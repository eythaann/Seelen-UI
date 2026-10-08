use std::{
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
    time::Duration,
};

use seelen_core::system_state::{SysTrayIcon, SystrayIconAction};
use windows::Win32::{
    Foundation::{HWND, LPARAM, WPARAM},
    UI::{
        Controls::{WM_MOUSEHOVER, WM_MOUSELEAVE},
        Shell::{NIN_POPUPCLOSE, NIN_POPUPOPEN, NIN_SELECT},
        WindowsAndMessaging::{
            AllowSetForegroundWindow, GetWindowThreadProcessId, HICON, SendNotifyMessageW,
            WM_CONTEXTMENU, WM_LBUTTONDBLCLK, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_MBUTTONDOWN,
            WM_MBUTTONUP, WM_MOUSEMOVE, WM_RBUTTONDOWN, WM_RBUTTONUP,
        },
    },
};

use crate::{
    modules::system_tray::application::{
        SystemTrayEvent, SystemTrayManager, find_registry_notify_icon, util::Util,
    },
    utils::{
        constants::SEELEN_COMMON, icon_extractor::convert_hicon_to_rgba_image, spawn_named_thread,
    },
    windows_api::{WindowsApi, window::Window},
};
use slu_ipc::messages::{IconEventData, Win32TrayEvent};

/// How many times (and how often) to look for the registry entry of a new icon,
/// Explorer writes it after the hook has already reported the icon.
const REGISTRY_LOOKUP_ATTEMPTS: u32 = 10;
const REGISTRY_LOOKUP_INTERVAL: Duration = Duration::from_millis(200);

/// Events that can be emitted by `Systray`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum SystrayEvent {
    IconAdd(SysTrayIcon),
    IconUpdate(SysTrayIcon),
    /// registry key of the removed icon
    IconRemove(String),
}

impl SystemTrayManager {
    /// Finds the icon the hook data refers to, by guid or by (window handle + uid),
    /// which is how the shell identifies icons during the session.
    fn find_stored_icon(&self, icon_data: &IconEventData) -> Option<SysTrayIcon> {
        self.icons.with_lock(|icons| {
            icons
                .values()
                .find(|icon| match icon_data.guid {
                    Some(guid) => icon.guid == Some(guid),
                    None => {
                        icon_data.window_handle.is_some()
                            && icon_data.uid.is_some()
                            && icon.window_handle == icon_data.window_handle
                            && icon.uid == icon_data.uid
                    }
                })
                .cloned()
        })
    }

    /// Removes icons whose owner window no longer exists.
    ///
    /// When a process dies without calling `NIM_DELETE` (crash, forced kill,
    /// in-place update), Explorer drops its icons internally, so the hook never
    /// sees an `IconRemove` for them and they would stay here forever.
    ///
    /// Returns `true` if any icon was removed.
    pub fn prune_dead_icons(&self) -> bool {
        let mut removed = Vec::new();
        self.icons.retain(|(id, icon)| {
            let alive = icon
                .window_handle
                .is_none_or(|handle| self.is_owner_alive(id, handle));
            if !alive {
                log::trace!("Tray icon removed, its window no longer exists: {}", id);
                removed.push(id.clone());
            }
            alive
        });
        for id in &removed {
            self.owners.remove(id);
        }
        !removed.is_empty()
    }

    /// Whether the icon's window still exists and still belongs to the process
    /// that registered the icon (window handles can be recycled).
    fn is_owner_alive(&self, registry_key: &str, handle: isize) -> bool {
        if !WindowsApi::is_window(HWND(handle as _)) {
            return false;
        }
        match self.owners.get(registry_key, |pid| *pid) {
            Some(pid) => window_pid(handle) == Some(pid),
            None => true,
        }
    }

    /// Remembers which process owns the icon's window.
    fn track_owner(&self, registry_key: &str, handle: isize) {
        match window_pid(handle) {
            Some(pid) => {
                self.owners.upsert(registry_key.to_string(), pid);
            }
            None => {
                self.owners.remove(registry_key);
            }
        }
    }

