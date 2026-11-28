use slu_ipc::{
    AppIpc,
    messages::{AppMessage, IconEventData, TrayIconPatch, Win32TaskbarButtonEvent, Win32TrayEvent},
};
use windows::Win32::{
    Foundation::{HANDLE, HWND, LPARAM, LRESULT, WPARAM},
    System::DataExchange::COPYDATASTRUCT,
    UI::{
        Shell::{
            NIF_GUID, NIF_ICON, NIF_MESSAGE, NIF_STATE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY,
            NIM_SETVERSION, NIS_HIDDEN, NIS_SHAREDICON, NOTIFY_ICON_DATA_FLAGS,
            NOTIFY_ICON_INFOTIP_FLAGS, NOTIFY_ICON_MESSAGE, NOTIFY_ICON_STATE, NOTIFYICONDATAW_0,
            SHLockShared, SHUnlockShared,
        },
        WindowsAndMessaging::{
            CWPSTRUCT, CallNextHookEx, GetClassNameW, GetWindowThreadProcessId, WM_COPYDATA,
            WM_USER,
        },
    },
};
use windows_core::{GUID, PCWSTR};

// ============================================================================
// Constants
// ============================================================================

/// Receives `Shell_NotifyIcon` calls (tray icons) via `WM_COPYDATA`.
const TRAY_CLASS: &str = "Shell_TrayWnd";
/// Window stored in the `TaskbandHWND` prop of `Shell_TrayWnd`, receives `ITaskbarList3` calls.
const TASKBAND_CLASS: &str = "MSTaskSwWClass";

// Undocumented taskband messages sent by `ITaskbarList3`, reference:
// https://github.com/cairoshell/ManagedShell/blob/master/src/ManagedShell.WindowsTasks/TasksService.cs
/// wParam: target window, lParam: HICON (null = overlay removed)
const TBM_SET_OVERLAY_ICON: u32 = WM_USER + 79;
/// wParam: target window, lParam: shared memory handle (`SHAllocShared`) with the description string
const TBM_SET_OVERLAY_DESCRIPTION: u32 = WM_USER + 85;

// ============================================================================
// Native Windows Structures
// ============================================================================

/// Tray message sent to `Shell_TrayWnd`
#[repr(C)]
struct ShellTrayMessage {
    magic_number: i32,
    message_type: u32,
    icon_data: NotifyIconData,
}

/// Contains the data for a system tray icon.
/// https://learn.microsoft.com/en-us/windows/win32/api/shellapi/ns-shellapi-notifyicondataw
/// in the windows crate NOTIFYICONDATAW use raw pointers that are 64-bit on 64-bit systems but this
/// need always be 32-bit, so we use a self-defined struct.
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
    guid_item: GUID,
    balloon_icon_handle: u32,
}

impl NotifyIconData {
    /// Identity of the icon: its guid (`NIF_GUID`), and its window handle + uid.
    fn identity(&self) -> (Option<uuid::Uuid>, Option<isize>, Option<u32>) {
        let guid = (self.flags.contains(NIF_GUID) && self.guid_item != GUID::default())
            .then(|| uuid::Uuid::from_u128(self.guid_item.to_u128()));
        let (window_handle, uid) = if self.window_handle != 0 {
            (Some(self.window_handle as isize), Some(self.uid))
        } else {
            (None, None)
        };
        (guid, window_handle, uid)
    }

    /// `NIF_TIP`: the tooltip as declared, may be empty (cleared).
    fn tooltip(&self) -> Option<String> {
        if !self.flags.contains(NIF_TIP) {
            return None;
        }
        let len = self
            .tooltip
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(self.tooltip.len());
        Some(String::from_utf16_lossy(&self.tooltip[..len]).replace('\r', ""))
    }

    /// `NIF_ICON` with a non null `HICON`.
    fn icon_handle(&self) -> Option<isize> {
        (self.flags.contains(NIF_ICON) && self.icon_handle != 0)
            .then_some(self.icon_handle as isize)
    }

