use std::path::{Path, PathBuf};

use schemars::JsonSchema;

#[macro_export(local_inner_macros)]
macro_rules! __switch {
    {
        if { $($if:tt)+ }
        do { $($do:tt)* }
        else { $($else:tt)* }
    } => { $($do)* };
    {
        if { }
        do { $($do:tt)* }
        else { $($else:tt)* }
    } => { $($else)* };
}

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

        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                ::std::write!(f, "{}", self.0)
            }
        }
    };
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema, TS)]
#[ts(type = "unknown")]
pub struct TsUnknown(pub serde_json::Value);

impl<T: Into<serde_json::Value>> From<T> for TsUnknown {
    fn from(value: T) -> Self {
        TsUnknown(value.into())
    }
}

static ALLOWED_ROOT_FILESTEMS: &[&str] = &["metadata", "index", "mod", "main"];
static ALLOWED_ROOT_EXTENSIONS: &[&str] = &["yml", "yaml", "slu", "json"];
pub fn search_resource_entrypoint(folder: &Path) -> Option<PathBuf> {
    if folder.is_file() {
        return None;
    }
    for filestem in ALLOWED_ROOT_FILESTEMS {
        for extension in ALLOWED_ROOT_EXTENSIONS {
            let path = folder.join(format!("{filestem}.{extension}"));
            if path.is_file() {
                return Some(path);
            }
        }
    }
    None
}
