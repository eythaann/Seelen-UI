use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM, RECT},
    Graphics::Gdi::{HDC, HMONITOR},
    UI::WindowsAndMessaging::{EnumChildWindows, EnumWindows},
};

use crate::{error_handler::Result, windows_api::WindowsApi};

use super::{monitor::Monitor, window::Window};

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
            unsafe { EnumChildWindows(Some(parent), Some(enum_proc), ptr).ok()? };
        } else {
            unsafe { EnumWindows(Some(enum_proc), ptr)? };
        }
        Ok(())
    }

    /// Will call the callback for each window while enumerating.
    /// If enumeration fails it will return error.
    pub fn for_each<F>(&self, cb: F) -> Result<()>
    where
        F: FnMut(Window),
    {
        type ForEachCallback<'a> = Box<dyn FnMut(Window) + 'a>;
        let mut callback: ForEachCallback = Box::new(cb);

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            if let Some(boxed) = (lparam.0 as *mut ForEachCallback).as_mut() {
                (*boxed)(Window::from(hwnd));
            }
            true.into()
        }

        self.enumerate(enum_proc, LPARAM(&mut callback as *mut _ as isize))
    }

    pub fn for_each_and_descendants<F>(&self, cb: F) -> Result<()>
    where
        F: FnMut(Window),
    {
        let mut cb = cb;
        self.for_each_and_descendants_impl(&mut cb)
    }

    fn for_each_and_descendants_impl<F>(&self, cb: &mut F) -> Result<()>
    where
        F: FnMut(Window),
    {
        self.for_each(|window| {
            cb(window);
            // ignore errors on recursive children enums
            let _ = Self::new()
                .with_parent(window.hwnd())
                .for_each_and_descendants_impl(cb);
        })
    }

    /// Will call the callback for each window while enumerating.
    /// If enumeration fails it will return error.
    pub fn map<F, T>(&self, cb: F) -> Result<Vec<T>>
    where
        F: FnMut(HWND) -> T,
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
}

#[derive(Debug, Clone, Default)]
pub struct MonitorEnumerator;

impl MonitorEnumerator {
    pub fn get_all_v2() -> Result<Vec<Monitor>> {
        let mut handles = Vec::new();

        unsafe extern "system" fn get_handles_proc(
            hmonitor: HMONITOR,
            _hdc: HDC,
            _rect_clip: *mut RECT,
            lparam: LPARAM,
        ) -> BOOL {
            let data_ptr = lparam.0 as *mut Vec<Monitor>;
            if let Some(data) = data_ptr.as_mut() {
                data.push(Monitor::from(hmonitor));
            }
            true.into()
        }

        WindowsApi::enum_display_monitors(Some(get_handles_proc), &mut handles as *mut _ as isize)?;
        Ok(handles)
    }
}