    /// `NIF_MESSAGE`
    fn callback_message(&self) -> Option<u32> {
        self.flags
            .contains(NIF_MESSAGE)
            .then_some(self.callback_message)
    }

    /// `uVersion`, it shares its memory with `uTimeout` (`NIF_INFO`).
    fn version(&self) -> Option<u32> {
        let version = unsafe { self.anonymous.uVersion };
        (version > 0 && version <= 4).then_some(version)
    }

    /// Only the state bits included in the mask are applied.
    fn state_bit(&self, bit: NOTIFY_ICON_STATE) -> Option<bool> {
        (self.flags.contains(NIF_STATE) && self.state_mask.contains(bit))
            .then(|| self.state.contains(bit))
    }

    /// Changes declared by an `NIM_MODIFY` call.
    fn modify_patch(&self) -> TrayIconPatch {
        let (guid, window_handle, uid) = self.identity();
        TrayIconPatch {
            uid,
            window_handle,
            guid,
            tooltip: self.tooltip(),
            icon_handle: self.icon_handle(),
            callback_message: self.callback_message(),
            version: None,
            is_visible: self.state_bit(NIS_HIDDEN).map(|hidden| !hidden),
            shared_icon: self.state_bit(NIS_SHAREDICON),
        }
    }

    /// `NIM_SETVERSION` only changes the version, the other fields are ignored by the shell.
    fn set_version_patch(&self) -> TrayIconPatch {
        let (guid, window_handle, uid) = self.identity();
        TrayIconPatch {
            uid,
            window_handle,
            guid,
            version: self.version(),
            ..Default::default()
        }
    }
}

impl From<NotifyIconData> for IconEventData {
    fn from(icon_data: NotifyIconData) -> Self {
        let (guid, window_handle, uid) = icon_data.identity();
        IconEventData {
            uid,
            window_handle,
            guid,
            tooltip: icon_data.tooltip().filter(|tooltip| !tooltip.is_empty()),
            icon_handle: icon_data.icon_handle(),
            callback_message: icon_data.callback_message(),
            version: icon_data.version(),
            is_visible: icon_data.state_bit(NIS_HIDDEN).map(|hidden| !hidden),
            shared_icon: icon_data.state_bit(NIS_SHAREDICON),
        }
    }
}

fn get_window_class(hwnd: HWND) -> String {
    let mut text: [u16; 512] = [0; 512];
    let len = unsafe { GetClassNameW(hwnd, &mut text) };
    let length = usize::try_from(len).unwrap_or(0);
    String::from_utf16_lossy(&text[..length])
}

// ============================================================================
// Hook Implementation
// ============================================================================

/// https://learn.microsoft.com/en-us/windows/win32/winmsg/about-hooks
/// https://learn.microsoft.com/en-us/windows/win32/winmsg/callwndproc
///
/// # Safety
#[unsafe(no_mangle)]
pub unsafe extern "system" fn CallWndProc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        let next = || CallNextHookEx(None, code, wparam, lparam);
        if code < 0 {
            return next();
        }

        let Some(msg) = (lparam.0 as *const CWPSTRUCT).as_ref() else {
            return next();
        };

        // filter by message first to avoid querying the class name on every message
        if !matches!(
            msg.message,
            WM_COPYDATA | TBM_SET_OVERLAY_ICON | TBM_SET_OVERLAY_DESCRIPTION
        ) {
            return next();
        }

        // Send debug message for all messages received
        /* let debug_msg = format!(
            "CallWndProc - code: {}, hwnd: {:?}, message: 0x{:X}, wparam: {:?}, lparam: {:?}",
            code, msg.hwnd, msg.message, msg.wParam, msg.lParam
        );
        let _ = AppIpc::send_sync(&AppMessage::Debug(debug_msg)); */

        match get_window_class(msg.hwnd).as_str() {
            TRAY_CLASS => {
                if let Some(event) = process_tray_message(msg) {
                    send_via_ipc(AppMessage::TrayChanged(event));
                }
            }
            TASKBAND_CLASS => {
                if let Some(event) = process_taskband_message(msg) {
                    send_via_ipc(AppMessage::TaskbarButtonChanged(event));
                }
            }
            _ => {}
        }

        next()
    }
}

