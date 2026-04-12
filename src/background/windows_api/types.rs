use serde::{Deserialize, Serialize};
use windows::Foundation::DateTime;

use super::WindowsApi;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum AppUserModelId {
    /// aumid added to the app start menu shortcut (eg: "com.squirrel.Discord.Discord")
    PropertyStore(String),
    /// Appx/Msix aumid (eg: "Microsoft.WindowsTerminal_8wekyb3d8bbwe!TerminalApp")
    Appx(String),
}

impl AppUserModelId {
    pub fn is_appx(&self) -> bool {
        matches!(self, AppUserModelId::Appx(_))
    }

    pub fn is_property_store(&self) -> bool {
        matches!(self, AppUserModelId::PropertyStore(_))
    }
}

impl std::fmt::Display for AppUserModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::ops::Deref for AppUserModelId {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        match self {
            AppUserModelId::PropertyStore(id) => id,
            AppUserModelId::Appx(id) => id,
        }
    }
}

impl From<AppUserModelId> for windows_core::HSTRING {
    fn from(val: AppUserModelId) -> Self {
        val.to_string().into()
    }
}

impl From<String> for AppUserModelId {
    fn from(value: String) -> Self {
        if WindowsApi::is_uwp_package_id(&value) {
            AppUserModelId::Appx(value)
        } else {
            AppUserModelId::PropertyStore(value)
        }
    }
}

/// Extension trait for converting Windows `DateTime` to standard Unix timestamps.
pub trait DateTimeExt {
    /// Returns Unix epoch milliseconds (ms since 1970-01-01 UTC).
    fn to_unix_ms(self) -> i64;
}

impl DateTimeExt for DateTime {
    fn to_unix_ms(self) -> i64 {
        // UniversalTime: 100-ns ticks since 1601-01-01 UTC.
        // Offset between Windows epoch (1601) and Unix epoch (1970): 11 644 473 600 s.
        self.UniversalTime / 10_000 - 11_644_473_600_000
    }
}
