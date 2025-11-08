use std::{os::raw::c_void, sync::OnceLock, thread::JoinHandle};

use tokio::sync::mpsc;
use windows::{
    core::Error,
    Win32::{
        Foundation::{CloseHandle, HANDLE, HWND, LPARAM, LRESULT, WPARAM},
        System::{
            DataExchange::COPYDATASTRUCT,
            Diagnostics::Debug::ReadProcessMemory,
            Memory::{VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, PAGE_READWRITE},
            Threading::{OpenProcess, PROCESS_ALL_ACCESS},
        },
        UI::{
            Controls::{TBBUTTON, TBSTATE_HIDDEN},
            Shell::{
                NIF_GUID, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
                NIM_SETVERSION, NIS_HIDDEN, NOTIFYICONDATAW_0, NOTIFY_ICON_DATA_FLAGS,
                NOTIFY_ICON_INFOTIP_FLAGS, NOTIFY_ICON_MESSAGE, NOTIFY_ICON_STATE,
            },
            WindowsAndMessaging::{
                DefWindowProcW, GetWindowThreadProcessId, PostMessageW, RegisterWindowMessageW,
                SendMessageW, SendNotifyMessageW, SetTimer, SetWindowPos, HWND_BROADCAST,
                HWND_TOPMOST, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, WM_ACTIVATEAPP, WM_COMMAND,
                WM_COPYDATA, WM_TIMER, WM_USER,
            },
        },
    },
};

use windows_core::w;

use crate::windows_api::window::Window;

use super::util::Util;

/// Global instance of sender for tray events.
///
/// For use with window procedure.
static TRAY_EVENT_TX: OnceLock<mpsc::UnboundedSender<Win32TrayEvent>> = OnceLock::new();

const TB_BUTTONCOUNT: u32 = WM_USER + 24;
const TB_GETBUTTON: u32 = WM_USER + 23;

/// Tray message sent to `Shell_TrayWnd` and intercepted by our spy window.
#[repr(C)]
struct ShellTrayMessage {
    magic_number: i32,
    message_type: u32,
    icon_data: NotifyIconData,
    version: u32,
}

/// Contains the data for a system tray icon.
///
/// When `Shell_NotifyIcon` sends its message to `Shell_Traywnd`, it
/// actually uses a 32-bit handle for the hwnd. This makes it slightly
/// different than the `windows` crate's `NOTIFYICONDATAW` type.
#[repr(C)]
#[derive(Clone, Copy)]
struct NotifyIconData {
    callback_size: u32,
    window_handle: u32,
    uid: u32,
    flags: NOTIFY_ICON_DATA_FLAGS,
    callback_message: u32,
    icon_handle: u32,
    tooltip: [u16; 128],
    state: NOTIFY_ICON_STATE,
    state_mask: NOTIFY_ICON_STATE,
    size_info: [u16; 256],
    anonymous: NOTIFYICONDATAW_0,
    info_title: [u16; 64],
    info_flags: NOTIFY_ICON_INFOTIP_FLAGS,
    guid_item: windows_core::GUID,
    balloon_icon_handle: u32,
}

#[repr(C)]
#[derive(Debug)]
struct NotifyIconIdentifier {
    magic_number: i32,
    message: i32,
    callback_size: i32,
    padding: i32,
    window_handle: u32,
    uid: u32,
    guid_item: windows_core::GUID,
}

/// Response from `ToolbarWindow32` with `TB_GETBUTTON` message.
///
/// Only available on Windows 10, since tray windows are XAML islands in
/// Windows 11.
#[repr(C)]
#[derive(Debug)]
struct TbButtonItem {
    window_handle: isize,
    uid: u32,
    callback_message: u32,
    state: u32,
    version: u32,
    icon_handle: isize,
    icon_demote_timer_id: isize,
    user_pref: u32,
    last_sound_time: u32,
    exe_name: [u16; 260],
    icon_text: [u16; 260],
    num_seconds: u32,
    guid_item: windows_core::GUID,
}

