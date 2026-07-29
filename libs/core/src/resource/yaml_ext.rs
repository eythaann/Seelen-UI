// the idea with this module is improve YAML with extensibility, via custom keywords

use std::{collections::HashMap, path::Path};

use serde_path_to_error;
use serde_yaml::{Mapping, Value};

use crate::error::Result;

/// Will deserialize a YAML file and parse the custom extended syntax
pub async fn deserialize_extended_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let value = read_and_parse_yml(path, true, &HashMap::new()).await?;
    serde_path_to_error::deserialize(value).map_err(|e| e.to_string().into())
}

/// Like [`deserialize_extended_yaml`] but leaves `${self.id}` literals intact so they can be
/// resolved at runtime with the published id.
///
/// This must also apply to `!extend`ed files (bundling resolves the whole tree), so the
/// `resolve_self: false` mode is threaded through [`resolve_extensions`] instead of only
/// being applied at the top level.
pub async fn deserialize_extended_yaml_no_vars<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> Result<T> {
    let value = read_and_parse_yml(path, false, &HashMap::new()).await?;
    serde_path_to_error::deserialize(value).map_err(|e| e.to_string().into())
}

/// `${self.id}` always refers to the id of the root document, never the id of whichever
/// `!extend`ed file is currently being read. `vars` is computed once, at the root (when
/// `inherited_vars` is empty), and then passed down unchanged through every `!extend` level -
/// it is inherited, not recomputed per file.
async fn read_and_parse_yml(
    path: &Path,
    resolve_self: bool,
    inherited_vars: &HashMap<String, String>,
) -> Result<Value> {
    let base = path.parent().ok_or("No parent directory")?.to_path_buf();
    let content = tokio::fs::read(path).await?;
    let raw_value = slice_to_yml_value(content).await?;

    if !resolve_self {
        return resolve_extensions(&base, raw_value, resolve_self, inherited_vars).await;
    }

    let vars = if inherited_vars.is_empty() {
        let mut vars = HashMap::new();
        vars.insert("self.id".to_string(), extract_self_id(&raw_value)?);
        vars
    } else {
        inherited_vars.clone()
    };

    let value = resolve_extensions(&base, raw_value, resolve_self, &vars).await?;
    Ok(resolve_vars_yaml(value, &vars))
}

// serde_yaml::from_str is CPU-bound and can take several ms even for small files.
// Offloading to the blocking thread pool prevents it from stalling tokio's async
// worker threads when many resources are loaded concurrently at startup.
async fn slice_to_yml_value(content: Vec<u8>) -> Result<Value> {
    tokio::task::spawn_blocking(move || {
        serde_yaml::from_slice::<Value>(&content).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(Into::into)
}

/// Extracts the top-level `id` field of a resource document, used as `self.id`.
///
/// Every Seelen UI resource document must declare an `id`, so a missing/invalid
/// field is an error rather than an absent value.
fn extract_self_id(value: &Value) -> Result<String> {
    value
        .as_mapping()
        .and_then(|m| m.get("id"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "Resource is missing required top-level 'id' field".into())
}

/// Extracts `self.id` for a `.slu` document, coming from `root["resource"]["id"]`
/// (the published UUID).
///
/// Every `.slu` file must carry this id, so a missing/invalid field is an error
/// rather than an absent value.
pub fn extract_self_id_slu(root: &Value) -> Result<String> {
    root.as_mapping()
        .and_then(|m| m.get("resource"))
        .and_then(Value::as_mapping)
        .and_then(|m| m.get("id"))
        .and_then(Value::as_str)
        .map(str::to_string)
        .ok_or_else(|| "SLU resource file is missing required 'resource.id' field".into())
}

/// Substitutes `${key}` placeholders in a string using the given variable map.
fn interpolate(s: String, vars: &HashMap<String, String>) -> String {
    let mut result = s;
    for (key, val) in vars {
        let placeholder = format!("${{{key}}}");
        if result.contains(&placeholder) {
            result = result.replace(&placeholder, val);
        }
    }
    result
}

/// Recursively substitutes `${self.id}` placeholders in all YAML string values.
pub fn resolve_vars_yaml(value: Value, vars: &HashMap<String, String>) -> Value {
    match value {
        Value::String(s) => Value::String(interpolate(s, vars)),

        Value::Mapping(map) => {
            let mut new_map = Mapping::new();
            for (k, v) in map {
                new_map.insert(k, resolve_vars_yaml(v, vars));
            }
            Value::Mapping(new_map)
        }

        Value::Sequence(seq) => Value::Sequence(
            seq.into_iter()
                .map(|v| resolve_vars_yaml(v, vars))
                .collect(),
        ),
        _ => value,
    }
}

async fn resolve_extensions(
    base: &Path,
    value: Value,
    resolve_self: bool,
    vars: &HashMap<String, String>,
) -> Result<Value> {
    match value {
        Value::Mapping(map) => {
            let futs = map.into_iter().map(|(key, val)| async move {
                let resolved = Box::pin(resolve_extensions(base, val, resolve_self, vars)).await?;
                Ok::<_, crate::error::SeelenLibError>((key, resolved))
            });
            let pairs: Vec<_> = futures::future::try_join_all(futs).await?;
            let mut new_map = Mapping::new();
            for (k, v) in pairs {
                new_map.insert(k, v);
            }
            Ok(Value::Mapping(new_map))
        }

        Value::Sequence(seq) => {
            let futs = seq
                .into_iter()
                .map(|val| Box::pin(resolve_extensions(base, val, resolve_self, vars)));
            let resolved = futures::future::try_join_all(futs).await?;
            Ok(Value::Sequence(resolved))
        }

        Value::Tagged(tag) => {
            if tag.tag == "!include" {
                if let Value::String(relative_path) = tag.value {
                    let to_include = base.join(relative_path);
                    let text = if to_include
                        .extension()
                        .is_some_and(|ext| ext == "scss" || ext == "sass")
                    {
                        // SCSS compilation is CPU-bound sync — offload to blocking pool
                        tokio::task::spawn_blocking(move || {
                            grass::from_path(&to_include, &grass::Options::default())
                        })
                        .await
                        .map_err(|e| crate::error::SeelenLibError::from(e.to_string()))??
                    } else {
                        tokio::fs::read_to_string(&to_include).await?
                    };
                    return Ok(Value::String(text));
                }
            }

            if tag.tag == "!extend" {
                if let Value::String(relative_path) = tag.value {
                    let value = Box::pin(read_and_parse_yml(
                        &base.join(relative_path),
                        resolve_self,
                        vars,
                    ))
                    .await?;
                    return Ok(value);
                }
            }

            Ok(Value::Tagged(tag))
        }
        _ => Ok(value),
    }
}
