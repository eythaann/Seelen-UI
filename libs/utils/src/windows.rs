//! Windows-specific helpers shared between the background process and the service,
//! for behavior that both need but that doesn't belong in either's own error/type system.

use std::mem;

use windows::Win32::{
    Foundation::HWND,
    System::Threading::{AttachThreadInput, GetCurrentThreadId},
    UI::{
        Input::KeyboardAndMouse::{
            SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYBD_EVENT_FLAGS,
            KEYEVENTF_KEYUP, VIRTUAL_KEY, VK_CONTROL, VK_MENU,
        },
        WindowsAndMessaging::{
            BringWindowToTop, GetForegroundWindow, GetWindowThreadProcessId, SetForegroundWindow,
        },
    },
};

const KEYEVENTF_KEYDOWN: KEYBD_EVENT_FLAGS = KEYBD_EVENT_FLAGS(0);

fn key_input(vk: VIRTUAL_KEY, key_up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: vk,
                wScan: 0,
                dwFlags: if key_up {
                    KEYEVENTF_KEYUP
                } else {
                    KEYEVENTF_KEYDOWN
                },
                time: 0,
                dwExtraInfo: 0,
            },
        },
    }
}

/// Simulates a tap of `Alt` immediately followed by a tap of `Ctrl`.
///
/// A lone `Alt` keypress is the well known trick to satisfy the "the calling process received
/// the last input event" rule checked by [`SetForegroundWindow`] (see [`set_foreground`]'s
/// docs). Its downside is that `Alt` alone is also the accelerator Windows uses to enter
/// keyboard-menu mode on whatever window currently has focus (highlighted menu bar, an open
/// menu left dangling, a changed focused list item, etc), since to the rest of the system this
/// looks like a real, physical keypress sent to that window.
///
/// Following the `Alt` tap immediately with a `Ctrl` tap still counts as "received input" for
/// the foreground-lock check, but apps generally only enter menu mode on a standalone `Alt`
/// press/release with no other key involved, so this avoids disturbing the previously focused
/// window in most cases.
fn simulate_alt_tap() -> windows::core::Result<()> {
    let inputs = [
        key_input(VK_MENU, false),
        key_input(VK_MENU, true),
        key_input(VK_CONTROL, false),
        key_input(VK_CONTROL, true),
    ];
    let sent = unsafe { SendInput(&inputs, mem::size_of::<INPUT>() as i32) };
    if sent as usize != inputs.len() {
        return Err(windows::core::Error::from_thread());
    }
    Ok(())
}

/// Temporarily attaches this thread's input queue to `target_thread`'s, runs `f`, then detaches
/// it again. While attached, both threads share focus/activation state, which is what lets
/// [`SetForegroundWindow`] succeed for a thread that otherwise has no foreground rights.
///
/// This can hang if `target_thread` is itself blocked/unresponsive while the queues are
/// attached, so callers should not rely on it alone and should keep `f` fast.
fn with_attached_thread_input<T>(target_thread: u32, f: impl FnOnce() -> T) -> T {
    let current_thread = unsafe { GetCurrentThreadId() };
    if target_thread == 0 || target_thread == current_thread {
        return f();
    }

    let attached = unsafe { AttachThreadInput(current_thread, target_thread, true) }.as_bool();
    let result = f();
    if attached {
        let _ = unsafe { AttachThreadInput(current_thread, target_thread, false) };
    }
    result
}

fn is_foreground(target: HWND) -> bool {
    (unsafe { GetForegroundWindow() }) == target
}

/// Polls [`GetForegroundWindow`] for up to `retries` milliseconds and reports whether `target`
/// ends up being the foreground window.
///
/// This is the only thing each escalation step below trusts to decide whether it worked. The
/// `BOOL` returned by Win32 calls like `SetForegroundWindow` or `BringWindowToTop` is not a
/// reliable signal here: it can be `TRUE` without the window actually taking real OS-level
/// input focus (this bit Seelen UI before, see [`set_foreground`]'s docs), and conversely
/// [`GetForegroundWindow`] itself can transiently return null while a window is losing/gaining
/// activation, per its docs.
fn wait_until_foreground(target: HWND, retries: u32) -> bool {
    for _ in 0..retries {
        if is_foreground(target) {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_millis(1));
    }
    is_foreground(target)
}

/// Forces the window `hwnd` to become the foreground window.
///
/// Windows heavily restricts [`SetForegroundWindow`], succeeding only under a handful of
/// conditions (see its docs), none of which a background process/service normally satisfies. A
/// plain call from Seelen UI's background process or service will therefore usually fail
/// silently: it returns without actually raising the window.
///
/// To work around that, this tries progressively more invasive methods, stopping as soon as one
/// *actually* moves the real foreground window (checked with [`wait_until_foreground`], never by
/// trusting a call's own return value — see its docs for why):
/// 1. A plain [`SetForegroundWindow`] call, in case we already have the rights (e.g. we are, or
///    were launched by, the current foreground process).
/// 2. [`AttachThreadInput`] with the current foreground window's thread, followed by both
///    `BringWindowToTop` and `SetForegroundWindow` while still attached. Attaching satisfies the
///    API's checks without generating any real input, so it can't disturb whatever the user was
///    doing. This `Attach + BringWindowToTop (+ SetForegroundWindow)` combo is the workaround
///    widely documented since Vista/UAC started blocking plain `SetForegroundWindow` calls (e.g.
///    <https://shlomio.wordpress.com/2012/09/04/solved-setforegroundwindow-win32-api-not-always-works/>).
///    Seelen UI used to call only `BringWindowToTop` here (no `SetForegroundWindow`), which
///    reorders the z-order but doesn't reliably move real input focus by itself — that's exactly
///    the kind of false success this function guards against by re-checking real state before
///    trusting any step worked, rather than trusting either call's own return value.
/// 3. As a last resort, simulate a keypress ([`simulate_alt_tap`]) to satisfy the "process
///    received the last input event" condition. This is the only condition we can trigger
///    ourselves without real user input, but it comes with the caveat documented on
///    [`simulate_alt_tap`].
///
/// [`SetForegroundWindow`]: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-setforegroundwindow
pub fn set_foreground(hwnd: isize) -> Result<(), String> {
    let target_hwnd = HWND(hwnd as _);

    if is_foreground(target_hwnd) {
        return Ok(());
    }

    if unsafe { SetForegroundWindow(target_hwnd).as_bool() } {
        return Ok(());
    }

    let foreground = unsafe { GetForegroundWindow() };
    let foreground_thread = unsafe { GetWindowThreadProcessId(foreground, None) };
    with_attached_thread_input(foreground_thread, || {
        let _ = unsafe { BringWindowToTop(target_hwnd) };
        let _ = unsafe { SetForegroundWindow(target_hwnd) };
    });
    if wait_until_foreground(target_hwnd, 10) {
        return Ok(());
    }

    // https://stackoverflow.com/questions/10740346/setforegroundwindow-only-working-while-visual-studio-is-open
    simulate_alt_tap().map_err(|e| format!("Failed to simulate alt tap: {e}"))?;
    let _ = unsafe { SetForegroundWindow(target_hwnd) };
    if wait_until_foreground(target_hwnd, 10) {
        return Ok(());
    }

    Err("Failed to set foreground window".to_string())
}
