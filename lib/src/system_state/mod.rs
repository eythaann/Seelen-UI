use serde::Serialize;

/// https://learn.microsoft.com/is-is/uwp/api/windows.ui.viewmanagement.uicolortype?view=winrt-19041
#[derive(Debug, Default, Serialize)]
pub struct UIColors {
    pub background: String,
    pub foreground: String,
    pub accent_darkest: String,
    pub accent_darker: String,
    pub accent_dark: String,
    pub accent: String,
    pub accent_light: String,
    pub accent_lighter: String,
    pub accent_lightest: String,
    pub complement: Option<String>,
}
