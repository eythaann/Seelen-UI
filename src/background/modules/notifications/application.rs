use parking_lot::Mutex as ParkingLotMutex;
use seelen_core::system_state::{
    AppNotification, Toast, ToastActionActivationType, ToastBindingChild, ToastText,
};
use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        LazyLock,
    },
};
use tauri::Manager;
use windows::{
    ApplicationModel::AppInfo,
    Foundation::{TypedEventHandler, Uri},
    UI::Notifications::{
        KnownNotificationBindings,
        Management::{UserNotificationListener, UserNotificationListenerAccessStatus},
        NotificationKinds, ToastNotificationManager, ToastNotificationManagerForUser,
        UserNotification, UserNotificationChangedEventArgs, UserNotificationChangedKind,
    },
};

use crate::{
    app::get_app_handle,
    error::{Result, ResultLogExt},
    event_manager, log_error,
    modules::{
        apps::application::msix::get_hightest_quality_posible_for_uwp_image,
        start::application::StartMenuManager,
    },
    utils::{
        convert_file_to_src, icon_extractor::request_icon_extraction_from_umid,
        lock_free::SyncHashMap, spawn_named_thread,
    },
    windows_api::{event_window::IS_INTERACTIVE_SESSION, types::AppUserModelId, WindowsApi},
};

static LOADED_NOTIFICATIONS: LazyLock<ParkingLotMutex<HashSet<u32>>> =
    LazyLock::new(|| ParkingLotMutex::new(HashSet::new()));
static RELEASED: AtomicBool = AtomicBool::new(true);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationEvent {
    Added(u32),
    Removed(u32),
    Cleared,
}

pub struct NotificationManager {
    notifications: SyncHashMap<u32, AppNotification>,
    manager: ToastNotificationManagerForUser,
    listener: UserNotificationListener,
    event_token: Option<i64>,
}

unsafe impl Send for NotificationManager {}
unsafe impl Sync for NotificationManager {}

event_manager!(NotificationManager, NotificationEvent);

