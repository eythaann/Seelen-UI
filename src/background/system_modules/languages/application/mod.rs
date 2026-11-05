#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use seelen_core::system_state::{ImeState, SystemLanguage};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::lock_free::SyncVec,
};

use super::domain::{LanguageEvent, LanguagesSysEvent, LanguagesSystem};

#[cfg(windows)]
type Platform = windows::WindowsLanguages;

pub struct LanguageManager {
    platform: Platform,
    /// Last known languages, refreshed when the layouts change.
    languages: SyncVec<SystemLanguage>,
}

event_manager!(LanguageManager, LanguageEvent);

impl LanguageManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<LanguageManager> = LazyLock::new(|| {
            let mut m = LanguageManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            languages: SyncVec::new(),
        }
    }

    fn init(&mut self) -> Result<()> {
        self.platform.init(Self::on_platform_event)?;
        self.languages.replace(self.platform.languages()?);
        Ok(())
    }

    fn on_platform_event(event: LanguagesSysEvent) {
        match event {
            LanguagesSysEvent::LayoutChanged => Self::instance().layout_changed(),
            LanguagesSysEvent::ImeChanged => Self::send(LanguageEvent::ImeChanged),
        }
    }

    fn layout_changed(&self) {
        if let Some(languages) = self.platform.languages().ok_logged() {
            self.languages.replace(languages);
        }
        Self::send(LanguageEvent::LayoutChanged);
    }

    pub fn languages(&self) -> Vec<SystemLanguage> {
        self.languages.to_vec()
    }

    pub fn ime_state(&self) -> Result<ImeState> {
        self.platform.ime_state()
    }

    pub fn set_keyboard_layout(&self, id: &str, handle: &str) -> Result<()> {
        self.platform.set_keyboard_layout(id, handle)?;
        self.layout_changed();
        Ok(())
    }
}
