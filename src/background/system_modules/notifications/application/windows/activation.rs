//! Activation of the notifications on their apps, replicating the Action Center as possible.
//!
//! https://learn.microsoft.com/en-us/windows/win32/api/notificationactivationcallback/nf-notificationactivationcallback-inotificationactivationcallback-activate
//! https://github.com/CommunityToolkit/WindowsCommunityToolkit/blob/c8f76d072df53d3622fb5440d63afb06cb9e7a10/Microsoft.Toolkit.Uwp.Notifications/Toasts/Compat/Desktop/DesktopNotificationManagerCompat.cs#L19

use std::collections::HashMap;

use seelen_core::system_state::ToastActionActivationType;
use windows::{
    ApplicationModel::AppInfo,
    Win32::UI::{
        Notifications::{INotificationActivationCallback, NOTIFICATION_USER_INPUT_DATA},
        Shell::{AO_NONE, ApplicationActivationManager, IApplicationActivationManager},
        WindowsAndMessaging::{ASFW_ANY, AllowSetForegroundWindow},
    },
    core::GUID,
};
use winreg::{
    RegKey,
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE},
};

use crate::{
    error::Result,
    system_modules::start::StartMenuManager,
    windows_api::msix::PackageManifest,
    windows_api::{Com, WindowsApi, string_utils::WindowsString, types::AppUserModelId},
};

pub fn activate(
    umid: &str,
    args: &str,
    activation_type: ToastActionActivationType,
    input_data: HashMap<String, String>,
) -> Result<()> {
    match activation_type {
        // Built-in Windows system actions (e.g. `dismiss`, `snooze`). Windows itself handles
        // these natively on the toast, there is nothing to run on the app.
        ToastActionActivationType::System => {
            log::trace!("System action '{args}': nothing to activate");
            return Ok(());
        }
        // Protocol activation: `args` is a URI. Delegate to the shell.
        ToastActionActivationType::Protocol => {
            return WindowsApi::execute(args.to_owned(), None, None, false);
        }
        ToastActionActivationType::Foreground
        | ToastActionActivationType::Background
        | ToastActionActivationType::Unknown => {}
    }

    let app_umid = AppUserModelId::from(umid.to_owned());
    match toast_activator_clsid(&app_umid) {
        Ok(activator_clsid) => {
            log::trace!(
                "Activating with INotificationActivationCallback, clsid: {activator_clsid}"
            );
            match activate_with_callback(&activator_clsid, &app_umid, args, input_data) {
                Ok(()) => return Ok(()),
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
            let args_w = WindowsString::from_str(args);
            activator.ActivateApplication(aumid.as_pcwstr(), args_w.as_pcwstr(), AO_NONE)?;
            Ok(())
        });
        match result {
            Ok(()) => return Ok(()),
            Err(error) => log::warn!("IApplicationActivationManager invocation failed: {error}"),
        }
    }

    Err("Failed to activate notification".into())
}

fn activate_with_callback(
    activator_clsid: &str,
    app_umid: &AppUserModelId,
    args: &str,
    input_data: HashMap<String, String>,
) -> Result<()> {
    let input_data: Vec<(WindowsString, WindowsString)> = input_data
        .iter()
        .map(|(key, value)| (WindowsString::from_str(key), WindowsString::from_str(value)))
        .collect();
    let data: Vec<NOTIFICATION_USER_INPUT_DATA> = input_data
        .iter()
        .map(|(key, value)| NOTIFICATION_USER_INPUT_DATA {
            Key: key.as_pcwstr(),
            Value: value.as_pcwstr(),
        })
        .collect();

    Com::run_with_context(|| unsafe {
        let clsid_activator = GUID::try_from(activator_clsid)?;
        let toast_activator: INotificationActivationCallback =
            Com::create_instance(&clsid_activator)?;

        let app_umid_w = WindowsString::from_str(app_umid);
        let args_w = WindowsString::from_str(args);

        // Real Action Center is a trusted broker exempt from the foreground-lock
        // restriction, so activated apps can always raise their window. We're a
        // regular process, so without this Windows may silently deny the
        // activated app's SetForegroundWindow call and the click appears to do
        // nothing (no error, no window).
        let _ = AllowSetForegroundWindow(ASFW_ANY);

        // Known limitation: Firefox (and Gecko forks) always bring their window
        // to the foreground here, but never open the tab/action tied to the
        // toast. This `Activate()` call genuinely reaches Firefox's COM object
        // (no error, no missing CLSID) — the failure is entirely on Firefox's
        // side, not ours. Firefox's native `ToastNotificationHandler::OnActivate`
        // doesn't parse `args` itself for a plain click — it just calls a JS
        // callback (`mAlertCallbacks->OnAlertClick()`) that was stashed on that
        // specific handler object back when the notification was first shown by
        // the page (e.g. a web push notification). That JS callback isn't tied to
        // anything reconstructible from `args`/`windowsTag`, and its lifetime is
        // shorter than the toast's entry in Windows' own notification history, so
        // by the time we replay an older notification from that history the
        // callback is long gone and only the native (foreground-only) part of
        // `OnActivate` still does anything. Real Action Center reaches Firefox
        // through a private broker (`WpnUserService`) that isn't available to
        // third parties, so there's no way to fully replicate its behavior here —
        // same class of limitation as the Telegram case above.
        toast_activator.Activate(app_umid_w.as_pcwstr(), args_w.as_pcwstr(), &data)?;
        Ok(())
    })
}

