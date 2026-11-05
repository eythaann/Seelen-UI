#[cfg(windows)]
mod windows;

use std::{collections::HashMap, sync::LazyLock};

use seelen_core::system_state::{AppNotification, NotificationsMode, ToastActionActivationType};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::lock_free::SyncHashMap,
};

use super::domain::{NotificationsEvent, NotificationsSysEvent, NotificationsSystem};

#[cfg(windows)]
type Platform = windows::WindowsNotifications;

pub struct NotificationManager {
    platform: Platform,
    notifications: SyncHashMap<u32, AppNotification>,
}

event_manager!(NotificationManager, NotificationsEvent);

impl NotificationManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<NotificationManager> = LazyLock::new(|| {
            let mut m = NotificationManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            notifications: SyncHashMap::new(),
        }
    }

    fn init(&mut self) -> Result<()> {
        self.platform.init(Self::on_platform_event)?;
        for id in self.platform.notification_ids()? {
            self.load_notification(id).log_error();
        }
        Ok(())
    }

    fn on_platform_event(event: NotificationsSysEvent) {
        let manager = Self::instance();
        match event {
            NotificationsSysEvent::Added { id } => {
                manager.load_notification(id).log_error();
                Self::send(NotificationsEvent::ListChanged);
            }
            NotificationsSysEvent::Removed { id } => {
                manager.notifications.remove(&id);
                Self::send(NotificationsEvent::ListChanged);
            }
            NotificationsSysEvent::ModeChanged { mode } => {
                Self::send(NotificationsEvent::ModeChanged { mode });
            }
        }
    }

    fn load_notification(&self, id: u32) -> Result<()> {
        if let Some(notification) = self.platform.notification(id)? {
            self.notifications.upsert(id, notification);
        }
        Ok(())
    }

    /// Newest first.
    pub fn notifications(&self) -> Vec<AppNotification> {
        let mut notifications = self.notifications.values();
        notifications.sort_by_key(|b| std::cmp::Reverse(b.date));
        notifications
    }

    pub fn remove_notification(&self, id: u32) -> Result<()> {
        self.platform.remove(id)?;
        self.notifications.remove(&id);
        Self::send(NotificationsEvent::ListChanged);
        Ok(())
    }

    pub fn clear_notifications(&self) {
        for id in self.notifications.keys() {
            self.platform.remove(id).log_error();
        }
        self.notifications.clear();
        Self::send(NotificationsEvent::ListChanged);
    }

    /// Runs the action on the app and, as the notification center does, removes the
    /// notification once handled.
    pub fn activate(
        &self,
        id: u32,
        umid: &str,
        args: &str,
        activation_type: ToastActionActivationType,
        input_data: HashMap<String, String>,
    ) -> Result<()> {
        log::trace!(
            "Activating notification '{umid}' (id={id}, type={activation_type:?}) with args '{args}'"
        );
        self.platform
            .activate(umid, args, activation_type, input_data)?;
        let _ = self.remove_notification(id);
        Ok(())
    }

    pub fn mode(&self) -> Result<NotificationsMode> {
        self.platform.mode()
    }

    /// Changes the "Do not disturb" state.
    pub fn set_mode(&self, mode: NotificationsMode) -> Result<()> {
        let notified = self.platform.set_mode(mode)?;
        if !notified {
            Self::send(NotificationsEvent::ModeChanged { mode });
        }
        Ok(())
    }
}
