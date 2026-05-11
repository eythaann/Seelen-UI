use std::{
    sync::{atomic::AtomicU8, Arc},
    time::Duration,
};

use seelen_core::state::{Widget, WidgetInstanceMode, WidgetStatus};
use tauri::{Emitter, Listener};
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use uuid::Uuid;

use crate::{
    app::get_app_handle,
    get_tokio_handle,
    modules::monitors::MonitorManager,
    resources::RESOURCES,
    state::application::FULL_STATE,
    utils::lock_free::SyncHashMap,
    widgets::{manager::WIDGET_MANAGER, webview::WidgetWebview, WidgetWebviewLabel},
    windows_api::event_window::IS_INTERACTIVE_SESSION,
};

const LIVENESS_PROVE_INTERVAL: Duration = Duration::from_secs(5);
const LIVENESS_PROVE_WAIT_TIMEOUT: Duration = Duration::from_secs(3);
const LIVENESS_PROVE_MAX_RETRIES: u8 = 5;
// Grace period after session resume or soft_restart to let the webview finish reloading.
const LIVENESS_RELOAD_GRACE_PERIOD: Duration = Duration::from_secs(10);

pub struct WidgetDeployment {
    pub definition: Arc<Widget>,
    pub pods: SyncHashMap<WidgetWebviewLabel, WidgetPod>,
}

impl WidgetDeployment {
    pub fn new(definition: Arc<Widget>) -> Self {
        Self {
            definition,
            pods: SyncHashMap::new(),
        }
    }

    /// Will revaluate all widget instances and remove or add them based on current user settings
    pub fn reconcile(&self) {
        match self.definition.instances {
            WidgetInstanceMode::Single => {
                if self.pods.is_empty() {
                    let instance = WidgetPod::create(&self.definition, None, None, None);
                    self.pods.upsert(instance.label.clone(), instance);
                }
            }
            WidgetInstanceMode::Multiple => {
                let nil_id = Uuid::nil();
                if self.pods.is_empty() {
                    let instance = WidgetPod::create(&self.definition, None, Some(&nil_id), None);
                    self.pods.upsert(instance.label.clone(), instance);
                }

                let replicas_ids = FULL_STATE
                    .load()
                    .get_widget_instances_ids(&self.definition.id);

                // Remove deleted instances
                self.pods.retain(|(label, _)| {
                    let instance_id = label.instance_id.expect("Missing instance id");
                    instance_id == nil_id || replicas_ids.contains(&instance_id)
                });

                // Add new instances
                for replica_id in replicas_ids {
                    if !self
                        .pods
                        .any(|(label, _)| label.instance_id == Some(replica_id))
                    {
                        let instance =
                            WidgetPod::create(&self.definition, None, Some(&replica_id), None);
                        self.pods.upsert(instance.label.clone(), instance);
                    }
                }
            }
            WidgetInstanceMode::ReplicaByMonitor => {
                let configs = FULL_STATE.load();

                // Remove disabled instances
                self.pods.retain(|(label, _)| {
                    let monitor_id = label.monitor_id.as_ref().expect("Missing monitor id");
                    configs.is_widget_enable_on_monitor(&self.definition.id, monitor_id)
                });

                // Add new/enabled instances
                for monitor_id in MonitorManager::instance().get_cached_ids() {
                    if self
                        .pods
                        .any(|(label, _)| label.monitor_id.as_ref() == Some(&monitor_id))
                    {
                        continue;
                    }

                    if !configs.is_widget_enable_on_monitor(&self.definition.id, &monitor_id) {
                        continue;
                    }

                    let instance =
                        WidgetPod::create(&self.definition, Some(&monitor_id), None, None);
                    self.pods.upsert(instance.label.clone(), instance);
                }
            }
        }
    }

    pub fn start_all_webviews(&self) {
        self.pods.for_each(|(_k, pod)| {
            pod.run(&self.definition);
        });
    }

    pub fn start_webview(&self, label: &WidgetWebviewLabel) {
        self.pods.get(label, |pod| {
            pod.run(&self.definition);
        });
    }

    pub fn create_runtime_instance(&self, instance_id: &Uuid, owner_hwnd: Option<isize>) {
        let instance = WidgetPod::create(&self.definition, None, Some(instance_id), owner_hwnd);
        self.pods.upsert(instance.label.clone(), instance);
    }

    pub fn kill_pod(&self, label: &WidgetWebviewLabel) {
        self.pods.remove(label);
    }
}

pub struct WidgetPod {
    pub label: WidgetWebviewLabel,

    window: Option<WidgetWebview>,
    _status: WidgetStatus,
    owner_hwnd: Option<isize>,

    live: Arc<tokio::sync::Notify>,
    liveness_prove_handle: Option<tokio::task::JoinHandle<()>>,
    retries: Arc<AtomicU8>,
}

impl WidgetPod {
    fn create(
        widget: &Widget,
        monitor_id: Option<&str>,
        instance_id: Option<&Uuid>,
        owner_hwnd: Option<isize>,
    ) -> Self {
        let label = WidgetWebviewLabel::new(&widget.id, monitor_id, instance_id);
        log::info!("Creating widget pod: {label}");
        Self {
            label,
            window: None,
            _status: WidgetStatus::Pending,
            owner_hwnd,
            live: Arc::new(tokio::sync::Notify::new()),
            liveness_prove_handle: None,
            retries: Arc::new(AtomicU8::new(0)),
        }
    }

