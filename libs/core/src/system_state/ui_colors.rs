/// https://learn.microsoft.com/is-is/uwp/api/windows.ui.viewmanagement.uicolortype?view=winrt-19041
#[derive(Debug, Default, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
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

/// since v2.2.0 this should be used to handle every used color in the app
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorFormat {
    Rgba(u32),
    Rgb(u32),
    Bgra(u32),
    Bgr(u32),
}

impl Color {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn parse(format: ColorFormat) -> Self {
        match format {
            ColorFormat::Rgba(rgba) => {
                let [r, g, b, a] = rgba.to_be_bytes();
                Color::new(r, g, b, a)
            }
            ColorFormat::Rgb(rgb) => {
                let [_, r, g, b] = rgb.to_be_bytes();
                Color::new(r, g, b, 0xFF)
            }
            ColorFormat::Bgra(bgra) => {
                let [b, g, r, a] = bgra.to_be_bytes();
                Color::new(r, g, b, a)
            }
            ColorFormat::Bgr(bgr) => {
                let [_, b, g, r] = bgr.to_be_bytes();
                Color::new(r, g, b, 0xFF)
            }
        }
    }
}
