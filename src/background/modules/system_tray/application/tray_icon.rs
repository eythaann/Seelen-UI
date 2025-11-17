use std::hash::{DefaultHasher, Hash, Hasher};

use seelen_core::system_state::{SysTrayIcon, SysTrayIconId, SystrayIconAction};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::{
        Controls::{WM_MOUSEHOVER, WM_MOUSELEAVE},
        Shell::{NIN_POPUPCLOSE, NIN_POPUPOPEN, NIN_SELECT},
        WindowsAndMessaging::{
            AllowSetForegroundWindow, GetWindowThreadProcessId, SendNotifyMessageW, HICON,
            WM_CONTEXTMENU, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN,
            WM_MBUTTONUP, WM_MOUSEMOVE, WM_RBUTTONDOWN, WM_RBUTTONUP,
        },
    },
};

use crate::{
    modules::system_tray::application::{util::Util, SystemTrayManager},
    utils::{constants::SEELEN_COMMON, icon_extractor::convert_hicon_to_rgba_image},
    windows_api::{window::Window, WindowsApi},
};
use slu_ipc::messages::{IconEventData, Win32TrayEvent};

/// Events that can be emitted by `Systray`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum SystrayEvent {
    IconAdd(SysTrayIcon),
    IconUpdate(SysTrayIcon),
    IconRemove(SysTrayIconId),
}

impl SystemTrayManager {
    /// Returns all icons managed by the `Systray`.
    pub fn icons(&self) -> Vec<SysTrayIcon> {
        self.icons.values()
    }

    /// Returns the icon with the given handle and uid.
    pub fn icon_by_handle(&self, handle: isize, uid: u32) -> Option<SysTrayIcon> {
        self.icons
            .get(&SysTrayIconId::HandleUid(handle, uid), |v| v.clone())
    }

    /// Returns the icon with the given guid.
    pub fn icon_by_guid(&self, guid: uuid::Uuid) -> Option<SysTrayIcon> {
        self.icons.get(&SysTrayIconId::Guid(guid), |v| v.clone())
    }

    fn find_icon(&self, icon_data: &IconEventData) -> Option<SysTrayIcon> {
        icon_data
            .guid
            .and_then(|guid| self.icon_by_guid(guid))
            .or_else(|| match (icon_data.window_handle, icon_data.uid) {
                (Some(handle), Some(uid)) => self.icon_by_handle(handle, uid),
                _ => None,
            })
    }

