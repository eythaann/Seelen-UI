pub mod cache;
pub mod event;

use seelen_core::state::WorkspaceId;
use seelen_core::{rect::Rect, system_state::MonitorId};
use slu_ipc::messages::SvcAction;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::{
    fmt::{Debug, Display},
    path::PathBuf,
    sync::{atomic::AtomicIsize, LazyLock},
};

use windows::{
    ApplicationModel::AppInfo,
    Win32::{
        Foundation::{HWND, RECT},
        UI::{
            Shell::FOLDERID_System,
            WindowsAndMessaging::{SET_WINDOW_POS_FLAGS, SHOW_WINDOW_CMD, SW_RESTORE},
        },
    },
};

use crate::virtual_desktops::SluWorkspacesManager2;
use crate::{
    cli::ServicePipe,
    error::Result,
    modules::{apps::application::is_interactable_window, start::application::StartMenuManager},
    utils::lock_free::TracedMutex,
    widgets::{
        toolbar::FancyToolbar, wallpaper_manager::SeelenWall, weg::instance::SeelenWeg,
        window_manager::instance::WindowManagerV2,
    },
};

use super::{
    monitor::Monitor, process::Process, types::AppUserModelId, WindowEnumerator, WindowsApi,
};

static DRAGGING_WINDOW: AtomicIsize = AtomicIsize::new(0);
static LAST_DRAGGED_WINDOW: AtomicIsize = AtomicIsize::new(0);
static DRAGGED_WINDOW_RECT_BEFORE_DRAG: LazyLock<Arc<TracedMutex<Option<Rect>>>> =
    LazyLock::new(|| Arc::new(TracedMutex::new(None)));

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
            .field(
                "exe",
                &self.process().program_exe_name().unwrap_or_default(),
            )
            .field("class", &self.class())
            .field("title", &self.title())
            .finish()
    }
}

impl Display for Window {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Window({:?})", self.0 .0)
    }
}

static APP_FRAME_HOST_PATH: LazyLock<PathBuf> = LazyLock::new(|| {
    WindowsApi::known_folder(FOLDERID_System)
        .expect("Failed to get system folder")
        .join("ApplicationFrameHost.exe")
});

impl Window {
    pub fn get_foregrounded() -> Window {
        Window(WindowsApi::get_foreground_window())
    }

    pub fn hwnd(&self) -> HWND {
        self.0
    }

    pub fn address(&self) -> isize {
        self.0 .0 as isize
    }

    pub fn is_electron(&self) -> bool {
        self.class() == "Chrome_WidgetWin_1"
    }

