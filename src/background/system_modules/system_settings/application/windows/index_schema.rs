use serde::Deserialize;

/// Schema of `%WINDIR%\ImmersiveControlPanel\Settings\AllSystemSettings_{GUID}.xml`,
/// the index used by Windows itself to search pages on the Settings app.
/// Only the fields needed to build the search entries are declared.
#[derive(Debug, Deserialize)]
pub struct PcSettingsIndex {
    #[serde(rename = "SearchableContent", default)]
    pub items: Vec<SearchableContent>,
}

#[derive(Debug, Deserialize)]
pub struct SearchableContent {
    #[serde(rename = "Filename")]
    pub filename: String,
    #[serde(rename = "ApplicationInformation")]
    pub app_info: Option<ApplicationInformation>,
    #[serde(rename = "SettingIdentity")]
    pub identity: Option<SettingIdentity>,
    #[serde(rename = "SettingInformation")]
    pub info: Option<SettingInformation>,
}

#[derive(Debug, Deserialize)]
pub struct ApplicationInformation {
    #[serde(rename = "FontFamily")]
    pub font_family: Option<String>,
    #[serde(rename = "Glyph")]
    pub glyph: Option<String>,
    /// Font file of private fonts, e.g. `%WINDIR%\SystemResources\...\SettingsIcons.ttf`
    #[serde(rename = "FontFilePath")]
    pub font_file_path: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SettingIdentity {
    #[serde(rename = "SettingPaths")]
    pub paths: Option<SettingPaths>,
}

#[derive(Debug, Deserialize)]
pub struct SettingPaths {
    #[serde(rename = "Path", default)]
    pub paths: Vec<SettingPath>,
}

#[derive(Debug, Deserialize)]
pub struct SettingPath {
    /// `;` separated list of `ms-settings:` page names, the first one is the main page.
    #[serde(rename = "PolicyIds")]
    pub policy_ids: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SettingInformation {
    /// Indirect string (`@{...?ms-resource://...}`) of the setting display name.
    #[serde(rename = "Description")]
    pub description: Option<String>,
    /// Indirect string of the `;` separated search keywords.
    #[serde(rename = "HighKeywords")]
    pub high_keywords: Option<String>,
    /// Indirect string of the `;` separated search keywords.
    #[serde(rename = "Keywords")]
    pub keywords: Option<String>,
}

impl SearchableContent {
    /// Main `ms-settings:` page name of this entry, if any.
    pub fn page(&self) -> Option<&str> {
        let paths = self.identity.as_ref()?.paths.as_ref()?;
        paths
            .paths
            .iter()
            .filter_map(|p| p.policy_ids.as_deref())
            .flat_map(|ids| ids.split(';'))
            .map(str::trim)
            .find(|id| !id.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_settings_index() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<PCSettings>
  <SearchableContent ExcludeWithFeature="SettingsKeywordsCleanup">
    <Filename>AAA_Bluetooth</Filename>
    <ApplicationInformation>
      <FontFamily>Segoe Fluent Icons</FontFamily>
      <Glyph>&#xE702;</Glyph>
    </ApplicationInformation>
    <SettingIdentity>
      <SettingPaths>
        <Path>
          <PageID>SettingsPageBluetooth</PageID>
          <GroupID>Bluetooth</GroupID>
          <PolicyIds>bluetooth;connecteddevices</PolicyIds>
        </Path>
      </SettingPaths>
    </SettingIdentity>
    <SettingInformation>
      <Description>@{windows?ms-resource://X/Description}</Description>
      <HighKeywords>@{windows?ms-resource://X/HighKeywords}</HighKeywords>
    </SettingInformation>
  </SearchableContent>
  <SearchableContent>
    <Filename>Defender_AccountProtection</Filename>
    <ApplicationInformation>
      <DeepLink>AccountProtection</DeepLink>
    </ApplicationInformation>
    <SettingIdentity>
      <PageID>{6E6DDBCB-9C89-434B-A994-D5F22239523B}</PageID>
    </SettingIdentity>
    <SettingInformation>
      <Description>@{Y}</Description>
    </SettingInformation>
  </SearchableContent>
</PCSettings>"#;

        let index: PcSettingsIndex = quick_xml::de::from_str(xml).unwrap();
        assert_eq!(index.items.len(), 2);

        let bluetooth = &index.items[0];
        assert_eq!(bluetooth.page(), Some("bluetooth"));
        let app_info = bluetooth.app_info.as_ref().unwrap();
        assert_eq!(app_info.glyph.as_deref(), Some("\u{E702}"));
        let info = bluetooth.info.as_ref().unwrap();
        assert_eq!(
            info.description.as_deref(),
            Some("@{windows?ms-resource://X/Description}")
        );

        assert_eq!(index.items[1].page(), None);
    }
}
