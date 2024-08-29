use std::slice::Iter;

use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT},
    Graphics::Gdi::{HDC, HMONITOR},
    UI::WindowsAndMessaging::EnumChildWindows,
};

use crate::{error_handler::Result, windows_api::WindowsApi};

#[derive(Debug, Clone)]
pub struct WindowEnumerator {
    parent: Option<HWND>,
    handles: Vec<HWND>,
}

impl IntoIterator for WindowEnumerator {
    type Item = HWND;
    type IntoIter = std::vec::IntoIter<HWND>;

    fn into_iter(self) -> Self::IntoIter {
        self.handles.into_iter()
    }
}

impl WindowEnumerator {
    pub fn new(parent: Option<HWND>) -> Self {
        Self {
            parent,
            handles: Vec::new(),
        }
    }

    pub fn new_refreshed() -> Result<Self> {
        let mut enumerator = Self::new(None);
        enumerator.refresh()?;
        Ok(enumerator)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.handles.clear();
        let ptr = &mut self.handles as *mut _ as isize;

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            let data_ptr = lparam.0 as *mut Vec<HWND>;
            if let Some(data) = data_ptr.as_mut() {
                data.push(hwnd);
            }
            true.into()
        }

        if let Some(parent) = self.parent {
            unsafe { EnumChildWindows(parent, Some(enum_proc), LPARAM(ptr)).ok()? };
        } else {
            WindowsApi::enum_windows(Some(enum_proc), ptr)?;
        }
        Ok(())
    }

    pub fn iter(&self) -> Iter<'_, HWND> {
        self.handles.iter()
    }
}

#[derive(Debug, Clone, Default)]
pub struct MonitorEnumerator {
    handles: Vec<HMONITOR>,
}

impl IntoIterator for MonitorEnumerator {
    type Item = HMONITOR;
    type IntoIter = std::vec::IntoIter<HMONITOR>;

    fn into_iter(self) -> Self::IntoIter {
        self.handles.into_iter()
    }
}

impl MonitorEnumerator {
    pub fn new_refreshed() -> Result<Self> {
        let mut enumerator = Self::default();
        enumerator.refresh()?;
        Ok(enumerator)
    }

    pub fn refresh(&mut self) -> Result<()> {
        self.handles.clear();

        unsafe extern "system" fn get_handles_proc(
            hmonitor: HMONITOR,
            _hdc: HDC,
            _rect_clip: *mut RECT,
            lparam: LPARAM,
        ) -> BOOL {
            let data_ptr = lparam.0 as *mut Vec<HMONITOR>;
            if let Some(data) = data_ptr.as_mut() {
                data.push(hmonitor);
            }
            true.into()
        }

        WindowsApi::enum_display_monitors(
            Some(get_handles_proc),
            &mut self.handles as *mut _ as isize,
        )
    }

    pub fn iter(&self) -> Iter<'_, HMONITOR> {
        self.handles.iter()
    }
}
