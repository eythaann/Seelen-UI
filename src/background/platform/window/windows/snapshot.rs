//! Serializable snapshots of a window, built from its Win32 data (umid, relaunch info, badges).

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

use super::NativeWindow;

impl NativeWindow {
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

    pub fn snapshot(&self) -> UserAppWindow {
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
            process: self.process().snapshot(),
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

    pub fn focused_app_snapshot(&self) -> FocusedApp {
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
            exe: process.exe_path().ok(),
            umid: self.app_user_model_id().map(|umid| umid.to_string()),
            is_maximized: self.is_maximized(),
            is_fullscreened: self.is_fullscreen(),
            rect: self.inner_rect().ok(),
        }
    }
}
