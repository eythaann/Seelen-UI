use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::system_state::{AppNotification, Toast, ToastBindingEntry};
use tauri::Manager;
use windows::{
    ApplicationModel::AppInfo,
    Foundation::{TypedEventHandler, Uri},
    Win32::System::WinRT::EventRegistrationToken,
    UI::Notifications::{
        KnownNotificationBindings,
        Management::{UserNotificationListener, UserNotificationListenerAccessStatus},
        NotificationKinds, ToastNotificationManager, ToastNotificationManagerForUser,
        UserNotification, UserNotificationChangedEventArgs,
    },
};

use crate::{
    error_handler::Result,
    event_manager, log_error,
    modules::uwp::get_hightest_quality_posible,
    seelen::get_app_handle,
    trace_lock,
    utils::{convert_file_to_src, icon_extractor::extract_and_save_icon_umid, spawn_named_thread},
    windows_api::traits::EventRegistrationTokenExt,
};

lazy_static! {
    pub static ref NOTIFICATION_MANAGER: Arc<Mutex<NotificationManager>> = Arc::new(Mutex::new(
        NotificationManager::new().expect("Failed to create notification manager")
    ));
    pub static ref LOADED_NOTIFICATIONS: Arc<Mutex<HashSet<u32>>> =
        Arc::new(Mutex::new(HashSet::new()));
}

static RELEASED: AtomicBool = AtomicBool::new(true);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationEvent {
    Added(u32),
    Removed(u32),
    Cleared,
}

pub struct NotificationManager {
    notifications: Vec<AppNotification>,
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
        let mut umids = HashSet::new();
        for n in self.notifications() {
            umids.insert(n.app_umid.clone());
        }
        for umid in umids {
            let history = self.manager.History()?;
            history.ClearWithId(&umid.into())?;
        }
        Self::event_tx().send(NotificationEvent::Cleared)?;
        Ok(())
    }

    pub fn initialize(&mut self) -> Result<()> {
        let access = self.listener.RequestAccessAsync()?.get()?;
        if access != UserNotificationListenerAccessStatus::Allowed {
            return Err("Failed to get notification access".into());
        }

        let u_notifications = self
            .listener
            .GetNotificationsAsync(NotificationKinds::Toast)?
            .get()?;
        for u_notification in u_notifications {
            log_error!(self.load_notification(u_notification));
        }

        // TODO: this only works on MSIX/APPX/UWP builds so idk how to make it work on win32 apps
        match self.listener.NotificationChanged(&self.event_handler) {
            Ok(token) => self.event_token = Some(token.as_event_token()),
            Err(error) => {
                log::warn!("Failed to register winrt notification change handler: {error}");
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
        let mut old_toasts = { trace_lock!(LOADED_NOTIFICATIONS).clone() };

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
        let mut manager = trace_lock!(NOTIFICATION_MANAGER);
        match event {
            NotificationEvent::Added(id) => {
                let u_notification = UserNotificationListener::Current()?.GetNotification(id)?;
                manager.load_notification(u_notification)?;
            }
            NotificationEvent::Removed(id) => {
                manager.notifications.retain(|n| n.id != id);
                trace_lock!(LOADED_NOTIFICATIONS).remove(&id);
            }
            NotificationEvent::Cleared => {
                manager.notifications.clear();
                trace_lock!(LOADED_NOTIFICATIONS).clear();
            }
        }
        Ok(())
    }

    fn clean_toast(toast: &mut Toast, umid: &str) -> Result<()> {
        let package_path = AppInfo::GetFromAppUserModelId(&umid.into())
            .and_then(|info| info.Package())
            .and_then(|package| package.InstalledPath())
            .map(|path| PathBuf::from(path.to_os_string()));

        for entry in &mut toast.visual.binding.entries {
            let ToastBindingEntry::Image(image) = entry else {
                continue;
            };

            if image.src.is_empty() {
                continue;
            }

            let uri = Uri::CreateUri(&image.src.clone().into())?;
            let scheme = uri.SchemeName()?.to_string_lossy();
            let uri_path = PathBuf::from(
                Uri::UnescapeComponent(&uri.Path()?)?
                    .to_string_lossy()
                    .trim_start_matches('/'),
            );

            // https://learn.microsoft.com/en-us/windows/uwp/app-resources/uri-schemes
            // https://learn.microsoft.com/en-us/uwp/schemas/tiles/toastschema/element-image
            match scheme.as_str() {
                "http" | "https" => {}
                "ms-appx" | "ms-appx-web" => {
                    let path = package_path.clone()?.join(uri_path);
                    if let Some((path, _)) = get_hightest_quality_posible(&path) {
                        log::debug!("  Resolved path: {}", path.display());
                        image.src = convert_file_to_src(&path);
                    } else {
                        log::warn!("  Unable to resolve path {}", path.display());
                    }
                }
                "ms-appdata" => {
                    let parent = if uri_path.starts_with("local") {
                        "LocalState"
                    } else if uri_path.starts_with("roaming") {
                        "LocalCache"
                    } else {
                        continue;
                    };

                    let uri_path = PathBuf::from(
                        Uri::UnescapeComponent(&uri.Path()?)?
                            .to_string_lossy()
                            .to_lowercase()
                            .trim_start_matches('/')
                            .trim_start_matches("local/")
                            .trim_start_matches("roaming/"),
                    );

                    let package_family_name = AppInfo::GetFromAppUserModelId(&umid.into())?
                        .PackageFamilyName()?
                        .to_string_lossy();

                    let path = get_app_handle()
                        .path()
                        .local_data_dir()?
                        .join("Packages")
                        .join(package_family_name)
                        .join(parent)
                        .join(uri_path);

                    log::debug!("  Resolved path: {}", path.display());
                    image.src = convert_file_to_src(&path);
                }
                "file" => {
                    image.src = convert_file_to_src(&uri_path);
                }
                _ => {}
            }
        }
        Ok(())
    }

    // this function an in general all the notification system can still be improved on usability and performance
    fn load_notification(&mut self, u_notification: UserNotification) -> Result<()> {
        {
            trace_lock!(LOADED_NOTIFICATIONS).insert(u_notification.Id()?);
        }
        let notification = u_notification.Notification()?;

        let app_info = match u_notification.AppInfo() {
            Ok(info) => info,
            Err(_) => {
                // will fail if the notification was added by an uninstalled app
                // log::error!("Unable to get app info: {}", error);
                return Ok(());
            }
        };

        let display_info = app_info.DisplayInfo()?;
        let app_umid = app_info.AppUserModelId()?;

        let visuals = notification.Visual()?;
        let text_sequence = visuals
            .GetBinding(&KnownNotificationBindings::ToastGeneric()?)?
            .GetTextElements()?;
        let mut notification_text = Vec::new();
        for text in text_sequence {
            let text = text.Text()?.to_string_lossy().trim().to_string();
            if !text.is_empty() {
                notification_text.push(text);
            }
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
                let mut toast: Toast = quick_xml::de::from_str(&data)?;

                let mut toast_text = Vec::new();
                for entry in &toast.visual.binding.entries {
                    if let ToastBindingEntry::Text(text) = entry {
                        if !text.content.is_empty() {
                            toast_text.push(text.content.clone().replace("\r\n", "\n"));
                        }
                    }
                }

                if notification_text == toast_text {
                    Self::clean_toast(&mut toast, &app_umid.to_string())?;
                    notification_content = Some(toast);
                    break;
                }
            }
        }

        if notification_content.is_none() {
            log::debug!("NONE FOR {notification_text:#?}");
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
