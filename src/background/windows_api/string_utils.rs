use windows_core::{PCWSTR, PWSTR};

pub struct WindowsString {
    pub inner: Vec<u16>,
}

impl WindowsString {
    pub fn new_to_fill(len: usize) -> Self {
        Self {
            inner: vec![0; len],
        }
    }

    pub fn as_pcwstr(&self) -> PCWSTR {
        PCWSTR(self.inner.as_ptr())
    }

    pub fn as_pwstr(&mut self) -> PWSTR {
        PWSTR(self.inner.as_mut_ptr())
    }
}

impl std::fmt::Display for WindowsString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            String::from_utf16_lossy(&self.inner).trim_end_matches("\0")
        )
    }
}
