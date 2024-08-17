use std::sync::Arc;

use crate::{error_handler::Result, log_error, trace_lock};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use windows::{
    Foundation::{EventRegistrationToken, TypedEventHandler},
    UI::ViewManagement::{UIColorType, UISettings},
};
use windows_core::IInspectable;

use super::domain::UIColors;

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

enum SettingsEvent {
    ColorChanged,
}

type ColorChangeCallback = Box<dyn Fn(&UIColors) + Send + Sync>;

pub struct SystemSettings {
    settings: UISettings,
    color_event_handler: TypedEventHandler<UISettings, IInspectable>,
    color_event_token: Option<EventRegistrationToken>,
    color_client_callbacks: Vec<ColorChangeCallback>,
}

unsafe impl Send for SystemSettings {}

impl SystemSettings {
    fn new() -> Result<Self> {
        let mut settings = Self {
            settings: UISettings::new()?,
            color_event_handler: TypedEventHandler::new(Self::internal_on_colors_change),
            color_event_token: None,
            color_client_callbacks: Vec::new(),
        };
        settings.init()?;
        Ok(settings)
    }

    fn init(&mut self) -> Result<()> {
        self.color_event_token = Some(
            self.settings
                .ColorValuesChanged(&self.color_event_handler)?,
        );
        Ok(())
    }

    pub fn release(&mut self) -> Result<()> {
        self.color_client_callbacks.clear();
        if let Some(token) = self.color_event_token.take() {
            self.settings.RemoveColorValuesChanged(token)?;
        }
        Ok(())
    }

    fn internal_on_colors_change(
        _listener: &Option<UISettings>,
        _args: &Option<IInspectable>,
    ) -> windows_core::Result<()> {
        log_error!(trace_lock!(SYSTEM_SETTINGS).on_change(SettingsEvent::ColorChanged));
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

    pub fn on_colors_change(&mut self, callback: ColorChangeCallback) {
        self.color_client_callbacks.push(callback);
    }

    fn on_change(&mut self, event: SettingsEvent) -> Result<()> {
        match event {
            SettingsEvent::ColorChanged => {
                let colors = self.get_colors()?;
                for callback in self.color_client_callbacks.iter() {
                    callback(&colors);
                }
            }
        }
        Ok(())
    }
}
