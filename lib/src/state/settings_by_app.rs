use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_alias::serde_alias;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AppExtraFlag {
    /// Start the app in the center of the screen as floating in the wm.
    Float,
    /// Force manage this app in the wm.
    Force,
    /// Unmanage this app in the wm.
    Unmanage,
    /// Pin this app in all the virtual desktops in the wm.
    Pinned,
    /// Hide this app on the dock/taskbar.
    Hidden,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, JsonSchema)]
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
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AppIdentifier {
    pub id: String,
    pub kind: AppIdentifierType,
    pub matching_strategy: MatchingStrategy,
    #[serde(default)]
    pub negation: bool,
    #[serde(default)]
    pub and: Vec<AppIdentifier>,
    #[serde(default)]
    pub or: Vec<AppIdentifier>,
    #[serde(skip)]
    pub regex: Option<Regex>,
}

impl AppIdentifier {
    pub fn cache_regex(&mut self) {
        if matches!(self.matching_strategy, MatchingStrategy::Regex) {
            let result = Regex::new(&self.id);
            if let Ok(re) = result {
                self.regex = Some(re);
            }
        }
    }

    pub fn validate(&self, title: &str, class: &str, exe: &str, path: &str) -> bool {
        let mut self_result = match self.matching_strategy {
            MatchingStrategy::Equals => match self.kind {
                AppIdentifierType::Title => title.eq(&self.id),
                AppIdentifierType::Class => class.eq(&self.id),
                AppIdentifierType::Exe => exe.eq(&self.id),
                AppIdentifierType::Path => path.eq(&self.id),
            },
            MatchingStrategy::StartsWith => match self.kind {
                AppIdentifierType::Title => title.starts_with(&self.id),
                AppIdentifierType::Class => class.starts_with(&self.id),
                AppIdentifierType::Exe => exe.starts_with(&self.id),
                AppIdentifierType::Path => path.starts_with(&self.id),
            },
            MatchingStrategy::EndsWith => match self.kind {
                AppIdentifierType::Title => title.ends_with(&self.id),
                AppIdentifierType::Class => class.ends_with(&self.id),
                AppIdentifierType::Exe => exe.ends_with(&self.id),
                AppIdentifierType::Path => path.ends_with(&self.id),
            },
            MatchingStrategy::Contains => match self.kind {
                AppIdentifierType::Title => title.contains(&self.id),
                AppIdentifierType::Class => class.contains(&self.id),
                AppIdentifierType::Exe => exe.contains(&self.id),
                AppIdentifierType::Path => path.contains(&self.id),
            },
            MatchingStrategy::Regex => match self.regex.as_ref() {
                Some(re) => match self.kind {
                    AppIdentifierType::Title => re.is_match(title),
                    AppIdentifierType::Class => re.is_match(class),
                    AppIdentifierType::Exe => re.is_match(exe),
                    AppIdentifierType::Path => re.is_match(path),
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
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    /// name of the app
    pub name: String,
    /// category to group the app under
    pub category: Option<String>,
    /// monitor index that the app should be bound to
    pub bound_monitor: Option<usize>,
    /// workspace name that the app should be bound to
    pub bound_workspace: Option<String>,
    /// app identifier
    pub identifier: AppIdentifier,
    /// extra specific options/settings for the app
    #[serde(default)]
    pub options: Vec<AppExtraFlag>,
    /// is this config bundled with seelen ui.
    #[serde(default)]
    pub is_bundled: bool,
}
