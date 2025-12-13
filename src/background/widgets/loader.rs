use std::sync::Arc;

use seelen_core::state::{Widget, WidgetInstanceMode, WidgetStatus};
use uuid::Uuid;

use crate::{
    state::application::FULL_STATE,
    utils::{lock_free::SyncHashMap, WidgetWebviewLabel},
    widgets::webview::WidgetWebview,
};

pub struct WidgetContainer {
    pub definition: Arc<Widget>,
    pub instances: SyncHashMap<WidgetWebviewLabel, WidgetInstance>,
}

impl WidgetContainer {
    pub fn create(widget: Arc<Widget>) -> Self {
        let instances = SyncHashMap::new();

        match widget.instances {
            WidgetInstanceMode::Single => {
                let instance = WidgetInstance::create(&widget, None, None);
                instances.upsert(instance.label.clone(), instance);
            }
            WidgetInstanceMode::Multiple => {
                for replica_id in FULL_STATE.load().get_widget_instances_ids(&widget.id) {
                    let instance = WidgetInstance::create(&widget, None, Some(&replica_id));
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
}

pub struct WidgetInstance {
    pub label: WidgetWebviewLabel,
    window: Option<WidgetWebview>,
    pub state: WidgetStatus,
}

impl WidgetInstance {
    fn create(widget: &Widget, monitor_id: Option<&str>, instance_id: Option<&Uuid>) -> Self {
        let label = WidgetWebviewLabel::new(&widget.id, monitor_id, instance_id);
        log::info!("Creating {:?}", label.decoded);
        Self {
            label,
            window: None,
            state: WidgetStatus::Pending,
        }
    }

    fn start_webview(&mut self, definition: &Widget) {
        if self.state == WidgetStatus::Pending {
            self.state = WidgetStatus::Creating;
            self.window = WidgetWebview::create(definition, &self.label).ok();
        }
    }
}

impl Drop for WidgetInstance {
    fn drop(&mut self) {
        log::info!("Dropping {:?}", self.label.decoded);
    }
}
