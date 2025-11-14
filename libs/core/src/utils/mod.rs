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
