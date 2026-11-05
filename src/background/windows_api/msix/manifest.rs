use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PackageManifest {
    pub identity: ManifestIdentity,
    pub properties: ManifestProperties,
    #[serde(default)]
    pub applications: ManifestApplications,
}

impl PackageManifest {
    pub fn get_app(&self, id: &str) -> Option<&ManifestApplication> {
        self.applications
            .application
            .iter()
            .find(|app| app.id == id)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestIdentity {
    #[serde(rename = "@Name")]
    pub name: String,
    #[serde(rename = "@Version")]
    pub version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManifestProperties {
    pub display_name: String,
    pub publisher_display_name: String,
    pub logo: String,
    pub description: Option<String>,
}

/// This struct makes reference to:
/// https://learn.microsoft.com/en-us/uwp/schemas/appxpackage/uapmanifestschema/element-applications
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "PascalCase")]
pub struct ManifestApplications {
    pub application: Vec<ManifestApplication>,
}

/// This struct makes reference to:
/// https://learn.microsoft.com/en-us/uwp/schemas/appxpackage/uapmanifestschema/element-application
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManifestApplication {
    #[serde(rename = "@Id")]
    pub id: String,
    #[serde(rename = "@Executable")]
    pub executable: Option<String>,
    pub visual_elements: ManifestApplicationVisualElements,
    pub extensions: Option<ManifestApplicationExtensions>,
}

/// Container for `<Application><Extensions>...</Extensions></Application>`.
/// Children can appear under multiple namespace prefixes (`uap3:Extension`,
/// `desktop:Extension`, ...). Using `$value` collects every child element into
/// `extension` regardless of prefix / local name.
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct ManifestApplicationExtensions {
    #[serde(rename = "$value", default)]
    pub extension: Vec<ManifestExtension>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManifestExtension {
    #[serde(rename = "@Category")]
    pub category: String,
    /// Present only when `Category == "windows.toastNotificationActivation"`.
    #[serde(default)]
    pub toast_notification_activation: Option<ToastNotificationActivation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToastNotificationActivation {
    #[serde(rename = "@ToastActivatorCLSID")]
    pub toast_activator_clsid: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManifestApplicationVisualElements {
    #[serde(rename = "@DisplayName")]
    pub display_name: String,
    #[serde(rename = "@Description")]
    pub description: String,
    #[serde(rename = "@BackgroundColor")]
    pub background_color: String,
    #[serde(rename = "@Square150x150Logo")]
    pub logo_150: String,
    #[serde(rename = "@Square44x44Logo")]
    pub logo_44: String,
}