    /// Application user model id asigned to the window via property-store or inherited from the process
    ///
    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-id
    pub fn app_user_model_id(&self) -> Option<AppUserModelId> {
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
            let path = process.program_path().ok()?;

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
    #[allow(dead_code)]
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

    pub fn app_display_name(&self) -> Result<String> {
        if let Some(AppUserModelId::Appx(umid)) = self.app_user_model_id() {
            let info = AppInfo::GetFromAppUserModelId(&umid.into())?;
            return Ok(info.DisplayInfo()?.DisplayName()?.to_string_lossy());
        }
        self.process().program_display_name()
    }

    #[allow(dead_code)]
    pub fn outer_rect(&self) -> Result<Rect> {
        let rect = WindowsApi::get_outer_window_rect(self.hwnd())?;
        Ok(Rect {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        })
    }

    pub fn inner_rect(&self) -> Result<Rect> {
        let rect = WindowsApi::get_inner_window_rect(self.hwnd())?;
        Ok(Rect {
            left: rect.left,
            top: rect.top,
            right: rect.right,
            bottom: rect.bottom,
        })
    }

    pub fn parent(&self) -> Option<Window> {
        WindowsApi::get_parent(self.0).ok().map(Window::from)
    }

    pub fn owner(&self) -> Option<Window> {
        WindowsApi::get_owner(self.0).ok().map(Window::from)
    }

    pub fn children(&self) -> Result<Vec<Window>> {
        WindowEnumerator::new()
            .with_parent(self.0)
            .map(Window::from)
    }

    pub fn monitor(&self) -> Monitor {
        Monitor::from(WindowsApi::monitor_from_window(self.0))
    }

    pub fn monitor_id(&self) -> MonitorId {
        self.monitor()
            .stable_id2()
            .unwrap_or_else(|_| MonitorId("null".to_string()))
    }

    pub fn workspace_id(&self) -> Result<WorkspaceId> {
        let win_id = self.address();
        let monitor_id = self.monitor_id();
        let workspace_id = SluWorkspacesManager2::instance()
            .monitors
            .get(&monitor_id, |monitor| {
                monitor.workspaces.iter().find_map(|w| {
                    if w.windows.contains(&win_id) {
                        Some(w.id.clone())
                    } else {
                        None
                    }
                })
            })
            .ok_or("Monitor not found")?
            .ok_or("This window is not binded to a seelen ui workspace")?;
        Ok(workspace_id)
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

    /// is the window an Application Frame Host
    pub fn is_frame(&self) -> Result<bool> {
        Ok(self
            .process()
            .program_path()?
            .as_os_str()
            .eq_ignore_ascii_case(&*APP_FRAME_HOST_PATH))
    }

    /// will fail if the window is not a frame
    pub fn get_frame_creator(&self) -> Result<Option<Window>> {
        if !self.is_frame()? {
            return Err("Window is not a frame".into());
        }

        let title = Some(self.title());
        let class = Some("Windows.UI.Core.CoreWindow".to_owned());

        // frame creator is a child of the window while rendering
        if let Ok(hwnd) =
            WindowsApi::find_window(Some(self.hwnd()), None, title.clone(), class.clone())
        {
            return Ok(Some(Window::from(hwnd)));
        }

        // while minimized, not rendering, the creator is detached as an top level window
        if let Ok(hwnd) = WindowsApi::find_window(None, None, title, class) {
            return Ok(Some(Window::from(hwnd)));
        }

        Ok(None)
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

    pub fn is_seelen_overlay(&self) -> bool {
        self.process().is_seelen() && {
            [
                FancyToolbar::TITLE,
                WindowManagerV2::TITLE,
                SeelenWeg::TITLE,
                SeelenWall::TITLE,
            ]
            .contains(&self.title().as_str())
        }
    }

    /// read inner called doc for more info
    pub fn is_interactable_and_not_hidden(&self) -> bool {
        is_interactable_window(self)
    }

    pub fn show_window(&self, command: SHOW_WINDOW_CMD) -> Result<()> {
        if self.process().open_handle().is_ok() {
            WindowsApi::show_window(self.hwnd(), command)
        } else {
            ServicePipe::request(SvcAction::ShowWindow {
                hwnd: self.address(),
                command: command.0,
            })
        }
    }

    pub fn show_window_async(&self, command: SHOW_WINDOW_CMD) -> Result<()> {
        if self.process().open_handle().is_ok() {
            WindowsApi::show_window_async(self.hwnd(), command)
        } else {
            ServicePipe::request(SvcAction::ShowWindowAsync {
                hwnd: self.address(),
                command: command.0,
            })
        }
    }

    #[allow(dead_code)]
    pub fn set_position(&self, rect: &RECT, flags: SET_WINDOW_POS_FLAGS) -> Result<()> {
        if self.process().open_handle().is_ok() {
            WindowsApi::set_position(self.hwnd(), None, rect, flags)
        } else {
            ServicePipe::request(SvcAction::SetWindowPosition {
                hwnd: self.address(),
                rect: Rect {
                    top: rect.top,
                    left: rect.left,
                    right: rect.right,
                    bottom: rect.bottom,
                },
                flags: flags.0,
            })
        }
    }

    pub fn focus(&self) -> Result<()> {
        if self.is_minimized() {
            self.show_window(SW_RESTORE)?;
        }

        /* if self.process().open_handle().is_ok() {
            WindowsApi::set_foreground(self.hwnd())
        } else {
            ServicePipe::request(SvcAction::SetForeground(self.address()))
        } */
        WindowsApi::set_foreground(self.hwnd())
    }

    pub fn is_dragging(&self) -> bool {
        DRAGGING_WINDOW.load(Ordering::SeqCst) == self.address()
    }

    pub fn is_last_dragged(&self) -> bool {
        LAST_DRAGGED_WINDOW.load(Ordering::SeqCst) == self.address()
    }

    pub fn set_dragging(&self, dragging: bool) {
        if dragging {
            DRAGGING_WINDOW.store(self.address(), Ordering::SeqCst);
            LAST_DRAGGED_WINDOW.store(self.address(), Ordering::SeqCst);
            *DRAGGED_WINDOW_RECT_BEFORE_DRAG.lock() = self.inner_rect().ok();
        } else {
            DRAGGING_WINDOW.store(0, Ordering::SeqCst);
            // *DRAGGING_WINDOW_RECT_BEFORE_DRAG.lock() = None; we don't clean up to allow be used on drag end
        }
    }

    /// if dragging returns the rect of the window before dragging
    /// otherwise returns the current rect
    pub fn get_rect_before_dragging(&self) -> Result<Rect> {
        if self.is_dragging() || self.is_last_dragged() {
            let guard = DRAGGED_WINDOW_RECT_BEFORE_DRAG.lock();
            if let Some(rect) = guard.as_ref() {
                Ok(rect.clone())
            } else {
                self.inner_rect()
            }
        } else {
            self.inner_rect()
        }
    }
}
