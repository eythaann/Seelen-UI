use std::sync::Arc;

use crate::{error::Result, event_manager, windows_api::traits::EventRegistrationTokenExt};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use seelen_core::system_state::UIColors;
use windows::{
    Foundation::TypedEventHandler,
    Win32::System::WinRT::EventRegistrationToken,
    UI::ViewManagement::{UIColorType, UISettings},
};
use windows_core::IInspectable;

lazy_static! {
    pub static ref SYSTEM_SETTINGS: Arc<Mutex<SystemSettings>> = Arc::new(Mutex::new(
        SystemSettings::new().expect("Failed to create settings manager")
    ));
}

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
    color_event_handler: TypedEventHandler<UISettings, IInspectable>,
    color_event_token: Option<EventRegistrationToken>,
    text_scale_event_handler: TypedEventHandler<UISettings, IInspectable>,
    text_scale_event_token: Option<EventRegistrationToken>,
}

unsafe impl Send for SystemSettings {}

event_manager!(SystemSettings, SystemSettingsEvent);

impl SystemSettings {
    fn new() -> Result<Self> {
        let settings = Self {
            settings: UISettings::new()?,
            color_event_handler: TypedEventHandler::new(Self::internal_on_colors_change),
            color_event_token: None,
            text_scale_event_handler: TypedEventHandler::new(Self::internal_on_text_scale_change),
            text_scale_event_token: None,
        };
        Ok(settings)
    }

    pub fn initialize(&mut self) -> Result<()> {
        self.color_event_token = Some(
            self.settings
                .ColorValuesChanged(&self.color_event_handler)?
                .as_event_token(),
        );
        self.text_scale_event_token = Some(
            self.settings
                .TextScaleFactorChanged(&self.text_scale_event_handler)?
                .as_event_token(),
        );
        Ok(())
    }

    pub fn release(&mut self) -> Result<()> {
        if let Some(token) = self.color_event_token.take() {
            self.settings.RemoveColorValuesChanged(token.value)?;
        }
        if let Some(token) = self.text_scale_event_token.take() {
            self.settings.RemoveTextScaleFactorChanged(token.value)?;
        }
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
