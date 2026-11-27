//! What Seelen UI and the taskbar guard component of `sluhk.dll` agree on.
//!
//! When Seelen hides the native Windows taskbar (`Shell_TrayWnd` and `Shell_SecondaryTrayWnd`),
//! Explorer can still try to show the taskbar on bottom-edge hover or window attention events.
//! Seelen hooks the taskbar threads with `WH_CALLWNDPROC` and sends [`ATTACH_MESSAGE`] so the
//! in-process subclass suppresses `SWP_SHOWWINDOW` in `WM_WINDOWPOSCHANGING` while Seelen is
//! alive.

/// Name of the hook procedure exported by `sluhk.dll` for the taskbar guard.
pub const HOOK_PROC: &str = "TaskbarGuardCallWndProc";

/// Registered window message (`RegisterWindowMessageW`) that attaches the guard to a taskbar
/// window. `wParam` is the Seelen UI process ID so the guard can fail open if Seelen exits.
pub const ATTACH_MESSAGE: &str = "SeelenUI.TaskbarGuard.Attach";

/// Registered window message that detaches the guard from a taskbar window.
pub const DETACH_MESSAGE: &str = "SeelenUI.TaskbarGuard.Detach";