    /// Handles an event from the `Systray`.
    ///
    /// Returns `None` if the event should be ignored (e.g. if an icon that
    /// doesn't exist was removed).
    pub(super) fn process_event(&self, mut event: Win32TrayEvent) -> Option<SystrayEvent> {
        // set application name if not tooltip is set
        match &mut event {
            Win32TrayEvent::IconAdd { data: icon_data }
            | Win32TrayEvent::IconUpdate { data: icon_data } => {
                if icon_data.tooltip.as_ref().is_none() {
                    if let Some(window_handle) = icon_data.window_handle {
                        let window = Window::from(window_handle);
                        if let Ok(name) = window.app_display_name() {
                            icon_data.tooltip = Some(name);
                        }
                    }
                }
            }
            _ => {}
        }

        match &event {
            Win32TrayEvent::IconAdd { data: icon_data }
            | Win32TrayEvent::IconUpdate { data: icon_data } => {
                let found_icon_id = self.find_icon(icon_data).map(|icon| icon.stable_id.clone());

                let found_icon = match found_icon_id {
                    Some(id) => self.icons.get(&id, |v| v.clone()),
                    None => None,
                };

                // Update the icon in-place if found.
                if let Some(found_icon) = found_icon {
                    // Avoid emitting update events for no-op changes.
                    if !has_change(&found_icon, icon_data) {
                        return None;
                    }

                    let mut to_update = found_icon.clone();

                    if let Some(uid) = icon_data.uid {
                        to_update.uid = Some(uid);
                    }

                    if let Some(window_handle) = icon_data.window_handle {
                        to_update.window_handle = Some(window_handle);
                    }

                    if let Some(guid) = icon_data.guid {
                        to_update.guid = Some(guid);
                    }

                    if let Some(tooltip) = &icon_data.tooltip {
                        to_update.tooltip = tooltip.clone();
                    }

                    if let Some(icon_handle) = icon_data.icon_handle {
                        // Avoid re-reading the icon image if it's the same as the existing icon.
                        if to_update.icon_handle != Some(icon_handle) {
                            if let Ok(img) = convert_hicon_to_rgba_image(&HICON(icon_handle as _)) {
                                to_update.icon_handle = Some(icon_handle);
                                to_update.icon_image_hash = Some(image_to_hash(&img));

                                let path = SEELEN_COMMON
                                    .app_temp_dir()
                                    .join(format!("{}.png", to_update.stable_id));
                                img.save(&path).unwrap();
                                to_update.icon_path = Some(path);
                            }
                        }
                    }

                    if let Some(callback_message) = icon_data.callback_message {
                        to_update.callback_message = Some(callback_message);
                    }

                    if let Some(version) = icon_data.version {
                        to_update.version = Some(version);
                    }

                    to_update.is_visible = icon_data.is_visible;

                    self.icons
                        .upsert(to_update.stable_id.clone(), to_update.clone());
                    Some(SystrayEvent::IconUpdate(to_update.clone()))
                } else {
                    log::info!("Tray icon added: {:?}", icon_data);

                    // Icon doesn't exist yet, so add new icon. Skip icons that
                    // cannot be identified.
                    let stable_id = icon_data.guid.map(SysTrayIconId::Guid).or({
                        match (icon_data.window_handle, icon_data.uid) {
                            (Some(handle), Some(uid)) => {
                                Some(SysTrayIconId::HandleUid(handle, uid))
                            }
                            _ => None,
                        }
                    })?;

                    let mut icon_image_hash = None;
                    let mut icon_path = None;

                    if let Some(icon_handle) = icon_data.icon_handle {
                        if let Ok(img) = convert_hicon_to_rgba_image(&HICON(icon_handle as _)) {
                            icon_image_hash = Some(image_to_hash(&img));
                            let path = SEELEN_COMMON
                                .app_temp_dir()
                                .join(format!("{}.png", stable_id));
                            img.save(&path).unwrap();
                            icon_path = Some(path);
                        }
                    }

                    let icon = SysTrayIcon {
                        stable_id,
                        uid: icon_data.uid,
                        window_handle: icon_data.window_handle,
                        guid: icon_data.guid,
                        tooltip: icon_data.tooltip.clone().unwrap_or_default(),
                        icon_handle: icon_data.icon_handle,
                        icon_path,
                        icon_image_hash,
                        callback_message: icon_data.callback_message,
                        version: icon_data.version,
                        is_visible: icon_data.is_visible,
                    };

                    self.icons.upsert(icon.stable_id.clone(), icon.clone());
                    Some(SystrayEvent::IconAdd(icon))
                }
            }
            Win32TrayEvent::IconRemove { data: icon_data } => {
                log::info!("Tray icon removed: {:?}", icon_data);

                let icon_id = self.find_icon(icon_data).map(|icon| icon.stable_id.clone());

                if let Some(icon_id) = icon_id {
                    self.icons.remove(&icon_id);
                    Some(SystrayEvent::IconRemove(icon_id))
                } else {
                    None
                }
            }
        }
    }

