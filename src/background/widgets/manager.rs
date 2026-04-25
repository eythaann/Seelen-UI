use std::sync::{
    atomic::{AtomicBool, Ordering},
    LazyLock,
};

use seelen_core::{
    resource::WidgetId,
    state::{WidgetLoader, WidgetStatus},
};

use crate::{
    error::Result,
    resources::RESOURCES,
    state::application::FULL_STATE,
    utils::lock_free::SyncHashMap,
    widgets::{loader::WidgetDeployment, WidgetWebviewLabel},
};

pub static WIDGET_MANAGER: LazyLock<WidgetManager> = LazyLock::new(WidgetManager::create);
pub static GAME_MODE_ACTIVE: AtomicBool = AtomicBool::new(false);

pub struct WidgetManager {
    /// group of widgets instances by widget resource id
    pub deployments: SyncHashMap<WidgetId, WidgetDeployment>,
}

impl WidgetManager {
    fn create() -> Self {
        Self {
            deployments: SyncHashMap::new(),
        }
    }

    pub fn is_ready(&self, label: &WidgetWebviewLabel) -> bool {
        self.deployments
            .get(&label.widget_id, |deploy| {
                deploy.pods.any(|(key, pod)| key == label && pod.is_ready())
            })
            .unwrap_or(false)
    }

    pub fn set_status(&self, label: &WidgetWebviewLabel, status: WidgetStatus) {
        self.deployments.get(&label.widget_id, |deploy| {
            deploy.pods.get(label, |instance| {
                instance.set_status(status);
            });
        });
    }

    pub fn suspend_all(&self) {
        GAME_MODE_ACTIVE.store(true, Ordering::Release);
        self.deployments.for_each(|(_, deploy)| {
            deploy.pods.clear();
        });
    }

    pub fn resume_all(&self) -> Result<()> {
        GAME_MODE_ACTIVE.store(false, Ordering::Release);
        self.reconcile()
    }

    pub fn reconcile(&self) -> Result<()> {
        // remove deleted resources
        self.deployments
            .retain(|(key, _)| RESOURCES.widgets.contains(key));

        let mut filtered = Vec::new();
        RESOURCES.widgets.scan(|k, w| {
            if w.loader != WidgetLoader::Legacy {
                filtered.push((k.clone(), w.clone()));
            }
        });

        let state = FULL_STATE.load();
        for (id, widget) in filtered {
            if !state.is_widget_enabled(&id) {
                self.deployments.remove(&id);
                continue;
            }

            if !self.deployments.contains_key(&id) {
                self.deployments
                    .upsert(id.clone(), WidgetDeployment::new(widget));
            }
        }

        // lazy creation of webviews to reduce startup time
        std::thread::spawn(|| {
            WIDGET_MANAGER.deployments.for_each(|(_, deployment)| {
                deployment.reconcile();

                if !deployment.definition.lazy && !GAME_MODE_ACTIVE.load(Ordering::Acquire) {
                    deployment.start_all_webviews();
                }
            });
        });

        Ok(())
    }
}
