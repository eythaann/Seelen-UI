use std::sync::LazyLock;

use serde::{de::Visitor, Deserialize, Deserializer};

use crate::error::Result;

#[macro_export(local_inner_macros)]
macro_rules! identifier_impl {
    ($type:ty, $inner:ty) => {
        impl std::ops::Deref for $type {
            type Target = $inner;
            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $type {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }

        impl From<&str> for $type {
            fn from(value: &str) -> Self {
                Self(<$inner>::from(value))
            }
        }

        impl From<String> for $type {
            fn from(value: String) -> Self {
                Self(<$inner>::from(value))
            }
        }

        impl From<&String> for $type {
            fn from(value: &String) -> Self {
                Self(<$inner>::from(value))
            }
        }

        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                ::std::write!(f, "{}", self.0)
            }
        }
    };
}

/// Visual id composed of the creator username and the resource name. e.g. `@username/resource-name`
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, JsonSchema, TS)]
#[ts(type = "string & { __brand: 'ResourceId' }")]
pub struct ResourceId(String);

static REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^@[a-zA-Z][\w\-]{1,30}[a-zA-Z0-9]\/[a-zA-Z][\w\-]+[a-zA-Z0-9]$").unwrap()
});

impl ResourceId {
    fn regex() -> &'static regex::Regex {
        &REGEX
    }

    pub fn is_valid(&self) -> bool {
        Self::regex().is_match(&self.0)
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.is_valid() {
            return Err(format!(
                "Invalid resource id ({}), should follow the regex: {}",
                self.0,
                Self::regex()
            ));
        }
        Ok(())
    }

    /// Creator username of the resource
    ///
    /// # Safety
    ///
    /// The string is a valid resource id
    pub fn creator(&self) -> String {
        self.0
            .split('/')
            .next()
            .unwrap()
            .trim_start_matches('@')
            .to_string()
    }

    /// Name of the resource
    ///
    /// # Safety
    ///
    /// The string is a valid resource id
    pub fn resource_name(&self) -> String {
        self.0.split('/').next_back().unwrap().to_string()
    }
}

identifier_impl!(ResourceId, String);

impl Default for ResourceId {
    fn default() -> Self {
        Self("@unknown/unknown".to_owned())
    }
}

impl<'de> Deserialize<'de> for ResourceId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ResourceIdVisitor;

        impl<'de> Visitor<'de> for ResourceIdVisitor {
            type Value = ResourceId;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    formatter,
                    "a string matching the resource ID pattern: {}",
                    REGEX.as_str()
                )
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                // this step is to allow deserialize old ids used on older schemas (themes)
                let id = match value {
                    "toolbar" => WidgetId::known_toolbar().0,
                    "weg" => WidgetId::known_weg().0,
                    "wm" => WidgetId::known_wm().0,
                    "wall" => WidgetId::known_wall().0,
                    "settings" => WidgetId::known_settings().0,
                    "launcher" => "@deprecated/launcher".into(),
                    "popup" => WidgetId::known_popup().0,
                    _ => ResourceId(value.to_string()),
                };
                id.validate().map_err(serde::de::Error::custom)?;
                Ok(id)
            }
        }

        deserializer.deserialize_str(ResourceIdVisitor)
    }
}

macro_rules! resource_id_variant {
    ($name:ident) => {
        /// Visual id composed of the creator username and the resource name. e.g. `@username/resource-name`
        #[derive(
            Debug, Clone, Hash, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS,
        )]
        pub struct $name(ResourceId);
        identifier_impl!($name, ResourceId);
    };
}

resource_id_variant!(PluginId);
resource_id_variant!(IconPackId);
resource_id_variant!(ThemeId);
resource_id_variant!(WidgetId);
resource_id_variant!(WallpaperId);

impl WidgetId {
    pub fn known_settings() -> Self {
        "@seelen/settings".into()
    }
    pub fn known_weg() -> Self {
        "@seelen/weg".into()
    }
    pub fn known_toolbar() -> Self {
        "@seelen/fancy-toolbar".into()
    }
    pub fn known_wm() -> Self {
        "@seelen/window-manager".into()
    }
    pub fn known_wall() -> Self {
        "@seelen/wallpaper-manager".into()
    }
    pub fn known_popup() -> Self {
        "@seelen/popup".into()
    }
}
