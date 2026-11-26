use seelen_core::rect::Rect;

use crate::{
    error::Result, modules::window_positioning::domain::WindowPositioningSystem,
    windows_api::WindowsApi,
};

pub struct WindowsWindowPositioning;

impl WindowPositioningSystem for WindowsWindowPositioning {
    fn create() -> Self {
        Self
    }

    fn show_window(&self, window: isize, command: i32, asynchronous: bool) -> Result<()> {
        if asynchronous {
            WindowsApi::show_window_async(window, command)
        } else {
            WindowsApi::show_window(window, command)
        }
    }

    fn set_position(&self, window: isize, rect: &Rect, flags: u32) -> Result<()> {
        WindowsApi::set_position(
            window,
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
            flags,
        )
    }

    fn set_foreground(&self, window: isize) -> Result<()> {
        WindowsApi::set_foreground(window)
    }
}
