use std::sync::Once;

use windows::{
    Win32::{
        Foundation::{COLORREF, HINSTANCE, HWND, LPARAM, LRESULT, RECT, SIZE, WPARAM},
        Graphics::Dwm::{
            DWM_THUMBNAIL_PROPERTIES, DWM_TNP_RECTDESTINATION, DWM_TNP_SOURCECLIENTAREAONLY,
            DWM_TNP_VISIBLE, DWMWA_EXTENDED_FRAME_BOUNDS, DwmFlush, DwmGetWindowAttribute,
            DwmRegisterThumbnail, DwmUnregisterThumbnail, DwmUpdateThumbnailProperties,
        },
        System::LibraryLoader::GetModuleHandleW,
        UI::HiDpi::{DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2, SetThreadDpiAwarenessContext},
        UI::WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, DispatchMessageW, GW_HWNDPREV,
            GetSystemMetrics, GetWindow, HWND_TOP, LWA_ALPHA, MSG, PM_REMOVE, PeekMessageW,
            RegisterClassW, SM_CXVIRTUALSCREEN, SM_CYVIRTUALSCREEN, SM_XVIRTUALSCREEN,
            SM_YVIRTUALSCREEN, SWP_NOACTIVATE, SWP_NOMOVE, SWP_NOSIZE, SWP_SHOWWINDOW,
            SetLayeredWindowAttributes, SetWindowPos, TranslateMessage, WM_DPICHANGED,
            WM_GETDPISCALEDSIZE, WNDCLASSW, WS_EX_LAYERED, WS_EX_NOACTIVATE,
            WS_EX_NOREDIRECTIONBITMAP, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT, WS_POPUP,
        },
    },
    core::{PCWSTR, w},
};

use crate::{api::get_window_rect, error::Result, rect::Rect};

const OVERLAY_CLASS: PCWSTR = w!("SeelenPositioningThumbnailOverlay");

/// Click-through, non-activating window spanning the virtual screen that hosts a DWM thumbnail
/// of another window. It has no redirection bitmap, so the overlay itself draws nothing and only
/// the thumbnail visual is composed. Covering the whole virtual screen (instead of the from/to
/// bounds) keeps overshooting easings (back, elastic) from being clipped, at no memory cost.
///
/// Must be created, used and dropped on the same thread.
pub struct ThumbnailOverlay {
    hwnd: HWND,
    thumbnail: isize,
    origin_x: i32,
    origin_y: i32,
    /// Invisible resize borders of the source (outer rect minus extended frame bounds), as
    /// positive margins. DWM thumbnails capture only the visible frame, so destinations given as
    /// outer rects must be shrunk by these, or the texture is stretched by the border ratio.
    insets: RECT,
}

impl ThumbnailOverlay {
    pub fn create(source: isize, initial: &Rect) -> Result<Self> {
        let instance = register_class()?;
        // Physical pixels everywhere, regardless of the host process manifest: a window's DPI
        // awareness is taken from its creating thread, and this thread's metrics, rects and the
        // thumbnail destinations must all share the same (real, unscaled) desktop coordinates.
        unsafe { SetThreadDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2) };
        let (origin_x, origin_y, width, height) = unsafe {
            (
                GetSystemMetrics(SM_XVIRTUALSCREEN),
                GetSystemMetrics(SM_YVIRTUALSCREEN),
                GetSystemMetrics(SM_CXVIRTUALSCREEN),
                GetSystemMetrics(SM_CYVIRTUALSCREEN),
            )
        };

        let hwnd = unsafe {
            CreateWindowExW(
                WS_EX_TOOLWINDOW
                    | WS_EX_NOACTIVATE
                    | WS_EX_TRANSPARENT
                    | WS_EX_LAYERED
                    | WS_EX_NOREDIRECTIONBITMAP,
                OVERLAY_CLASS,
                PCWSTR::null(),
                WS_POPUP,
                origin_x,
                origin_y,
                width,
                height,
                None,
                None,
                Some(instance),
                None,
            )?
        };

        // Built right away so Drop cleans up the window if any step below fails.
        let mut overlay = Self {
            hwnd,
            thumbnail: 0,
            origin_x,
            origin_y,
            insets: get_frame_insets(source, initial).unwrap_or_default(),
        };

