use std::{
    fmt::{Debug, Display},
    path::PathBuf,
};

use seelen_core::{
    rect::Rect,
    state::{AppConfig, WorkspaceId},
    system_state::{FocusedApp, MonitorId, Relaunch, UserAppWindow},
};
use windows::Win32::{
    Foundation::{HWND, RECT},
    UI::WindowsAndMessaging::{
        SET_WINDOW_POS_FLAGS, SHOW_WINDOW_CMD, SW_FORCEMINIMIZE, SW_HIDE, SW_MAXIMIZE, SW_MINIMIZE,
        SW_NORMAL, SW_RESTORE, SW_SHOW, SW_SHOWNOACTIVATE,
    },
};

use crate::{
    error::Result,
    platform::monitor::Monitor,
    windows_api::{
        WindowEnumerator, process::Process, types::AppUserModelId, window::Window as Win32Window,
    },
};

use super::{NativeWindow, Window, WindowState};

/// Delegates to the `HWND` based implementation of `windows_api`, so the behavior (and the
/// shared state like the drag tracking) is exactly the same while both types coexist.
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct WindowsWindow(Win32Window);

impl NativeWindow for WindowsWindow {
    fn get_foregrounded() -> Self {
        Self(Win32Window::get_foregrounded())
    }

    fn enumerate() -> Result<Vec<Self>> {
        WindowEnumerator::new().map(|hwnd| Self(Win32Window::from(hwnd)))
    }

    fn address(&self) -> isize {
        self.0.address()
    }

    fn is_electron(&self) -> bool {
        self.0.is_electron()
    }

    fn title(&self) -> String {
        self.0.title()
    }

    fn class(&self) -> String {
        self.0.class()
    }

    fn process_id(&self) -> u32 {
        self.0.process().id()
    }

    fn exe_path(&self) -> Result<PathBuf> {
        self.0.process().program_path()
    }

    fn app_display_name(&self) -> Result<String> {
        self.0.app_display_name()
    }

    fn outer_rect(&self) -> Result<Rect> {
        self.0.outer_rect()
    }

    fn inner_rect(&self) -> Result<Rect> {
        self.0.inner_rect()
    }

    fn parent(&self) -> Option<Self> {
        self.0.parent().map(Self)
    }

    fn owner(&self) -> Option<Self> {
        self.0.owner().map(Self)
    }

    fn children(&self) -> Result<Vec<Self>> {
        Ok(self.0.children()?.into_iter().map(Self).collect())
    }

    fn monitor(&self) -> Monitor {
        Monitor::from(self.0.monitor())
    }

    fn monitor_id(&self) -> MonitorId {
        self.0.monitor_id()
    }

    fn slu_category(&self) -> Option<String> {
        self.0.slu_category()
    }

    fn workspace_id(&self) -> Result<WorkspaceId> {
        self.0.workspace_id()
    }

    fn get_app_config(&self) -> Result<Option<AppConfig>> {
        self.0.get_app_config()
    }

    fn snapshot(&self) -> UserAppWindow {
        self.0.snapshot()
    }

    fn focused_app_snapshot(&self) -> FocusedApp {
        self.0.focused_app_snapshot()
    }

    fn is_window(&self) -> bool {
        self.0.is_window()
    }

    fn is_visible(&self) -> bool {
        self.0.is_visible()
    }

    fn is_minimized(&self) -> bool {
        self.0.is_minimized()
    }

    fn is_maximized(&self) -> bool {
        self.0.is_maximized()
    }

    fn is_cloaked(&self) -> bool {
        self.0.is_cloaked()
    }

    fn is_focused(&self) -> bool {
        self.0.is_focused()
    }

    fn is_fullscreen(&self) -> bool {
        self.0.is_fullscreen()
    }

    fn is_desktop(&self) -> bool {
        self.0.is_desktop()
    }

    fn is_interactable_and_not_hidden(&self) -> bool {
        self.0.is_interactable_and_not_hidden()
    }

    fn set_state(&self, state: WindowState) -> Result<()> {
        self.0.show_window(to_show_command(state))
    }

    fn set_state_async(&self, state: WindowState) -> Result<()> {
        self.0.show_window_async(to_show_command(state))
    }

    fn set_rect(&self, rect: &Rect) -> Result<()> {
        self.0
            .set_position(&to_win32_rect(rect), SET_WINDOW_POS_FLAGS::default())
    }

    fn unminimize(&self) -> Result<()> {
        self.0.unminimize()
    }

