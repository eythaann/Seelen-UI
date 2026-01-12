// the idea with this module is improve YAML with extensibility, via custom keywords

use std::{fs::File, path::Path};

use serde_yaml::{Mapping, Value};

use crate::error::Result;

/// Will deserialize a YAML file and parse the custom extended syntax
pub fn deserialize_extended_yaml<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let value = read_and_parse_yml(path)?;
    Ok(serde_yaml::from_value(value)?)
}

fn read_and_parse_yml(path: &Path) -> Result<Value> {
    let file = File::open(path)?;
    file.lock_shared()?;
    let base = path.parent().ok_or("No parent directory")?;
    let value: Value = serde_yaml::from_reader(file)?;
    parse_yaml(base, value)
}

fn parse_yaml(base: &Path, value: Value) -> Result<Value> {
    match value {
        Value::Mapping(map) => {
            let mut new_map = Mapping::new();
            for (key, value) in map {
                let value = parse_yaml(base, value)?;
                new_map.insert(key, value);
            }
            Ok(Value::Mapping(new_map))
        }
        Value::Sequence(seq) => {
            let mut new_seq = Vec::new();
            for value in seq {
                let value = parse_yaml(base, value)?;
                new_seq.push(value);
            }
            Ok(Value::Sequence(new_seq))
        }
        Value::Tagged(tag) => {
            if tag.tag == "!include" {
                if let Value::String(relative_path) = tag.value {
                    let to_include = base.join(relative_path);
                    let text = if to_include
                        .extension()
                        .is_some_and(|ext| ext == "scss" || ext == "sass")
                    {
                        grass::from_path(&to_include, &grass::Options::default())?
                    } else {
                        std::fs::read_to_string(&to_include)?
                    };
                    return Ok(Value::String(text));
                }
            }

            if tag.tag == "!extend" {
                if let Value::String(relative_path) = tag.value {
                    let value = read_and_parse_yml(&base.join(relative_path))?;
                    return Ok(value);
                }
            }

            Ok(Value::Tagged(tag))
        }
        _ => Ok(value),
    }
}