    /// Sends an action to the systray icon.
    pub fn send_action(
        &self,
        icon_id: &SysTrayIconId,
        action: &SystrayIconAction,
    ) -> crate::Result<()> {
        log::info!("Sending icon action: {:?} to: {:?}", action, icon_id);
        let icon = self
            .icons
            .get(icon_id, |v| v.clone())
            .ok_or("Icon not found")?;

        // Early return if we don't have the required fields.
        let window_handle = icon
            .window_handle
            .ok_or("Inoperable icon, missing window handle")?;
        let uid = icon.uid.ok_or("Inoperable icon, missing uid")?;
        let callback = icon
            .callback_message
            .ok_or("Inoperable icon, missing callback")?;

        if !WindowsApi::is_window(HWND(window_handle as _)) {
            return Err("Window handle is invalid".into());
        }

        let is_mouse_click = matches!(
            action,
            SystrayIconAction::LeftClick
                | SystrayIconAction::RightClick
                | SystrayIconAction::MiddleClick
        );

        // For mouse clicks, there is often a menu that appears after the
        // click. Allow the notify icon to gain focus so that the menu can be
        // dismissed after clicking outside.
        if is_mouse_click {
            let mut proc_id = u32::default();
            unsafe { GetWindowThreadProcessId(HWND(window_handle as _), Some(&mut proc_id)) };
            let _ = unsafe { AllowSetForegroundWindow(proc_id) };
        }

        let wm_messages = match action {
            SystrayIconAction::LeftClick => vec![WM_LBUTTONDOWN, WM_LBUTTONUP],
            SystrayIconAction::LeftDoubleClick => vec![WM_LBUTTONDBLCLK, WM_LBUTTONUP],
            SystrayIconAction::RightClick => {
                vec![WM_RBUTTONDOWN, WM_RBUTTONUP]
            }
            SystrayIconAction::MiddleClick => {
                vec![WM_MBUTTONDOWN, WM_MBUTTONUP]
            }
            SystrayIconAction::HoverEnter => vec![WM_MOUSEHOVER],
            SystrayIconAction::HoverLeave => vec![WM_MOUSELEAVE],
            SystrayIconAction::HoverMove => vec![WM_MOUSEMOVE],
        };

        for wm_message in wm_messages {
            Self::notify_icon(window_handle, callback, uid, icon.version, wm_message)?;
        }

        // Additional messages are sent for version 4 and above. Explorer sends
        // these for version 3 as well though, so we do the same.
        // Ref: https://learn.microsoft.com/en-us/windows/win32/api/shellapi/nf-shellapi-shell_notifyicona#remarks
        if icon.version.is_some_and(|version| version >= 3) {
            let v3_message = match action {
                SystrayIconAction::HoverEnter => NIN_POPUPOPEN,
                SystrayIconAction::HoverLeave => NIN_POPUPCLOSE,
                SystrayIconAction::LeftClick => NIN_SELECT,
                SystrayIconAction::RightClick => WM_CONTEXTMENU,
                _ => return Ok(()),
            };

            Self::notify_icon(window_handle, callback, uid, icon.version, v3_message)?;
        }

        Ok(())
    }

    /// Sends a message to the systray icon window.
    fn notify_icon(
        window_handle: isize,
        callback: u32,
        uid: u32,
        version: Option<u32>,
        message: u32,
    ) -> crate::Result<()> {
        // The wparam is the mouse position for version > 3 (with the low and
        // high word being the x and y-coordinates respectively), and the UID
        // for version <= 3.
        let wparam = if version.is_some_and(|version| version > 3) {
            let cursor_pos = Util::cursor_position()?;
            Util::pack_i32(cursor_pos.0 as i16, cursor_pos.1 as i16) as u32
        } else {
            uid
        };

        // The high word for the lparam is the UID for version > 3, and 0 for
        // version <= 3. The low word is always the message.
        let lparam = if version.is_some_and(|version| version > 3) {
            Util::pack_i32(message as i16, uid as i16)
        } else {
            Util::pack_i32(message as i16, 0)
        };

        unsafe {
            SendNotifyMessageW(
                HWND(window_handle as _),
                callback,
                WPARAM(wparam as _),
                LPARAM(lparam as _),
            )
        }?;

        Ok(())
    }
}

/// Computes a hash of the icon image.
fn image_to_hash(icon_image: &image::RgbaImage) -> String {
    let mut hasher = DefaultHasher::new();
    icon_image.as_raw().hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Checks if the icon would change from the given icon data.
fn has_change(icon: &SysTrayIcon, data: &IconEventData) -> bool {
    data.uid.is_some_and(|uid| icon.uid != Some(uid))
        || data
            .window_handle
            .is_some_and(|handle| icon.window_handle != Some(handle))
        || data.guid.is_some_and(|guid| icon.guid != Some(guid))
        || data.tooltip.as_ref().is_some_and(|t| &icon.tooltip != t)
        || data
            .icon_handle
            .is_some_and(|handle| icon.icon_handle != Some(handle))
        || data
            .callback_message
            .is_some_and(|msg| icon.callback_message != Some(msg))
        || data.version.is_some_and(|ver| icon.version != Some(ver))
        || icon.is_visible != data.is_visible
}
