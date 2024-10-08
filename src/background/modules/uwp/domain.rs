use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct PackageManifest {
    pub identity: ManifestIdentity,
    pub properties: ManifestProperties,
    pub applications: Option<ManifestApplications>,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManifestApplications {
    pub application: Vec<ManifestApplication>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct ManifestApplication {
    #[serde(rename = "@Id")]
    pub id: String,
    pub visual_elements: ManifestApplicationVisualElements,
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