    pub fn status(&self) -> &WidgetStatus {
        &self._status
    }

    pub fn set_status(&mut self, status: WidgetStatus) {
        log::trace!("{} status changed to: {:?}", self.label, status);
        self._status = status;
    }

    pub fn is_ready(&self) -> bool {
        self.window.is_some() && self.status() == &WidgetStatus::Ready
    }

    pub fn soft_restart(&mut self) {
        if self.window.is_none() {
            // Pod was never started; leave it in Pending so run() can initialize it.
            return;
        }
        self.set_status(WidgetStatus::Restarting);
        if let Some(window) = &self.window {
            window.reload();
        }
    }

    fn run(&mut self, definition: &Widget) {
        if self.status() != &WidgetStatus::Pending {
            return;
        }

        self.set_status(WidgetStatus::Creating);
        let window = match WidgetWebview::create(definition, &self.label, self.owner_hwnd) {
            Ok(window) => window,
            Err(err) => {
                log::error!("Failed to create webview: {}", err);
                self.set_status(WidgetStatus::CrashedOnCreation);
                return;
            }
        };
        self.set_status(WidgetStatus::Mounting);

        let label = self.label.clone();
        window.0.on_window_event(move |event| {
            if let tauri::WindowEvent::Destroyed = event {
                // Defer window creation off the UI message-loop thread.
                // Calling start_all_webviews() (→ WidgetWebview::create → builder.build)
                // synchronously here triggers a re-entrant ZwUserDestroyWindow while the
                // message pump is still inside the destruction handler, which causes the
                // APPLICATION_HANG_ENDTASK_HungThreadIsIdle crash.
                let label = label.clone();
                std::thread::spawn(move || {
                    WIDGET_MANAGER.deployments.get(&label.widget_id, |deploy| {
                        deploy.kill_pod(&label);
                        deploy.reconcile();
                        if !deploy.definition.lazy {
                            deploy.start_all_webviews();
                        }
                    });
                });
            }
        });

        if definition.debug {
            window.0.open_devtools();
        }

        self.window = Some(window);
        self.start_liveness_prove();
    }

    fn start_liveness_prove(&mut self) {
        if let Some(window) = &self.window {
            let live = self.live.clone();
            window.0.listen("internal::liveness-pong", move |_event| {
                live.notify_waiters();
            });
        }

        let live = self.live.clone();
        let label = self.label.clone();
        let retries = self.retries.clone();

        let handle = get_tokio_handle().spawn(async move {
            let app = get_app_handle();
            let mut was_suspended = false;

            loop {
                tokio::time::sleep(LIVENESS_PROVE_INTERVAL).await;
                if !IS_INTERACTIVE_SESSION.load(std::sync::atomic::Ordering::Acquire) {
                    was_suspended = true;
                    continue;
                }

                // After session resume, reset state and wait for webview to finish reloading.
                if was_suspended {
                    was_suspended = false;
                    retries.store(0, std::sync::atomic::Ordering::SeqCst);
                    tokio::time::sleep(LIVENESS_RELOAD_GRACE_PERIOD).await;
                    continue;
                }

                let _ = app.emit_to(&label.raw, "internal::liveness-ping", ());

                tokio::select! {
                    _ = live.notified() => {
                        // Widget is healthy: reset consecutive failure counter.
                        retries.store(0, std::sync::atomic::Ordering::SeqCst);
                    }
                    _ = tokio::time::sleep(LIVENESS_PROVE_WAIT_TIMEOUT) => {
                        let attempt = retries.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        log::warn!("Liveness prove failed for {label} (attempt {}/{LIVENESS_PROVE_MAX_RETRIES}), reloading webview.", attempt + 1);

                        if attempt < LIVENESS_PROVE_MAX_RETRIES {
                            WIDGET_MANAGER.deployments.get(&label.widget_id, |deployment| {
                                deployment.pods.get(&label, |pod| {
                                    pod.soft_restart();
                                });
                            });
                        } else {
                            log::error!("Liveness prove failed for {label} too many times, giving up.");
                            let lang = rust_i18n::locale();
                            let widget_name = RESOURCES
                                .widgets
                                .read(&label.widget_id, |_, w| w.metadata.display_name.get(&lang).to_string())
                                .unwrap_or_else(|| label.widget_id.to_string());
                            app.dialog()
                                .message(t!("widget_liveness.failed_description", widget_name = widget_name))
                                .title(t!("widget_liveness.failed_title"))
                                .kind(MessageDialogKind::Error)
                                .buttons(MessageDialogButtons::Ok)
                                .show(|_| {});
                            break;
                        }
                    }
                }
            }
        });

        self.liveness_prove_handle = Some(handle);
    }
}

impl Drop for WidgetPod {
    fn drop(&mut self) {
        log::trace!("Dropping widget pod: {}", self.label);
        if let Some(handle) = self.liveness_prove_handle.take() {
            handle.abort();
        }
    }
}
