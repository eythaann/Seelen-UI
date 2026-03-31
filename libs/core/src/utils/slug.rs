use std::sync::LazyLock;

use schemars::JsonSchema;
use serde::{de::Visitor, Deserialize, Deserializer, Serialize};
use ts_rs::TS;

use crate::error::Result;

/// A URL-safe slug: lowercase ASCII letters, digits, and hyphens.
/// Must start and end with an alphanumeric character.
/// Example: `my-cool-theme`, `dark-mode-2`
#[derive(Debug, Default, Clone, PartialEq, Eq, Hash, JsonSchema, TS)]
#[ts(type = "string & { __brand: 'Slug' }")]
pub struct Slug(String);

impl Slug {
    fn regex() -> &'static regex::Regex {
        static REGEX: LazyLock<regex::Regex> =
            LazyLock::new(|| regex::Regex::new(r"^[a-z0-9]+(?:-[a-z0-9]+)*$").unwrap());
        &REGEX
    }

    pub fn is_valid(s: &str) -> bool {
        Self::regex().is_match(s)
    }

    pub fn validate(s: &str) -> Result<(), String> {
        if !Self::is_valid(s) {
            return Err(format!(
                "Invalid slug ({:?}): must match {}",
                s,
                Self::regex().as_str()
            ));
        }
        Ok(())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Converts an arbitrary string into the closest valid slug.
    /// Non-alphanumeric characters become hyphens; consecutive hyphens are collapsed;
    /// leading/trailing hyphens are removed. Returns an empty `Slug` if no
    /// alphanumeric characters are found.
    pub fn from_lossy(s: &str) -> Self {
        let slug = s
            .to_lowercase()
            .chars()
            .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
            .collect::<String>();

        let normalized = slug
            .split('-')
            .filter(|part| !part.is_empty())
            .collect::<Vec<_>>()
            .join("-");

        Self(normalized)
    }
}

impl std::fmt::Display for Slug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TryFrom<String> for Slug {
    type Error = String;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        if value.is_empty() {
            return Ok(Self(value));
        }
        Self::validate(&value)?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for Slug {
    type Error = String;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::try_from(value.to_string())
    }
}

impl Serialize for Slug {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Slug {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SlugVisitor;

        impl<'de> Visitor<'de> for SlugVisitor {
            type Value = Slug;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(
                    formatter,
                    "a slug matching the pattern: {}",
                    Slug::regex().as_str()
                )
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Slug::try_from(value).map_err(serde::de::Error::custom)
            }
        }

        deserializer.deserialize_str(SlugVisitor)
    }
}
