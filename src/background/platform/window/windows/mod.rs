mod interactable;
mod snapshot;

pub(super) use interactable::is_interactable_window;

use std::{
    fmt::{Debug, Display},
    path::PathBuf,
    sync::LazyLock,
};

use seelen_core::{rect::Rect, system_state::Relaunch};
use slu_ipc::messages::SvcAction;
use windows::{
    ApplicationModel::AppInfo,
    Win32::{
        Foundation::{HWND, RECT},
        UI::{
            Shell::FOLDERID_System,
            WindowsAndMessaging::{
                SET_WINDOW_POS_FLAGS, SHOW_WINDOW_CMD, SW_FORCEMINIMIZE, SW_HIDE, SW_MAXIMIZE,
                SW_MINIMIZE, SW_NORMAL, SW_RESTORE, SW_SHOW, SW_SHOWNOACTIVATE,
            },
        },
    },
};

use crate::{
    cli::ServicePipe,
    error::Result,
    platform::{
        hook::{HookManager, WindowEvent},
        monitor::Monitor,
        process::{Process, WindowsProcessExt},
    },
    system_modules::start::StartMenuManager,
    windows_api::{WindowEnumerator, WindowsApi, types::AppUserModelId},
};

use super::{Window, WindowState};

static APP_FRAME_HOST_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    WindowsApi::known_folder(FOLDERID_System)
        .expect("Failed to get system folder")
        .join("ApplicationFrameHost.exe")
});

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct NativeWindow(HWND);

unsafe impl Send for NativeWindow {}
unsafe impl Sync for NativeWindow {}

impl NativeWindow {
    pub fn get_foregrounded() -> Self {
        Self(WindowsApi::get_foreground_window())
    }

    pub fn enumerate() -> Result<Vec<Self>> {
        WindowEnumerator::new().map(Self)
    }

    pub fn address(&self) -> isize {
        self.0.0 as isize
    }

    pub fn is_electron(&self) -> bool {
        self.class() == "Chrome_WidgetWin_1"
    }

    pub fn title(&self) -> String {
        WindowsApi::get_window_text(self.0)
    }

    pub fn class(&self) -> String {
        WindowsApi::get_class(self.0).unwrap_or_default()
    }

    pub fn process_id(&self) -> u32 {
        WindowsApi::window_thread_process_id(self.0).0
    }

    pub fn app_display_name(&self) -> Result<String> {
        if let Some(umid) = self.app_user_model_id() {
            match umid {
                AppUserModelId::Appx(umid) => {
                    let info = AppInfo::GetFromAppUserModelId(&umid.into())?;
                    return Ok(info.DisplayInfo()?.DisplayName()?.to_string_lossy());
                }
                AppUserModelId::PropertyStore(ref umid) => {
                    if let Some(display_name) = self.relaunch_display_name() {
                        return Ok(display_name);
                    }
                    let guard = StartMenuManager::instance();
                    if let Some(item) = guard.get_by_file_umid(umid) {
                        return Ok(item.display_name.clone());
                    }
                }
            }
        }
        self.process().display_name()
    }

    pub fn outer_rect(&self) -> Result<Rect> {
        Ok(from_win32_rect(WindowsApi::get_outer_window_rect(self.0)?))
    }

    pub fn inner_rect(&self) -> Result<Rect> {
        Ok(from_win32_rect(WindowsApi::get_inner_window_rect(self.0)?))
    }

    pub fn parent(&self) -> Option<Self> {
        WindowsApi::get_parent(self.0).ok().map(Self)
    }

    pub fn owner(&self) -> Option<Self> {
        WindowsApi::get_owner(self.0).ok().map(Self)
    }

    pub fn children(&self) -> Result<Vec<Self>> {
        WindowEnumerator::new().with_parent(self.0).map(Self)
    }

    pub fn monitor(&self) -> Monitor {
        Monitor::from(WindowsApi::monitor_from_window(self.0))
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
        WindowsApi::is_zoomed(self.0)
    }

    pub fn is_cloaked(&self) -> bool {
        WindowsApi::is_cloaked(self.0).unwrap_or(false)
    }

    pub fn is_focused(&self) -> bool {
        WindowsApi::get_foreground_window() == self.0
    }

