//! Windows have a lot of api to get information about the window.
//! Also provides events for theses to be listened, but some events like fullscreen, maximize, etc.
//! are not standard windows events, so we handle cached windows data to check for these changes, and emit
//! synthetic events for them.

use seelen_core::system_state::{
    FocusedApp, Relaunch, RelaunchArguments, UserAppWindow, WindowAttention,
};

use crate::{
    modules::{
        apps::application::WindowBadges, notifications::wpn_service::WpnService,
        start::application::StartMenuManager,
    },
    utils::get_parts_of_inline_command,
    windows_api::types::AppUserModelId,
};

use super::Window;

impl Window {
    /// `Relaunch` info and `prevent_pinning` both only apply to windows with a
    /// property-store assigned umid, so they're derived from it together.
    pub fn relaunch_info(&self, umid: &Option<AppUserModelId>) -> Option<(Relaunch, bool)> {
        let Some(AppUserModelId::PropertyStore(umid)) = umid else {
            return None;
        };

        // apps with a start menu shortcut are launched through `shell:AppsFolder\umid`, which
        // already applies the shortcut arguments and working dir
        if StartMenuManager::instance().has_shortcut_with_umid(umid) {
            return None;
        }

        let cmd = self.relaunch_command()?;
        let (command, args) = get_parts_of_inline_command(&cmd);

        let relaunch = Relaunch {
            command,
            args: args.map(RelaunchArguments::String),
            working_dir: None,
            icon: self.relaunch_icon(),
        };
        Some((relaunch, self.prevent_pinning()))
    }

    pub fn to_serializable(self: &Window) -> UserAppWindow {
        let umid = self.app_user_model_id();
        let (relaunch, prevent_pinning) = self.relaunch_info(&umid).unzip();
        let badge = WindowBadges::instance().get(self.address());
        let badge_value = umid
            .as_ref()
            .and_then(|umid| WpnService::instance().ok()?.get_badge_value(umid.as_str()));

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
            prevent_pinning: prevent_pinning.unwrap_or(false),
            relaunch,
            rect: self.inner_rect().ok(),
            last_foreground_at: 0,
            attention: WindowAttention::None,
            badge_icon_path: badge.as_ref().map(|b| b.icon_path.clone()),
            badge_updated_at: badge.map_or(0, |b| b.updated_at),
            badge_value,
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
