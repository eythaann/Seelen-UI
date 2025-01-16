use seelen_core::rect::Rect;
use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{WS_EX_APPWINDOW, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW},
};

use crate::{
    error_handler::Result,
    modules::virtual_desk::{get_vd_manager, VirtualDesktop},
    seelen_bar::FancyToolbar,
    seelen_rofi::SeelenRofi,
    seelen_wall::SeelenWall,
    seelen_weg::instance::SeelenWeg,
    seelen_wm_v2::instance::WindowManagerV2,
};

use super::{monitor::Monitor, process::Process, WindowEnumerator, WindowsApi};

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Window(HWND);
unsafe impl Send for Window {}
unsafe impl Sync for Window {}

impl From<HWND> for Window {
    fn from(hwnd: HWND) -> Self {
        Self(hwnd)
    }
}

impl From<isize> for Window {
    fn from(addr: isize) -> Self {
        Self(HWND(addr as _))
    }
}

impl Debug for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Window")
            .field("handle", &self.0 .0)
            .field("title", &self.title())
            .field("class", &self.class())
            .field("exe", &self.exe())
            .finish()
    }
}

impl Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Window({:?})", self.0)
    }
}

pub const APP_FRAME_HOST_PATH: &str = "C:\\Windows\\System32\\ApplicationFrameHost.exe";
impl Window {
    pub fn hwnd(&self) -> HWND {
        self.0
    }

    pub fn address(&self) -> isize {
        self.0 .0 as isize
    }

