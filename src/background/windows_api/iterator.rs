use std::slice::Iter;

use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT},
    Graphics::Gdi::{HDC, HMONITOR},
    UI::WindowsAndMessaging::{EnumChildWindows, EnumWindows},
};

use crate::{error_handler::Result, windows_api::WindowsApi};

use super::window::Window;

#[derive(Debug, Clone)]
pub struct WindowEnumerator {
    parent: Option<HWND>,
}

impl WindowEnumerator {
    pub fn new() -> Self {
        Self { parent: None }
    }

    pub fn with_parent(mut self, parent: HWND) -> Self {
        self.parent = Some(parent);
        self
    }

    fn enumerate(
        &self,
        enum_proc: unsafe extern "system" fn(HWND, LPARAM) -> BOOL,
        ptr: LPARAM,
    ) -> Result<()> {
        if let Some(parent) = self.parent {
            unsafe { EnumChildWindows(parent, Some(enum_proc), ptr).ok()? };
        } else {
            unsafe { EnumWindows(Some(enum_proc), ptr)? };
        }
        Ok(())
    }

    /// Will call the callback for each window while enumerating.
    /// If enumeration fails it will return error.
    pub fn for_each<F>(&self, cb: F) -> Result<()>
    where
        F: FnMut(HWND) + Sync,
    {
        type ForEachCallback<'a> = Box<dyn FnMut(HWND) + 'a>;
        let mut callback: ForEachCallback = Box::new(cb);

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            if let Some(boxed) = (lparam.0 as *mut ForEachCallback).as_mut() {
                (*boxed)(hwnd)
            }
            true.into()
        }

        self.enumerate(enum_proc, LPARAM(&mut callback as *mut _ as isize))
    }

    /// Will call the callback for each window while enumerating.
    /// If enumeration fails it will return error.
    pub fn map<F, T>(&self, cb: F) -> Result<Vec<T>>
    where
        F: FnMut(HWND) -> T + Sync,
        T: Sync,
    {
        struct MapCallbackWrapper<'a, T> {
            cb: Box<dyn FnMut(HWND) -> T + 'a>,
            processed: Vec<T>,
        }

        unsafe extern "system" fn enum_proc<T>(hwnd: HWND, lparam: LPARAM) -> BOOL {
            if let Some(wrapper) = (lparam.0 as *mut MapCallbackWrapper<T>).as_mut() {
                wrapper.processed.push((wrapper.cb)(hwnd));
            }
            true.into()
        }

        let mut wrapper = MapCallbackWrapper {
            cb: Box::new(cb),
            processed: Vec::new(),
        };

        self.enumerate(enum_proc::<T>, LPARAM(&mut wrapper as *mut _ as isize))?;
        Ok(wrapper.processed)
    }

    /// Will return the first window that matches the condition specified by the callback.
    /// If no window matches the condition, it will return None.
    pub fn find<F>(&self, cb: F) -> Result<Option<Window>>
    where
        F: FnMut(Window) -> bool + Sync,
    {
        struct FindCallbackWrapper<'a> {
            cb: Box<dyn FnMut(Window) -> bool + 'a>,
            result: Option<Window>,
        }

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            if let Some(wrapper) = (lparam.0 as *mut FindCallbackWrapper).as_mut() {
                if wrapper.result.is_none() && (wrapper.cb)(Window::from(hwnd)) {
                    wrapper.result = Some(Window::from(hwnd));
                    // for some reason returning false is not stopping the enumeration
                    // return false.into();
                }
            }
            true.into()
        }

        let mut wrapper = FindCallbackWrapper {
            cb: Box::new(cb),
            result: None,
        };

        self.enumerate(enum_proc, LPARAM(&mut wrapper as *mut _ as isize))?;
        Ok(wrapper.result)
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