/// Events emitted by the spy window.
#[derive(Clone, Debug, PartialEq, Eq)]
#[allow(clippy::enum_variant_names)]
pub(crate) enum Win32TrayEvent {
    IconAdd(IconEventData),
    IconUpdate(IconEventData),
    IconRemove(IconEventData),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct IconEventData {
    pub uid: Option<u32>,
    pub window_handle: Option<isize>,
    pub guid: Option<uuid::Uuid>,
    pub tooltip: Option<String>,
    pub icon_handle: Option<isize>,
    pub callback_message: Option<u32>,
    pub version: Option<u32>,
    pub is_visible: bool,
}

impl From<NotifyIconData> for IconEventData {
    fn from(icon_data: NotifyIconData) -> Self {
        let icon_handle = if icon_data.icon_handle != 0 && icon_data.flags.0 & NIF_ICON.0 != 0 {
            Some(icon_data.icon_handle as isize)
        } else {
            None
        };

        let guid = if icon_data.guid_item != windows_core::GUID::default()
            && icon_data.flags.0 & NIF_GUID.0 != 0
        {
            Some(uuid::Uuid::from_u128(icon_data.guid_item.to_u128()))
        } else {
            None
        };

        let tooltip = if icon_data.flags.0 & NIF_TIP.0 != 0 {
            let tooltip_len = icon_data.tooltip.iter().position(|&c| c == 0).unwrap_or(0);

            let tooltip_str = String::from_utf16_lossy(&icon_data.tooltip[..tooltip_len])
                .replace('\r', "")
                .to_string();

            (!tooltip_str.is_empty()).then_some(tooltip_str)
        } else if icon_data.window_handle != 0 {
            Window::from(icon_data.window_handle as isize)
                .app_display_name()
                .ok()
        } else {
            None
        };

        let (window_handle, uid) = if icon_data.window_handle != 0 {
            (Some(icon_data.window_handle as isize), Some(icon_data.uid))
        } else {
            (None, None)
        };

        let callback_message = if icon_data.flags.contains(NIF_MESSAGE) {
            Some(icon_data.callback_message)
        } else {
            None
        };

        let version = if unsafe { icon_data.anonymous.uVersion } > 0
            && unsafe { icon_data.anonymous.uVersion } <= 4
        {
            Some(unsafe { icon_data.anonymous.uVersion })
        } else {
            None
        };

        let is_visible = icon_data.state.0 & NIS_HIDDEN.0 == 0;

        IconEventData {
            uid,
            window_handle,
            guid,
            tooltip,
            icon_handle,
            callback_message,
            version,
            is_visible,
        }
    }
}

impl From<TbButtonItem> for IconEventData {
    fn from(tb_item: TbButtonItem) -> Self {
        let icon_handle = if tb_item.icon_handle != 0 {
            Some(tb_item.icon_handle)
        } else {
            None
        };

        let guid = if tb_item.guid_item != windows_core::GUID::default() {
            Some(uuid::Uuid::from_u128(tb_item.guid_item.to_u128()))
        } else {
            None
        };

        let tooltip_len = tb_item.icon_text.iter().position(|&c| c == 0).unwrap_or(0);

        let mut tooltip = String::from_utf16_lossy(&tb_item.icon_text[..tooltip_len])
            .replace('\r', "")
            .to_string();

        if tooltip.is_empty() {
            tooltip = Window::from(tb_item.window_handle)
                .app_display_name()
                .unwrap_or_default();
        }

        IconEventData {
            uid: Some(tb_item.uid),
            window_handle: Some(tb_item.window_handle),
            guid,
            tooltip: Some(tooltip),
            icon_handle,
            callback_message: Some(tb_item.callback_message),
            version: Some(tb_item.version),
            // Determined by the parent `TBBUTTON` struct. This value is set
            // after initialization.
            is_visible: true,
        }
    }
}

/// A window that spies on system tray icon messages and broadcasts events.
#[derive(Debug)]
pub(crate) struct TraySpy {
    window_thread: Option<JoinHandle<()>>,
}

impl TraySpy {
    /// Creates a new `TraySpy` instance.
    pub fn new() -> crate::Result<(Self, mpsc::UnboundedReceiver<Win32TrayEvent>)> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Add the sender for tray events to global state.
        TRAY_EVENT_TX
            .set(event_tx)
            .expect("Tray event sender already set.");

        let spy = Self {
            window_thread: Some(Self::spawn()?),
        };

        Ok((spy, event_rx))
    }

    /// Starts the spy window in its own thread.
    ///
    /// Returns a thread handle for the spy window.
    fn spawn() -> crate::Result<std::thread::JoinHandle<()>> {
        let handle = std::thread::spawn(move || {
            if let Err(err) = Self::run() {
                log::error!("SpyWindow error: {:?}", err);
            }
        });

        Ok(handle)
    }

