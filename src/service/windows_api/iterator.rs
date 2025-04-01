use windows::Win32::{
    Foundation::{BOOL, HWND, LPARAM},
    UI::WindowsAndMessaging::{EnumChildWindows, EnumWindows},
};

use crate::error::Result;

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
        F: FnMut(HWND),
    {
        type ForEachCallback<'a> = Box<dyn FnMut(HWND) + 'a>;
        let mut callback: ForEachCallback = Box::new(cb);

        unsafe extern "system" fn enum_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
            if let Some(boxed) = (lparam.0 as *mut ForEachCallback).as_mut() {
                (*boxed)(hwnd);
            }
            true.into()
        }

        self.enumerate(enum_proc, LPARAM(&mut callback as *mut _ as isize))
    }
}