impl NotificationManager {
    fn new() -> Result<Self> {
        Ok(Self {
            notifications: SyncHashMap::new(),
            manager: ToastNotificationManager::GetDefault()?,
            listener: UserNotificationListener::Current()?,
            event_token: None,
        })
    }

    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<NotificationManager> = LazyLock::new(|| {
            let mut manager =
                NotificationManager::new().expect("Failed to create notification manager");
            manager.init().log_error();
            manager
        });
        &MANAGER
    }

    pub fn notifications(&self) -> Vec<AppNotification> {
        let mut notifications = self.notifications.values();
        notifications.sort_by(|a, b| b.date.cmp(&a.date));
        notifications
    }

    pub fn remove_notification(&self, id: u32) -> Result<()> {
        self.listener.RemoveNotification(id)?;
        Self::send(NotificationEvent::Removed(id));
        Ok(())
    }

    pub fn clear_notifications(&self) -> Result<()> {
        let mut umids = HashSet::new();
        self.notifications.for_each(|(_, n)| {
            umids.insert(n.app_umid.clone());
        });
        for umid in umids {
            let history = self.manager.History()?;
            history.ClearWithId(&umid.into())?;
        }
        Self::send(NotificationEvent::Cleared);
        Ok(())
    }

    fn init(&mut self) -> Result<()> {
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
        match self
            .listener
            .NotificationChanged(&TypedEventHandler::new(Self::on_notifications_change))
        {
            Ok(token) => self.event_token = Some(token),
            Err(error) => {
                log::warn!("Failed to register winrt notification change handler: {error}");
                // intead we use a thread
                spawn_named_thread("Notification Manager", || -> Result<()> {
                    RELEASED.store(false, Ordering::SeqCst);
                    while !RELEASED.load(Ordering::Acquire) {
                        std::thread::sleep(std::time::Duration::from_secs(1));
                        // Pause when session is not interactive to reduce CPU usage
                        if !IS_INTERACTIVE_SESSION.load(Ordering::Acquire) {
                            continue;
                        }
                        Self::manual_notifications_loop().log_error();
                    }
                    Ok(())
                });
            }
        }

        let eid = Self::subscribe(|e| log_error!(Self::process_event(e)));
        Self::set_event_handler_priority(&eid, 1);
        Ok(())
    }

    fn on_notifications_change(
        listener: &Option<UserNotificationListener>,
        args: &Option<UserNotificationChangedEventArgs>,
    ) -> windows_core::Result<()> {
        let (Some(_listener), Some(args)) = (listener, args) else {
            return Ok(());
        };

        let change_kind = args.ChangeKind()?;
        let changed_id = args.UserNotificationId()?;

        match change_kind {
            UserNotificationChangedKind::Added => {
                Self::send(NotificationEvent::Added(changed_id));
            }
            UserNotificationChangedKind::Removed => {
                Self::send(NotificationEvent::Removed(changed_id));
            }
            _ => {}
        }
        Ok(())
    }

    fn manual_notifications_loop() -> Result<()> {
        let listener = { UserNotificationListener::Current()? };
        let mut old_toasts = LOADED_NOTIFICATIONS.lock().clone();

        for u_notification in listener
            .GetNotificationsAsync(NotificationKinds::Toast)?
            .get()?
        {
            let id: u32 = u_notification.Id()?;
            if !old_toasts.contains(&id) {
                Self::send(NotificationEvent::Added(id))
            }
            old_toasts.remove(&id);
        }

        for id in old_toasts {
            Self::send(NotificationEvent::Removed(id))
        }
        Ok(())
    }

    fn process_event(event: NotificationEvent) -> Result<()> {
        let manager = NotificationManager::instance();
        match event {
            NotificationEvent::Added(id) => {
                let u_notification = UserNotificationListener::Current()?.GetNotification(id)?;
                manager.load_notification(u_notification)?;
            }
            NotificationEvent::Removed(id) => {
                manager.notifications.remove(&id);
                LOADED_NOTIFICATIONS.lock().remove(&id);
            }
            NotificationEvent::Cleared => {
                manager.notifications.clear();
                LOADED_NOTIFICATIONS.lock().clear();
            }
        }
        Ok(())
    }

    fn clean_toast(toast: &mut Toast, umid: &str) -> Result<()> {
        if toast.launch.is_none() {
            toast.launch = Some(format!("shell:AppsFolder\\{umid}"));
            toast.activation_type = ToastActionActivationType::Protocol;
        }

        let package_path = AppInfo::GetFromAppUserModelId(&umid.into())
            .and_then(|info| info.Package())
            .and_then(|package| package.InstalledPath())
            .map(|path| PathBuf::from(path.to_os_string()));

        for entry in &mut toast.visual.binding.children {
            let ToastBindingChild::Image(image) = entry else {
                continue;
            };

            if image.src.is_empty() {
                continue;
            }

            log::trace!("Resolving image src:e \"{}\"", image.src);
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
                    if let Some((path, _)) = get_hightest_quality_posible_for_uwp_image(&path) {
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
                    image.src = if uri_path.exists() {
                        convert_file_to_src(&uri_path)
                    } else {
                        // telegram desktop from ms store uses file intead of ms-appdata
                        // so this causes the image to be missing, windows doesn't show image as well
                        // so we decide to follow same behavior.
                        "".to_string()
                    }
                }
                _ => {}
            }
        }
        Ok(())
    }

    // this function an in general all the notification system can still be improved on usability and performance
    fn load_notification(&self, u_notification: UserNotification) -> Result<()> {
        let notification_id = u_notification.Id()?;
        LOADED_NOTIFICATIONS.lock().insert(notification_id);
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
        let binding = visuals.GetBinding(&KnownNotificationBindings::ToastGeneric()?)?;
        let text_sequence = binding.GetTextElements()?;

        let mut notification_text = String::new();
        for text in &text_sequence {
            let text = text.Text()?.to_string_lossy().trim().replace("\r\n", "\n");
            notification_text.push_str(&text);
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
                let toast_xml = content.GetXml()?.to_string();
                let mut toast: Toast = quick_xml::de::from_str(&toast_xml)?;
                let toast_text = get_text_from_toast(&toast);

                if notification_text == toast_text {
                    Self::clean_toast(&mut toast, &app_umid.to_string())?;
                    notification_content = Some(toast);
                    break;
                }
            }
        }

        let notification_content = match notification_content {
            Some(content) => content,
            None => {
                log::debug!("Toast content not found, generating one from plain text");
                let mut toast = Toast::default();
                let content = &mut toast.visual.binding.children;
                for text in text_sequence {
                    let text = text
                        .Text()?
                        .to_string_lossy()
                        .replace("\r\n", "\n")
                        .trim()
                        .to_owned();
                    content.push(ToastBindingChild::Text(ToastText {
                        id: None,
                        content: text,
                    }));
                }
                Self::clean_toast(&mut toast, &app_umid.to_string())?;
                toast
            }
        };

        // pre-extraction to avoid flickering on the ui
        request_icon_extraction_from_umid(&app_umid.to_string().into());

        self.notifications.upsert(
            notification_id,
            AppNotification {
                id: notification_id,
                app_umid: app_umid.to_string(),
                app_name: display_info.DisplayName()?.to_string(),
                app_description: display_info.Description()?.to_string(),
                date: u_notification.CreationTime()?.UniversalTime,
                content: notification_content,
            },
        );
        Ok(())
    }
}

impl Drop for NotificationManager {
    fn drop(&mut self) {
        if let Some(token) = self.event_token.take() {
            self.listener.RemoveNotificationChanged(token).log_error();
        }
        RELEASED.store(true, Ordering::Release);
    }
}

fn get_text_from_toast(toast: &Toast) -> String {
    let mut text = String::new();
    for entry in &toast.visual.binding.children {
        // text inside groups are intended to be ignored for the comparison
        if let ToastBindingChild::Text(entry) = entry {
            text.push_str(entry.content.replace("\r\n", "\n").trim());
        }
    }
    text
}

pub fn get_toast_activator_clsid(app_umid: &AppUserModelId) -> Result<String> {
    match app_umid {
        AppUserModelId::PropertyStore(umid) => {
            let guard = StartMenuManager::instance();
            if let Some(item) = guard.get_by_file_umid(umid) {
                let clsid = WindowsApi::get_file_toast_activator(&item.path)?;
                return Ok(clsid);
            }
        }
        _ => {
            // todo search for the clsid in the AppManifest
        }
    };
    Err(format!("Unable to get toast activator clsid for: {app_umid:?}").into())
}
