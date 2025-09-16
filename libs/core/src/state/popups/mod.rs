use std::collections::HashMap;

use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct SluPopupConfig {
    pub width: f64,
    pub height: f64,
    pub title: Vec<SluPopupContent>,
    pub content: Vec<SluPopupContent>,
    pub footer: Vec<SluPopupContent>,
}

impl Default for SluPopupConfig {
    fn default() -> Self {
        Self {
            width: 400.0,
            height: 200.0,
            title: vec![],
            content: vec![],
            footer: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(
    tag = "type",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum SluPopupContent {
    Text {
        value: String,
        styles: Option<CssStyles>,
    },
    Icon {
        /// react icon name. ex: `FaGithub`
        name: String,
        styles: Option<CssStyles>,
    },
    Image {
        href: Url,
        styles: Option<CssStyles>,
    },
    Button {
        inner: Vec<SluPopupContent>,
        styles: Option<CssStyles>,
        /// event name to be emitted on click ex: `test::clicked`
        on_click: String,
    },
    Group {
        items: Vec<SluPopupContent>,
        styles: Option<CssStyles>,
    },
}

impl SluPopupContent {
    pub fn set_styles(&mut self, new_styles: CssStyles) {
        match self {
            SluPopupContent::Text { styles, .. } => *styles = Some(new_styles),
            SluPopupContent::Icon { styles, .. } => *styles = Some(new_styles),
            SluPopupContent::Image { styles, .. } => *styles = Some(new_styles),
            SluPopupContent::Button { styles, .. } => *styles = Some(new_styles),
            SluPopupContent::Group { styles, .. } => *styles = Some(new_styles),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize, TS)]
pub struct CssStyles(HashMap<String, String>);

impl CssStyles {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(mut self, key: &str, value: &str) -> Self {
        self.0.insert(key.to_string(), value.to_string());
        self
    }
}
