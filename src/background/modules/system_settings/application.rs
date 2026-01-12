use std::sync::LazyLock;

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
};
use seelen_core::system_state::UIColors;
use windows::{
    Foundation::TypedEventHandler,
    UI::ViewManagement::{UIColorType, UISettings},
};
use windows_core::IInspectable;

fn color_to_string(color: windows::UI::Color) -> String {
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        color.R, color.G, color.B, color.A
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SystemSettingsEvent {
    ColorChanged,
    TextScaleChanged,
}

pub struct SystemSettings {
    settings: UISettings,
    color_event_token: Option<i64>,
    text_scale_event_token: Option<i64>,
}

unsafe impl Send for SystemSettings {}

event_manager!(SystemSettings, SystemSettingsEvent);

impl SystemSettings {
    pub fn instance() -> &'static Self {
        static SYSTEM_SETTINGS: LazyLock<SystemSettings> = LazyLock::new(|| {
            let mut settings = SystemSettings::new();
            settings.init().log_error();
            settings
        });
        &SYSTEM_SETTINGS
    }

    fn new() -> Self {
        Self {
            settings: UISettings::new().expect("Failed to create UISettings"),
            color_event_token: None,
            text_scale_event_token: None,
        }
    }

    fn init(&mut self) -> Result<()> {
        // Register color change event
        // Windows-rs clones the handler internally, so we don't need to store it
        let color_token = self
            .settings
            .ColorValuesChanged(&TypedEventHandler::new(Self::internal_on_colors_change))?;
        self.color_event_token = Some(color_token);

        // Register text scale change event
        let text_scale_token = self
            .settings
            .TextScaleFactorChanged(&TypedEventHandler::new(Self::internal_on_text_scale_change))?;
        self.text_scale_event_token = Some(text_scale_token);

        Ok(())
    }

    fn internal_on_colors_change(
        _listener: &Option<UISettings>,
        _args: &Option<IInspectable>,
    ) -> windows_core::Result<()> {
        let _ = Self::event_tx().send(SystemSettingsEvent::ColorChanged);
        Ok(())
    }

    fn internal_on_text_scale_change(
        _listener: &Option<UISettings>,
        _args: &Option<IInspectable>,
    ) -> windows_core::Result<()> {
        let _ = Self::event_tx().send(SystemSettingsEvent::TextScaleChanged);
        Ok(())
    }

    pub fn get_colors(&self) -> Result<UIColors> {
        let settings = &self.settings;
        Ok(UIColors {
            background: color_to_string(settings.GetColorValue(UIColorType::Background)?),
            foreground: color_to_string(settings.GetColorValue(UIColorType::Foreground)?),
            accent_darkest: color_to_string(settings.GetColorValue(UIColorType::AccentDark3)?),
            accent_darker: color_to_string(settings.GetColorValue(UIColorType::AccentDark2)?),
            accent_dark: color_to_string(settings.GetColorValue(UIColorType::AccentDark1)?),
            accent: color_to_string(settings.GetColorValue(UIColorType::Accent)?),
            accent_light: color_to_string(settings.GetColorValue(UIColorType::AccentLight1)?),
            accent_lighter: color_to_string(settings.GetColorValue(UIColorType::AccentLight2)?),
            accent_lightest: color_to_string(settings.GetColorValue(UIColorType::AccentLight3)?),
            // https://learn.microsoft.com/is-is/uwp/api/windows.ui.viewmanagement.uisettings.getcolorvalue?view=winrt-19041#remarks
            complement: None,
        })
    }
}

impl Drop for SystemSettings {
    fn drop(&mut self) {
        if let Some(token) = self.color_event_token.take() {
            self.settings.RemoveColorValuesChanged(token).log_error();
        }

        if let Some(deferral) = self.text_scale_event_token.take() {
            self.settings
                .RemoveTextScaleFactorChanged(deferral)
                .log_error();
        }
    }
}
