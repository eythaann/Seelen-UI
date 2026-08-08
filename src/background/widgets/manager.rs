use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, LazyLock,
};

use seelen_core::{
    resource::WidgetId,
    state::{WidgetLoader, WidgetStatus},
};

use crate::{
    error::{Result, ResultLogExt},
    modules::monitors::MonitorManager,
    resources::RESOURCES,
    state::application::FULL_STATE,
    utils::lock_free::SyncHashMap,
    widgets::{loader::WidgetDeployment, WidgetWebviewLabel},
};

pub static WIDGET_MANAGER: LazyLock<WidgetManager> = LazyLock::new(WidgetManager::create);
pub static GAME_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);

pub struct WidgetManager {
    /// group of widgets instances by widget resource id
    pub deployments: SyncHashMap<WidgetId, Arc<WidgetDeployment>>,
}

impl WidgetManager {
    fn create() -> Self {
        let sub_id = MonitorManager::subscribe(|_event| {
            WIDGET_MANAGER.reconcile().log_error();
        });
        MonitorManager::set_event_handler_priority(&sub_id, 1);
        Self {
            deployments: SyncHashMap::new(),
        }
    }

    pub fn is_ready(&self, label: &WidgetWebviewLabel) -> bool {
        self.deployments
            .get_cloned(&label.widget_id)
            .map(|deploy| deploy.pods.any(|(key, pod)| key == label && pod.is_ready()))
            .unwrap_or(false)
    }

    pub fn set_status(&self, label: &WidgetWebviewLabel, status: WidgetStatus) {
        if let Some(deploy) = self.deployments.get_cloned(&label.widget_id) {
            deploy.pods.get(label, |instance| {
                instance.set_status(status);
            });
        }
    }

    pub fn suspend_all(&self) {
        GAME_MODE_ACTIVE.store(true, Ordering::Release);
        for deploy in self.deployments.values() {
            deploy.pods.clear();
        }
    }

    pub fn resume_all(&self) -> Result<()> {
        GAME_MODE_ACTIVE.store(false, Ordering::Release);
        self.reconcile()
    }

    pub fn reconcile(&self) -> Result<()> {
        // remove deleted resources
        self.deployments
            .retain(|(key, _)| RESOURCES.widgets.contains_sync(key));

        let mut filtered = Vec::new();
        RESOURCES.widgets.iter_sync(|k, w| {
            if w.loader != WidgetLoader::Legacy {
                filtered.push((k.clone(), w.clone()));
            }
            true
        });

        let state = FULL_STATE.load();
        for (id, widget) in filtered {
            if !state.is_widget_enabled(&id) {
                self.deployments.remove(&id);
                continue;
            }

            if !self.deployments.contains_key(&id) {
                self.deployments
                    .upsert(id.clone(), Arc::new(WidgetDeployment::new(widget)));
            }
        }

        // lazy creation of webviews to reduce startup time
        std::thread::spawn(|| {
            fn reconcile(deployment: &WidgetDeployment) {
                deployment.reconcile();
                if !deployment.definition.lazy && !GAME_MODE_ACTIVE.load(Ordering::Acquire) {
                    deployment.start_all_webviews();
                }
            }

            // More visual widgets load first
            for priority in [
                WidgetId::known_wall(),
                WidgetId::known_toolbar(),
                WidgetId::known_weg(),
            ] {
                if let Some(deployment) = WIDGET_MANAGER.deployments.get_cloned(&priority) {
                    reconcile(&deployment);
                }
            }

            // Clone the Arc handles while locked, then perform webview work outside the map lock.
            for deployment in WIDGET_MANAGER.deployments.values() {
                reconcile(&deployment);
            }
        });

        Ok(())
    }
}
