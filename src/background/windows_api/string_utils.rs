use std::{
    ffi::{OsStr, OsString},
    os::windows::ffi::{OsStrExt, OsStringExt},
};

use windows_core::{HSTRING, PCWSTR, PWSTR};

#[derive(Debug, Clone)]
pub struct WindowsString {
    pub inner: Vec<u16>,
}

impl WindowsString {
    pub fn new_to_fill(capacity: usize) -> Self {
        Self {
            inner: vec![0; capacity],
        }
    }

    pub fn from_slice(s: &[u16]) -> Self {
        Self { inner: s.to_vec() }
    }

    pub fn from_str(s: &str) -> Self {
        Self {
            inner: s.encode_utf16().chain(Some(0)).collect(),
        }
    }

    pub fn from_os_string(s: &OsStr) -> Self {
        Self {
            inner: s.encode_wide().chain(Some(0)).collect(),
        }
    }

    pub fn len(&self) -> usize {
        self.inner
            .iter()
            .position(|c| c == &0)
            .expect("Invalid UTF16 Windows String")
    }

    pub fn as_mut_slice(&mut self) -> &mut [u16] {
        &mut self.inner
    }

    pub fn as_pcwstr(&self) -> PCWSTR {
        PCWSTR(self.inner.as_ptr())
    }

    pub fn as_pwstr(&mut self) -> PWSTR {
        PWSTR(self.inner.as_mut_ptr())
    }

    pub fn to_hstring(&self) -> HSTRING {
        HSTRING::from_wide(&self.inner[..self.len()])
    }

    pub fn to_os_string(&self) -> std::ffi::OsString {
        std::ffi::OsString::from_wide(&self.inner[..self.len()])
    }
}

impl std::fmt::Display for WindowsString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            String::from_utf16_lossy(&self.inner[..self.len()]).trim_end_matches("\0")
        )
    }
}

impl From<&str> for WindowsString {
    fn from(value: &str) -> Self {
        Self::from_str(value)
    }
}

impl From<String> for WindowsString {
    fn from(value: String) -> Self {
        Self::from_str(&value)
    }
}

impl From<&String> for WindowsString {
    fn from(value: &String) -> Self {
        Self::from_str(value)
    }
}

impl From<&OsStr> for WindowsString {
    fn from(value: &OsStr) -> Self {
        Self::from_os_string(value)
    }
}

impl From<OsString> for WindowsString {
    fn from(value: OsString) -> Self {
        Self::from_os_string(&value)
    }
}

impl From<&[u16]> for WindowsString {
    fn from(value: &[u16]) -> Self {
        Self::from_slice(value)
    }
}