    /// App user model id asigned to the window via property-store
    /// To get UWP app user model id use `self.process().package_app_user_model_id()`
    ///
    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-id
    pub fn app_user_model_id(&self) -> Option<String> {
        WindowsApi::get_window_app_user_model_id(self.0).ok()
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-preventpinning
    pub fn prevent_pinning(&self) -> bool {
        WindowsApi::get_window_prevent_pinning(self.0).unwrap_or(false)
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchcommand
    pub fn relaunch_command(&self) -> Option<String> {
        WindowsApi::get_window_relaunch_command(self.0).ok()
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchdisplaynameresource
    pub fn relaunch_display_name(&self) -> Option<String> {
        if let Ok(name) = WindowsApi::get_window_relaunch_display_name(self.0) {
            if name.starts_with("@") {
                return WindowsApi::resolve_indirect_string(&name).ok();
            }
            return Some(name);
        }
        None
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchiconresource
    pub fn relaunch_icon(&self) -> Option<String> {
        WindowsApi::get_window_relaunch_icon_resource(self.0).ok()
    }

    pub fn title(&self) -> String {
        WindowsApi::get_window_text(self.0)
    }

    pub fn class(&self) -> String {
        WindowsApi::get_class(self.0).unwrap_or_default()
    }

    pub fn process(&self) -> Process {
        Process::from_window(self)
    }

    /// will fail if process is restricted and the invoker is not running as admin
    pub fn exe(&self) -> Result<PathBuf> {
        WindowsApi::exe_path_v2(self.0)
    }

    pub fn app_display_name(&self) -> Result<String> {
        if let Ok(info) = self.process().package_app_info() {
            return Ok(info.DisplayInfo()?.DisplayName()?.to_string_lossy());
        }
        self.process().program_display_name()
    }

    pub fn outer_rect(&self) -> Result<Rect> {
        Ok(WindowsApi::get_outer_window_rect(self.hwnd())?.into())
    }

    pub fn inner_rect(&self) -> Result<Rect> {
        Ok(WindowsApi::get_inner_window_rect(self.hwnd())?.into())
    }

    pub fn parent(&self) -> Option<Window> {
        let parent = WindowsApi::get_parent(self.0);
        if !parent.is_invalid() {
            Some(Window(parent))
        } else {
            None
        }
    }

    pub fn children(&self) -> Result<Vec<Window>> {
        WindowEnumerator::new()
            .with_parent(self.0)
            .map(Window::from)
    }

    pub fn monitor(&self) -> Monitor {
        Monitor::from(WindowsApi::monitor_from_window(self.0))
    }

    pub fn workspace(&self) -> Result<VirtualDesktop> {
        get_vd_manager().get_by_window(self.address())
    }

    pub fn is_window(&self) -> bool {
        WindowsApi::is_window(self.0)
    }

    pub fn is_visible(&self) -> bool {
        WindowsApi::is_window_visible(self.0)
    }

    pub fn is_minimized(&self) -> bool {
        WindowsApi::is_iconic(self.0)
    }

    pub fn is_maximized(&self) -> bool {
        WindowsApi::is_maximized(self.0)
    }

    pub fn is_cloaked(&self) -> bool {
        WindowsApi::is_cloaked(self.0).unwrap_or(false)
    }

    pub fn is_foreground(&self) -> bool {
        WindowsApi::get_foreground_window() == self.0
    }

    pub fn is_fullscreen(&self) -> bool {
        WindowsApi::is_fullscreen(self.0).unwrap_or(false)
    }

    /// is the window an Application Frame Host
    pub fn is_frame(&self) -> Result<bool> {
        Ok(self.exe()? == PathBuf::from(APP_FRAME_HOST_PATH))
    }

    /// will fail if the window is not a frame
    pub fn get_frame_creator(&self) -> Result<Option<Window>> {
        if !self.is_frame()? {
            return Err("Window is not a frame".into());
        }
        for window in self.children()? {
            if !window.class().starts_with("ApplicationFrame") {
                return Ok(Some(window));
            }
        }
        Ok(None)
    }

    /// this means all windows that are part of the UI desktop not the real desktop window
    pub fn is_desktop(&self) -> bool {
        let class = self.class();
        WindowsApi::get_desktop_window() == self.0
            || class == "Progman"
            || (class == "WorkerW"
                && self.children().is_ok_and(|children| {
                    children
                        .iter()
                        .any(|child| child.class() == "SHELLDLL_DefView")
                }))
    }

    pub fn is_seelen_overlay(&self) -> bool {
        if let Ok(exe) = self.exe() {
            return exe.ends_with("seelen-ui.exe")
                && [
                    FancyToolbar::TITLE,
                    WindowManagerV2::TITLE,
                    SeelenWeg::TITLE,
                    SeelenRofi::TITLE,
                    SeelenWall::TITLE,
                ]
                .contains(&self.title().as_str());
        }
        false
    }

    pub fn is_real_window(&self) -> bool {
        let path = match self.process().program_path() {
            Ok(path) => path,
            Err(_) => return false,
        };

        if !self.is_visible()
            || path.starts_with("C:\\Windows\\SystemApps")
            || path.starts_with("C:\\Windows\\ImmersiveControlPanel")
            || self.parent().is_some()
            || self.is_seelen_overlay()
        {
            return false;
        }

        // this class is used for edge tabs to be shown as independent windows on alt + tab
        // this only applies when the new tab is created it is binded to explorer.exe for some reason
        // maybe we can search/learn more about edge tabs later.
        // fix: https://github.com/eythaann/Seelen-UI/issues/83
        if self.class() == "Windows.Internal.Shell.TabProxyWindow" {
            return false;
        }

        let ex_style = WindowsApi::get_ex_styles(self.hwnd());
        if (ex_style.contains(WS_EX_TOOLWINDOW) || ex_style.contains(WS_EX_NOACTIVATE))
            && !ex_style.contains(WS_EX_APPWINDOW)
        {
            return false;
        }

        /* if let Ok(frame_creator) = window.get_frame_creator() {
            if frame_creator.is_none() {
                return false;
            }
        }

        if WindowsApi::window_is_uwp_suspended(window.hwnd()).unwrap_or_default() {
            return false;
        } */

        /* if let Some(config) = FULL_STATE.load().get_app_config_by_window(hwnd) {
            if config.options.contains(&AppExtraFlag::Hidden) {
                log::trace!("Skipping by config: {:?}", window);
                return false;
            }
        }

        !TITLE_BLACK_LIST.contains(&window.title().as_str()) */
        true
    }
}
