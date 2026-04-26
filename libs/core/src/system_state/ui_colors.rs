/// https://learn.microsoft.com/is-is/uwp/api/windows.ui.viewmanagement.uicolortype?view=winrt-19041
#[derive(Debug, Default, Serialize, Deserialize, TS)]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct UIColors {
    pub accent_lightest: Color,
    pub accent_lighter: Color,
    pub accent_light: Color,
    pub accent: Color,
    pub accent_dark: Color,
    pub accent_darker: Color,
    pub accent_darkest: Color,
    pub complement: Color,
}

impl UIColors {
    pub fn from_bytes(bytes: &[u8; 32]) -> Self {
        let chunks = bytes.chunks_exact(4);
        let colors: Vec<Color> = chunks
            .map(|chunk| Color::new(chunk[0], chunk[1], chunk[2], chunk[3]))
            .collect();
        Self {
            accent_lightest: colors[0],
            accent_lighter: colors[1],
            accent_light: colors[2],
            accent: colors[3],
            accent_dark: colors[4],
            accent_darker: colors[5],
            accent_darkest: colors[6],
            complement: colors[7],
        }
    }

    pub fn to_bytes(&self) -> [u8; 32] {
        let mut bytes = [0u8; 32];
        let colors = [
            self.accent_lightest,
            self.accent_lighter,
            self.accent_light,
            self.accent,
            self.accent_dark,
            self.accent_darker,
            self.accent_darkest,
            self.complement,
        ];
        for (i, color) in colors.iter().enumerate() {
            bytes[i * 4..(i + 1) * 4].copy_from_slice(&[color.r, color.g, color.b, color.a]);
        }
        bytes
    }
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
