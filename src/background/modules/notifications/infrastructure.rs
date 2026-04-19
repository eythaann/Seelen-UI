use std::{collections::HashMap, sync::Once};

use seelen_core::{
    handlers::SeelenEvent,
    system_state::{AppNotification, NotificationsMode, ToastActionActivationType},
};
use windows::{
    core::GUID,
    Win32::UI::{
        Notifications::{INotificationActivationCallback, NOTIFICATION_USER_INPUT_DATA},
        Shell::{ApplicationActivationManager, IApplicationActivationManager, AO_NONE},
    },
};

use crate::{
    app::emit_to_webviews,
    error::Result,
    modules::notifications::application::{
        get_toast_activator_clsid, NotificationEvent, NotificationManager,
    },
    windows_api::{string_utils::WindowsString, types::AppUserModelId, Com},
};

fn get_notification_manager() -> &'static NotificationManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        NotificationManager::subscribe(|event| match event {
            NotificationEvent::ModeChanged(mode) => {
                emit_to_webviews(SeelenEvent::NotificationsModeChanged, mode);
            }
            _ => {
                emit_to_webviews(
                    SeelenEvent::Notifications,
                    NotificationManager::instance().notifications(),
                );
            }
        });
    });
    NotificationManager::instance()
}

#[tauri::command(async)]
pub fn get_notifications() -> Vec<AppNotification> {
    get_notification_manager().notifications()
}

// https://learn.microsoft.com/en-us/windows/win32/api/notificationactivationcallback/nf-notificationactivationcallback-inotificationactivationcallback-activate
// https://github.com/CommunityToolkit/WindowsCommunityToolkit/blob/c8f76d072df53d3622fb5440d63afb06cb9e7a10/Microsoft.Toolkit.Uwp.Notifications/Toasts/Compat/Desktop/DesktopNotificationManagerCompat.cs#L19
#[tauri::command(async)]
pub fn activate_notification(
    id: u32,
    umid: String,
    args: String,
    activation_type: ToastActionActivationType,
    input_data: HashMap<String, String>,
) -> Result<()> {
    log::trace!(
        "Activating notification \'{umid}\' (id={id}, type={activation_type:?}) with args \'{args}\'"
    );

    let app_umid = AppUserModelId::from(umid);

    match activation_type {
        // Built-in Windows system actions (e.g. `dismiss`, `snooze`). Windows
        // itself handles these natively on the toast; we just mirror the
        // side-effect locally by removing the notification from our list,
        // which is what Action Center does on any action click.
        ToastActionActivationType::System => {
            log::trace!("System action '{args}': removing notification locally");
            let _ = get_notification_manager().remove_notification(id);
            return Ok(());
        }
        // Protocol activation: `args` is a URI. Delegate to the shell.
        ToastActionActivationType::Protocol => {
            crate::exposed::open_file_inner(args)?;
            let _ = get_notification_manager().remove_notification(id);
            return Ok(());
        }
        ToastActionActivationType::Foreground
        | ToastActionActivationType::Background
        | ToastActionActivationType::Unknown => {}
    }

    match get_toast_activator_clsid(&app_umid) {
        Ok(activator_clsid) => {
            log::trace!(
                "Activating with INotificationActivationCallback, clsid: {activator_clsid}"
            );

            let mut data = Vec::new();
            for (key, value) in input_data {
                let key = WindowsString::from_str(&key);
                let value = WindowsString::from_str(&value);
                data.push(NOTIFICATION_USER_INPUT_DATA {
                    Key: key.as_pcwstr(),
                    Value: value.as_pcwstr(),
                });
            }

            let result = Com::run_with_context(|| unsafe {
                let clsid_activator = GUID::try_from(activator_clsid.as_str())?;
                let toast_activator: INotificationActivationCallback =
                    Com::create_instance(&clsid_activator)?;

                let app_umid_w = WindowsString::from_str(&app_umid);
                let args_w = WindowsString::from_str(&args);
                toast_activator.Activate(app_umid_w.as_pcwstr(), args_w.as_pcwstr(), &data)?;
                Ok(())
            });

            match result {
                Ok(()) => {
                    let _ = get_notification_manager().remove_notification(id);
                    return Ok(());
                }
                Err(error) => {
                    log::warn!(
                        "Toast activator invocation failed, falling back to launch: {error}"
                    );
                }
            }
        }
        Err(error) => {
            log::trace!("No toast activator CLSID available ({error}); falling back to launch");
        }
    }

    // Fallback for APPX/MSIX apps without a ToastActivatorCLSID. We only relaunch
    // the app by AUMID — best effort.
    //
    // Known limitation: apps like Telegram Desktop (Store) don't declare a
    // ToastActivatorCLSID and instead register `ToastNotification.Activated`
    // handlers in-process when each toast is created. The real Windows Action
    // Center routes clicks back to those handlers through a private RPC channel
    // in `WpnUserService` (`INotificationPlatform::ActivateNotification`) which
    // isn't available to third parties. Without that channel the toast `args`
    // (e.g. `action=open&peer=...&msg=...`) never reach the app, so the click
    // opens the window but not the correct chat. Replicating Action Center's
    // behavior would require reverse-engineered, unstable COM interfaces, so we
    // accept this limitation — opening the app is still better than nothing.
    if app_umid.is_appx() {
        log::trace!("Activating with IApplicationActivationManager");
        let result = Com::run_with_context(|| unsafe {
            let activator: IApplicationActivationManager =
                Com::create_instance(&ApplicationActivationManager)?;
            let aumid = WindowsString::from_str(&app_umid);
            let args_w = WindowsString::from_str(&args);
            activator.ActivateApplication(aumid.as_pcwstr(), args_w.as_pcwstr(), AO_NONE)?;
            Ok(())
        });

        match result {
            Ok(()) => {
                let _ = get_notification_manager().remove_notification(id);
                return Ok(());
            }
            Err(error) => {
                log::warn!("IApplicationActivationManager invocation failed: {error}");
            }
        }
    }

    // Last resort: mirror Action Center behavior by removing the toast anyway.
    // let _ = get_notification_manager().remove_notification(id);
    Err("Failed to activate notification".into())
}

#[tauri::command(async)]
pub fn notifications_close(id: u32) -> Result<()> {
    get_notification_manager().remove_notification(id)
}

#[tauri::command(async)]
pub fn notifications_close_all() -> Result<()> {
    get_notification_manager().clear_notifications()
}

#[tauri::command(async)]
pub fn get_notifications_mode() -> Result<NotificationsMode> {
    get_notification_manager().get_notifications_mode()
}

#[tauri::command(async)]
pub fn set_notifications_mode(mode: NotificationsMode) -> Result<()> {
    NotificationManager::set_mode(mode)
}
