use seelen_core::{
    Rect,
    system_state::{AppBarEdge, ZOrder},
};
use tauri::Manager;
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::{
        RDW_ALLCHILDREN, RDW_ERASE, RDW_FRAME, RDW_INVALIDATE, RDW_UPDATENOW, RedrawWindow,
    },
    UI::WindowsAndMessaging::{GW_HWNDNEXT, GetTopWindow, GetWindow, SWP_ASYNCWINDOWPOS},
};

use crate::{
    app::get_app_handle,
    error::{Result, ResultLogExt},
    platform::{
        hook::{HookManager, WindowEvent},
        window::Window,
    },
    windows_api::{AppBarData, WindowsApi},
};

const WINUI_WINDOW_CLASS: &str = "WinUIDesktopWin32WindowClass";

/// Workaround for microsoft/microsoft-ui-xaml#9990 (Seelen UI #2020).
///
/// Since Windows App SDK 1.6, creating a WinUI 3 window (`WinUIDesktopWin32WindowClass`, e.g.
/// WhatsApp) corrupts the z-order of every always-on-top window of any process: they keep
/// `WS_EX_TOPMOST` but end up below normal windows until they are activated again. Per the
/// WinUI team, it comes from WinUI attaching its `DesktopWindowXamlSource` to the host window
/// during `WM_NCCREATE` instead of `WM_CREATE`. The fix is on its way upstream, but each affected
/// app has to ship the updated runtime.
///
/// The z-order only breaks on the creation of a WinUI 3 window, so on each one this re-asserts
/// topmost on all our visible topmost widgets, keeping their relative order. No detection is done:
/// at creation time the WinUI window is still hidden, so a "normal window above a topmost one"
/// check would miss it.
pub fn start_topmost_guard() {
    HookManager::subscribe(|(event, origin)| {
        if event == WindowEvent::Created && origin.class() == WINUI_WINDOW_CLASS {
            restore_broken_topmost();
        }
    });
}

fn restore_broken_topmost() {
    let mut pending: Vec<HWND> = get_app_handle()
        .webview_windows()
        .values()
        .filter_map(|webview| hwnd(webview).ok())
        .filter(|hwnd| {
            let window = Window::from(*hwnd);
            window.is_visible() && window.is_always_on_top()
        })
        .collect();

    if pending.is_empty() {
        return;
    }

    // Single walk of the z-order from top to bottom, until all our topmost widgets are found.
    let mut ordered = Vec::with_capacity(pending.len());
    let mut current = unsafe { GetTopWindow(None) }.ok();
    while let Some(hwnd) = current {
        if hwnd.is_invalid() {
            break;
        }

        if let Some(idx) = pending.iter().position(|w| *w == hwnd) {
            pending.swap_remove(idx);
            ordered.push(hwnd);
            if pending.is_empty() {
                break;
            }
        }

        current = unsafe { GetWindow(hwnd, GW_HWNDNEXT) }.ok();
    }

    log::debug!("WinUI 3 window created, restoring the topmost widgets");
    // bottom to top, so the originally highest one ends on top again
    for hwnd in ordered.into_iter().rev() {
        Window::from(hwnd).set_z_order(ZOrder::TopMost).log_error();
    }
}

pub fn native_window(webview: &tauri::WebviewWindow) -> Result<Window> {
    Ok(Window::from(hwnd(webview)?))
}

pub fn set_owner<'a, R: tauri::Runtime, M: tauri::Manager<R>>(
    builder: tauri::WebviewWindowBuilder<'a, R, M>,
    owner: &Window,
) -> tauri::WebviewWindowBuilder<'a, R, M> {
    // SAFETY: HWND in windows 0.61 (tauri) and 0.62 (ours) share the same memory layout
    #[allow(clippy::missing_transmute_annotations)]
    builder.owner_raw(unsafe { std::mem::transmute(owner.handle()) })
}

pub fn disable_system_transitions(webview: &tauri::WebviewWindow) -> Result<()> {
    WindowsApi::set_system_transitions_disabled(hwnd(webview)?, true)
}

pub fn set_position(webview: &tauri::WebviewWindow, rect: &Rect) -> Result<()> {
    let hwnd = hwnd(webview)?;
    let rect = to_win32_rect(rect);

    // pre set position for resize in case of multiples dpi
    WindowsApi::move_window(hwnd, &rect)?;
    WindowsApi::set_position(hwnd, None, &rect, SWP_ASYNCWINDOWPOS)?;
    // ensure child windows are redrawn
    unsafe {
        let _ = RedrawWindow(
            Some(hwnd),
            None,
            None,
            RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN | RDW_FRAME | RDW_ERASE,
        );
    }
    Ok(())
}

