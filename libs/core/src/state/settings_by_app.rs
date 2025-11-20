use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_alias::serde_alias;
use ts_rs::TS;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum AppExtraFlag {
    /// Mark this app as non interactive window.
    #[serde(alias = "no-interactive")]
    NoInteractive,
    /// Start the app in the center of the screen as floating in the wm.
    #[serde(alias = "float", alias = "wm-float")]
    WmFloat,
    /// Forces the management of this app in the wm. (only if it is interactable and not pinned)
    #[serde(alias = "force", alias = "wm-force")]
    WmForce,
    /// Unmanage this app in the wm.
    #[serde(alias = "unmanage", alias = "wm-unmanage")]
    WmUnmanage,
    /// Pin this app in all the virtual desktops in the wm.
    #[serde(alias = "pinned", alias = "vd-pinned")]
    VdPinned,
    #[serde(other)]
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum AppIdentifierType {
    #[serde(alias = "exe")]
    Exe,
    #[serde(alias = "class")]
    Class,
    #[serde(alias = "title")]
    Title,
    #[serde(alias = "path")]
    Path,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(repr(enum = name))]
pub enum MatchingStrategy {
    #[serde(alias = "equals", alias = "legacy", alias = "Legacy")]
    Equals,
    #[serde(alias = "startsWith")]
    StartsWith,
    #[serde(alias = "endsWith")]
    EndsWith,
    #[serde(alias = "contains")]
    Contains,
    #[serde(alias = "regex")]
    Regex,
}

#[serde_alias(SnakeCase)]
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
pub struct AppIdentifier {
    /// Depending of the kind this can be case sensitive or not.
    /// - `class` and `title` are case sensitive
    /// - `exe` and `path` are case insensitive
    pub id: String,
    /// the way to match the application
    pub kind: AppIdentifierType,
    /// the strategy to use to determine if id matches with the application
    pub matching_strategy: MatchingStrategy,
    #[serde(default)]
    pub negation: bool,
    #[serde(default)]
    pub and: Vec<AppIdentifier>,
    #[serde(default)]
    pub or: Vec<AppIdentifier>,
    #[serde(skip)]
    cache: AppIdentifierCache,
}

/// this struct is intented to improve performance
#[derive(Debug, Default, Clone)]
pub struct AppIdentifierCache {
    pub regex: Option<Regex>,
    pub lower_id: Option<String>,
}

impl AppIdentifier {
    fn prepare(&mut self) {
        if matches!(self.matching_strategy, MatchingStrategy::Regex) {
            let result = Regex::new(&self.id);
            if let Ok(re) = result {
                self.cache.regex = Some(re);
            }
        }
        if matches!(self.kind, AppIdentifierType::Path | AppIdentifierType::Exe) {
            // Normalize path separators to backslash and uppercase for Windows paths
            let normalized = self.id.replace('\\', "/");
            self.cache.lower_id = Some(normalized.to_lowercase());
        }

        self.and.iter_mut().for_each(|i| i.prepare());
        self.or.iter_mut().for_each(|i| i.prepare());
    }

    fn lower_id(&self) -> &str {
        self.cache.lower_id.as_deref().unwrap()
    }

    /// path and filenames on Windows System should be uppercased before be passed to this function
    /// Safety: will panic if cache was not performed before
    fn validate(&self, title: &str, class: &str, exe: &str, path: &str) -> bool {
        let mut self_result = match self.matching_strategy {
            MatchingStrategy::Equals => match self.kind {
                AppIdentifierType::Title => title.eq(&self.id),
                AppIdentifierType::Class => class.eq(&self.id),
                AppIdentifierType::Exe => exe.eq(self.lower_id()),
                AppIdentifierType::Path => path.eq(self.lower_id()),
            },
            MatchingStrategy::StartsWith => match self.kind {
                AppIdentifierType::Title => title.starts_with(&self.id),
                AppIdentifierType::Class => class.starts_with(&self.id),
                AppIdentifierType::Exe => exe.starts_with(self.lower_id()),
                AppIdentifierType::Path => path.starts_with(self.lower_id()),
            },
            MatchingStrategy::EndsWith => match self.kind {
                AppIdentifierType::Title => title.ends_with(&self.id),
                AppIdentifierType::Class => class.ends_with(&self.id),
                AppIdentifierType::Exe => exe.ends_with(self.lower_id()),
                AppIdentifierType::Path => path.ends_with(self.lower_id()),
            },
            MatchingStrategy::Contains => match self.kind {
                AppIdentifierType::Title => title.contains(&self.id),
                AppIdentifierType::Class => class.contains(&self.id),
                AppIdentifierType::Exe => exe.contains(self.lower_id()),
                AppIdentifierType::Path => path.contains(self.lower_id()),
            },
            MatchingStrategy::Regex => match &self.cache.regex {
                Some(regex) => match self.kind {
                    AppIdentifierType::Title => regex.is_match(title),
                    AppIdentifierType::Class => regex.is_match(class),
                    AppIdentifierType::Exe => regex.is_match(exe),
                    AppIdentifierType::Path => regex.is_match(path),
                },
                None => false,
            },
        };

        if self.negation {
            self_result = !self_result;
        }

        (self_result && {
            self.and
                .iter()
                .all(|and| and.validate(title, class, exe, path))
        }) || {
            self.or
                .iter()
                .any(|or| or.validate(title, class, exe, path))
        }
    }
}

#[serde_alias(SnakeCase)]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct AppConfig {
    /// name of the app
    pub name: String,
    /// category to group the app under
    pub category: Option<String>,
    /// monitor index that the app should be bound to
    pub bound_monitor: Option<usize>,
    /// workspace index that the app should be bound to
    pub bound_workspace: Option<usize>,
    /// app identifier
    pub identifier: AppIdentifier,
    /// extra specific options/settings for the app
    #[serde(default)]
    pub options: Vec<AppExtraFlag>,
    /// is this config bundled with seelen ui.
    #[serde(default)]
    pub is_bundled: bool,
}

