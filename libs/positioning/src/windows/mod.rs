mod overlay;
mod shell_view;
mod timer;
mod window;

use crate::{
    domain::{PlaceOptions, PositioningSystem, WindowQuirks},
    error::Result,
    rect::Rect,
};

pub struct WindowsPositioning;

impl PositioningSystem for WindowsPositioning {
    type FrameTimer = timer::WaitableFrameTimer;
    type Overlay = overlay::DwmThumbnailOverlay;
    type Cloak = shell_view::CloakGuard;

    fn window_rect(window: isize) -> Result<Rect> {
        window::get_window_rect(window)
    }

    fn window_quirks(window: isize) -> Result<WindowQuirks> {
        Ok(quirks_for_class(&window::get_class(window)?))
    }

    fn place_window(window: isize, rect: &Rect, options: PlaceOptions) -> Result<()> {
        window::set_window_pos(
            window,
            rect,
            options.redraw,
            options.keep_size,
            options.asynchronous,
        )
    }

    fn redraw_window(window: isize) -> Result<()> {
        window::force_redraw_window(window)
    }

    fn boost_current_thread_priority() {
        timer::boost_current_thread_priority();
    }

    fn create_frame_timer() -> Result<Self::FrameTimer> {
        timer::WaitableFrameTimer::new()
    }

    fn create_overlay(source: isize, initial: &Rect) -> Result<Self::Overlay> {
        overlay::DwmThumbnailOverlay::create(source, initial)
    }

    fn cloak(window: isize) -> Result<Self::Cloak> {
        shell_view::CloakGuard::cloak(window)
    }
}

/// Quirks of the window of an app, identified by its window class.
///
/// - Explorer reacts badly to frames placed without redraw.
/// - Chromium/Electron suspend rendering while their HWND reports any cloak state, so a resize
///   done while cloaked would leave a stale/black surface.
/// - Firefox: when its startup session restore completes it checks whether all of its windows
///   are cloaked and, if they are, assumes they live on another virtual desktop and opens a new
///   window (Mozilla bug 1812489). The first placement of its restored window lands right in that
///   check, so a buffered cloak made every launch open a second, blank window.
fn quirks_for_class(class: &str) -> WindowQuirks {
    WindowQuirks {
        redraw_each_frame: class == "CabinetWClass" || class == "ExplorerWClass",
        pauses_rendering_while_cloaked: class.starts_with("Chrome_WidgetWin_"),
        breaks_when_cloaked: class == "MozillaWindowClass",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firefox_breaks_when_cloaked() {
        assert!(quirks_for_class("MozillaWindowClass").breaks_when_cloaked);
    }

    #[test]
    fn known_classes_get_their_quirks() {
        assert!(quirks_for_class("CabinetWClass").redraw_each_frame);
        assert!(quirks_for_class("ExplorerWClass").redraw_each_frame);
        assert!(quirks_for_class("Chrome_WidgetWin_1").pauses_rendering_while_cloaked);
        assert_eq!(
            quirks_for_class("CASCADIA_HOSTING_WINDOW_CLASS"),
            WindowQuirks::default()
        );
    }
}
