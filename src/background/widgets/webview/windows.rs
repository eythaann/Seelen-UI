use seelen_core::{Rect, system_state::AppBarEdge};
use windows::Win32::{
    Foundation::{HWND, RECT},
    Graphics::Gdi::{
        RDW_ALLCHILDREN, RDW_ERASE, RDW_FRAME, RDW_INVALIDATE, RDW_UPDATENOW, RedrawWindow,
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
    builder.owner_raw(unsafe { std::mem::transmute(owner.address()) })
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
