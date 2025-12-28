use std::collections::HashMap;

use seelen_core::{handlers::SeelenEvent, system_state::AppNotification};
use windows::{
    core::GUID,
    Win32::UI::Notifications::{INotificationActivationCallback, NOTIFICATION_USER_INPUT_DATA},
};

use crate::{
    app::emit_to_webviews,
    error::Result,
    log_error,
    modules::notifications::application::{get_toast_activator_clsid, NotificationManager},
    trace_lock,
    windows_api::{string_utils::WindowsString, types::AppUserModelId, Com},
};

use super::application::NOTIFICATION_MANAGER;

pub fn register_notification_events() {
    std::thread::spawn(|| {
        log_error!(trace_lock!(NOTIFICATION_MANAGER).initialize());

        NotificationManager::subscribe(|_event| {
            emit_to_webviews(
                SeelenEvent::Notifications,
                trace_lock!(NOTIFICATION_MANAGER).notifications(),
            );
        });
    });
}

pub fn release_notification_events() {
    log_error!(trace_lock!(NOTIFICATION_MANAGER).release());
}

#[tauri::command(async)]
pub fn get_notifications() -> Vec<AppNotification> {
    trace_lock!(NOTIFICATION_MANAGER).notifications().clone()
}

// https://learn.microsoft.com/en-us/windows/win32/api/notificationactivationcallback/nf-notificationactivationcallback-inotificationactivationcallback-activate
// https://github.com/CommunityToolkit/WindowsCommunityToolkit/blob/c8f76d072df53d3622fb5440d63afb06cb9e7a10/Microsoft.Toolkit.Uwp.Notifications/Toasts/Compat/Desktop/DesktopNotificationManagerCompat.cs#L19
#[tauri::command(async)]
pub fn activate_notification(
    umid: String,
    args: String,
    input_data: HashMap<String, String>,
) -> Result<()> {
    log::trace!("Activating notification \'{umid}\' with args \'{args}\'");

    let app_umid = AppUserModelId::from(umid);

    if let Ok(activator_clsid) = get_toast_activator_clsid(&app_umid) {
        log::trace!("Activating with clsid: {activator_clsid}");

        let mut data = Vec::new();
        for (key, value) in input_data {
            let key = WindowsString::from_str(&key);
            let value = WindowsString::from_str(&value);
            data.push(NOTIFICATION_USER_INPUT_DATA {
                Key: key.as_pcwstr(),
                Value: value.as_pcwstr(),
            });
        }

        return Com::run_with_context(|| unsafe {
            let clsid_activator = GUID::try_from(activator_clsid.as_str())?;
            let toast_activator: INotificationActivationCallback =
                Com::create_instance(&clsid_activator)?;

            let app_umid = WindowsString::from_str(&app_umid);
            let args = WindowsString::from_str(&args);
            toast_activator.Activate(app_umid.as_pcwstr(), args.as_pcwstr(), &data)?;
            Ok(())
        });
    }

    // as fallback in case of not being able to use the toast activator, just open the app.
    log::trace!("Using activation fallback (open app - no arguments)");
    crate::exposed::open_file(format!("shell:AppsFolder\\{app_umid}"))?;
    Ok(())
}

#[tauri::command(async)]
pub fn notifications_close(id: u32) -> Result<()> {
    trace_lock!(NOTIFICATION_MANAGER).remove_notification(id)?;
    Ok(())
}

#[tauri::command(async)]
pub fn notifications_close_all() -> Result<()> {
    trace_lock!(NOTIFICATION_MANAGER).clear_notifications()
}
