use windows::Win32::{
    Foundation::{LPARAM, POINT, WPARAM},
    UI::WindowsAndMessaging::{
        GetCursorPos, RegisterWindowMessageW, SendNotifyMessageW, HWND_BROADCAST,
    },
};

use windows_core::w;

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
    pub fn refresh_icons() -> crate::Result<()> {
        log::info!("Refreshing icons by sending `TaskbarCreated` message.");
        let msg = unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) };
        if msg == 0 {
            return Err("Failed to register message".into());
        }
        unsafe { SendNotifyMessageW(HWND_BROADCAST, msg, WPARAM::default(), LPARAM::default()) }?;
        Ok(())
    }
}
