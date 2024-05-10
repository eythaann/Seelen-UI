use std::{
    collections::{hash_map::Entry, HashMap, VecDeque},
    path::PathBuf,
    sync::Arc,
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

#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct AppIdentifier {
    id: String,
    kind: AppIdentifierType,
    matching_strategy: MatchingStrategy,
    negate: Option<bool>,
    and: Option<Vec<AppIdentifier>>,
    or: Option<Vec<AppIdentifier>>,
}

impl AppIdentifier {
    pub fn cache_regex(&self) {
        if matches!(self.matching_strategy, MatchingStrategy::Regex) {
            let result = Regex::new(&self.id);
            if let Some(re) = result.ok() {
                let mut regex_identifiers = REGEX_IDENTIFIERS.lock();
                regex_identifiers.insert(self.id.clone(), re);
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
            MatchingStrategy::Regex => match REGEX_IDENTIFIERS.lock().get(&self.id) {
                Some(re) => match self.kind {
                    AppIdentifierType::Title => re.is_match(title),
                    AppIdentifierType::Class => re.is_match(class),
                    AppIdentifierType::Exe => re.is_match(exe),
                    AppIdentifierType::Path => re.is_match(path),
                },
                None => false,
            },
        };

        if self.negate.is_some_and(|negation| negation) {
            self_result = !self_result;
        }

        (self_result && {
            match self.and.as_ref() {
                Some(and) => and.iter().all(|and| and.validate(title, class, exe, path)),
                None => true,
            }
        }) || {
            match self.or.as_ref() {
                Some(or) => or.iter().any(|or| or.validate(title, class, exe, path)),
                None => false,
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppConfig {
    name: String,
    category: Option<String>,
    bound_monitor_idx: Option<usize>,
    bound_workspace_name: Option<String>,
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

    pub fn options_contains(&self, option: AppExtraFlag) -> bool {
        self.options
            .as_ref()
            .map_or(false, |options| options.contains(&option))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct AppsConfigurations {
    apps: VecDeque<AppConfig>,
    cache: HashMap<isize, Option<usize>>,
    user_path: PathBuf,
    apps_template_path: PathBuf,
}

impl Default for AppsConfigurations {
    fn default() -> Self {
        Self {
            apps: VecDeque::new(),
            cache: HashMap::new(),
            user_path: PathBuf::new(),
            apps_template_path: PathBuf::new(),
        }
    }
}

impl AppsConfigurations {
    pub fn set_paths(&mut self, user_path: PathBuf, apps_template_path: PathBuf) {
        self.user_path = user_path;
        self.apps_template_path = apps_template_path;
    }

    pub fn load(&mut self) -> Result<()> {
        log::trace!("Loading apps configurations from {:?}", self.user_path);
        REGEX_IDENTIFIERS.lock().clear();
        self.cache.clear();
        self.apps.clear();

        if self.user_path.exists() {
            let content = std::fs::read_to_string(&self.user_path)?;
            let apps: Vec<AppConfig> = serde_yaml::from_str(&content)?;
            self.apps.extend(apps);
        }

        for entry in self.apps_template_path.read_dir()? {
            if let Ok(entry) = entry {
                let content = std::fs::read_to_string(entry.path())?;
                let apps: Vec<AppConfig> = serde_yaml::from_str(&content)?;
                self.apps.extend(apps);
            }
        }

        self.apps
            .iter()
            .for_each(|app| app.identifier.cache_regex());
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

#[tauri::command]
pub fn reload_apps_configurations() {
    std::thread::spawn(|| -> Result<()> { SETTINGS_BY_APP.lock().load() });
}