    pub fn is_fullscreen(&self) -> bool {
        WindowsApi::is_fullscreen(self.0).unwrap_or(false)
            && !self.is_desktop()
            && !self.process().is_seelen() // we ignore seelen widgets
    }

    /// this means all windows that are part of the UI desktop not the real desktop window
    pub fn is_desktop(&self) -> bool {
        WindowsApi::get_desktop_window() == self.0 || {
            let class = self.class();
            class == "Progman" || {
                class == "WorkerW"
                    && self.children().is_ok_and(|children| {
                        children
                            .iter()
                            .any(|child| child.class() == "SHELLDLL_DefView")
                    })
            }
        }
    }

    pub fn set_state(&self, state: WindowState) -> Result<()> {
        let command = to_show_command(state);
        if self.is_manageable_from_unelevated() {
            WindowsApi::show_window(self.0, command)
        } else {
            ServicePipe::request(SvcAction::ShowWindow {
                hwnd: self.address(),
                command: command.0,
            })
        }
    }

    pub fn set_state_async(&self, state: WindowState) -> Result<()> {
        let command = to_show_command(state);
        if self.is_manageable_from_unelevated() {
            WindowsApi::show_window_async(self.0, command)
        } else {
            ServicePipe::request(SvcAction::ShowWindowAsync {
                hwnd: self.address(),
                command: command.0,
            })
        }
    }

    pub fn set_rect(&self, rect: &Rect) -> Result<()> {
        let flags = SET_WINDOW_POS_FLAGS::default();
        if self.is_manageable_from_unelevated() {
            WindowsApi::set_position(self.0, None, &to_win32_rect(rect), flags)
        } else {
            ServicePipe::request(SvcAction::SetWindowPosition {
                hwnd: self.address(),
                rect: rect.clone(),
                flags: flags.0,
            })
        }
    }

    /// The caller already skips minimized windows.
    pub fn focus(&self) -> Result<()> {
        if Self::get_foregrounded().is_manageable_from_unelevated() {
            return WindowsApi::set_foreground(self.0);
        }
        ServicePipe::request_sync(SvcAction::SetForeground(self.address()))?;
        if WindowsApi::get_foreground_window() == self.0 {
            HookManager::send((WindowEvent::Foregrounded, Window(*self)));
            Ok(())
        } else {
            Err("Failed to set foreground window".into())
        }
    }

    // Windows only, exposed through `WindowsWindowExt`.

    /// Application user model id asigned to the window via property-store or inherited from the process
    ///
    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-id
    fn app_user_model_id(&self) -> Option<AppUserModelId> {
        if let Ok(umid) = WindowsApi::get_window_app_user_model_id(self.0) {
            return match WindowsApi::is_uwp_package_id(&umid) {
                true => Some(AppUserModelId::Appx(umid)),
                false => Some(AppUserModelId::PropertyStore(umid)),
            };
        }

        let process = self.process();
        if let Ok(umid) = process.package_app_user_model_id() {
            return Some(umid);
        }

        if self.is_electron() {
            let path = process.exe_path().ok()?;

            // special manual case like there's no way to call GetCurrentProcessExplicitAppUserModelID without code injection
            if path.file_name()?.to_string_lossy().to_lowercase() == "discord.exe" {
                return Some(AppUserModelId::PropertyStore(
                    "com.squirrel.Discord.Discord".to_string(),
                ));
            }

            let guard = StartMenuManager::instance();
            let item = guard.get_by_target(&path)?;
            Some(AppUserModelId::PropertyStore(item.umid.clone()?))
        } else {
            None
        }
    }

    /// Cloaked by a Seelen buffered animation (transient), not by the app or the shell.
    fn is_cloaked_by_seelen(&self) -> bool {
        WindowsApi::has_prop(self.0, "SluCloaked")
    }

    /// Removed from the taskbar via `ITaskbarList::DeleteTab`, which marks the window with this prop.
    fn is_taskbar_tab_deleted(&self) -> bool {
        WindowsApi::has_prop(self.0, "ITaskList_Deleted")
    }

    /// is the window an Application Frame Host
    fn is_frame(&self) -> Result<bool> {
        Ok(self
            .process()
            .exe_path()?
            .as_os_str()
            .eq_ignore_ascii_case(&*APP_FRAME_HOST_PATH))
    }

