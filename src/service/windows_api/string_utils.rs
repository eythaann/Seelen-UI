use windows_core::PCWSTR;

/// Null terminated UTF-16 string, to pass `&str`s to Win32.
#[derive(Debug, Clone)]
pub struct WindowsString {
    pub inner: Vec<u16>,
}

impl WindowsString {
    pub fn from_str<S: AsRef<str>>(s: S) -> Self {
        Self {
            inner: s.as_ref().encode_utf16().chain(Some(0)).collect(),
        }
    }

    // pcwstr: pointer constant wide string
    pub fn as_pcwstr(&self) -> PCWSTR {
        PCWSTR(self.inner.as_ptr())
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

impl From<&str> for WindowsString {
    fn from(value: &str) -> Self {
        Self::from_str(value)
    }
}

impl From<String> for WindowsString {
    fn from(value: String) -> Self {
        Self::from_str(value)
    }
}

impl From<&String> for WindowsString {
    fn from(value: &String) -> Self {
        Self::from_str(value)
    }
}
