use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    time::Duration,
};

use lazy_static::lazy_static;
use parking_lot::Mutex;
use serde::Serialize;
use windows::{
    Foundation::{EventRegistrationToken, TypedEventHandler},
    UI::Notifications::{
        KnownNotificationBindings,
        Management::{UserNotificationListener, UserNotificationListenerAccessStatus},
        NotificationKinds, UserNotification, UserNotificationChangedEventArgs,
    },
};

use crate::{
    error_handler::Result, log_error, seelen_weg::icon_extractor::extract_and_save_icon_umid,
    utils::spawn_named_thread,
};

lazy_static! {
    pub static ref NOTIFICATION_MANAGER: Arc<Mutex<NotificationManager>> = Arc::new(Mutex::new(
        NotificationManager::new().expect("Failed to create notification manager")
    ));
}

#[derive(Debug, Serialize)]
#[allow(dead_code)]
pub struct AppNotification {
    pub id: u32,
    app_umid: String,
    app_name: String,
    app_description: String,
    body: Vec<String>,
    date: i64,
}

enum NotificationEvent {
    Added(u32),
    Removed(u32),
}

type OnNotificationsChange = Box<dyn Fn(&Vec<AppNotification>) + Send + Sync>;

pub struct NotificationManager {
    listener: UserNotificationListener,
    notifications: Vec<AppNotification>,
    notifications_ids: Vec<u32>,
    callbacks: Vec<OnNotificationsChange>,
    #[allow(dead_code)]
    event_handler: TypedEventHandler<UserNotificationListener, UserNotificationChangedEventArgs>,
    event_token: Option<EventRegistrationToken>,
}

unsafe impl Send for NotificationManager {}

impl NotificationManager {
    pub fn notifications(&self) -> &Vec<AppNotification> {
        &self.notifications
    }
}

static RELEASED: AtomicBool = AtomicBool::new(true);

impl NotificationManager {
    fn new() -> Result<Self> {
        let mut manager = Self {
            listener: UserNotificationListener::Current()?,
            callbacks: Vec::new(),
            notifications: Vec::new(),
            notifications_ids: Vec::new(),
            event_handler: TypedEventHandler::new(Self::internal_notifications_change),
            event_token: None,
        };
        manager.initialize()?;
        Ok(manager)
    }

    pub fn remove_notification(&mut self, id: u32) -> Result<()> {
        self.notifications.retain(|n| n.id != id);
        self.notify_changes();
        self.listener.RemoveNotification(id)?;
        Ok(())
    }

    pub fn clear_notifications(&mut self) -> Result<()> {
        self.notifications.clear();
        self.notify_changes();
        for notification in self.notifications() {
            self.listener.RemoveNotification(notification.id)?;
        }
        Ok(())
    }

    fn initialize(&mut self) -> Result<()> {
        let access = self.listener.RequestAccessAsync()?.get()?;
        if access != UserNotificationListenerAccessStatus::Allowed {
            return Err("Failed to get notification access".into());
        }

        // TODO: this only works on MSIX/APPX/UWP builds so idk how to make it work on win32 apps
        // self.listener.NotificationChanged(&self.event_handler)?;
        // intead we use a thread
        spawn_named_thread("Notification Manager", || -> Result<()> {
            RELEASED.store(false, Ordering::SeqCst);
            while !RELEASED.load(Ordering::Acquire) {
                log_error!(Self::internal_notifications_change(&None, &None));
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            Ok(())
        })?;

        let u_notifications = self
            .listener
            .GetNotificationsAsync(NotificationKinds::Toast)?
            .get()?;

        for u_notification in u_notifications {
            log_error!(self.load_notification(u_notification));
        }

        Ok(())
    }

    pub fn release(&mut self) -> Result<()> {
        if let Some(token) = self.event_token.take() {
            self.listener.RemoveNotificationChanged(token)?;
        }
        RELEASED.store(true, Ordering::Release);
        Ok(())
    }

    fn internal_notifications_change(
        _listener: &Option<UserNotificationListener>,
        _args: &Option<UserNotificationChangedEventArgs>,
    ) -> windows_core::Result<()> {
        let mut manager = NOTIFICATION_MANAGER
            .try_lock_for(Duration::from_secs(5))
            .expect("Failed to lock");
        let mut current_list = manager.notifications_ids.clone();

        for u_notification in manager
            .listener
            .GetNotificationsAsync(NotificationKinds::Toast)?
            .get()?
        {
            let id = u_notification.Id()?;
            if !current_list.contains(&id) {
                manager.emit_event(NotificationEvent::Added(id));
            }
            current_list.retain(|x| *x != id);
        }

        for id in current_list {
            manager.emit_event(NotificationEvent::Removed(id));
        }
        Ok(())
    }

    pub fn notify_changes(&self) {
        for callback in &self.callbacks {
            callback(self.notifications());
        }
    }

    fn emit_event(&mut self, event: NotificationEvent) {
        log_error!(self.process_event(event));
        self.notify_changes();
    }

    fn process_event(&mut self, event: NotificationEvent) -> Result<()> {
        match event {
            NotificationEvent::Added(id) => {
                let u_notification = self.listener.GetNotification(id)?;
                self.load_notification(u_notification)?;
            }
            NotificationEvent::Removed(id) => {
                self.notifications.retain(|n| n.id != id);
                self.notifications_ids.retain(|x| *x != id);
            }
        }
        Ok(())
    }

    pub fn on_notifications_change<F>(&mut self, callback: F)
    where
        F: Fn(&Vec<AppNotification>) + Send + Sync + 'static,
    {
        self.callbacks.push(Box::new(callback));
    }

    fn load_notification(&mut self, u_notification: UserNotification) -> Result<()> {
        let notification = u_notification.Notification()?;

        let app_info = u_notification.AppInfo()?;
        let display_info = app_info.DisplayInfo()?;

        let visuals = notification.Visual()?;

        let text_sequence = visuals
            .GetBinding(&KnownNotificationBindings::ToastGeneric()?)?
            .GetTextElements()?;

        let mut body = Vec::new();
        for text in text_sequence {
            body.push(text.Text()?.to_string());
        }

        let umid = app_info.AppUserModelId()?.to_string_lossy();
        log_error!(extract_and_save_icon_umid(umid.clone()));

        self.notifications.push(AppNotification {
            id: u_notification.Id()?,
            app_umid: umid,
            app_name: display_info.DisplayName()?.to_string(),
            app_description: display_info.Description()?.to_string(),
            body,
            date: u_notification.CreationTime()?.UniversalTime,
        });
        self.notifications_ids.push(u_notification.Id()?);
        Ok(())
    }
}
