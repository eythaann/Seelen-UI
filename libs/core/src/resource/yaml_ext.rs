// the idea with this module is improve YAML with extensibility, via custom keywords

use std::{collections::HashMap, path::Path};

use serde_path_to_error;
use serde_yaml::{Mapping, Value};

use crate::error::Result;

/// Will deserialize a YAML file and parse the custom extended syntax
pub async fn deserialize_extended_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let value = read_and_parse_yml(path).await?;
    serde_path_to_error::deserialize(value).map_err(|e| e.to_string().into())
}

// serde_yaml::from_str is CPU-bound and can take several ms even for small files.
// Offloading to the blocking thread pool prevents it from stalling tokio's async
// worker threads when many resources are loaded concurrently at startup.
async fn parse_yaml_content(content: Vec<u8>) -> Result<Value> {
    tokio::task::spawn_blocking(move || {
        serde_yaml::from_slice::<Value>(&content).map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
    .map_err(Into::into)
}

async fn read_and_parse_yml(path: &Path) -> Result<Value> {
    let base = path.parent().ok_or("No parent directory")?.to_path_buf();
    let content = tokio::fs::read(path).await?;
    let value = parse_yaml_content(content).await?;
    let value = resolve_extensions(&base, value).await?;
    let vars = collect_vars(value.as_mapping(), value.as_mapping(), true);
    Ok(resolve_vars_yaml(value, &vars))
}

/// Like [`deserialize_extended_yaml`] but only resolves `vars.*` placeholders, leaving
/// `${self.*}` literals intact so they can be resolved at runtime with the published id.
pub async fn deserialize_extended_yaml_no_vars<T: serde::de::DeserializeOwned>(
    path: &Path,
) -> Result<T> {
    let base = path.parent().ok_or("No parent directory")?.to_path_buf();
    let content = tokio::fs::read(path).await?;
    let value = parse_yaml_content(content).await?;
    let value = resolve_extensions(&base, value).await?;
    let vars = collect_vars(value.as_mapping(), value.as_mapping(), false);

    serde_path_to_error::deserialize(resolve_vars_yaml(value, &vars))
        .map_err(|e| e.to_string().into())
}

/// Builds a variable map for a `.slu` document.
///
/// `self.id` comes from `root["resource"]["id"]` (the published UUID).
/// Other `self.*` and `vars.*` come from `root["data"]`.
pub fn extract_vars_slu(root: &Value) -> HashMap<String, String> {
    let self_id_source = root
        .as_mapping()
        .and_then(|m| m.get("resource"))
        .and_then(Value::as_mapping);

    let data = root
        .as_mapping()
        .and_then(|m| m.get("data"))
        .and_then(Value::as_mapping);

    let mut vars = collect_vars(self_id_source, data, true);

    // self.id from resource overrides whatever may be in data
    if let Some(id) = self_id_source
        .and_then(|m| m.get("id"))
        .and_then(Value::as_str)
    {
        vars.insert("self.id".to_string(), id.to_string());
    }

    vars
}

/// Collects `self.*` (from `self_map`) and `vars.*` (from `vars_map["vars"]`) into one map.
/// Pass `resolve_self: false` to skip `self.*` collection (bundle mode).
fn collect_vars(
    self_map: Option<&Mapping>,
    vars_map: Option<&Mapping>,
    resolve_self: bool,
) -> HashMap<String, String> {
    let mut vars = HashMap::new();

    if resolve_self {
        if let Some(map) = self_map {
            for (k, v) in map {
                if let (Some(k), Some(v)) = (k.as_str(), v.as_str()) {
                    vars.entry(format!("self.{k}"))
                        .or_insert_with(|| v.to_string());
                }
            }
        }
    }

    if let Some(user_vars) = vars_map
        .and_then(|m| m.get("vars"))
        .and_then(Value::as_mapping)
    {
        for (k, v) in user_vars {
            if let (Some(k), Some(v)) = (k.as_str(), v.as_str()) {
                vars.insert(format!("vars.{k}"), v.to_string());
            }
        }
    }

    vars
}

/// Substitutes `${key}` placeholders in a string using the given variable map.
fn interpolate(s: String, vars: &HashMap<String, String>) -> String {
    let mut result = s;
    for (key, val) in vars {
        result = result.replace(&format!("${{{key}}}"), val);
    }
    result
}

/// Recursively substitutes `${key}` placeholders in all YAML string values.
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

async fn resolve_extensions(base: &Path, value: Value) -> Result<Value> {
    match value {
        Value::Mapping(map) => {
            let futs = map.into_iter().map(|(key, val)| async move {
                let resolved = Box::pin(resolve_extensions(base, val)).await?;
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
                .map(|val| Box::pin(resolve_extensions(base, val)));
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
                    let value = Box::pin(read_and_parse_yml(&base.join(relative_path))).await?;
                    return Ok(value);
                }
            }

            Ok(Value::Tagged(tag))
        }
        _ => Ok(value),
    }
}
