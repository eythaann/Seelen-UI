use super::WindowsApi;

#[derive(Debug, Clone)]
pub enum AppUserModelId {
    PropertyStore(String),
    Appx(String),
}

impl AppUserModelId {
    pub fn is_appx(&self) -> bool {
        matches!(self, AppUserModelId::Appx(_))
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
