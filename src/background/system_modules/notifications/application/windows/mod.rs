mod activation;
mod content;

use std::{
    collections::{HashMap, HashSet},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};

use seelen_core::system_state::{AppNotification, NotificationsMode, ToastActionActivationType};
use windows::{
    Foundation::TypedEventHandler,
    UI::Notifications::{
        Management::{UserNotificationListener, UserNotificationListenerAccessStatus},
        NotificationKinds, ToastNotificationManager, ToastNotificationManagerForUser,
        ToastNotificationMode, UserNotificationChangedEventArgs, UserNotificationChangedKind,
    },
};

use crate::{
    error::{Result, ResultLogExt},
    resources::icon_extractor::request_icon_extraction_from_umid,
    system_modules::notifications::domain::{NotificationsSysEvent, NotificationsSystem},
    utils::spawn_named_thread,
    windows_api::{
        WindowsApi,
        types::DateTimeExt,
        undocumented::{
            QUIET_HOURS_PROFILE_ALARMS_ONLY, QUIET_HOURS_PROFILE_PRIORITY_ONLY,
            QUIET_HOURS_PROFILE_UNRESTRICTED,
        },
    },
};

/// Interval of the polling used when the change notifications are not available.
const POLL_INTERVAL: Duration = Duration::from_secs(1);

type EventCallback = Arc<dyn Fn(NotificationsSysEvent) + Send + Sync>;

pub struct WindowsNotifications {
    manager: Option<ToastNotificationManagerForUser>,
    listener: Option<UserNotificationListener>,
    changed_token: Option<i64>,
    mode_changed_token: Option<i64>,
    /// Stops the polling thread, if any.
    polling: Option<Arc<AtomicBool>>,
}

// SAFETY: the WinRT objects are only used to call their methods and remove the event handlers
// (as the previous manager did, also with `unsafe impl Send/Sync`).
unsafe impl Send for WindowsNotifications {}
unsafe impl Sync for WindowsNotifications {}

