use std::collections::HashMap;

use seelen_core::system_state::{AppNotification, NotificationsMode, ToastActionActivationType};

use crate::error::Result;

/// OS contract of the notifications module (the notification center).
pub trait NotificationsSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Requests access to the notifications and starts listening them, reported through
    /// `on_event`. Called once; the subscriptions live as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(NotificationsSysEvent) + Send + Sync + 'static;

    /// Ids of the notifications currently in the notification center.
    fn notification_ids(&self) -> Result<Vec<u32>>;

    /// Reads a notification. `None` if it can't be shown (e.g. its app was uninstalled).
    fn notification(&self, id: u32) -> Result<Option<AppNotification>>;

    /// Removes the notification from the notification center.
    fn remove(&self, id: u32) -> Result<()>;

    /// Runs the action of a notification (or the notification itself) on its app.
    fn activate(
        &self,
        umid: &str,
        args: &str,
        activation_type: ToastActionActivationType,
        input_data: HashMap<String, String>,
    ) -> Result<()>;

    fn mode(&self) -> Result<NotificationsMode>;

    /// Returns whether the change will be notified as `ModeChanged`.
    fn set_mode(&self, mode: NotificationsMode) -> Result<bool>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationsSysEvent {
    Added { id: u32 },
    Removed { id: u32 },
    ModeChanged { mode: NotificationsMode },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationsEvent {
    ListChanged,
    ModeChanged { mode: NotificationsMode },
}
