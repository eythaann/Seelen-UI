use std::{
    collections::{hash_map::Entry, HashMap, VecDeque}, path::PathBuf, sync::Arc
};

use lazy_static::lazy_static;
use parking_lot::Mutex;
use regex::Regex;
use serde::{Deserialize, Serialize};
use windows::Win32::Foundation::HWND;

use crate::{error_handler::Result, windows_api::WindowsApi};

lazy_static! {
    pub static ref REGEX_IDENTIFIERS: Arc<Mutex<HashMap<String, Regex>>> =
        Arc::new(Mutex::new(HashMap::new()));
    pub static ref SETTINGS_BY_APP: Arc<Mutex<AppsConfigurations>> =
        Arc::new(Mutex::new(AppsConfigurations::default()));
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all(deserialize = "snake_case"))]
pub enum AppExtraFlag {
    Float,
    Unmanage,
    // only for backguard compatibility
    ObjectNameChange,
    Layered,
    BorderOverflow,
    TrayAndMultiWindow,
    Force,
}

#[derive(Clone, Debug, Deserialize)]
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
    Legacy,
    Equals,
    StartsWith,
    EndsWith,
    Contains,
    Regex,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppIdentifier {
    id: String,
    kind: AppIdentifierType,
    matching_strategy: MatchingStrategy,
}

impl AppIdentifier {
    pub fn cache_regex(&mut self) {
        if matches!(self.matching_strategy, MatchingStrategy::Regex) {
            let result = Regex::new(&self.id);
            if let Some(re) = result.ok() {
                let mut regex_identifiers = REGEX_IDENTIFIERS.lock();
                regex_identifiers.insert(self.id.clone(), re);
            }
        }
    }

    pub fn validate(&self, title: &str, class: &str, exe: &str, path: &str) -> bool {
        match self.matching_strategy {
            MatchingStrategy::Legacy => match self.kind {
                AppIdentifierType::Title => {
                    title.starts_with(&self.id) || title.ends_with(&self.id)
                }
                AppIdentifierType::Class => {
                    class.starts_with(&self.id) || class.ends_with(&self.id)
                }
                AppIdentifierType::Exe => exe.eq(&self.id),
                AppIdentifierType::Path => path.eq(&self.id),
            },
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
            MatchingStrategy::Regex => {
                let regex_identifiers = REGEX_IDENTIFIERS.lock();
                if let Some(re) = regex_identifiers.get(&self.id) {
                    return match self.kind {
                        AppIdentifierType::Title => re.is_match(title),
                        AppIdentifierType::Class => re.is_match(class),
                        AppIdentifierType::Exe => re.is_match(exe),
                        AppIdentifierType::Path => re.is_match(path),
                    };
                }
                false
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    name: String,
    category: Option<String>,
    binded_monitor_idx: Option<usize>,
    binded_workspace_name: Option<String>,
    identifier: AppIdentifier,
    options: Option<Vec<AppExtraFlag>>,
}

impl AppConfig {
    pub fn match_window(&self, hwnd: HWND) -> bool {
        if let (title, Ok(path), Ok(exe), Ok(class)) = (
            WindowsApi::get_window_text(hwnd),
            WindowsApi::exe_path(hwnd),
            WindowsApi::exe(hwnd),
            WindowsApi::get_class(hwnd),
        ) {
            return self.identifier.validate(&title, &class, &exe, &path);
        }
        false
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppsConfigurations {
    apps: VecDeque<AppConfig>,
    cache: HashMap<isize, Option<usize>>,
}

impl Default for AppsConfigurations {
    fn default() -> Self {
        Self {
            apps: VecDeque::new(),
            cache: HashMap::new(),
        }
    }
}

impl AppsConfigurations {
    pub fn load(&mut self, path: PathBuf) -> Result<()> {
        let mut content = String::from("");
        if path.exists() {
            content = std::fs::read_to_string(path)?;
        }
        self.apps = serde_yaml::from_str(&content).unwrap();
        self.cache.clear();
        Ok(())
    }

    pub fn get_by_window(&mut self, hwnd: HWND) -> Option<&AppConfig> {
        match self.cache.entry(hwnd.0) {
            Entry::Occupied(entry) => entry.get().and_then(|index| self.apps.get(index)),
            Entry::Vacant(entry) => {
                for (i, app) in self.apps.iter().enumerate() {
                    if app.match_window(hwnd) {
                        entry.insert(Some(i));
                        return Option::from(app);
                    }
                }
                entry.insert(None);
                None
            }
        }
    }
}