impl NotificationsSystem for WindowsNotifications {
    fn create() -> Self {
        Self {
            manager: None,
            listener: None,
            changed_token: None,
            mode_changed_token: None,
            polling: None,
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(NotificationsSysEvent) + Send + Sync + 'static,
    {
        let on_event: EventCallback = Arc::new(on_event);
        let manager = ToastNotificationManager::GetDefault()?;
        let listener = UserNotificationListener::Current()?;

        let access = listener.RequestAccessAsync()?.join()?;
        if access != UserNotificationListenerAccessStatus::Allowed {
            return Err("Failed to get notification access".into());
        }

        // TODO: this only works on MSIX/APPX/UWP builds so idk how to make it work on win32 apps
        let cb = on_event.clone();
        match listener.NotificationChanged(&TypedEventHandler::new(
            move |_listener: windows_core::Ref<UserNotificationListener>,
                  args: windows_core::Ref<UserNotificationChangedEventArgs>| {
                let Some(args) = args.as_ref() else {
                    return Ok(());
                };
                let id = args.UserNotificationId()?;
                match args.ChangeKind()? {
                    UserNotificationChangedKind::Added => cb(NotificationsSysEvent::Added { id }),
                    UserNotificationChangedKind::Removed => {
                        cb(NotificationsSysEvent::Removed { id })
                    }
                    _ => {}
                }
                Ok(())
            },
        )) {
            Ok(token) => self.changed_token = Some(token),
            Err(error) => {
                log::warn!("Failed to register winrt notification change handler: {error}");
                // instead we poll the notifications
                self.polling = Some(spawn_polling(on_event.clone())?);
            }
        }

        // `NotificationModeChanged` is only implemented on Windows 11. On Windows 10 the
        // call fails with E_NOINTERFACE, so the mode API is treated as optional.
        match manager.NotificationModeChanged(&TypedEventHandler::new(
            move |sender: windows_core::Ref<ToastNotificationManagerForUser>,
                  _args: windows_core::Ref<windows_core::IInspectable>| {
                if let Some(sender) = sender.as_ref() {
                    let mode = mode_from_toast_mode(sender.NotificationMode()?);
                    on_event(NotificationsSysEvent::ModeChanged { mode });
                }
                Ok(())
            },
        )) {
            Ok(token) => self.mode_changed_token = Some(token),
            Err(error) => {
                log::warn!(
                    "Notification mode is not supported on this OS, the do not disturb \
                     state will not be tracked in real time: {error}"
                );
            }
        }

        self.manager = Some(manager);
        self.listener = Some(listener);
        Ok(())
    }

    fn notification_ids(&self) -> Result<Vec<u32>> {
        toast_ids(self.listener()?)
    }

    // this function an in general all the notification system can still be improved on usability and performance
    fn notification(&self, id: u32) -> Result<Option<AppNotification>> {
        let u_notification = self.listener()?.GetNotification(id)?;
        let notification = u_notification.Notification()?;

        let app_info = match u_notification.AppInfo() {
            Ok(info) => info,
            // will fail if the notification was added by an uninstalled app
            Err(_) => return Ok(None),
        };

        let display_info = app_info.DisplayInfo()?;
        let app_umid = app_info.AppUserModelId()?.to_string();

        let manager = self
            .manager
            .as_ref()
            .ok_or("Notifications not initialized")?;
        let toast_notifications = manager
            .History()?
            .GetHistoryWithId(&app_umid.clone().into())?;

        log::trace!("Loading notification, ID: {id}, AppID: {app_umid}");
        let content = content::toast_content(&notification, toast_notifications, &app_umid)?;

        // pre-extraction to avoid flickering on the ui
        request_icon_extraction_from_umid(&app_umid);

        Ok(Some(AppNotification {
            id,
            app_umid,
            app_name: display_info.DisplayName()?.to_string(),
            app_description: display_info.Description()?.to_string(),
            date: u_notification.CreationTime()?.to_unix_ms(),
            content,
        }))
    }

    fn remove(&self, id: u32) -> Result<()> {
        self.listener()?.RemoveNotification(id)?;
        Ok(())
    }

    fn activate(
        &self,
        umid: &str,
        args: &str,
        activation_type: ToastActionActivationType,
        input_data: HashMap<String, String>,
    ) -> Result<()> {
        activation::activate(umid, args, activation_type, input_data)
    }

    fn mode(&self) -> Result<NotificationsMode> {
        // Only available on Windows 11. This backs the do not disturb toggle, so degrade to
        // `All` on systems without the API instead of failing the command: the frontend
        // awaits it at module top level, and a rejection there prevents the widget from
        // mounting, which surfaces to the user as "notifications never show up".
        if let Some(Ok(mode)) = self.manager.as_ref().map(|m| m.NotificationMode()) {
            return Ok(mode_from_toast_mode(mode));
        }

        // Windows 10 fallback: read the focus assist profile directly
        match WindowsApi::get_quiet_hours_profile() {
            Ok(profile) => Ok(mode_from_quiet_hours_profile(&profile)),
            Err(error) => {
                log::debug!("Failed to read the quiet hours profile: {error}");
                Ok(NotificationsMode::All)
            }
        }
    }

    /// Changes the "Do not disturb" (Windows 11) / "Focus assist" (Windows 10) state.
    fn set_mode(&self, mode: NotificationsMode) -> Result<bool> {
        let profile = quiet_hours_profile_from_mode(mode);
        log::debug!("Setting notifications mode to {mode:?} ({profile})");

        // there is no official api to do this, so if the undocumented one fails
        // we let the user change it by hand
        if let Err(error) = WindowsApi::set_quiet_hours_profile(profile) {
            log::warn!("Failed to set the quiet hours profile, opening system settings: {error}");
            std::process::Command::new("explorer.exe")
                .arg("ms-settings:notifications")
                .spawn()?;
            // nothing changed (yet), so there is nothing to notify
            return Ok(true);
        }

        // without `NotificationModeChanged` (Windows 10) the change is never notified
        Ok(self.mode_changed_token.is_some())
    }
}

impl WindowsNotifications {
    fn listener(&self) -> Result<&UserNotificationListener> {
        Ok(self
            .listener
            .as_ref()
            .ok_or("Notifications not initialized")?)
    }
}

impl Drop for WindowsNotifications {
    fn drop(&mut self) {
        if let (Some(listener), Some(token)) = (&self.listener, self.changed_token.take()) {
            listener.RemoveNotificationChanged(token).log_error();
        }
        if let (Some(manager), Some(token)) = (&self.manager, self.mode_changed_token.take()) {
            manager.RemoveNotificationModeChanged(token).log_error();
        }
        if let Some(running) = self.polling.take() {
            running.store(false, Ordering::Release);
        }
    }
}

/// Polls the notifications list, reporting the added/removed ones.
/// Returns the flag that keeps the thread running.
fn spawn_polling(on_event: EventCallback) -> Result<Arc<AtomicBool>> {
    let running = Arc::new(AtomicBool::new(true));
    let mut known: HashSet<u32> = toast_ids(&UserNotificationListener::Current()?)?
        .into_iter()
        .collect();

    let flag = running.clone();
    spawn_named_thread("Notification Manager", move || -> Result<()> {
        while flag.load(Ordering::Acquire) {
            std::thread::sleep(POLL_INTERVAL);
            // Pause when session is not interactive to reduce CPU usage
            if !crate::platform::session::is_interactive() {
                continue;
            }
            let Some(current) = UserNotificationListener::Current()
                .map_err(Into::into)
                .and_then(|listener| toast_ids(&listener))
                .ok_logged()
            else {
                continue;
            };

            for id in &current {
                if !known.contains(id) {
                    on_event(NotificationsSysEvent::Added { id: *id });
                }
            }
            for id in &known {
                if !current.contains(id) {
                    on_event(NotificationsSysEvent::Removed { id: *id });
                }
            }
            known = current.into_iter().collect();
        }
        Ok(())
    });
    Ok(running)
}

fn toast_ids(listener: &UserNotificationListener) -> Result<Vec<u32>> {
    let mut ids = Vec::new();
    for notification in listener
        .GetNotificationsAsync(NotificationKinds::Toast)?
        .join()?
    {
        ids.push(notification.Id()?);
    }
    Ok(ids)
}

fn mode_from_toast_mode(mode: ToastNotificationMode) -> NotificationsMode {
    match mode {
        ToastNotificationMode::AlarmsOnly => NotificationsMode::AlarmsOnly,
        ToastNotificationMode::PriorityOnly => NotificationsMode::PriorityOnly,
        _ => NotificationsMode::All,
    }
}

fn mode_from_quiet_hours_profile(profile: &str) -> NotificationsMode {
    match profile {
        QUIET_HOURS_PROFILE_ALARMS_ONLY => NotificationsMode::AlarmsOnly,
        QUIET_HOURS_PROFILE_PRIORITY_ONLY => NotificationsMode::PriorityOnly,
        _ => NotificationsMode::All,
    }
}

fn quiet_hours_profile_from_mode(mode: NotificationsMode) -> &'static str {
    match mode {
        NotificationsMode::All => QUIET_HOURS_PROFILE_UNRESTRICTED,
        NotificationsMode::PriorityOnly => QUIET_HOURS_PROFILE_PRIORITY_ONLY,
        NotificationsMode::AlarmsOnly => QUIET_HOURS_PROFILE_ALARMS_ONLY,
    }
}
