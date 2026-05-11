//! Windows have a lot of api to get information about the window.
//! Also provides events for theses to be listened, but some events like fullscreen, maximize, etc.
//! are not standard windows events, so we handle cached windows data to check for these changes, and emit
//! synthetic events for them.

use seelen_core::system_state::{FocusedApp, Relaunch, RelaunchArguments, UserAppWindow};

use crate::{utils::get_parts_of_inline_command, windows_api::types::AppUserModelId};

use super::Window;

impl Window {
    pub fn to_serializable(self: &Window) -> UserAppWindow {
        let umid = self.app_user_model_id();
        let mut prevent_pinning = false;

        let relaunch = match umid {
            Some(AppUserModelId::PropertyStore(_)) => {
                if let Some(cmd) = self.relaunch_command() {
                    let (command, args) = get_parts_of_inline_command(&cmd);
                    let args = args.map(RelaunchArguments::String);

                    let icon = self.relaunch_icon();
                    prevent_pinning = self.prevent_pinning();

                    Some(Relaunch {
                        command,
                        args,
                        working_dir: None,
                        icon,
                    })
                } else {
                    None
                }
            }
            _ => None,
        };

        UserAppWindow {
            hwnd: self.address(),
            monitor: self.monitor().stable_id().unwrap_or_default(),
            title: self.title(),
            app_name: self.app_display_name().unwrap_or_default(),
            is_iconic: self.is_minimized(),
            is_zoomed: self.is_maximized(),
            is_fullscreen: self.is_fullscreen(),
            umid: umid.map(|umid| umid.to_string()),
            process: self.process().to_serializable(),
            prevent_pinning,
            relaunch,
            rect: self.inner_rect().ok(),
        }
    }

    pub fn as_focused_app_information(&self) -> FocusedApp {
        let process = self.process();

        FocusedApp {
            hwnd: self.address(),
            owner_hwnd: self.owner().map(|w| w.address()).unwrap_or(0),
            monitor: self.monitor().stable_id().unwrap_or_default(),
            title: self.title(),
            class: self.class(),
            name: self
                .app_display_name()
                .unwrap_or(String::from("Error on App Name")),
            exe: process.program_path().ok(),
            umid: self.app_user_model_id().map(|umid| umid.to_string()),
            is_maximized: self.is_maximized(),
            is_fullscreened: self.is_fullscreen(),
            rect: self.inner_rect().ok(),
        }
    }
}
