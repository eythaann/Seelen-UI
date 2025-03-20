use std::{
    collections::HashSet,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::system_state::{AppNotification, Toast, ToastBindingEntry};
use windows::{
    Foundation::TypedEventHandler,
    Win32::System::WinRT::EventRegistrationToken,
    UI::Notifications::{
        KnownNotificationBindings,
        Management::{UserNotificationListener, UserNotificationListenerAccessStatus},
        NotificationKinds, ToastNotificationManager, ToastNotificationManagerForUser,
        UserNotification, UserNotificationChangedEventArgs,
    },
};

use crate::{
    error_handler::Result, event_manager, log_error,
    seelen_weg::icon_extractor::extract_and_save_icon_umid, trace_lock, utils::spawn_named_thread,
    windows_api::traits::EventRegistrationTokenExt,
};

lazy_static! {
    pub static ref NOTIFICATION_MANAGER: Arc<Mutex<NotificationManager>> = Arc::new(Mutex::new(
        NotificationManager::new().expect("Failed to create notification manager")
    ));
}

static RELEASED: AtomicBool = AtomicBool::new(true);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationEvent {
    Added(u32),
    Removed(u32),
}

pub struct NotificationManager {
    notifications: Vec<AppNotification>,
    notifications_id: HashSet<u32>,
    manager: ToastNotificationManagerForUser,
    listener: UserNotificationListener,
    event_handler: TypedEventHandler<UserNotificationListener, UserNotificationChangedEventArgs>,
    event_token: Option<EventRegistrationToken>,
}

unsafe impl Send for NotificationManager {}

event_manager!(NotificationManager, NotificationEvent);

impl NotificationManager {
    fn new() -> Result<Self> {
        Ok(Self {
            notifications: Vec::new(),
            notifications_id: HashSet::new(),
            manager: ToastNotificationManager::GetDefault()?,
            listener: UserNotificationListener::Current()?,
            event_handler: TypedEventHandler::new(Self::internal_notifications_change),
            event_token: None,
        })
    }

    pub fn notifications(&self) -> &Vec<AppNotification> {
        &self.notifications
    }

    pub fn remove_notification(&mut self, id: u32) -> Result<()> {
        self.listener.RemoveNotification(id)?;
        Self::event_tx().send(NotificationEvent::Removed(id))?;
        Ok(())
    }

    pub fn clear_notifications(&mut self) -> Result<()> {
        for notification in self.notifications() {
            self.listener.RemoveNotification(notification.id)?;
            Self::event_tx().send(NotificationEvent::Removed(notification.id))?;
        }
        Ok(())
    }

    pub fn initialize(&mut self) -> Result<()> {
        let access = self.listener.RequestAccessAsync()?.get()?;
        if access != UserNotificationListenerAccessStatus::Allowed {
            return Err("Failed to get notification access".into());
        }

        // TODO: this only works on MSIX/APPX/UWP builds so idk how to make it work on win32 apps
        match self.listener.NotificationChanged(&self.event_handler) {
            Ok(token) => self.event_token = Some(token.as_event_token()),
            Err(error) => {
                log::warn!(
                    "Failed to register winrt notification change handler: {}",
                    error
                );
                // intead we use a thread
                spawn_named_thread("Notification Manager", || -> Result<()> {
                    RELEASED.store(false, Ordering::SeqCst);
                    while !RELEASED.load(Ordering::Acquire) {
                        log_error!(Self::internal_notifications_change(&None, &None));
                        std::thread::sleep(std::time::Duration::from_secs(1));
                    }
                    Ok(())
                })?;
            }
        }

        let u_notifications = self
            .listener
            .GetNotificationsAsync(NotificationKinds::Toast)?
            .get()?;

        for u_notification in u_notifications {
            log_error!(self.load_notification(u_notification));
        }

        Self::subscribe(|e| log_error!(Self::process_event(e)));
        Ok(())
    }

    pub fn release(&mut self) -> Result<()> {
        if let Some(token) = self.event_token.take() {
            self.listener.RemoveNotificationChanged(token.value)?;
        }
        RELEASED.store(true, Ordering::Release);
        Ok(())
    }

    fn internal_notifications_change(
        _listener: &Option<UserNotificationListener>,
        _args: &Option<UserNotificationChangedEventArgs>,
    ) -> windows_core::Result<()> {
        let listener = { UserNotificationListener::Current()? };
        let mut old_toasts: HashSet<u32> =
            { trace_lock!(NOTIFICATION_MANAGER).notifications_id.clone() };

        for u_notification in listener
            .GetNotificationsAsync(NotificationKinds::Toast)?
            .get()?
        {
            let id: u32 = u_notification.Id()?;
            if !old_toasts.contains(&id) {
                log_error!(Self::event_tx().send(NotificationEvent::Added(id)));
            }
            old_toasts.remove(&id);
        }

        for id in old_toasts {
            log_error!(Self::event_tx().send(NotificationEvent::Removed(id)));
        }
        Ok(())
    }

    fn process_event(event: NotificationEvent) -> Result<()> {
        match event {
            NotificationEvent::Added(id) => {
                let u_notification = UserNotificationListener::Current()?.GetNotification(id)?;
                trace_lock!(NOTIFICATION_MANAGER).load_notification(u_notification)?;
            }
            NotificationEvent::Removed(id) => {
                let mut manager = trace_lock!(NOTIFICATION_MANAGER);
                manager.notifications_id.remove(&id);
                manager.notifications.retain(|n| n.id != id);
            }
        }
        Ok(())
    }

    // this function an in general all the notification system can still be improved on usability and performance
    fn load_notification(&mut self, u_notification: UserNotification) -> Result<()> {
        self.notifications_id.insert(u_notification.Id()?);
        let notification = u_notification.Notification()?;

        let app_info = u_notification.AppInfo()?;
        let display_info = app_info.DisplayInfo()?;
        let app_umid = app_info.AppUserModelId()?;

        let visuals = notification.Visual()?;
        let text_sequence = visuals
            .GetBinding(&KnownNotificationBindings::ToastGeneric()?)?
            .GetTextElements()?;
        let mut notification_text = Vec::new();
        for text in text_sequence {
            notification_text.push(text.Text()?.to_string());
        }

        let history = self.manager.History()?;
        let toast_notifications = history.GetHistoryWithId(&app_umid)?;

        log::trace!(
            "Loading notification, ID: {}, AppID: {}",
            u_notification.Id()?,
            app_umid
        );

        let mut notification_content = None;
        for toast_notification in toast_notifications {
            // this can be null when the notification count is bigger than the max allowed by default 20
            if let Ok(content) = toast_notification.Content() {
                let data = content.GetXml()?.to_string();
                let toast: Toast = quick_xml::de::from_str(&data)?;

                let mut toast_text = Vec::new();
                for entry in &toast.visual.binding.entries {
                    if let ToastBindingEntry::Text(text) = entry {
                        toast_text.push(text.content.clone());
                    }
                }

                if notification_text == toast_text {
                    notification_content = Some(toast);
                    break;
                }
            }
        }

        // pre-extraction to avoid flickering on the ui
        let _ = extract_and_save_icon_umid(&app_umid.to_string().into());

        self.notifications.push(AppNotification {
            id: u_notification.Id()?,
            app_umid: app_umid.to_string(),
            app_name: display_info.DisplayName()?.to_string(),
            app_description: display_info.Description()?.to_string(),
            date: u_notification.CreationTime()?.UniversalTime,
            content: notification_content.ok_or("Failed to get notification content")?,
        });
        self.notifications.sort_by(|a, b| b.date.cmp(&a.date));
        Ok(())
    }
}
