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
    error::ResultLogExt,
    get_tokio_handle,
    resources::RESOURCES,
    state::application::FULL_STATE,
    utils::lock_free::SyncHashMap,
    widgets::{manager::WIDGET_MANAGER, webview::WidgetWebview, WidgetWebviewLabel},
};

const LIVENESS_PROVE_INTERVAL: Duration = Duration::from_secs(10);
const LIVENESS_PROVE_WAIT_TIMEOUT: Duration = Duration::from_secs(5);
const LIVENESS_PROVE_MAX_RETRIES: u8 = 5;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstanceType {
    /// Instances defined in user settings
    Static,
    /// Instances created dynamically during runtime
    Runtime,
}

pub struct WidgetContainer {
    pub definition: Arc<Widget>,
    pub instances: SyncHashMap<WidgetWebviewLabel, WidgetInstance>,
}

impl WidgetContainer {
    pub fn create(widget: Arc<Widget>) -> Self {
        let instances = SyncHashMap::new();

        match widget.instances {
            WidgetInstanceMode::Single => {
                let instance = WidgetInstance::create(&widget, None, None, InstanceType::Static);
                instances.upsert(instance.label.clone(), instance);
            }
            WidgetInstanceMode::Multiple => {
                for replica_id in FULL_STATE.load().get_widget_instances_ids(&widget.id) {
                    let instance = WidgetInstance::create(
                        &widget,
                        None,
                        Some(&replica_id),
                        InstanceType::Static,
                    );
                    instances.upsert(instance.label.clone(), instance);
                }
            }
            WidgetInstanceMode::ReplicaByMonitor => {}
        }

        Self {
            definition: widget,
            instances,
        }
    }

    pub fn start_all_webviews(&self) {
        self.instances.for_each(|(_k, v)| {
            v.start_webview(&self.definition);
        });
    }

    pub fn start_webview(&self, label: &WidgetWebviewLabel) {
        self.instances.get(label, |instance| {
            instance.start_webview(&self.definition);
        });
    }

    pub fn create_runtime_instance(&self, instance_id: &Uuid) {
        let instance = WidgetInstance::create(
            &self.definition,
            None,
            Some(instance_id),
            InstanceType::Runtime,
        );
        self.instances.upsert(instance.label.clone(), instance);
    }
}

pub struct WidgetInstance {
    pub label: WidgetWebviewLabel,
    pub instance_type: InstanceType,
    window: Option<WidgetWebview>,
    _status: WidgetStatus,

    live: Arc<tokio::sync::Notify>,
    retries: Arc<AtomicU8>,
}

impl WidgetInstance {
    fn create(
        widget: &Widget,
        monitor_id: Option<&str>,
        instance_id: Option<&Uuid>,
        instance_type: InstanceType,
    ) -> Self {
        let label = WidgetWebviewLabel::new(&widget.id, monitor_id, instance_id);
        log::info!("Starting widget instance: {label}");
        Self {
            label,
            instance_type,
            window: None,
            _status: WidgetStatus::Pending,
            live: Arc::new(tokio::sync::Notify::new()),
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

    fn start_webview(&mut self, definition: &Widget) {
        if self.status() != &WidgetStatus::Pending {
            return;
        }

        self.set_status(WidgetStatus::Creating);
        let window = match WidgetWebview::create(definition, &self.label) {
            Ok(window) => window,
            Err(err) => {
                log::error!("Failed to create webview: {}", err);
                self.set_status(WidgetStatus::CrashedOnCreation);
                return;
            }
        };
        self.set_status(WidgetStatus::Mounting);

        let live = self.live.clone();
        let label = self.label.clone();
        let retries = self.retries.clone();
        let liveness_prove = get_tokio_handle().spawn(async move {
            let app = get_app_handle();

            loop {
                tokio::time::sleep(LIVENESS_PROVE_INTERVAL).await;
                let _ = app.emit_to(&label.raw, "internal::liveness-ping", ());

                tokio::select! {
                    _ = live.notified() => {
                        // log::trace!("Liveness prove succeeded for {label}");
                    }
                    _ = tokio::time::sleep(LIVENESS_PROVE_WAIT_TIMEOUT) => {
                        log::warn!("Liveness prove failed for {label}, reloading webview.");

                        let attempt = retries.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                        if attempt < LIVENESS_PROVE_MAX_RETRIES {
                            WIDGET_MANAGER.groups.get(&label.widget_id, |c| {
                                c.instances.get(&label, |w| {
                                    w.set_status(WidgetStatus::Pending);
                                    if let Some(window) = &w.window {
                                        window.0.reload().log_error();
                                    }
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

        let label = self.label.clone();
        let instance_type = self.instance_type;
        window.0.on_window_event(move |event| {
            if let tauri::WindowEvent::Destroyed = event {
                WIDGET_MANAGER.groups.get(&label.widget_id, |c| {
                    liveness_prove.abort();
                    match instance_type {
                        // Remove runtime instances on destroy
                        InstanceType::Runtime => {
                            c.instances.remove(&label);
                        }
                        // Reset static instances to pending state
                        InstanceType::Static => {
                            c.instances.get(&label, |w| {
                                w.window = None;
                                w.retries.store(0, std::sync::atomic::Ordering::SeqCst);
                                w.set_status(WidgetStatus::Pending);
                            });
                        }
                    }
                });
            }
        });

        let live = self.live.clone();
        window.0.listen("internal::liveness-pong", move |_event| {
            live.notify_waiters();
        });

        self.window = Some(window);
    }
}

impl Drop for WidgetInstance {
    fn drop(&mut self) {
        log::info!("Dropping {:?}", self.label);
    }
}
