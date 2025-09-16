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

impl From<serde_json::Value> for TsUnknown {
    fn from(value: serde_json::Value) -> Self {
        TsUnknown(value)
    }
}

pub fn search_for_metadata_file(folder: &Path) -> Option<PathBuf> {
    for entry in std::fs::read_dir(folder).ok()?.flatten() {
        if entry
            .file_name()
            .to_string_lossy()
            .to_lowercase()
            .starts_with("metadata")
        {
            return Some(entry.path());
        }
    }
    None
}
