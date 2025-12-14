use std::sync::LazyLock;

use seelen_core::{
    resource::WidgetId,
    state::{WidgetLoader, WidgetStatus},
};

use crate::{
    error::Result,
    resources::RESOURCES,
    state::application::FULL_STATE,
    utils::{lock_free::SyncHashMap, WidgetWebviewLabel},
    widgets::loader::WidgetContainer,
};

pub static WIDGET_MANAGER: LazyLock<WidgetManager> = LazyLock::new(WidgetManager::create);

pub struct WidgetManager {
    /// group of widgets instances by widget resource id
    pub groups: SyncHashMap<WidgetId, WidgetContainer>,
}

impl WidgetManager {
    fn create() -> Self {
        Self {
            groups: SyncHashMap::new(),
        }
    }

    pub fn is_ready(&self, label: &WidgetWebviewLabel) -> bool {
        self.groups
            .get(&label.widget_id, |c| {
                c.instances.any(|(k, i)| k == label && i.is_ready())
            })
            .unwrap_or(false)
    }

    pub fn set_status(&self, label: &WidgetWebviewLabel, status: WidgetStatus) {
        self.groups.get(&label.widget_id, |c| {
            c.instances.get(label, |instance| {
                instance.set_status(status);
            });
        });
    }

    pub fn refresh(&self) -> Result<()> {
        // remove deleted resources
        self.groups.retain(|key, _| RESOURCES.widgets.contains(key));

        let mut filtered = Vec::new();
        RESOURCES.widgets.scan(|k, w| {
            if w.loader != WidgetLoader::Legacy {
                filtered.push((k.clone(), w.clone()));
            }
        });

        let state = FULL_STATE.load();
        for (id, widget) in filtered {
            if !state.is_widget_enabled(&id) {
                self.groups.remove(&id);
                continue;
            }

            if !self.groups.contains_key(&id) {
                self.groups
                    .upsert(id.clone(), WidgetContainer::create(widget));
            }
        }

        // lazy creation of webviews to reduce startup time
        std::thread::spawn(|| {
            WIDGET_MANAGER.groups.for_each(|(_, c)| {
                if !c.definition.lazy {
                    c.start_all_webviews();
                }
            });
        });

        Ok(())
    }
}