fn toast_activator_clsid(app_umid: &AppUserModelId) -> Result<String> {
    match app_umid {
        AppUserModelId::PropertyStore(umid) => {
            // Win32 apps using a dedicated toast AUMID (Firefox's `FirefoxToast-*`,
            // FanControl, DSX...) have no start menu shortcut carrying that AUMID, so
            // the shortcut lookup below can't find them. Their CLSID lives in the
            // registry instead.
            if let Some(clsid) = registered_custom_activator(umid) {
                return Ok(clsid);
            }
            if let Some(item) = StartMenuManager::instance().get_by_file_umid(umid) {
                return WindowsApi::get_file_toast_activator(&item.path);
            }
        }
        AppUserModelId::Appx(umid) => {
            return appx_toast_activator_clsid(umid);
        }
    };
    Err(format!("Unable to get toast activator clsid for: {app_umid:?}").into())
}

/// Reads `CustomActivator` from `SOFTWARE\Classes\AppUserModelId\<aumid>`, where
/// Win32 apps register the CLSID of their `INotificationActivationCallback` when they
/// use a dedicated toast AUMID. HKCU takes precedence over HKLM, matching how the
/// shell resolves `Classes`.
fn registered_custom_activator(umid: &str) -> Option<String> {
    let roots = [
        (HKEY_CURRENT_USER, r"SOFTWARE\Classes\AppUserModelId"),
        (HKEY_LOCAL_MACHINE, r"SOFTWARE\Classes\AppUserModelId"),
    ];

    for (root, subkey) in roots {
        let Ok(key) = RegKey::predef(root).open_subkey(subkey) else {
            continue;
        };
        let Ok(key) = key.open_subkey(umid) else {
            continue;
        };
        let Ok(clsid) = key.get_value::<String, _>("CustomActivator") else {
            continue;
        };
        let clsid = clsid.trim().trim_start_matches('{').trim_end_matches('}');
        if !clsid.is_empty() {
            return Some(clsid.to_owned());
        }
    }
    None
}

/// Reads the package's `AppxManifest.xml` (via the shared `PackageManifest`
/// deserializer) and returns the `ToastActivatorCLSID` declared by the
/// `windows.toastNotificationActivation` extension for the given AppId.
/// This is what enables MSIX/APPX apps (WhatsApp, Telegram, Teams, etc.)
/// to handle toast action activation.
fn appx_toast_activator_clsid(app_umid: &str) -> Result<String> {
    let (_family, app_id) = app_umid
        .split_once('!')
        .ok_or("Invalid Appx AUMID (missing '!')")?;

    let package = AppInfo::GetFromAppUserModelId(&app_umid.into())?.Package()?;
    let manifest = PackageManifest::try_read_for(&package)?;

    let app = manifest
        .get_app(app_id)
        .ok_or_else(|| format!("AppId {app_id} not found in manifest"))?;

    let extensions = app
        .extensions
        .as_ref()
        .ok_or("Application has no <Extensions> block")?;

    for extension in &extensions.extension {
        if extension.category != "windows.toastNotificationActivation" {
            continue;
        }
        if let Some(activation) = &extension.toast_notification_activation {
            let clsid = activation
                .toast_activator_clsid
                .trim()
                .trim_start_matches('{')
                .trim_end_matches('}')
                .to_string();
            return Ok(clsid);
        }
    }

    Err(format!("ToastActivatorCLSID not found in manifest for {app_umid}").into())
}
