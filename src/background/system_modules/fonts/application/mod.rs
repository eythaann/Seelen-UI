#[cfg(windows)]
mod windows;

use std::sync::LazyLock;

use seelen_core::system_state::SeelenFont;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
};

use super::domain::{FontsEvent, FontsSysEvent, FontsSystem};

#[cfg(windows)]
type Platform = windows::WindowsFonts;

/// Stateless: the font list is queried on demand (it is big and rarely needed).
pub struct FontsManager {
    platform: Platform,
}

event_manager!(FontsManager, FontsEvent);

impl FontsManager {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<FontsManager> = LazyLock::new(|| {
            let mut m = FontsManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
        }
    }

    fn init(&mut self) -> Result<()> {
        self.platform.init(Self::on_platform_event)
    }

    fn on_platform_event(event: FontsSysEvent) {
        match event {
            FontsSysEvent::Changed => match Self::instance().fonts() {
                Ok(fonts) => Self::send(FontsEvent::Changed { fonts }),
                Err(err) => log::error!("Failed to refresh fonts: {err}"),
            },
        }
    }

    /// Installed font families, sorted by name.
    pub fn fonts(&self) -> Result<Vec<SeelenFont>> {
        let mut fonts = self.platform.fonts()?;
        fonts.sort_by(|a, b| a.family.cmp(&b.family));
        Ok(fonts)
    }
}