/// Processes a tray message and returns an event if relevant
unsafe fn process_tray_message(msg: &CWPSTRUCT) -> Option<Win32TrayEvent> {
    unsafe {
        if msg.message != WM_COPYDATA {
            return None;
        }

        let copy_data = (msg.lParam.0 as *const COPYDATASTRUCT).as_ref()?;

        // Type 1 is the tray icon message
        if copy_data.dwData != 1 || copy_data.lpData.is_null() {
            return None;
        }

        // Any process can send WM_COPYDATA to the tray window, never read past the received
        // buffer as this runs inside explorer.exe.
        if (copy_data.cbData as usize) < size_of::<ShellTrayMessage>() {
            return None;
        }

        let tray_message = &*copy_data.lpData.cast::<ShellTrayMessage>();
        let icon_data = tray_message.icon_data;

        match NOTIFY_ICON_MESSAGE(tray_message.message_type) {
            NIM_ADD => Some(Win32TrayEvent::IconAdd {
                data: icon_data.into(),
            }),
            NIM_MODIFY => Some(Win32TrayEvent::IconUpdate {
                patch: icon_data.modify_patch(),
            }),
            NIM_SETVERSION => Some(Win32TrayEvent::IconUpdate {
                patch: icon_data.set_version_patch(),
            }),
            NIM_DELETE => Some(Win32TrayEvent::IconRemove {
                data: icon_data.into(),
            }),
            _ => None,
        }
    }
}

/// Processes an `ITaskbarList3` message sent to the taskband and returns an event if relevant.
///
/// The description must be read here: the sender is blocked in `SendMessage` while the hook runs,
/// but it is free to release the shared memory as soon as the call returns.
unsafe fn process_taskband_message(msg: &CWPSTRUCT) -> Option<Win32TaskbarButtonEvent> {
    unsafe {
        let hwnd = msg.wParam.0 as isize;
        match msg.message {
            TBM_SET_OVERLAY_ICON => Some(Win32TaskbarButtonEvent::OverlayIconChanged {
                hwnd,
                icon_handle: (msg.lParam.0 != 0).then_some(msg.lParam.0),
            }),
            TBM_SET_OVERLAY_DESCRIPTION => {
                Some(Win32TaskbarButtonEvent::OverlayDescriptionChanged {
                    hwnd,
                    description: read_shared_string(HWND(hwnd as _), msg.lParam),
                })
            }
            _ => None,
        }
    }
}

/// Reads a string allocated via `SHAllocShared` by the process owning `owner`.
unsafe fn read_shared_string(owner: HWND, lparam: LPARAM) -> Option<String> {
    unsafe {
        if lparam.0 == 0 {
            return None;
        }

        let mut process_id = 0;
        GetWindowThreadProcessId(owner, Some(&mut process_id));
        if process_id == 0 {
            return None;
        }

        // SHLockShared duplicates the handle without closing the source, so explorer can still read it
        let ptr = SHLockShared(HANDLE(lparam.0 as _), process_id);
        if ptr.is_null() {
            return None;
        }
        let text = PCWSTR(ptr as _).to_string().ok();
        let _ = SHUnlockShared(ptr);
        text.filter(|t| !t.is_empty())
    }
}

// ============================================================================
// IPC Implementation
// ============================================================================

/// Sends a message through the IPC channel to the main process
fn send_via_ipc(message: AppMessage) {
    // If it fails, we simply ignore it (we don't want to crash the hook)
    let _ = AppIpc::send_sync(&message);
}

// ============================================================================
// DLL Entry Point
// ============================================================================

/// DLL entry point
/// This is called when the DLL is loaded/unloaded in any process
///
/// # Safety
#[unsafe(no_mangle)]
#[allow(non_snake_case)]
pub extern "system" fn DllMain(
    _hinst_dll: windows::Win32::Foundation::HINSTANCE,
    _fdw_reason: u32,
    _lpv_reserved: *const std::ffi::c_void,
) -> bool {
    // No special initialization needed
    // The hook will be installed by the main process
    true
}
