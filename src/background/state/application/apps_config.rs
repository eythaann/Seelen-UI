use lazy_static::lazy_static;
use parking_lot::Mutex;
use regex::Regex;
use std::{collections::HashMap, sync::Arc};
use windows::Win32::Foundation::HWND;

use crate::{
    state::domain::{AppConfig, AppExtraFlag, AppIdentifier, AppIdentifierType, MatchingStrategy},
    trace_lock,
    windows_api::WindowsApi,
};

use super::FullState;

lazy_static! {
    pub static ref REGEX_IDENTIFIERS: Arc<Mutex<HashMap<String, Regex>>> =
        Arc::new(Mutex::new(HashMap::new()));
}

impl AppIdentifier {
    pub fn cache_regex(&self) {
        if matches!(self.matching_strategy, MatchingStrategy::Regex) {
            let result = Regex::new(&self.id);
            if let Ok(re) = result {
                let mut regex_identifiers = trace_lock!(REGEX_IDENTIFIERS);
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
            MatchingStrategy::Regex => match trace_lock!(REGEX_IDENTIFIERS).get(&self.id) {
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
        self.options.contains(&option)
    }
}

impl FullState {
    pub fn get_app_config_by_window(&mut self, hwnd: HWND) -> Option<&AppConfig> {
        // Can no cache apps that changes titles
        /* match self.cache.entry(hwnd.0) {
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
        } */

        if let (title, Ok(path), Ok(exe), Ok(class)) = (
            WindowsApi::get_window_text(hwnd),
            WindowsApi::exe_path(hwnd),
            WindowsApi::exe(hwnd),
            WindowsApi::get_class(hwnd),
        ) {
            for app in self.settings_by_app.iter() {
                if app.identifier.validate(&title, &class, &exe, &path) {
                    return Option::from(app);
                }
            }
        }

        None
    }
}
