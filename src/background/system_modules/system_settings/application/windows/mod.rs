mod index;
mod index_schema;
mod nightlight;

use std::sync::Arc;

use seelen_core::system_state::{Color, NightlightSettings, SystemSettingEntry, UIColors};
use windows::{
    Foundation::TypedEventHandler,
    UI::ViewManagement::{UIColorType, UISettings},
};
use windows_core::IInspectable;
use winreg::{
    RegKey, RegValue,
    enums::{HKEY_CURRENT_USER, KEY_SET_VALUE, REG_BINARY},
};

use crate::{
    error::{Result, ResultLogExt},
    system_modules::system_settings::domain::{SystemSettingsSysEvent, SystemSettingsSystem},
    windows_api::WindowsApi,
};

const ACCENT_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\Accent";
const PERSONALIZE_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Themes\Personalize";

type EventCallback = Arc<dyn Fn(SystemSettingsSysEvent) + Send + Sync>;

pub struct WindowsSystemSettings {
    settings: Option<UISettings>,
    color_event_token: Option<i64>,
    text_scale_event_token: Option<i64>,
}

// SAFETY: `UISettings` is only used to read values and to remove the event handlers
// (as the previous manager did, also with `unsafe impl Send`).
unsafe impl Send for WindowsSystemSettings {}
unsafe impl Sync for WindowsSystemSettings {}

impl SystemSettingsSystem for WindowsSystemSettings {
    fn create() -> Self {
        Self {
            settings: None,
            color_event_token: None,
            text_scale_event_token: None,
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(SystemSettingsSysEvent) + Send + Sync + 'static,
    {
        let on_event: EventCallback = Arc::new(on_event);
        let settings = UISettings::new()?;

        // Windows-rs clones the handler internally, so we don't need to store it
        let cb = on_event.clone();
        self.color_event_token = Some(settings.ColorValuesChanged(&TypedEventHandler::new(
            move |_: windows_core::Ref<UISettings>, _: windows_core::Ref<IInspectable>| {
                cb(SystemSettingsSysEvent::ColorsChanged);
                Ok(())
            },
        ))?);

        self.text_scale_event_token =
            Some(settings.TextScaleFactorChanged(&TypedEventHandler::new(
                move |_: windows_core::Ref<UISettings>, _: windows_core::Ref<IInspectable>| {
                    on_event(SystemSettingsSysEvent::TextScaleChanged);
                    Ok(())
                },
            ))?);

        self.settings = Some(settings);
        Ok(())
    }

    fn colors(&self) -> Result<UIColors> {
        let settings = self.settings()?;
        let color =
            |kind| -> Result<Color> { Ok(color_from_native(settings.GetColorValue(kind)?)) };
        Ok(UIColors {
            accent_darkest: color(UIColorType::AccentDark3)?,
            accent_darker: color(UIColorType::AccentDark2)?,
            accent_dark: color(UIColorType::AccentDark1)?,
            accent: color(UIColorType::Accent)?,
            accent_light: color(UIColorType::AccentLight1)?,
            accent_lighter: color(UIColorType::AccentLight2)?,
            accent_lightest: color(UIColorType::AccentLight3)?,
            // https://learn.microsoft.com/is-is/uwp/api/windows.ui.viewmanagement.uisettings.getcolorvalue?view=winrt-19041#remarks
            complement: Color::default(),
        })
    }

    fn set_accent_color(&self, color: Color, palette: &UIColors) -> Result<()> {
        let key =
            RegKey::predef(HKEY_CURRENT_USER).open_subkey_with_flags(ACCENT_KEY, KEY_SET_VALUE)?;
        // Registry stores the accent color as 0xAABBGGRR
        let dword: u32 = u32::from_be_bytes([color.a, color.b, color.g, color.r]);
        key.set_value("AccentColor", &dword)?;
        key.set_value("AccentColorMenu", &dword)?;
        key.set_value("StartColorMenu", &dword)?;
        key.set_raw_value(
            "AccentPalette",
            &RegValue {
                bytes: palette.to_bytes().to_vec(),
                vtype: REG_BINARY,
            },
        )?;
        WindowsApi::broadcast_setting_change("ImmersiveColorSet");
        Ok(())
    }

    fn is_dark_mode(&self) -> Result<bool> {
        let key = RegKey::predef(HKEY_CURRENT_USER).open_subkey(PERSONALIZE_KEY)?;
        let apps_use_light_theme: u32 = key.get_value("AppsUseLightTheme").unwrap_or(1);
        Ok(apps_use_light_theme == 0)
    }

    fn set_dark_mode(&self, enabled: bool) -> Result<()> {
        let key = RegKey::predef(HKEY_CURRENT_USER)
            .open_subkey_with_flags(PERSONALIZE_KEY, KEY_SET_VALUE)?;
        let dword: u32 = if enabled { 0 } else { 1 };
        key.set_value("AppsUseLightTheme", &dword)?;
        key.set_value("SystemUsesLightTheme", &dword)?;
        WindowsApi::broadcast_setting_change("ImmersiveColorSet");
        Ok(())
    }

    fn text_scale_factor(&self) -> Result<f64> {
        Ok(self.settings()?.TextScaleFactor()?)
    }

    fn settings_entries(&self) -> Vec<SystemSettingEntry> {
        index::build_entries()
    }

    fn night_light_settings(&self) -> Result<NightlightSettings> {
        nightlight::get_settings()
    }

    fn is_night_light_enabled(&self) -> Result<bool> {
        nightlight::get_enabled()
    }

    fn set_night_light_enabled(&self, enabled: bool) -> Result<()> {
        nightlight::set_enabled(enabled)
    }

    fn set_night_light_temperature(&self, temperature: u16) -> Result<()> {
        nightlight::set_color_temperature(temperature)
    }
}

impl WindowsSystemSettings {
    fn settings(&self) -> Result<&UISettings> {
        Ok(self
            .settings
            .as_ref()
            .ok_or("System settings not initialized")?)
    }
}

impl Drop for WindowsSystemSettings {
    fn drop(&mut self) {
        let Some(settings) = &self.settings else {
            return;
        };
        if let Some(token) = self.color_event_token.take() {
            settings.RemoveColorValuesChanged(token).log_error();
        }
        if let Some(token) = self.text_scale_event_token.take() {
            settings.RemoveTextScaleFactorChanged(token).log_error();
        }
    }
}

fn color_from_native(color: windows::UI::Color) -> Color {
    Color {
        r: color.R,
        g: color.G,
        b: color.B,
        a: color.A,
    }
}
