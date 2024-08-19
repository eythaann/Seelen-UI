use regex::Regex;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all(deserialize = "snake_case"))]
pub enum AppExtraFlag {
    Float,
    Force,
    Unmanage,
    Pinned,
    // only for backwards compatibility
    ObjectNameChange,
    Layered,
    BorderOverflow,
    TrayAndMultiWindow,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum MatchingStrategy {
    #[serde(alias = "equals")]
    Equals,
    #[serde(alias = "startsWith")]
    StartsWith,
    #[serde(alias = "endsWith")]
    EndsWith,
    #[serde(alias = "contains")]
    Contains,
    #[serde(alias = "regex")]
    Regex,
    // only for backwards compatibility
    #[serde(alias = "legacy")]
    Legacy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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
    // cache
    #[serde(skip)]
    pub regex: Option<Regex>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct AppConfig {
    pub name: String,
    pub category: Option<String>,
    pub bound_monitor_idx: Option<usize>,
    pub bound_workspace_name: Option<String>,
    pub identifier: AppIdentifier,
    #[serde(default)]
    pub options: Vec<AppExtraFlag>,
    #[serde(default)]
    pub is_bundled: bool,
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
            MatchingStrategy::Legacy | MatchingStrategy::Equals => match self.kind {
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
