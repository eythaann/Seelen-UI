use std::{borrow::Cow, fmt::Display, str::FromStr};

use schemars::{JsonSchema, Schema, SchemaGenerator, json_schema};
use serde::{Deserializer, de::Visitor};

/// Native handle of a window (`HWND` on Windows).
///
/// Unique among the live windows of the session, so it is enough (and cheaper) to identify a window
/// at runtime. But the OS can reuse it once the window is destroyed, so it must never be used for
/// persisted, async or security sensitive data, where it could end up pointing to another window:
/// use [`WindowIdentity`] for those.
pub type WindowHandle = isize;

/// Unique identifier of a window that, unlike its handle (`hwnd`), survives app restarts and
/// reboots: the OS can reuse a handle value once the window is destroyed, but the pair
/// `(hwnd, process_created_at)` is never reused. `pid` is not needed for uniqueness, it is a cheap
/// early check before querying the process creation time.
///
/// Serialized as the hex string `"{hwnd}_{pid}_{process_created_at}"`, so it can be used as a map
/// key in JSON/YAML. For backward compatibility it can also be deserialized from the window handle
/// used as identifier before: a plain number, or its decimal string (as map keys were sent), see
/// [`WindowIdentity::is_handle_only`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(
    all(feature = "gen-binds", not(feature = "salvo")),
    ts(type = "string")
)]
#[serde(into = "String")]
pub struct WindowIdentity {
    pub hwnd: WindowHandle,
    pub pid: u32,
    /// FILETIME (100ns ticks since 1601-01-01 UTC)
    pub process_created_at: u64,
}

impl WindowIdentity {
    /// Identity with only the window handle known, the process data is left as 0.
    pub fn from_handle(hwnd: WindowHandle) -> Self {
        Self {
            hwnd,
            pid: 0,
            process_created_at: 0,
        }
    }

    /// Created from a bare handle (e.g. stored by an old version), so it can not be validated
    /// against the window's process: the handle could belong to another window by now.
    pub fn is_handle_only(&self) -> bool {
        self.pid == 0 && self.process_created_at == 0
    }
}

impl Display for WindowIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // hwnd as usize so a (theoretical) negative handle is written as its bits, not with a sign
        write!(
            f,
            "{:x}_{:x}_{:x}",
            self.hwnd as usize, self.pid, self.process_created_at
        )
    }
}

impl FromStr for WindowIdentity {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || format!("invalid window identity: {s:?}");
        let mut parts = s.split('_');
        let mut next = || parts.next().ok_or_else(invalid);
        let identity = Self {
            hwnd: usize::from_str_radix(next()?, 16).map_err(|_| invalid())? as WindowHandle,
            pid: u32::from_str_radix(next()?, 16).map_err(|_| invalid())?,
            process_created_at: u64::from_str_radix(next()?, 16).map_err(|_| invalid())?,
        };
        if parts.next().is_some() {
            return Err(invalid());
        }
        Ok(identity)
    }
}

impl From<WindowIdentity> for String {
    fn from(value: WindowIdentity) -> Self {
        value.to_string()
    }
}

impl<'de> serde::Deserialize<'de> for WindowIdentity {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(WindowIdentityVisitor)
    }
}

/// Documented as a string, the legacy number form is only accepted for backward compatibility.
impl JsonSchema for WindowIdentity {
    fn schema_name() -> Cow<'static, str> {
        "WindowIdentity".into()
    }

    fn json_schema(_generator: &mut SchemaGenerator) -> Schema {
        json_schema!({ "type": "string" })
    }
}

struct WindowIdentityVisitor;

impl Visitor<'_> for WindowIdentityVisitor {
    type Value = WindowIdentity;

    fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("a window identity string or a window handle number")
    }

    fn visit_str<E: serde::de::Error>(self, v: &str) -> Result<Self::Value, E> {
        // legacy: a handle as decimal string, e.g. the keys of `{ [hwnd]: value }` maps
        if !v.contains('_')
            && let Ok(hwnd) = v.parse::<WindowHandle>()
        {
            return Ok(WindowIdentity::from_handle(hwnd));
        }
        v.parse().map_err(E::custom)
    }

    fn visit_i64<E: serde::de::Error>(self, v: i64) -> Result<Self::Value, E> {
        Ok(WindowIdentity::from_handle(v as WindowHandle))
    }

    fn visit_u64<E: serde::de::Error>(self, v: u64) -> Result<Self::Value, E> {
        Ok(WindowIdentity::from_handle(v as WindowHandle))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::WindowIdentity;

    const IDENTITY: WindowIdentity = WindowIdentity {
        hwnd: 132456,
        pid: 1234,
        process_created_at: 133_456_789_012_345_678,
    };

    #[test]
    fn identity_roundtrips_as_string() {
        let json = serde_json::to_string(&IDENTITY).unwrap();
        assert_eq!(json, "\"20568_4d2_1da223e15f1f34e\"");
        assert_eq!(
            serde_json::from_str::<WindowIdentity>(&json).unwrap(),
            IDENTITY
        );
    }

    #[test]
    fn identity_works_as_map_key() {
        let map = HashMap::from([(IDENTITY, 1)]);
        let json = serde_json::to_string(&map).unwrap();
        let back: HashMap<WindowIdentity, i32> = serde_json::from_str(&json).unwrap();
        assert_eq!(back, map);

        let yaml = serde_yaml::to_string(&map).unwrap();
        let back: HashMap<WindowIdentity, i32> = serde_yaml::from_str(&yaml).unwrap();
        assert_eq!(back, map);
    }

    #[test]
    fn identity_rejects_malformed_strings() {
        for s in ["", "1_2", "1_2_3_4", "g_2_3", "1__3", "1_-2_3"] {
            assert!(s.parse::<WindowIdentity>().is_err(), "{s:?} should fail");
        }
    }

    #[test]
    fn identity_accepts_legacy_handle_numbers() {
        let ids: Vec<WindowIdentity> = serde_json::from_str("[132456]").unwrap();
        assert_eq!(ids, vec![WindowIdentity::from_handle(132456)]);
        assert!(ids[0].is_handle_only());
        assert_eq!(serde_json::to_string(&ids).unwrap(), "[\"20568_0_0\"]");
    }

    #[test]
    fn identity_accepts_legacy_handle_map_keys() {
        let map: HashMap<WindowIdentity, i32> = serde_json::from_str(r#"{"132456": 1}"#).unwrap();
        assert_eq!(
            map,
            HashMap::from([(WindowIdentity::from_handle(132456), 1)])
        );
        assert!(serde_json::from_str::<WindowIdentity>("\"\"").is_err());
    }

    #[test]
    fn identity_schema_is_a_string() {
        let schema = schemars::schema_for!(WindowIdentity);
        assert_eq!(schema.get("type").and_then(|t| t.as_str()), Some("string"));
    }
}
