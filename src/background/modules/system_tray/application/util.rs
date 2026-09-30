use windows::Win32::{
    Foundation::{HWND, LPARAM, POINT, WPARAM},
    UI::WindowsAndMessaging::{
        GetCursorPos, HWND_BROADCAST, RegisterWindowMessageW, SendNotifyMessageW,
    },
};

use windows_core::w;

use crate::modules::apps::application::USER_APPS_MANAGER;

pub struct Util;
impl Util {
    /// Packs two 16-bit values into a 32-bit value. This is commonly used
    /// for `WPARAM` and `LPARAM` values.
    ///
    /// Equivalent to the Win32 `MAKELPARAM` and `MAKEWPARAM` macros.
    pub fn pack_i32(low: i16, high: i16) -> i32 {
        low as i32 | ((high as i32) << 16)
    }

    /// Gets the mouse position in screen coordinates.
    pub fn cursor_position() -> crate::Result<(i32, i32)> {
        let mut point = POINT { x: 0, y: 0 };
        unsafe { GetCursorPos(&mut point) }?;
        Ok((point.x, point.y))
    }

    /// Refreshes the icons of the tray.
    ///
    /// Simulates the Windows taskbar being re-created. Some windows fail to
    /// re-add their icons, in which case it's an implementation error on
    /// their side. These windows that fail also do not re-add their icons
    /// to the Windows taskbar when `explorer.exe` is restarted ordinarily.
    pub fn refresh_tray_icons() -> crate::Result<()> {
        log::info!("Refreshing tray icons by sending `TaskbarCreated` message.");
        let msg = unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) };
        if msg == 0 {
            return Err("Failed to register message".into());
        }
        unsafe { SendNotifyMessageW(HWND_BROADCAST, msg, WPARAM::default(), LPARAM::default()) }?;
        Ok(())
    }

    /// Refreshes the taskbar buttons state (overlay icons/badges, progress, etc.).
    ///
    /// Sends `TaskbarButtonCreated` to the already opened windows so they re-apply
    /// their `ITaskbarList3` state, as ManagedShell does on startup. Only meant to be
    /// called once, as some apps re-add their thumbbar buttons on this message.
    ///
    /// Doesn't work for Electron apps: they don't listen to `TaskbarButtonCreated`, and on
    /// `TaskbarCreated` they only restore their thumbbar buttons, so their overlay icon is
    /// only sent again when the app itself changes it.
    pub fn refresh_taskbar_buttons() -> crate::Result<()> {
        log::info!("Refreshing taskbar buttons by sending `TaskbarButtonCreated` message.");
        let msg = unsafe { RegisterWindowMessageW(w!("TaskbarButtonCreated")) };
        if msg == 0 {
            return Err("Failed to register message".into());
        }
        USER_APPS_MANAGER.interactable_windows.for_each(|w| {
            // notify (non-blocking) to avoid hanging on unresponsive windows
            let _ = unsafe {
                SendNotifyMessageW(HWND(w.hwnd as _), msg, WPARAM::default(), LPARAM::default())
            };
        });
        Ok(())
    }
}
