mod slug;
pub mod traits;

pub use slug::*;

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

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(all(feature = "gen-binds", not(feature = "salvo")), derive(ts_rs::TS))]
#[cfg_attr(
    all(feature = "gen-binds", not(feature = "salvo")),
    ts(type = "unknown")
)]
pub struct TsUnknown(pub serde_json::Value);

impl<T: Into<serde_json::Value>> From<T> for TsUnknown {
    fn from(value: T) -> Self {
        TsUnknown(value.into())
    }
}

static ALLOWED_ROOT_FILESTEMS: &[&str] = &["metadata"];
static ALLOWED_ROOT_EXTENSIONS: &[&str] = &["yml", "slu", "yaml", "json"];
pub async fn search_resource_entrypoint(folder: &Path) -> Option<PathBuf> {
    // Build all candidates in priority order (stem × extension),
    // then probe all of them concurrently instead of sequentially.
    // The first candidate in priority order that exists is returned.
    let candidates: Vec<PathBuf> = ALLOWED_ROOT_FILESTEMS
        .iter()
        .flat_map(|stem| {
            ALLOWED_ROOT_EXTENSIONS
                .iter()
                .map(move |ext| folder.join(format!("{stem}.{ext}")))
        })
        .collect();

    let exists: Vec<bool> = futures::future::join_all(
        candidates
            .iter()
            .map(|p| async move { tokio::fs::metadata(p).await.is_ok_and(|m| m.is_file()) }),
    )
    .await;

    candidates
        .into_iter()
        .zip(exists)
        .find_map(|(path, found)| found.then_some(path))
}