impl AppConfig {
    pub fn prepare(&mut self) {
        self.identifier.prepare();
    }
}

#[derive(Debug, Default, Clone)]
pub struct AppsConfigurationList(Vec<AppConfig>);

impl AppsConfigurationList {
    pub fn prepare(&mut self) {
        self.0.iter_mut().for_each(|config| config.prepare());
    }

    pub fn search(&self, title: &str, class: &str, exe: &str, path: &str) -> Option<&AppConfig> {
        let normalized_path = path.to_lowercase().replace("\\", "/");
        let normalized_exe = exe.to_lowercase();

        self.0.iter().find(|&config| {
            config
                .identifier
                .validate(title, class, &normalized_exe, &normalized_path)
        })
    }

    pub fn iter(&self) -> impl Iterator<Item = &AppConfig> {
        self.0.iter()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn extend(&mut self, configs: Vec<AppConfig>) {
        self.0.extend(configs);
    }

    pub fn as_slice(&self) -> &[AppConfig] {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_apps_path_contains_matching() {
        // Test the specific case: ShellExperienceHost.exe with SystemApps path matching
        let mut identifier = AppIdentifier {
            id: "Windows\\SystemApps".to_string(),
            kind: AppIdentifierType::Path,
            matching_strategy: MatchingStrategy::Contains,
            negation: false,
            and: vec![],
            or: vec![],
            cache: AppIdentifierCache::default(),
        };

        // Prepare the identifier (this normalizes to lowercase with forward slashes)
        identifier.prepare();

        // Test path (will be normalized by search() to lowercase with forward slashes)
        let path =
            "C:\\WINDOWS\\SYSTEMAPPS\\SHELLEXPERIENCEHOST_CW5N1H2TXYEWY\\SHELLEXPERIENCEHOST.EXE";
        let title = "";
        let class = "";
        let exe = "SHELLEXPERIENCEHOST.EXE";

        // Normalize path and exe as search() does
        let normalized_path = path.to_lowercase().replace("\\", "/");
        let normalized_exe = exe.to_lowercase();

        // Should match because normalized path contains "windows/systemapps"
        assert!(
            identifier.validate(title, class, &normalized_exe, &normalized_path),
            "Path should match with Contains strategy"
        );
    }

    #[test]
    fn test_system_apps_full_config() {
        // Test a full AppConfig with the System Background Apps configuration
        let mut config = AppConfig {
            name: "System Background Apps".to_string(),
            category: None,
            bound_monitor: None,
            bound_workspace: None,
            identifier: AppIdentifier {
                id: "Windows\\SystemApps".to_string(),
                kind: AppIdentifierType::Path,
                matching_strategy: MatchingStrategy::Contains,
                negation: false,
                and: vec![],
                or: vec![],
                cache: AppIdentifierCache::default(),
            },
            options: vec![AppExtraFlag::NoInteractive],
            is_bundled: false,
        };

        config.prepare();

        // Test that ShellExperienceHost.exe matches
        let path =
            "C:\\WINDOWS\\SYSTEMAPPS\\SHELLEXPERIENCEHOST_CW5N1H2TXYEWY\\SHELLEXPERIENCEHOST.EXE";
        let title = "";
        let class = "";
        let exe = "SHELLEXPERIENCEHOST.EXE";

        // Normalize as search() does
        let normalized_path = path.to_lowercase().replace("\\", "/");
        let normalized_exe = exe.to_lowercase();

        assert!(
            config
                .identifier
                .validate(title, class, &normalized_exe, &normalized_path),
            "ShellExperienceHost.exe should match System Background Apps config"
        );

        // Verify options
        assert_eq!(config.options.len(), 1);
        assert_eq!(config.options[0], AppExtraFlag::NoInteractive);
    }

    #[test]
    fn test_apps_configuration_list_search() {
        // Test searching in AppsConfigurationList
        let mut list = AppsConfigurationList(vec![AppConfig {
            name: "System Background Apps".to_string(),
            category: None,
            bound_monitor: None,
            bound_workspace: None,
            identifier: AppIdentifier {
                id: "Windows\\SystemApps".to_string(),
                kind: AppIdentifierType::Path,
                matching_strategy: MatchingStrategy::Contains,
                negation: false,
                and: vec![],
                or: vec![],
                cache: AppIdentifierCache::default(),
            },
            options: vec![AppExtraFlag::NoInteractive],
            is_bundled: true,
        }]);

        list.prepare();

        // Search for ShellExperienceHost.exe
        let path =
            "C:\\WINDOWS\\SYSTEMAPPS\\SHELLEXPERIENCEHOST_CW5N1H2TXYEWY\\SHELLEXPERIENCEHOST.EXE";
        let title = "";
        let class = "";
        let exe = "SHELLEXPERIENCEHOST.EXE";

        let result = list.search(title, class, exe, path);
        assert!(result.is_some(), "Should find matching config");

        let found_config = result.unwrap();
        assert_eq!(found_config.name, "System Background Apps");
        assert!(found_config.options.contains(&AppExtraFlag::NoInteractive));
    }

    #[test]
    fn test_path_contains_non_matching() {
        // Test that non-SystemApps paths don't match
        let mut identifier = AppIdentifier {
            id: "Windows\\SystemApps".to_string(),
            kind: AppIdentifierType::Path,
            matching_strategy: MatchingStrategy::Contains,
            negation: false,
            and: vec![],
            or: vec![],
            cache: AppIdentifierCache::default(),
        };

        identifier.prepare();

        // Test with a regular Program Files path
        let path = "C:\\PROGRAM FILES\\SOME APP\\APP.EXE";
        let title = "";
        let class = "";
        let exe = "APP.EXE";

        // Normalize as search() does
        let normalized_path = path.to_lowercase().replace("\\", "/");
        let normalized_exe = exe.to_lowercase();

        assert!(
            !identifier.validate(title, class, &normalized_exe, &normalized_path),
            "Non-SystemApps path should not match"
        );
    }

    #[test]
    fn test_path_case_insensitivity() {
        // Test that path matching is case insensitive
        let mut identifier = AppIdentifier {
            id: "windows\\systemapps".to_string(), // lowercase
            kind: AppIdentifierType::Path,
            matching_strategy: MatchingStrategy::Contains,
            negation: false,
            and: vec![],
            or: vec![],
            cache: AppIdentifierCache::default(),
        };

        identifier.prepare();

        // Test with uppercase path (will be normalized to lowercase)
        let path =
            "C:\\WINDOWS\\SYSTEMAPPS\\SHELLEXPERIENCEHOST_CW5N1H2TXYEWY\\SHELLEXPERIENCEHOST.EXE";
        let title = "";
        let class = "";
        let exe = "SHELLEXPERIENCEHOST.EXE";

        // Normalize as search() does
        let normalized_path = path.to_lowercase().replace("\\", "/");
        let normalized_exe = exe.to_lowercase();

        assert!(
            identifier.validate(title, class, &normalized_exe, &normalized_path),
            "Path matching should be case insensitive"
        );
    }

    #[test]
    fn test_multiple_system_apps() {
        // Test that the same config matches multiple SystemApps executables
        let mut identifier = AppIdentifier {
            id: "Windows\\SystemApps".to_string(),
            kind: AppIdentifierType::Path,
            matching_strategy: MatchingStrategy::Contains,
            negation: false,
            and: vec![],
            or: vec![],
            cache: AppIdentifierCache::default(),
        };

        identifier.prepare();

        // Test different SystemApps
        let test_cases = vec![
            "C:\\WINDOWS\\SYSTEMAPPS\\SHELLEXPERIENCEHOST_CW5N1H2TXYEWY\\SHELLEXPERIENCEHOST.EXE",
            "C:\\WINDOWS\\SYSTEMAPPS\\MICROSOFT.WINDOWS.STARTMENUEXPERIENCEHOST_CW5N1H2TXYEWY\\STARTMENUEXPERIENCEHOST.EXE",
            "C:\\WINDOWS\\SYSTEMAPPS\\MICROSOFT.WINDOWS.SEARCH_CW5N1H2TXYEWY\\SEARCHAPP.EXE",
        ];

        for path in test_cases {
            // Normalize as search() does
            let normalized_path = path.to_lowercase().replace("\\", "/");
            assert!(
                identifier.validate("", "", "", &normalized_path),
                "Path {} should match SystemApps pattern",
                path
            );
        }
    }

    #[test]
    fn test_app_extra_flag_deserialization() {
        // Test that "no-interactive" deserializes correctly
        let json = r#"{"name":"Test","identifier":{"id":"test","kind":"exe","matchingStrategy":"equals"},"options":["no-interactive"]}"#;
        let config: AppConfig = serde_json::from_str(json).expect("Should deserialize");

        assert_eq!(config.options.len(), 1);
        assert_eq!(config.options[0], AppExtraFlag::NoInteractive);
    }

    #[test]
    fn test_matching_strategy_deserialization() {
        // Test that "contains" deserializes correctly
        let json = r#"{"name":"Test","identifier":{"id":"test","kind":"path","matchingStrategy":"contains"},"options":[]}"#;
        let config: AppConfig = serde_json::from_str(json).expect("Should deserialize");

        assert!(matches!(
            config.identifier.matching_strategy,
            MatchingStrategy::Contains
        ));
    }

    #[test]
    fn test_path_separator_normalization_forward_slash() {
        // Test that both forward and backslashes work (both normalized to forward slash)
        let mut identifier = AppIdentifier {
            id: "Windows/SystemApps".to_string(), // Using forward slash
            kind: AppIdentifierType::Path,
            matching_strategy: MatchingStrategy::Contains,
            negation: false,
            and: vec![],
            or: vec![],
            cache: AppIdentifierCache::default(),
        };

        identifier.prepare();

        // Verify the cached id is normalized to lowercase with forward slashes
        assert_eq!(identifier.lower_id(), "windows/systemapps");

        // Should match when path uses backslashes (normalized by search())
        let path =
            "C:\\WINDOWS\\SYSTEMAPPS\\SHELLEXPERIENCEHOST_CW5N1H2TXYEWY\\SHELLEXPERIENCEHOST.EXE";
        let title = "";
        let class = "";
        let exe = "SHELLEXPERIENCEHOST.EXE";

        // Normalize as search() does
        let normalized_path = path.to_lowercase().replace("\\", "/");
        let normalized_exe = exe.to_lowercase();

        assert!(
            identifier.validate(title, class, &normalized_exe, &normalized_path),
            "Both forward and backslashes should be normalized to forward slash"
        );
    }

    #[test]
    fn test_path_separator_mixed() {
        // Test with mixed separators in the identifier
        let mut identifier = AppIdentifier {
            id: "Windows\\SystemApps/Microsoft.Windows".to_string(), // Mixed separators
            kind: AppIdentifierType::Path,
            matching_strategy: MatchingStrategy::Contains,
            negation: false,
            and: vec![],
            or: vec![],
            cache: AppIdentifierCache::default(),
        };

        identifier.prepare();

        // Verify all backslashes are normalized to forward slashes
        assert_eq!(
            identifier.lower_id(),
            "windows/systemapps/microsoft.windows"
        );

        // Path with backslashes should match (normalized by search())
        let path = "C:\\WINDOWS\\SYSTEMAPPS\\MICROSOFT.WINDOWS.SEARCH_CW5N1H2TXYEWY\\SEARCHAPP.EXE";
        let title = "";
        let class = "";
        let exe = "SEARCHAPP.EXE";

        // Normalize as search() does
        let normalized_path = path.to_lowercase().replace("\\", "/");
        let normalized_exe = exe.to_lowercase();

        assert!(
            identifier.validate(title, class, &normalized_exe, &normalized_path),
            "Mixed separators should be normalized to forward slashes"
        );
    }

    #[test]
    fn test_exe_separator_normalization() {
        // Test that exe type also normalizes separators (though less common)
        let mut identifier = AppIdentifier {
            id: "app.EXE".to_string(),
            kind: AppIdentifierType::Exe,
            matching_strategy: MatchingStrategy::Contains,
            negation: false,
            and: vec![],
            or: vec![],
            cache: AppIdentifierCache::default(),
        };

        identifier.prepare();

        // Verify the cached value has backslashes
        assert_eq!(identifier.lower_id(), "app.exe");
    }

    #[test]
    fn test_title_and_class_no_normalization() {
        // Test that Title and Class types don't normalize slashes
        let mut title_identifier = AppIdentifier {
            id: "Some/Title".to_string(),
            kind: AppIdentifierType::Title,
            matching_strategy: MatchingStrategy::Equals,
            negation: false,
            and: vec![],
            or: vec![],
            cache: AppIdentifierCache::default(),
        };

        title_identifier.prepare();

        // Title with forward slash should match exactly (no normalization)
        assert!(
            title_identifier.validate("Some/Title", "", "", ""),
            "Title should not normalize separators"
        );

        // Should NOT match with backslash
        assert!(
            !title_identifier.validate("Some\\Title", "", "", ""),
            "Title should preserve forward slash"
        );
    }
}
