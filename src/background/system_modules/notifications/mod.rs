mod application;
mod domain;

use std::{collections::HashMap, sync::Once};

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{AppNotification, NotificationsMode, ToastActionActivationType},
};

use crate::{app::emit_to_webviews, error::Result};

use self::{application::NotificationManager, domain::NotificationsEvent};

/// Manager instance, bridging its events to the webviews on first access.
fn get_manager() -> &'static NotificationManager {
    static REGISTER: Once = Once::new();
    REGISTER.call_once(|| {
        NotificationManager::subscribe(|event| match event {
            NotificationsEvent::ListChanged => {
                emit_to_webviews(
                    SeelenEvent::Notifications,
                    NotificationManager::instance().notifications(),
                );
            }
            NotificationsEvent::ModeChanged { mode } => {
                emit_to_webviews(SeelenEvent::NotificationsModeChanged, mode);
            }
        });
    });
    NotificationManager::instance()
}

impl crate::tauri_handlers::Handlers {
    pub fn get_notifications() -> Vec<AppNotification> {
        get_manager().notifications()
    }

    pub fn activate_notification(
        id: u32,
        umid: String,
        args: Option<String>,
        activation_type: ToastActionActivationType,
        input_data: HashMap<String, String>,
    ) -> Result<()> {
        get_manager().activate(
            id,
            &umid,
            &args.unwrap_or_default(),
            activation_type,
            input_data,
        )
    }

    pub fn notifications_close(id: u32) -> Result<()> {
        get_manager().remove_notification(id)
    }

    pub fn notifications_close_all() -> Result<()> {
        get_manager().clear_notifications();
        Ok(())
    }

    pub fn get_notifications_mode() -> Result<NotificationsMode> {
        get_manager().mode()
    }

    pub fn set_notifications_mode(mode: NotificationsMode) -> Result<()> {
        get_manager().set_mode(mode)
    }
}