    fn focus(&self) -> Result<()> {
        self.0.focus()
    }

    fn is_dragging(&self) -> bool {
        self.0.is_dragging()
    }

    fn is_last_dragged(&self) -> bool {
        self.0.is_last_dragged()
    }

    fn set_dragging(&self, dragging: bool) {
        self.0.set_dragging(dragging)
    }

    fn get_rect_before_dragging(&self) -> Result<Rect> {
        self.0.get_rect_before_dragging()
    }
}

impl From<isize> for WindowsWindow {
    fn from(address: isize) -> Self {
        Self(Win32Window::from(address))
    }
}

impl Debug for WindowsWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl Display for WindowsWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

/// Windows only concepts of a window, use it only from Windows specific code.
pub trait WindowsWindowExt {
    fn hwnd(&self) -> HWND;
    fn process(&self) -> Process;
    fn app_user_model_id(&self) -> Option<AppUserModelId>;
    fn relaunch_info(&self, umid: &Option<AppUserModelId>) -> Option<(Relaunch, bool)>;
    fn prevent_pinning(&self) -> bool;
    fn relaunch_command(&self) -> Option<String>;
    fn relaunch_display_name(&self) -> Option<String>;
    fn relaunch_icon(&self) -> Option<String>;
    fn is_cloaked_by_seelen(&self) -> bool;
    fn is_taskbar_tab_deleted(&self) -> bool;
    fn is_frame(&self) -> Result<bool>;
    fn get_frame_creator(&self) -> Result<Option<Window>>;
    fn is_manageable_from_unelevated(&self) -> bool;
    fn show_window(&self, command: SHOW_WINDOW_CMD) -> Result<()>;
    fn show_window_async(&self, command: SHOW_WINDOW_CMD) -> Result<()>;
    fn set_position(&self, rect: &RECT, flags: SET_WINDOW_POS_FLAGS) -> Result<()>;
}

impl WindowsWindowExt for Window {
    fn hwnd(&self) -> HWND {
        self.0.0.hwnd()
    }

    fn process(&self) -> Process {
        self.0.0.process()
    }

    fn app_user_model_id(&self) -> Option<AppUserModelId> {
        self.0.0.app_user_model_id()
    }

    fn relaunch_info(&self, umid: &Option<AppUserModelId>) -> Option<(Relaunch, bool)> {
        self.0.0.relaunch_info(umid)
    }

    fn prevent_pinning(&self) -> bool {
        self.0.0.prevent_pinning()
    }

    fn relaunch_command(&self) -> Option<String> {
        self.0.0.relaunch_command()
    }

    fn relaunch_display_name(&self) -> Option<String> {
        self.0.0.relaunch_display_name()
    }

    fn relaunch_icon(&self) -> Option<String> {
        self.0.0.relaunch_icon()
    }

    fn is_cloaked_by_seelen(&self) -> bool {
        self.0.0.is_cloaked_by_seelen()
    }

    fn is_taskbar_tab_deleted(&self) -> bool {
        self.0.0.is_taskbar_tab_deleted()
    }

    fn is_frame(&self) -> Result<bool> {
        self.0.0.is_frame()
    }

    fn get_frame_creator(&self) -> Result<Option<Window>> {
        Ok(self.0.0.get_frame_creator()?.map(Window::from))
    }

    fn is_manageable_from_unelevated(&self) -> bool {
        self.0.0.is_manageable_from_unelevated()
    }

    fn show_window(&self, command: SHOW_WINDOW_CMD) -> Result<()> {
        self.0.0.show_window(command)
    }

    fn show_window_async(&self, command: SHOW_WINDOW_CMD) -> Result<()> {
        self.0.0.show_window_async(command)
    }

    fn set_position(&self, rect: &RECT, flags: SET_WINDOW_POS_FLAGS) -> Result<()> {
        self.0.0.set_position(rect, flags)
    }
}

// Interop with `windows_api` while the app migrates to the platform layer.

impl From<HWND> for Window {
    fn from(hwnd: HWND) -> Self {
        Self(WindowsWindow(Win32Window::from(hwnd)))
    }
}

impl From<Win32Window> for Window {
    fn from(window: Win32Window) -> Self {
        Self(WindowsWindow(window))
    }
}

impl From<Window> for Win32Window {
    fn from(window: Window) -> Self {
        window.0.0
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

fn to_win32_rect(rect: &Rect) -> RECT {
    RECT {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}