    /// will fail if the window is not a frame
    fn get_frame_creator(&self) -> Result<Option<Self>> {
        if !self.is_frame()? {
            return Err("Window is not a frame".into());
        }

        let title = Some(self.title());
        let class = Some("Windows.UI.Core.CoreWindow".to_owned());

        // frame creator is a child of the window while rendering
        if let Ok(hwnd) = WindowsApi::find_window(Some(self.0), None, title.clone(), class.clone())
        {
            return Ok(Some(Self(hwnd)));
        }

        // while minimized, not rendering, the creator is detached as an top level window
        if let Ok(hwnd) = WindowsApi::find_window(None, None, title, class) {
            return Ok(Some(Self(hwnd)));
        }

        Ok(None)
    }

    /// windows created on elevated process like task manager are not manageable from a non elevated process
    fn is_manageable_from_unelevated(&self) -> bool {
        self.process().open_handle().is_ok()
    }

    // Internal helpers.

    fn process(&self) -> Process {
        Process::from_id(self.process_id())
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-preventpinning
    fn prevent_pinning(&self) -> bool {
        WindowsApi::get_window_prevent_pinning(self.0).unwrap_or(false)
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchcommand
    fn relaunch_command(&self) -> Option<String> {
        WindowsApi::get_window_relaunch_command(self.0).ok()
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchdisplaynameresource
    fn relaunch_display_name(&self) -> Option<String> {
        if let Ok(name) = WindowsApi::get_window_relaunch_display_name(self.0) {
            if name.starts_with("@") {
                return WindowsApi::resolve_indirect_string(&name).ok();
            }
            return Some(name);
        }
        None
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchiconresource
    fn relaunch_icon(&self) -> Option<String> {
        WindowsApi::get_window_relaunch_icon_resource(self.0).ok()
    }
}

impl From<isize> for NativeWindow {
    fn from(address: isize) -> Self {
        Self(HWND(address as _))
    }
}

impl Debug for NativeWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Window")
            .field("handle", &self.0.0)
            .field("exe", &self.process().exe_name().unwrap_or_default())
            .field("class", &self.class())
            .field("title", &self.title())
            .finish()
    }
}

impl Display for NativeWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Window({:?})", self.0.0)
    }
}

/// Windows only concepts of a window, use it only from Windows specific code.
pub trait WindowsWindowExt {
    fn hwnd(&self) -> HWND;
    fn app_user_model_id(&self) -> Option<AppUserModelId>;
    fn relaunch_info(&self, umid: &Option<AppUserModelId>) -> Option<(Relaunch, bool)>;
    fn is_cloaked_by_seelen(&self) -> bool;
    fn is_taskbar_tab_deleted(&self) -> bool;
    fn is_frame(&self) -> Result<bool>;
    fn get_frame_creator(&self) -> Result<Option<Window>>;
    fn is_manageable_from_unelevated(&self) -> bool;
}

impl WindowsWindowExt for Window {
    fn hwnd(&self) -> HWND {
        self.0.0
    }

    fn app_user_model_id(&self) -> Option<AppUserModelId> {
        self.0.app_user_model_id()
    }

    fn relaunch_info(&self, umid: &Option<AppUserModelId>) -> Option<(Relaunch, bool)> {
        self.0.relaunch_info(umid)
    }

    fn is_cloaked_by_seelen(&self) -> bool {
        self.0.is_cloaked_by_seelen()
    }

    fn is_taskbar_tab_deleted(&self) -> bool {
        self.0.is_taskbar_tab_deleted()
    }

    fn is_frame(&self) -> Result<bool> {
        self.0.is_frame()
    }

    fn get_frame_creator(&self) -> Result<Option<Window>> {
        Ok(self.0.get_frame_creator()?.map(Window))
    }

    fn is_manageable_from_unelevated(&self) -> bool {
        self.0.is_manageable_from_unelevated()
    }
}

impl From<HWND> for Window {
    fn from(hwnd: HWND) -> Self {
        Self(NativeWindow(hwnd))
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

fn from_win32_rect(rect: RECT) -> Rect {
    Rect {
        left: rect.left,
        top: rect.top,
        right: rect.right,
        bottom: rect.bottom,
    }
}
