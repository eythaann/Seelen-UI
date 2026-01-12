use std::sync::LazyLock;

use serde::{de::Visitor, Deserialize, Deserializer, Serialize};

use crate::error::Result;

/// For local resources this will be an visual id composed of the creator username and the resource name. e.g. `@username/resource-name`
/// For downloaded resources this will be an UUID.
#[derive(Debug, Clone, PartialEq, Eq, Hash, JsonSchema, TS)]
#[ts(type = "string & { __brand: 'ResourceId' }")]
pub enum ResourceId {
    Local(String),
    Remote(uuid::Uuid),
}

impl ResourceId {
    fn regex() -> &'static regex::Regex {
        static REGEX: LazyLock<regex::Regex> = LazyLock::new(|| {
            regex::Regex::new(r"^@[a-zA-Z][\w\-]{1,30}[a-zA-Z0-9]\/[a-zA-Z][\w\-]+[a-zA-Z0-9]$")
                .unwrap()
        });
        &REGEX
    }

    pub fn is_valid(&self) -> bool {
        match self {
            ResourceId::Local(id) => Self::regex().is_match(id),
            ResourceId::Remote(_) => true, // UUIDs are always valid
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        if !self.is_valid() {
            let id_str = match self {
                ResourceId::Local(id) => id.as_str(),
                ResourceId::Remote(_) => return Ok(()), // UUIDs are always valid
            };
            return Err(format!(
                "Invalid resource id ({}), should follow the regex: {}",
                id_str,
                Self::regex()
            ));
        }
        Ok(())
    }

    pub fn as_str(&self) -> &str {
        match self {
            ResourceId::Local(id) => id.as_str(),
            ResourceId::Remote(_) => "",
        }
    }

    pub fn starts_with(&self, prefix: &str) -> bool {
        match self {
            ResourceId::Local(id) => id.starts_with(prefix),
            ResourceId::Remote(_) => false,
        }
    }

    /// Creator username of the resource, will be none for remote resources
    pub fn creator(&self) -> Option<String> {
        match self {
            ResourceId::Local(id) => Some(
                id.split('/')
                    .next()
                    .unwrap()
                    .trim_start_matches('@')
                    .to_string(),
            ),
            ResourceId::Remote(_) => None,
        }
    }

    /// Name of the resource, will be none for remote resources
    pub fn resource_name(&self) -> Option<String> {
        match self {
            ResourceId::Local(id) => Some(id.split('/').nth(1).unwrap().to_string()),
            ResourceId::Remote(_) => None,
        }
    }
}

impl Default for ResourceId {
    fn default() -> Self {
        Self::Local("@unknown/unknown".to_owned())
    }
}

impl From<&str> for ResourceId {
    fn from(value: &str) -> Self {
        Self::from(value.to_string())
    }
}

impl From<String> for ResourceId {
    fn from(value: String) -> Self {
        // Try to parse as UUID first, otherwise treat as local ID
        if let Ok(uuid) = uuid::Uuid::try_parse(&value) {
            ResourceId::Remote(uuid)
        } else {
            ResourceId::Local(value)
        }
    }
}

impl From<uuid::Uuid> for ResourceId {
    fn from(value: uuid::Uuid) -> Self {
        ResourceId::Remote(value)
    }
}

impl std::fmt::Display for ResourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResourceId::Local(id) => write!(f, "{}", id),
            ResourceId::Remote(uuid) => write!(f, "{}", uuid),
        }
    }
}

impl Serialize for ResourceId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ResourceId::Local(id) => serializer.serialize_str(id),
            ResourceId::Remote(id) => serializer.serialize_str(&id.to_string()),
        }
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
                    ResourceId::regex().as_str()
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
                    _ => {
                        // Try to parse as UUID first, otherwise treat as local ID
                        if let Ok(uuid) = uuid::Uuid::try_parse(value) {
                            ResourceId::Remote(uuid)
                        } else {
                            ResourceId::Local(value.to_string())
                        }
                    }
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
        crate::identifier_impl!($name, ResourceId);

        impl From<ResourceId> for $name {
            fn from(value: ResourceId) -> Self {
                Self(value)
            }
        }
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
