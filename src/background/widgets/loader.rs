use std::sync::Arc;

use seelen_core::state::{Widget, WidgetInstanceMode, WidgetStatus};
use uuid::Uuid;

use crate::{
    state::application::FULL_STATE,
    utils::lock_free::SyncHashMap,
    widgets::{manager::WIDGET_MANAGER, webview::WidgetWebview, WidgetWebviewLabel},
};

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
        if self.status() == &WidgetStatus::Pending {
            self.set_status(WidgetStatus::Creating);

            match WidgetWebview::create(definition, &self.label) {
                Ok(window) => {
                    let label = self.label.clone();
                    let instance_type = self.instance_type;
                    window.0.on_window_event(move |event| {
                        if let tauri::WindowEvent::Destroyed = event {
                            WIDGET_MANAGER.groups.get(&label.widget_id, |c| {
                                match instance_type {
                                    InstanceType::Runtime => {
                                        // Remove runtime instances on destroy
                                        c.instances.remove(&label);
                                    }
                                    InstanceType::Static => {
                                        // Reset static instances to pending state
                                        c.instances.get(&label, |w| {
                                            w.window = None;
                                            w.set_status(WidgetStatus::Pending);
                                        });
                                    }
                                }
                            });
                        }
                    });

                    self.window = Some(window);
                    self.set_status(WidgetStatus::Mounting);
                }
                Err(err) => {
                    log::error!("Failed to create webview: {}", err);
                    self.set_status(WidgetStatus::CrashedOnCreation);
                }
            }
        }
    }
}

impl Drop for WidgetInstance {
    fn drop(&mut self) {
        log::info!("Dropping {:?}", self.label);
    }
}
