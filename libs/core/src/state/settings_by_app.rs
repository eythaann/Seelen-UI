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
    pub cache: AppIdentifierCache,
}

/// this struct is intented to improve performance
#[derive(Debug, Default, Clone)]
pub struct AppIdentifierCache {
    pub regex: Option<Regex>,
    pub uppercased_id: Option<String>,
}

impl AppIdentifier {
    pub fn prepare(&mut self) {
        if matches!(self.matching_strategy, MatchingStrategy::Regex) {
            let result = Regex::new(&self.id);
            if let Ok(re) = result {
                self.cache.regex = Some(re);
            }
        }
        if matches!(self.kind, AppIdentifierType::Path | AppIdentifierType::Exe) {
            self.cache.uppercased_id = Some(self.id.to_uppercase());
        }

        self.and.iter_mut().for_each(|i| i.prepare());
        self.or.iter_mut().for_each(|i| i.prepare());
    }

    pub fn uppercased_id(&self) -> &str {
        self.cache.uppercased_id.as_deref().unwrap()
    }

    /// path and filenames on Windows System should be uppercased before be passed to this function
    /// Safety: will panic if cache was not performed before
    pub fn validate(&self, title: &str, class: &str, exe: &str, path: &str) -> bool {
        let mut self_result = match self.matching_strategy {
            MatchingStrategy::Equals => match self.kind {
                AppIdentifierType::Title => title.eq(&self.id),
                AppIdentifierType::Class => class.eq(&self.id),
                AppIdentifierType::Exe => exe.eq(self.uppercased_id()),
                AppIdentifierType::Path => path.eq(self.uppercased_id()),
            },
            MatchingStrategy::StartsWith => match self.kind {
                AppIdentifierType::Title => title.starts_with(&self.id),
                AppIdentifierType::Class => class.starts_with(&self.id),
                AppIdentifierType::Exe => exe.starts_with(self.uppercased_id()),
                AppIdentifierType::Path => path.starts_with(self.uppercased_id()),
            },
            MatchingStrategy::EndsWith => match self.kind {
                AppIdentifierType::Title => title.ends_with(&self.id),
                AppIdentifierType::Class => class.ends_with(&self.id),
                AppIdentifierType::Exe => exe.ends_with(self.uppercased_id()),
                AppIdentifierType::Path => path.ends_with(self.uppercased_id()),
            },
            MatchingStrategy::Contains => match self.kind {
                AppIdentifierType::Title => title.contains(&self.id),
                AppIdentifierType::Class => class.contains(&self.id),
                AppIdentifierType::Exe => exe.contains(self.uppercased_id()),
                AppIdentifierType::Path => path.contains(self.uppercased_id()),
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
        self.0
            .iter()
            .find(|&config| config.identifier.validate(title, class, exe, path))
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
