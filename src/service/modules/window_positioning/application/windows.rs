use seelen_core::{
    rect::Rect,
    system_state::{WindowHandle, WindowState},
};
use windows::Win32::UI::WindowsAndMessaging::{
    SHOW_WINDOW_CMD, SW_FORCEMINIMIZE, SW_HIDE, SW_MAXIMIZE, SW_MINIMIZE, SW_NORMAL, SW_RESTORE,
    SW_SHOW, SW_SHOWNOACTIVATE,
};

use crate::{
    error::Result, modules::window_positioning::domain::WindowPositioningSystem,
    windows_api::WindowsApi,
};

pub struct WindowsWindowPositioning;

impl WindowPositioningSystem for WindowsWindowPositioning {
    fn create() -> Self {
        Self
    }

    fn set_state(
        &self,
        window: WindowHandle,
        state: WindowState,
        asynchronous: bool,
    ) -> Result<()> {
        let command = to_show_command(state).0;
        if asynchronous {
            WindowsApi::show_window_async(window, command)
        } else {
            WindowsApi::show_window(window, command)
        }
    }

    fn set_position(&self, window: WindowHandle, rect: &Rect) -> Result<()> {
        WindowsApi::set_position(
            window,
            rect.left,
            rect.top,
            rect.right - rect.left,
            rect.bottom - rect.top,
        )
    }

    fn set_foreground(&self, window: WindowHandle) -> Result<()> {
        WindowsApi::set_foreground(window)
    }
}

fn to_show_command(state: WindowState) -> SHOW_WINDOW_CMD {
    match state {
        WindowState::Shown => SW_SHOW,
        WindowState::ShownNoActivate => SW_SHOWNOACTIVATE,
        WindowState::Hidden => SW_HIDE,
        WindowState::Normal => SW_NORMAL,
        WindowState::Minimized => SW_MINIMIZE,
        WindowState::ForceMinimized => SW_FORCEMINIMIZE,
        WindowState::Maximized => SW_MAXIMIZE,
        WindowState::Restored => SW_RESTORE,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `WindowState` is serialized with the `SW_*` codes previous versions sent.
    #[test]
    fn show_commands_match_the_serialized_codes() {
        for state in [
            WindowState::Shown,
            WindowState::ShownNoActivate,
            WindowState::Hidden,
            WindowState::Normal,
            WindowState::Minimized,
            WindowState::ForceMinimized,
            WindowState::Maximized,
            WindowState::Restored,
        ] {
            assert_eq!(to_show_command(state).0, state.code());
        }
    }
}