    /// Handles an event from the `Systray`.
    ///
    /// Returns `None` if the event should be ignored (e.g. if an icon that
    /// doesn't exist was removed).
    pub(super) fn process_event(&self, mut event: Win32TrayEvent) -> Option<SystrayEvent> {
        // set application name if not tooltip is set
        match &mut event {
            Win32TrayEvent::IconAdd { data: icon_data }
            | Win32TrayEvent::IconUpdate { data: icon_data }
                if icon_data.tooltip.as_ref().is_none() =>
            {
                if let Some(window_handle) = icon_data.window_handle {
                    let window = Window::from(window_handle);
                    if let Ok(name) = window.app_display_name() {
                        icon_data.tooltip = Some(name);
                    }
                }
            }
            _ => {}
        }

        match &event {
            Win32TrayEvent::IconAdd { data: icon_data }
            | Win32TrayEvent::IconUpdate { data: icon_data } => {
                // Update the icon in-place if found.
                if let Some(found_icon) = self.find_stored_icon(icon_data) {
                    // Avoid emitting update events for no-op changes.
                    if !has_change(&found_icon, icon_data) {
                        return None;
                    }

                    let mut to_update = found_icon.clone();

                    if let Some(uid) = icon_data.uid {
                        to_update.uid = Some(uid);
                    }

                    if let Some(window_handle) = icon_data.window_handle
                        && to_update.window_handle != Some(window_handle)
                    {
                        to_update.window_handle = Some(window_handle);
                        self.track_owner(&to_update.registry_key, window_handle);
                    }

                    if let Some(guid) = icon_data.guid {
                        to_update.guid = Some(guid);
                    }

                    if let Some(tooltip) = &icon_data.tooltip {
                        to_update.tooltip = tooltip.clone();
                    }

                    if let Some(icon_handle) = icon_data.icon_handle {
                        // Avoid re-reading the icon image if it's the same as the existing icon.
                        if to_update.icon_handle != Some(icon_handle)
                            && let Ok(img) = convert_hicon_to_rgba_image(&HICON(icon_handle as _))
                        {
                            to_update.icon_handle = Some(icon_handle);
                            to_update.icon_image_hash = Some(image_to_hash(&img));

                            let path = SEELEN_COMMON
                                .app_temp_dir()
                                .join(format!("{}.png", to_update.registry_key));
                            if let Err(err) = img.save(&path) {
                                log::warn!("Failed to save tray icon snapshot to {path:?}: {err}");
                            } else {
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
                        .upsert(to_update.registry_key.clone(), to_update.clone());
                    Some(SystrayEvent::IconUpdate(to_update.clone()))
                } else {
                    // Icon doesn't exist yet, so add new icon. Skip icons that
                    // cannot be identified.
                    let identity = session_identity(icon_data)?;

                    let window_path = icon_data
                        .window_handle
                        .and_then(|handle| Window::from(handle).process().program_path().ok());
                    let Some(entry) = find_registry_notify_icon(
                        window_path.as_deref(),
                        icon_data.guid,
                        icon_data.uid,
                    ) else {
                        self.defer_until_registered(identity, icon_data.clone(), window_path);
                        return None;
                    };
                    self.pending.remove(&identity);

                    let registry_key = entry.key;
                    let executable_path = window_path.unwrap_or(entry.executable_path);
                    log::trace!("Tray icon added: {registry_key} ({executable_path:?})");

                    let mut icon_image_hash = None;
                    let mut icon_path = None;

                    if let Some(icon_handle) = icon_data.icon_handle
                        && let Ok(img) = convert_hicon_to_rgba_image(&HICON(icon_handle as _))
                    {
                        icon_image_hash = Some(image_to_hash(&img));
                        let path = SEELEN_COMMON
                            .app_temp_dir()
                            .join(format!("{}.png", registry_key));
                        if let Err(err) = img.save(&path) {
                            log::warn!("Failed to save tray icon snapshot to {path:?}: {err}");
                        } else {
                            icon_path = Some(path);
                        }
                    }

                    let icon = SysTrayIcon {
                        registry_key,
                        executable_path,
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
                        is_promoted: entry.is_promoted,
                    };

                    if let Some(window_handle) = icon.window_handle {
                        self.track_owner(&icon.registry_key, window_handle);
                    }
                    self.icons.upsert(icon.registry_key.clone(), icon.clone());
                    Some(SystrayEvent::IconAdd(icon))
                }
            }
            Win32TrayEvent::IconRemove { data: icon_data } => {
                if let Some(identity) = session_identity(icon_data) {
                    self.pending.remove(&identity);
                }
                let registry_key = self.find_stored_icon(icon_data)?.registry_key;
                log::trace!("Tray icon removed: {}", registry_key);
                self.icons.remove(&registry_key);
                self.owners.remove(&registry_key);
                Some(SystrayEvent::IconRemove(registry_key))
            }
        }
    }

    /// Holds an icon whose registry entry doesn't exist yet and re-dispatches it
    /// once Explorer writes the entry. Later updates are merged into the held data.
    fn defer_until_registered(
        &self,
        identity: String,
        icon_data: IconEventData,
        window_path: Option<PathBuf>,
    ) {
        let already_waiting = self
            .pending
            .with_lock(|pending| match pending.get_mut(&identity) {
                Some(held) => {
                    merge_icon_data(held, &icon_data);
                    true
                }
                None => {
                    pending.insert(identity.clone(), icon_data.clone());
                    false
                }
            });
        if already_waiting {
            return;
        }

        log::trace!("Tray icon {identity} not registered yet, waiting for its registry entry");
        spawn_named_thread("Tray Registry Lookup", move || {
            for _ in 0..REGISTRY_LOOKUP_ATTEMPTS {
                std::thread::sleep(REGISTRY_LOOKUP_INTERVAL);
                let registered = find_registry_notify_icon(
                    window_path.as_deref(),
                    icon_data.guid,
                    icon_data.uid,
                );
                if registered.is_none() {
                    continue;
                }
                // None means it was removed meanwhile
                if let Some(data) = SystemTrayManager::instance().pending.remove(&identity) {
                    SystemTrayManager::handle_tray_event(Win32TrayEvent::IconAdd { data });
                }
                return;
            }
            SystemTrayManager::instance().pending.remove(&identity);
            log::warn!("Tray icon {identity} ({window_path:?}) ignored, no registry entry found");
        });
    }

    /// Sends an action to the systray icon.
    pub fn send_action(&self, registry_key: &str, action: &SystrayIconAction) -> crate::Result<()> {
        log::trace!("Sending icon action: {:?} to: {}", action, registry_key);
        let icon = self
            .icons
            .get(registry_key, |v| v.clone())
            .ok_or("Icon not found")?;

        // Early return if we don't have the required fields.
        let window_handle = icon
            .window_handle
            .ok_or("Inoperable icon, missing window handle")?;
        let uid = icon.uid.ok_or("Inoperable icon, missing uid")?;
        let callback = icon
            .callback_message
            .ok_or("Inoperable icon, missing callback")?;

        if !self.is_owner_alive(&icon.registry_key, window_handle) {
            // The owner died without removing its icon, drop it (Explorer does
            // the same when the mouse passes over a dead icon).
            if self.prune_dead_icons() {
                SystemTrayManager::send(SystemTrayEvent::Changed);
            }
            return Ok(());
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

/// Returns the id of the process that owns the window, if it exists.
fn window_pid(handle: isize) -> Option<u32> {
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(HWND(handle as _), Some(&mut pid)) };
    (pid != 0).then_some(pid)
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

/// Key identifying an icon during the session, as the shell does: its guid, or
/// its (window handle + uid).
fn session_identity(data: &IconEventData) -> Option<String> {
    match (data.guid, data.window_handle, data.uid) {
        (Some(guid), _, _) => Some(guid.to_string()),
        (None, Some(handle), Some(uid)) => Some(format!("{handle:x}_{uid}")),
        _ => None,
    }
}

/// Applies the fields set in `newer` over `held`.
fn merge_icon_data(held: &mut IconEventData, newer: &IconEventData) {
    held.uid = newer.uid.or(held.uid);
    held.window_handle = newer.window_handle.or(held.window_handle);
    held.guid = newer.guid.or(held.guid);
    held.tooltip = newer.tooltip.clone().or(held.tooltip.take());
    held.icon_handle = newer.icon_handle.or(held.icon_handle);
    held.callback_message = newer.callback_message.or(held.callback_message);
    held.version = newer.version.or(held.version);
    held.is_visible = newer.is_visible;
}
