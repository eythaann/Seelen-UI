//! Windows have a lot of api to get information about the window.
//! Also provides events for theses to be listened, but some events like fullscreen, maximize, etc.
//! are not standard windows events, so we handle cached windows data to check for these changes, and emit
//! synthetic events for them.

use seelen_core::system_state::{FocusedApp, UserAppWindow};

use super::Window;

impl Window {
    pub fn to_serializable(self: &Window) -> UserAppWindow {
        UserAppWindow {
            hwnd: self.address(),
            monitor: self.monitor().stable_id().unwrap_or_default().into(),
            title: self.title(),
            app_name: self.app_display_name().unwrap_or_default(),
            is_iconic: self.is_minimized(),
            is_zoomed: self.is_maximized(),
            is_fullscreen: self.is_fullscreen(),
            umid: self.app_user_model_id().map(|umid| umid.to_string()),
            process: self.process().to_serializable(),
        }
    }

    pub fn as_focused_app_information(&self) -> FocusedApp {
        let process = self.process();

        FocusedApp {
            hwnd: self.address(),
            monitor: self.monitor().stable_id().unwrap_or_default().into(),
            title: self.title(),
            class: self.class(),
            name: self
                .app_display_name()
                .unwrap_or(String::from("Error on App Name")),
            exe: process.program_path().ok(),
            umid: self.app_user_model_id().map(|umid| umid.to_string()),
            is_maximized: self.is_maximized(),
            is_fullscreened: self.is_fullscreen(),
            is_seelen_overlay: self.is_seelen_overlay(),
            rect: self.inner_rect().ok(),
        }
    }
}
