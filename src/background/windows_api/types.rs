use serde::{Deserialize, Serialize};

use super::WindowsApi;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
