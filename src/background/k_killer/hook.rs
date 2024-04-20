use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{
        EVENT_OBJECT_CREATE, EVENT_OBJECT_DESTROY, EVENT_OBJECT_SHOW, EVENT_OBJECT_UNCLOAKED,
    },
};

use crate::{
    error_handler::{log_if_error, Result},
    seelen::SEELEN,
    utils::virtual_desktop::VirtualDesktopManager,
};

use super::WindowManager;

impl WindowManager {
    pub fn process_event(event: u32, _hwnd: HWND) -> Result<()> {
        match event {
            EVENT_OBJECT_CREATE
            | EVENT_OBJECT_DESTROY
            | EVENT_OBJECT_SHOW
            | EVENT_OBJECT_UNCLOAKED => {
                let mut seelen = SEELEN.lock();
                if let Some(wm) = seelen.wm_mut() {
                    if let Ok(virtual_desktop) = VirtualDesktopManager::get_current_virtual_desktop() {
                        log_if_error(wm.set_active_workspace(virtual_desktop.id()));
                    };
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
