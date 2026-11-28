use std::{
    collections::HashSet,
    hash::{DefaultHasher, Hash, Hasher},
    path::PathBuf,
};

use seelen_core::system_state::{SysTrayIcon, SystrayIconAction, WindowHandle, WindowIdentity};
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
    error::ResultLogExt,
    platform::window::Window,
    system_modules::system_tray::application::windows::{TrayState, util::Util},
    utils::constants::SEELEN_COMMON,
    windows_api::{WindowsApi, icon::convert_hicon_to_rgba_image},
};
use slu_ipc::messages::{IconEventData, TrayIconPatch, Win32TrayEvent};

/// Events that can be emitted by `Systray`.
#[derive(Clone, Debug, Eq, PartialEq)]
#[allow(clippy::enum_variant_names)]
pub enum SystrayEvent {
    IconAdd(SysTrayIcon),
    IconUpdate(SysTrayIcon),
    /// session id of the removed icon
    IconRemove(String),
}

impl TrayState {
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
                            && icon.window_handle.map(|w| w.handle) == icon_data.window_handle
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
                .is_none_or(|handle| self.is_owner_alive(id, handle.handle));
            if !alive {
                log::trace!("Tray icon removed, its window no longer exists: {}", id);
                removed.push(id.clone());
            }
            alive
        });
        for id in &removed {
            self.owners.remove(id);
        }
        if removed.is_empty() {
            return false;
        }
        self.release_unused_images();
        true
    }

    /// Image of the icon handle, converted and written to disk only the first time
    /// it is seen for the owner.
    fn icon_image(&self, owner_key: &str, icon_handle: isize) -> Option<SavedIconImage> {
        if let Some(image) = self
            .images
            .get(owner_key, |images| images.get(&icon_handle).cloned())
            .flatten()
        {
            return Some(image);
        }

        let image = save_icon_image(owner_key, icon_handle)?;
        self.images.with_lock(|images| {
            images
                .entry(owner_key.to_string())
                .or_default()
                .insert(icon_handle, image.clone());
        });
        Some(image)
    }

    /// Drops (and deletes from disk) the cached images of owners that no longer have icons.
    fn release_unused_images(&self) {
        let owners: HashSet<String> = self
            .icons
            .map(|(id, icon)| image_owner_key(icon.window_handle, id))
            .into_iter()
            .collect();
        let mut released = Vec::new();
        self.images.retain(|(owner, images)| {
            let keep = owners.contains(owner);
            if !keep {
                released.extend(images.drain().map(|(_, image)| image.path));
            }
            keep
        });
        for path in released {
            let _ = std::fs::remove_file(path);
        }
    }

    /// Whether the icon's window still exists and still belongs to the process
    /// that registered the icon (window handles can be recycled).
    fn is_owner_alive(&self, session_id: &str, handle: WindowHandle) -> bool {
        if !WindowsApi::is_window(HWND(handle as _)) {
            return false;
        }
        match self.owners.get(session_id, |pid| *pid) {
            Some(pid) => window_pid(handle) == Some(pid),
            None => true,
        }
    }

    /// Remembers which process owns the icon's window.
    fn track_owner(&self, session_id: &str, handle: WindowHandle) {
        match window_pid(handle) {
            Some(pid) => {
                self.owners.upsert(session_id.to_string(), pid);
            }
            None => {
                self.owners.remove(session_id);
            }
        }
    }

    /// Handles an event from the `Systray`.
    ///
    /// Returns `None` if the event should be ignored (e.g. if an icon that
    /// doesn't exist was removed).
    pub(super) fn process_event(&self, event: Win32TrayEvent) -> Option<SystrayEvent> {
        match event {
            Win32TrayEvent::IconAdd { data } => self.process_nim_add(&data),
            Win32TrayEvent::IconUpdate { patch } => self.process_nim_update(&patch),
            Win32TrayEvent::IconRemove { data } => self.process_nim_delete(&data),
        }
    }

    /// `NIM_ADD`: registers a new icon. Apps re-add their icons when they receive
    /// `TaskbarCreated`, so an already stored icon is updated instead.
    fn process_nim_add(&self, icon_data: &IconEventData) -> Option<SystrayEvent> {
        if let Some(stored) = self.find_stored_icon(icon_data) {
            return self.update_stored_icon(stored, &TrayIconPatch::from(icon_data.clone()));
        }
        self.store_new_icon(icon_data)
    }

    /// `NIM_MODIFY` / `NIM_SETVERSION`: patches a known icon. An unknown icon was added
    /// before the hook was installed (e.g. shell icons), so it is added as on `NIM_ADD`.
    fn process_nim_update(&self, patch: &TrayIconPatch) -> Option<SystrayEvent> {
        let icon_data = &IconEventData::from(patch.clone());
        if let Some(stored) = self.find_stored_icon(icon_data) {
            return self.update_stored_icon(stored, patch);
        }
        self.store_new_icon(icon_data)
    }

    /// `NIM_DELETE`: removes the icon.
    fn process_nim_delete(&self, icon_data: &IconEventData) -> Option<SystrayEvent> {
        let session_id = self.find_stored_icon(icon_data)?.session_id;
        log::trace!("Tray icon removed: {}", session_id);
        self.icons.remove(&session_id);
        self.owners.remove(&session_id);
        self.release_unused_images();
        Some(SystrayEvent::IconRemove(session_id))
    }

    /// Applies the fields declared by the patch over the stored icon, the others are kept.
    /// Returns `None` if nothing changed.
    fn update_stored_icon(
        &self,
        mut icon: SysTrayIcon,
        patch: &TrayIconPatch,
    ) -> Option<SystrayEvent> {
        let mut icon_data = patch.clone();
        // a cleared tooltip falls back to the app name, as on creation
        if icon_data.tooltip.as_ref().is_some_and(|t| t.is_empty()) {
            let window_handle = icon_data
                .window_handle
                .or(icon.window_handle.map(|w| w.handle));
            icon_data.tooltip = Some(app_name(window_handle).unwrap_or_default());
        }
        let icon_data = &icon_data;

        if !has_change(&icon, icon_data) {
            return None;
        }

        if let Some(uid) = icon_data.uid {
            icon.uid = Some(uid);
        }

        if let Some(window_handle) = icon_data.window_handle
            && icon.window_handle.map(|w| w.handle) != Some(window_handle)
        {
            icon.window_handle = Some(Window::from(window_handle).id());
            self.track_owner(&icon.session_id, window_handle);
        }

        if let Some(guid) = icon_data.guid {
            icon.guid = Some(guid);
        }

        if let Some(tooltip) = &icon_data.tooltip {
            icon.tooltip = tooltip.clone();
        }

        // Avoid re-reading the icon image if it's the same as the existing icon.
        if let Some(icon_handle) = icon_data.icon_handle
            && icon.icon_handle != Some(icon_handle)
            && let Some(image) = self.icon_image(
                &image_owner_key(icon.window_handle, &icon.session_id),
                icon_handle,
            )
        {
            icon.icon_handle = Some(icon_handle);
            icon.icon_image_hash = Some(image.hash);
            icon.icon_path = Some(image.path);
            icon.is_glyph = image.is_glyph;
        }

        if let Some(callback_message) = icon_data.callback_message {
            icon.callback_message = Some(callback_message);
        }

        if let Some(version) = icon_data.version {
            icon.version = Some(version);
        }

        if let Some(is_visible) = icon_data.is_visible {
            icon.is_visible = is_visible;
        }

        self.icons.upsert(icon.session_id.clone(), icon.clone());
        Some(SystrayEvent::IconUpdate(icon))
    }

    /// Stores an icon seen for the first time.
    fn store_new_icon(&self, icon_data: &IconEventData) -> Option<SystrayEvent> {
        // Skip icons that cannot be identified.
        let session_id = session_identity(icon_data)?;
        let executable_path = window_program_path(icon_data).unwrap_or_default();
        log::trace!("Tray icon added: {session_id} ({executable_path:?})");

        let window_handle = icon_data.window_handle.map(|h| Window::from(h).id());
        let image = icon_data.icon_handle.and_then(|handle| {
            self.icon_image(&image_owner_key(window_handle, &session_id), handle)
        });

        let icon = SysTrayIcon {
            session_id,
            // the user settings are applied by the manager
            key: None,
            executable_path,
            uid: icon_data.uid,
            window_handle,
            guid: icon_data.guid,
            // icons without tooltip are named after their app
            tooltip: icon_data
                .tooltip
                .clone()
                .filter(|tooltip| !tooltip.is_empty())
                .or_else(|| app_name(icon_data.window_handle))
                .unwrap_or_default(),
            icon_handle: icon_data.icon_handle,
            icon_image_hash: image.as_ref().map(|image| image.hash.clone()),
            icon_path: image.as_ref().map(|image| image.path.clone()),
            is_glyph: image.is_some_and(|image| image.is_glyph),
            callback_message: icon_data.callback_message,
            version: icon_data.version,
            // an icon added without state is visible
            is_visible: icon_data.is_visible.unwrap_or(true),
            is_promoted: false,
        };

        if let Some(window_handle) = icon.window_handle {
            self.track_owner(&icon.session_id, window_handle.handle);
        }
        self.icons.upsert(icon.session_id.clone(), icon.clone());
        Some(SystrayEvent::IconAdd(icon))
    }

    /// Sends an action to the systray icon.
    pub fn send_action(&self, session_id: &str, action: &SystrayIconAction) -> crate::Result<()> {
        log::trace!("Sending icon action: {:?} to: {}", action, session_id);
        let icon = self
            .icons
            .get(session_id, |v| v.clone())
            .ok_or("Icon not found")?;

        // Early return if we don't have the required fields.
        let window_handle = icon
            .window_handle
            .ok_or("Inoperable icon, missing window handle")?
            .handle;
        let uid = icon.uid.ok_or("Inoperable icon, missing uid")?;
        let callback = icon
            .callback_message
            .ok_or("Inoperable icon, missing callback")?;

        if !self.is_owner_alive(&icon.session_id, window_handle) {
            // The owner died without removing its icon, drop it (Explorer does
            // the same when the mouse passes over a dead icon).
            if self.prune_dead_icons() {
                self.notify_changed();
            }
            return Ok(());
        }

        let is_mouse_click = matches!(
            action,
            SystrayIconAction::LeftClick
                | SystrayIconAction::LeftDoubleClick
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
        window_handle: WindowHandle,
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
            Util::pack_u32(cursor_pos.0 as u16, cursor_pos.1 as u16)
        } else {
            uid
        };

        // The high word for the lparam is the UID for version > 3, and 0 for
        // version <= 3. The low word is always the message.
        let lparam = if version.is_some_and(|version| version > 3) {
            Util::pack_u32(message as u16, uid as u16)
        } else {
            Util::pack_u32(message as u16, 0)
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

/// Icon image written to disk by `save_icon_image`.
#[derive(Clone)]
pub(super) struct SavedIconImage {
    hash: String,
    path: PathBuf,
    is_glyph: bool,
}

/// Returns the id of the process that owns the window, if it exists.
fn window_pid(handle: WindowHandle) -> Option<u32> {
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(HWND(handle as _), Some(&mut pid)) };
    (pid != 0).then_some(pid)
}

/// Display name of the app owning the window, used as tooltip of icons without one.
fn app_name(window_handle: Option<WindowHandle>) -> Option<String> {
    Window::from(window_handle?).app_display_name().ok()
}

/// Executable of the process that owns the icon's window.
fn window_program_path(icon_data: &IconEventData) -> Option<PathBuf> {
    icon_data
        .window_handle
        .and_then(|handle| Window::from(handle).exe_path().ok_logged())
}

/// Key the icon images are cached and named by: the owner window identity, as
/// `HICON` handles are only unique per process lifetime, or the session id for
/// icons without window.
fn image_owner_key(window: Option<WindowIdentity>, session_id: &str) -> String {
    match window {
        Some(window) => window.to_string(),
        None => session_id.to_string(),
    }
}

/// Saves the icon image as png in the temp dir, unless it is already there.
fn save_icon_image(owner_key: &str, icon_handle: isize) -> Option<SavedIconImage> {
    let img = convert_hicon_to_rgba_image(&HICON(icon_handle as _)).ok()?;
    let path = SEELEN_COMMON
        .app_temp_dir()
        .join(format!("{owner_key}__{:x}.png", icon_handle as usize));
    if !path.exists() {
        img.save(&path).ok_logged()?;
    }
    Some(SavedIconImage {
        hash: image_to_hash(&img),
        path,
        is_glyph: is_glyph_image(&img),
    })
}

/// Computes a hash of the icon image.
fn image_to_hash(icon_image: &image::RgbaImage) -> String {
    let mut hasher = DefaultHasher::new();
    icon_image.as_raw().hash(&mut hasher);
    format!("{:x}", hasher.finish())
}

/// Checks if every visible pixel has the same color, so the shape only lives in the
/// alpha channel (theme dependent glyphs drawn for the taskbar) and it can be used as mask.
fn is_glyph_image(icon_image: &image::RgbaImage) -> bool {
    let mut visible = icon_image
        .pixels()
        .filter(|p| p[3] > 0)
        .map(|p| [p[0], p[1], p[2]]);
    let Some(base) = visible.next() else {
        return false;
    };
    visible.all(|rgb| rgb == base)
}

/// Checks if the icon would change from the given patch.
fn has_change(icon: &SysTrayIcon, data: &TrayIconPatch) -> bool {
    data.uid.is_some_and(|uid| icon.uid != Some(uid))
        || data
            .window_handle
            .is_some_and(|handle| icon.window_handle.map(|w| w.handle) != Some(handle))
        || data.guid.is_some_and(|guid| icon.guid != Some(guid))
        || data.tooltip.as_ref().is_some_and(|t| &icon.tooltip != t)
        || data
            .icon_handle
            .is_some_and(|handle| icon.icon_handle != Some(handle))
        || data
            .callback_message
            .is_some_and(|msg| icon.callback_message != Some(msg))
        || data.version.is_some_and(|ver| icon.version != Some(ver))
        || data
            .is_visible
            .is_some_and(|visible| icon.is_visible != visible)
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