        unsafe {
            // Layered + transparent = clicks pass through to whatever is below.
            SetLayeredWindowAttributes(hwnd, COLORREF(0), 255, LWA_ALPHA)?;
            overlay.thumbnail = DwmRegisterThumbnail(hwnd, HWND(source as _))?;
        }
        overlay.set_destination(initial)?;

        // Right above the source in z-order, so windows that covered it keep covering the thumbnail.
        let insert_after = unsafe { GetWindow(HWND(source as _), GW_HWNDPREV) }.unwrap_or(HWND_TOP);
        unsafe {
            SetWindowPos(
                hwnd,
                Some(insert_after),
                0,
                0,
                0,
                0,
                SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE | SWP_SHOWWINDOW,
            )?;
        }

        Ok(overlay)
    }

    /// Places the thumbnail where a window with outer rect `rect` (screen coordinates) would show
    /// its visible frame, stretching it to fit.
    pub fn set_destination(&self, rect: &Rect) -> Result<()> {
        let left = rect.x - self.origin_x;
        let top = rect.y - self.origin_y;
        let props = DWM_THUMBNAIL_PROPERTIES {
            dwFlags: DWM_TNP_RECTDESTINATION | DWM_TNP_VISIBLE | DWM_TNP_SOURCECLIENTAREAONLY,
            rcDestination: RECT {
                left: left + self.insets.left,
                top: top + self.insets.top,
                right: left + rect.width - self.insets.right,
                bottom: top + rect.height - self.insets.bottom,
            },
            opacity: 255,
            fVisible: true.into(),
            fSourceClientAreaOnly: false.into(),
            ..Default::default()
        };
        unsafe { DwmUpdateThumbnailProperties(self.thumbnail, &props)? };
        Ok(())
    }

    /// The animation thread has no message loop; drain the queue so the overlay never looks hung
    /// to apps that broadcast or send messages to every top-level window.
    pub fn pump_messages(&self) {
        let mut msg = MSG::default();
        unsafe {
            while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
                let _ = TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

impl Drop for ThumbnailOverlay {
    fn drop(&mut self) {
        unsafe {
            // Let DWM present whatever changed before (e.g. the real window being uncloaked)
            // so removing the thumbnail does not leave a blank frame.
            let _ = DwmFlush();
            if self.thumbnail != 0 {
                let _ = DwmUnregisterThumbnail(self.thumbnail);
            }
            let _ = DestroyWindow(self.hwnd);
        }
    }
}

/// `outer` must be the current `GetWindowRect` of `hwnd`.
fn get_frame_insets(hwnd: isize, outer: &Rect) -> Result<RECT> {
    let mut frame = RECT::default();
    unsafe {
        DwmGetWindowAttribute(
            HWND(hwnd as _),
            DWMWA_EXTENDED_FRAME_BOUNDS,
            &mut frame as *mut RECT as _,
            std::mem::size_of::<RECT>() as u32,
        )?;
    }
    Ok(RECT {
        left: frame.left - outer.x,
        top: frame.top - outer.y,
        right: (outer.x + outer.width) - frame.right,
        bottom: (outer.y + outer.height) - frame.bottom,
    })
}

fn register_class() -> Result<HINSTANCE> {
    static REGISTER: Once = Once::new();
    let instance: HINSTANCE = unsafe { GetModuleHandleW(None)? }.into();
    REGISTER.call_once(|| {
        let class = WNDCLASSW {
            lpfnWndProc: Some(overlay_wnd_proc),
            hInstance: instance,
            lpszClassName: OVERLAY_CLASS,
            ..Default::default()
        };
        if unsafe { RegisterClassW(&class) } == 0 {
            log::error!("Failed to register thumbnail overlay window class");
        }
    });
    Ok(instance)
}

unsafe extern "system" fn overlay_wnd_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        // The overlay is a canvas in real desktop coordinates: a DPI change (e.g. its majority
        // monitor changing) must never rescale or move it. Answer the size query with the current
        // size so no linear DPI scaling is applied, and ignore the change itself.
        WM_GETDPISCALEDSIZE => {
            if let Ok(rect) = get_window_rect(hwnd.0 as isize) {
                unsafe {
                    *(lparam.0 as *mut SIZE) = SIZE {
                        cx: rect.width,
                        cy: rect.height,
                    };
                }
            }
            LRESULT(1)
        }
        WM_DPICHANGED => LRESULT(0),
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
