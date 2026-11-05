#[cfg(windows)]
mod windows;

use std::sync::{
    LazyLock, OnceLock,
    atomic::{AtomicBool, Ordering},
};

use seelen_core::system_state::{Color, NightlightSettings, SystemSettingEntry, UIColors};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
};

use super::domain::{
    SystemSettingsEvent, SystemSettingsSysEvent, SystemSettingsSystem, generate_accent_palette,
};

#[cfg(windows)]
type Platform = windows::WindowsSystemSettings;

pub struct SystemSettings {
    platform: Platform,
    /// Last known theme, to tell apart the color changes that switch it.
    is_dark_mode: AtomicBool,
    /// Built on first use, it is slow and doesn't change while running.
    entries: OnceLock<Vec<SystemSettingEntry>>,
}

event_manager!(SystemSettings, SystemSettingsEvent);

impl SystemSettings {
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<SystemSettings> = LazyLock::new(|| {
            let mut m = SystemSettings::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            is_dark_mode: AtomicBool::new(false),
            entries: OnceLock::new(),
        }
    }

    fn init(&mut self) -> Result<()> {
        self.is_dark_mode.store(
            self.platform.is_dark_mode().unwrap_or(false),
            Ordering::SeqCst,
        );
        self.platform.init(Self::on_platform_event)
    }

    fn on_platform_event(event: SystemSettingsSysEvent) {
        match event {
            SystemSettingsSysEvent::ColorsChanged => {
                Self::send(SystemSettingsEvent::ColorChanged);
                let manager = Self::instance();
                if let Ok(is_dark) = manager.platform.is_dark_mode() {
                    let previous = manager.is_dark_mode.swap(is_dark, Ordering::SeqCst);
                    if previous != is_dark {
                        Self::send(SystemSettingsEvent::ColorSchemeSwitched);
                    }
                }
            }
            SystemSettingsSysEvent::TextScaleChanged => {
                Self::send(SystemSettingsEvent::TextScaleChanged);
            }
        }
    }

    pub fn get_colors(&self) -> Result<UIColors> {
        self.platform.colors()
    }

    pub fn set_accent_color(&self, color: Color) -> Result<()> {
        log::info!(
            "Setting accent color to #{:02X}{:02X}{:02X}{:02X}",
            color.r,
            color.g,
            color.b,
            color.a
        );
        self.platform
            .set_accent_color(color, &generate_accent_palette(color))
    }

    pub fn get_dark_mode(&self) -> Result<bool> {
        self.platform.is_dark_mode()
    }

    pub fn set_dark_mode(&self, enabled: bool) -> Result<()> {
        log::info!("Setting dark mode to {enabled}");
        self.platform.set_dark_mode(enabled)
    }

    pub fn get_text_scale_factor(&self) -> Result<f64> {
        self.platform.text_scale_factor()
    }

    pub fn settings_entries(&self) -> Vec<SystemSettingEntry> {
        self.entries
            .get_or_init(|| self.platform.settings_entries())
            .clone()
    }

    pub fn night_light_settings(&self) -> Result<NightlightSettings> {
        self.platform.night_light_settings()
    }

    pub fn is_night_light_enabled(&self) -> Result<bool> {
        self.platform.is_night_light_enabled()
    }

    pub fn set_night_light_enabled(&self, enabled: bool) -> Result<()> {
        self.platform.set_night_light_enabled(enabled)
    }

    pub fn set_night_light_temperature(&self, temperature: u16) -> Result<()> {
        self.platform.set_night_light_temperature(temperature)
    }
}