pub fn register_app_bar(
    webview: &tauri::WebviewWindow,
    rect: &Rect,
    edge: AppBarEdge,
) -> Result<()> {
    let mut app_bar = AppBarData::from_handle(hwnd(webview)?);
    app_bar.set_rect(to_win32_rect(rect));
    app_bar.set_edge(edge);
    app_bar.register_as_new_bar()
}

pub fn unregister_app_bar(webview: &tauri::WebviewWindow) -> Result<()> {
    let mut app_bar = AppBarData::from_handle(hwnd(webview)?);
    app_bar.unregister_bar()
}

/// Makes the webview render 1 CSS px = 1 physical px (`devicePixelRatio` = 1) on every monitor.
///
/// By default WebView2 tracks the monitor DPI itself (`ShouldDetectMonitorScaleChanges`) and
/// updates its `RasterizationScale` asynchronously after the window crosses monitors, so any
/// zoom-based compensation done from the webview races against it (and against `WM_DPICHANGED`
/// not being delivered to hidden windows). Taking ownership of the scale and pinning it to 1.0
/// removes the race entirely: the scale never changes again, whatever monitor the window is on.
pub fn lock_rasterization_scale_to_one(webview: &tauri::WebviewWindow) -> Result<()> {
    use webview2_com::Microsoft::Web::WebView2::Win32::ICoreWebView2Controller3;
    use windows_core::Interface;

    webview.with_webview(|platform_webview| {
        let controller = platform_webview.controller();
        let result = (|| unsafe {
            // reset any zoom compensation applied before (zoom persists across reloads)
            controller.SetZoomFactor(1.0)?;
            let controller = controller.cast::<ICoreWebView2Controller3>()?;
            controller.SetShouldDetectMonitorScaleChanges(false)?;
            controller.SetRasterizationScale(1.0)
        })();

        if let Err(err) = result {
            log::error!("Failed to lock webview rasterization scale: {err:?}");
        }
    })?;
    Ok(())
}

/// Forces the WebView2 renderer to react as if the OS was under memory pressure,
/// via the same CDP call DevTools issues internally. Unlike `window.gc()`, this
/// actually makes V8/Blink release freed pages back to the OS, because real pressure
/// almost never reaches an individual background widget when the system overall has
/// plenty of RAM free.
pub fn simulate_memory_pressure(webview: &tauri::WebviewWindow) {
    if webview.is_focused().unwrap_or(false) {
        return;
    }

    webview
        .with_webview(|platform_webview| {
            let controller = platform_webview.controller();
            let core_webview = match unsafe { controller.CoreWebView2() } {
                Ok(core_webview) => core_webview,
                Err(err) => {
                    log::warn!("Failed to get ICoreWebView2 for memory pressure simulation: {err}");
                    return;
                }
            };

            // Can't use our own `WindowsString` here: webview2-com pins its own `windows`
            // crate version (0.61) which is semver-incompatible with the one this project
            // depends on directly (0.62.2), so their `PCWSTR` types don't unify. Must go
            // through webview2-com's own string helper, built against its own `windows`.
            let result =
                webview2_com::CallDevToolsProtocolMethodCompletedHandler::wait_for_async_operation(
                    Box::new(move |handler| unsafe {
                        let method = webview2_com::CoTaskMemPWSTR::from(
                            "Memory.simulatePressureNotification",
                        );
                        let params = webview2_com::CoTaskMemPWSTR::from(r#"{"level":"critical"}"#);
                        core_webview
                            .CallDevToolsProtocolMethod(
                                *method.as_ref().as_pcwstr(),
                                *params.as_ref().as_pcwstr(),
                                &handler,
                            )
                            .map_err(webview2_com::Error::WindowsError)
                    }),
                    Box::new(|error_code, _result| error_code),
                );

            if let Err(err) = result {
                log::warn!("Memory.simulatePressureNotification failed: {err:?}");
            }
        })
        .log_error();
}

// SAFETY: HWND in windows 0.61 (tauri) and 0.62 (ours) share the same memory layout
fn hwnd(webview: &tauri::WebviewWindow) -> Result<HWND> {
    Ok(HWND(webview.hwnd()?.0))
}

fn to_win32_rect(rect: &Rect) -> RECT {
    RECT {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}
