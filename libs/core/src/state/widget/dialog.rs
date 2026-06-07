use std::collections::HashMap;

use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(default, rename_all = "camelCase")]
#[cfg_attr(feature = "gen-binds", ts(export))]
pub struct Dialog {
    pub identifier: uuid::Uuid,
    pub width: f64,
    pub height: f64,
    pub title: Vec<DialogContent>,
    pub content: Vec<DialogContent>,
    pub footer: Vec<DialogContent>,
}

impl Default for Dialog {
    fn default() -> Self {
        Self {
            identifier: uuid::Uuid::new_v4(),
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
#[cfg_attr(feature = "gen-binds", ts(export, optional_fields = nullable))]
pub enum DialogContent {
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
        skin: Option<String>,
        inner: Vec<DialogContent>,
        styles: Option<CssStyles>,
        /// event name to be emitted on click ex: `test::clicked`
        on_click: String,
    },
    Group {
        items: Vec<DialogContent>,
        styles: Option<CssStyles>,
    },
    Loader {
        styles: Option<CssStyles>,
    },
}

impl DialogContent {
    pub fn set_styles(&mut self, new_styles: CssStyles) {
        match self {
            DialogContent::Text { styles, .. } => *styles = Some(new_styles),
            DialogContent::Icon { styles, .. } => *styles = Some(new_styles),
            DialogContent::Image { styles, .. } => *styles = Some(new_styles),
            DialogContent::Button { styles, .. } => *styles = Some(new_styles),
            DialogContent::Group { styles, .. } => *styles = Some(new_styles),
            DialogContent::Loader { styles, .. } => *styles = Some(new_styles),
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
