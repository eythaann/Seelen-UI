use seelen_core::{Rect, system_state::AppBarEdge};
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::{
        MONITOR_DEFAULTTOPRIMARY, MonitorFromRect, MonitorFromWindow, RDW_ALLCHILDREN, RDW_ERASE,
        RDW_FRAME, RDW_INVALIDATE, RDW_UPDATENOW, RedrawWindow,
    },
    UI::WindowsAndMessaging::SWP_ASYNCWINDOWPOS,
};

use crate::{
    error::{Result, ResultLogExt},
    platform::window::Window,
    windows_api::{AppBarData, WindowsApi},
};

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
    let current = WindowsApi::get_outer_window_rect(hwnd)?;
    if current == rect {
        return Ok(());
    }

    // pre set position for resize in case of multiples dpi
    let current_monitor = unsafe { MonitorFromWindow(hwnd, MONITOR_DEFAULTTOPRIMARY) };
    let target_monitor = unsafe { MonitorFromRect(&rect, MONITOR_DEFAULTTOPRIMARY) };
    let different_dpi = if current_monitor == target_monitor {
        false
    } else {
        match (
            WindowsApi::get_monitor_scale_factor(current_monitor),
            WindowsApi::get_monitor_scale_factor(target_monitor),
        ) {
            (Ok(current), Ok(target)) => current != target,
            _ => true,
        }
    };
    if different_dpi {
        WindowsApi::move_window(hwnd, &rect)?;
    }
    WindowsApi::set_position(hwnd, None, &rect, SWP_ASYNCWINDOWPOS)?;
    if different_dpi {
        unsafe {
            let _ = RedrawWindow(
                Some(hwnd),
                None,
                None,
                RDW_INVALIDATE | RDW_UPDATENOW | RDW_ALLCHILDREN | RDW_FRAME | RDW_ERASE,
            );
        }
    } else if current.right - current.left != rect.right - rect.left
        || current.bottom - current.top != rect.bottom - rect.top
    {
        unsafe {
            let _ = RedrawWindow(Some(hwnd), None, None, RDW_INVALIDATE | RDW_ALLCHILDREN);
        }
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
    let Ok(window) = native_window(webview) else {
        return;
    };
    if window.is_visible() {
        return;
    }

    webview
        .with_webview(move |platform_webview| {
            // The window can become visible while the callback is queued on the UI thread.
            if window.is_visible() {
                return;
            }
            let controller = platform_webview.controller();
            let core_webview = match unsafe { controller.CoreWebView2() } {
                Ok(core_webview) => core_webview,
                Err(err) => {
                    log::warn!("Failed to get ICoreWebView2 for memory pressure simulation: {err}");
                    return;
                }
            };

            let handler = webview2_com::CallDevToolsProtocolMethodCompletedHandler::create(
                Box::new(|result, _| {
                    if let Err(err) = result {
                        log::warn!("Memory.simulatePressureNotification failed: {err:?}");
                    }
                    Ok(())
                }),
            );
            let method = webview2_com::CoTaskMemPWSTR::from("Memory.simulatePressureNotification");
            let params = webview2_com::CoTaskMemPWSTR::from(r#"{"level":"critical"}"#);
            let result = unsafe {
                core_webview.CallDevToolsProtocolMethod(
                    *method.as_ref().as_pcwstr(),
                    *params.as_ref().as_pcwstr(),
                    &handler,
                )
            };

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
