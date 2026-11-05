//! Rebuilds the toast XML of a notification, as the listener only exposes its plain text.

use std::path::PathBuf;

use seelen_core::system_state::{Toast, ToastBindingChild, ToastText};
use tauri::Manager;
use windows::{
    ApplicationModel::AppInfo,
    Foundation::Uri,
    UI::Notifications::{KnownNotificationBindings, Notification, ToastNotification},
};
use windows_collections::IVectorView;

use crate::{
    app::get_app_handle, error::Result, utils::convert_file_to_src,
    windows_api::msix::get_hightest_quality_posible_for_uwp_image,
};

/// The toast of the app history matching the notification text, or one generated from the
/// plain text if none matches.
pub fn toast_content(
    notification: &Notification,
    toast_notifications: IVectorView<ToastNotification>,
    umid: &str,
) -> Result<Toast> {
    let visuals = notification.Visual()?;
    let binding = visuals.GetBinding(&KnownNotificationBindings::ToastGeneric()?)?;
    let text_sequence = binding.GetTextElements()?;

    let mut notification_text = String::new();
    for text in &text_sequence {
        let text = text.Text()?.to_string_lossy().trim().replace("\r\n", "\n");
        notification_text.push_str(&text);
    }

    for toast_notification in toast_notifications {
        // this can be null when the notification count is bigger than the max allowed by default 20
        if let Ok(content) = toast_notification.Content() {
            let toast_xml = content.GetXml()?.to_string();
            let mut toast: Toast = quick_xml::de::from_str(&toast_xml)?;
            if notification_text == text_for_comparison(&toast) {
                clean_toast(&mut toast, umid)?;
                return Ok(toast);
            }
        }
    }

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
    clean_toast(&mut toast, umid)?;
    Ok(toast)
}

/// Resolves the image sources of the toast into files the webviews can load.
fn clean_toast(toast: &mut Toast, umid: &str) -> Result<()> {
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

fn text_for_comparison(toast: &Toast) -> String {
    let mut text = String::new();
    // only the first two top-level text entries are exposed by UserNotification::Visual,
    // so only those are relevant for comparison
    for entry in toast
        .visual
        .binding
        .children
        .iter()
        .filter_map(|entry| match entry {
            ToastBindingChild::Text(entry) => Some(entry),
            _ => None,
        })
        .take(2)
    {
        text.push_str(entry.content.replace("\r\n", "\n").trim());
    }
    text
}