    /// Creates the spy window and runs its message loop.
    fn run() -> crate::Result<()> {
        let window = Util::create_message_window("Shell_TrayWnd", Some(Self::window_proc))?;

        // TODO: Check whether this can be done in a better way. Check out
        unsafe { SetTimer(Some(HWND(window as _)), 1, 100, None) };

        let event_tx = TRAY_EVENT_TX.get().expect("Tray event sender not set.");

        Self::refresh_icons()?;

        if let Ok(icons) = Self::initial_tray_icons(window) {
            for icon in icons {
                if let Err(err) = event_tx.send(Win32TrayEvent::IconAdd(icon)) {
                    log::error!("Failed to send tray event: {:?}", err);
                }
            }
        } else {
            log::warn!("Failed to retrieve initial tray icons. This is expected on W11.");
        }

        Util::run_message_loop();

        Ok(())
    }

    extern "system" fn window_proc(
        hwnd: HWND,
        msg: u32,
        wparam: WPARAM,
        lparam: LPARAM,
    ) -> LRESULT {
        match msg {
            WM_TIMER => {
                // Regain tray priority.
                let _ = Self::bring_to_top(hwnd);
                LRESULT(0)
            }
            WM_COPYDATA => Self::handle_copy_data(hwnd, msg, wparam, lparam),
            _ => {
                if Self::should_forward_message(msg) {
                    Self::forward_message(hwnd, msg, wparam, lparam)
                } else {
                    unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) }
                }
            }
        }
    }

    fn handle_copy_data(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        // Extract `COPYDATASTRUCT` and return early if invalid.
        let Some(copy_data) = (unsafe { (lparam.0 as *const COPYDATASTRUCT).as_ref() }) else {
            return LRESULT(0);
        };

        match copy_data.dwData {
            1 if !copy_data.lpData.is_null() => {
                let tray_message = unsafe { &*copy_data.lpData.cast::<ShellTrayMessage>() };

                let event_tx = TRAY_EVENT_TX.get().expect("Tray event sender not set.");

                let tray_event = match NOTIFY_ICON_MESSAGE(tray_message.message_type) {
                    NIM_ADD => Some(Win32TrayEvent::IconAdd(tray_message.icon_data.into())),
                    NIM_MODIFY | NIM_SETVERSION => {
                        Some(Win32TrayEvent::IconUpdate(tray_message.icon_data.into()))
                    }
                    NIM_DELETE => Some(Win32TrayEvent::IconRemove(tray_message.icon_data.into())),
                    _ => None,
                };

                log::debug!("Tray event: {:?}", tray_event);

                if let Some(event) = tray_event {
                    event_tx.send(event).expect("Failed to send tray event.");
                }

                Self::forward_message(hwnd, msg, wparam, lparam)
            }
            3 if !copy_data.lpData.is_null() => {
                let icon_identifier = unsafe { &*copy_data.lpData.cast::<NotifyIconIdentifier>() };

                let cursor_pos = match Util::cursor_position() {
                    Ok(pos) => pos,
                    Err(_) => {
                        log::error!("Failed to get cursor position.");
                        return Self::forward_message(hwnd, msg, wparam, lparam);
                    }
                };

                match icon_identifier.message {
                    1 => LRESULT(Util::pack_i32(cursor_pos.0 as i16, cursor_pos.0 as i16) as _),
                    2 => LRESULT(
                        Util::pack_i32(cursor_pos.1 as i16 + 1, cursor_pos.1 as i16 + 1) as _,
                    ),
                    _ => LRESULT(0),
                }
            }
            _ => Self::forward_message(hwnd, msg, wparam, lparam),
        }
    }

    /// Brings the spy window to the top of the z-order.
    fn bring_to_top(window_handle: HWND) -> crate::Result<()> {
        unsafe {
            SetWindowPos(
                window_handle,
                Some(HWND_TOPMOST),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE,
            )
        }?;

        Ok(())
    }

    /// Refreshes the icons of the tray.
    ///
    /// Simulates the Windows taskbar being re-created. Some windows fail to
    /// re-add their icons, in which case it's an implementation error on
    /// their side. These windows that fail also do not re-add their icons
    /// to the Windows taskbar when `explorer.exe` is restarted ordinarily.
    fn refresh_icons() -> crate::Result<()> {
        log::info!("Refreshing icons by sending `TaskbarCreated` message.");

        let msg = unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) };

        if msg == 0 {
            return Err(windows::core::Error::from_win32().into());
        }

        unsafe { SendNotifyMessageW(HWND_BROADCAST, msg, WPARAM::default(), LPARAM::default()) }?;

        Ok(())
    }

    pub fn initial_tray_icons(window_handle: isize) -> crate::Result<Vec<IconEventData>> {
        log::info!("Finding initial tray icons.");

        let tray = Util::find_tray_window(window_handle).ok_or("tray window not found")?;

        let toolbars = [
            Util::find_tray_toolbar_window(tray),
            Util::find_overflow_toolbar_window(),
        ];

        // Get process handle of tray window.
        let mut process_id = u32::default();
        unsafe {
            GetWindowThreadProcessId(HWND(tray as _), Some(&mut process_id));
        }

        let tray_process = unsafe { OpenProcess(PROCESS_ALL_ACCESS, false, process_id) }?;

        // Allocate memory in target process.
        let buffer = unsafe {
            VirtualAllocEx(
                tray_process,
                None,
                std::mem::size_of::<TBBUTTON>(),
                MEM_COMMIT,
                PAGE_READWRITE,
            )
        };

        if buffer.is_null() {
            return Err(Error::from_win32().into());
        }

        let mut icons = Vec::new();

        for toolbar in toolbars.into_iter().flatten() {
            // Get number of tray icons.
            let count =
                unsafe { SendMessageW(HWND(toolbar as _), TB_BUTTONCOUNT, None, None) }.0 as usize;

            log::info!("Found {} buttons in toolbar.", count);

            // Read each button.
            for index in 0..count {
                if let Ok(icon) = Self::read_tray_icon(tray_process, buffer, toolbar, index) {
                    icons.push(icon);
                }
            }
        }

        // Cleanup.
        let _ = unsafe { VirtualFreeEx(tray_process, buffer, 0, MEM_RELEASE) };
        let _ = unsafe { CloseHandle(tray_process) };

        log::info!("Retrieved {} icons from system tray.", icons.len());

        Ok(icons)
    }

    fn read_tray_icon(
        tray_process: HANDLE,
        buffer: *mut c_void,
        toolbar: isize,
        index: usize,
    ) -> crate::Result<IconEventData> {
        // Get button info via a taskbar message.
        unsafe {
            SendMessageW(
                HWND(toolbar as _),
                TB_GETBUTTON,
                Some(WPARAM(index)),
                Some(LPARAM(buffer as isize)),
            )
        };

        // Read shared memory containing the taskbar button data.
        let mut button: TBBUTTON = unsafe { std::mem::zeroed() };
        unsafe {
            ReadProcessMemory(
                tray_process,
                buffer,
                &mut button as *mut _ as _,
                std::mem::size_of::<TBBUTTON>(),
                None,
            )
        }?;

        // Read shared memory containing the tray icon data.
        let mut tray_item: TbButtonItem = unsafe { std::mem::zeroed() };
        unsafe {
            ReadProcessMemory(
                tray_process,
                button.dwData as _,
                &mut tray_item as *mut _ as _,
                std::mem::size_of::<TbButtonItem>(),
                None,
            )
        }?;

        let mut icon_event: IconEventData = tray_item.into();

        // Hidden state is determined by the toolbar button state.
        icon_event.is_visible = button.fsState & TBSTATE_HIDDEN as u8 == 0;

        Ok(icon_event)
    }

    /// Whether a message should be forwarded to the real tray window.
    fn should_forward_message(msg: u32) -> bool {
        msg == WM_COPYDATA || msg == WM_ACTIVATEAPP || msg == WM_COMMAND || msg >= WM_USER
    }

    /// Forwards a message to the real tray window.
    fn forward_message(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
        // log::debug!("Forwarding msg: {:#x} - {} to real tray window.", msg, msg);
        unsafe {
            let Some(real_tray) = Util::find_tray_window(hwnd.0 as isize) else {
                log::warn!("No real tray found.");
                return DefWindowProcW(hwnd, msg, wparam, lparam);
            };

            if msg > WM_USER {
                let _ = PostMessageW(Some(HWND(real_tray as _)), msg, wparam, lparam);
                DefWindowProcW(hwnd, msg, wparam, lparam)
            } else {
                SendMessageW(HWND(real_tray as _), msg, Some(wparam), Some(lparam))
            }
        }
    }
}

impl Drop for TraySpy {
    fn drop(&mut self) {
        if let Some(window_thread) = self.window_thread.take() {
            if let Err(err) = Util::kill_message_loop(&window_thread) {
                log::warn!("Failed to kill message loop: {:?}", err);
            }
        }
    }
}
