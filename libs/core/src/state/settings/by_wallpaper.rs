#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(default, rename_all = "camelCase")]
pub struct WallpaperInstanceSettings {
    /// playback speed for video backgrounds
    pub playback_speed: PlaybackSpeed,
    /// will flip the image/video vertically
    pub flip_vertical: bool,
    /// will flip the image/video horizontally
    pub flip_horizontal: bool,
    /// blur factor to apply to the image
    pub blur: u32,
    /// method to fill the monitor background
    pub object_fit: ObjectFit,
    /// position of the background
    pub object_position: ObjectPosition,
    /// number between 0 and 2
    pub saturation: f32,
    /// number between 0 and 2
    pub contrast: f32,
    /// will overlay the image/video with a color filter
    pub with_overlay: bool,
    pub overlay_mix_blend_mode: MixBlendMode,
    pub overlay_color: String,
    /// mute video backgrounds
    pub muted: bool,
}

impl Default for WallpaperInstanceSettings {
    fn default() -> Self {
        Self {
            playback_speed: PlaybackSpeed::default(),
            flip_vertical: false,
            flip_horizontal: false,
            blur: 0,
            object_fit: ObjectFit::default(),
            object_position: ObjectPosition::default(),
            saturation: 1.0,
            contrast: 1.0,
            with_overlay: false,
            overlay_mix_blend_mode: MixBlendMode::default(),
            overlay_color: "#ff0000".to_string(),
            muted: true,
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(repr(enum = name))]
pub enum ObjectFit {
    Fill,
    Contain,
    #[default]
    Cover,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(repr(enum = name))]
pub enum ObjectPosition {
    Top,
    #[default]
    Center,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(repr(enum = name))]
pub enum MixBlendMode {
    Normal,
    #[default]
    Multiply,
    Screen,
    Overlay,
    Darken,
    Lighten,
    ColorDodge,
    ColorBurn,
    HardLight,
    SoftLight,
    Difference,
    Exclusion,
    Hue,
    Saturation,
    Color,
    Luminosity,
    PlusDarker,
    PlusLighter,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema, TS)]
#[serde(rename_all = "camelCase")]
#[ts(repr(enum = name))]
pub enum PlaybackSpeed {
    XDot25,
    XDot5,
    XDot75,
    #[default]
    X1,
    X1Dot25,
    X1Dot5,
    X1Dot75,
    X2,
}
