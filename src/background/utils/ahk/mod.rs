use std::{collections::HashMap, env::temp_dir, path::PathBuf};

use lazy_static::lazy_static;
use regex::Regex;
use tauri::{path::BaseDirectory, Manager};
use tauri_plugin_shell::ShellExt;

use crate::{error_handler::Result, seelen::get_app_handle, state::AhkShortcutConfig};

lazy_static! {
    pub static ref LIB_AHK_PATH: PathBuf = {
        let mut lib_ahk = AutoHotKey::new(include_str!("mocks/seelen.lib.ahk"));

        lib_ahk.inner = lib_ahk.inner.replace(
            "SEELEN_UI_EXE_PATH",
            &std::env::current_exe()
                .expect("Failed to get current exe path")
                .to_string_lossy(),
        );

        lib_ahk.save().expect("Failed to load lib.ahk")
    };
}

pub struct AutoHotKey {
    inner: String,
}

impl AutoHotKey {
    pub fn new(contents: &str) -> Self {
        Self {
            inner: contents.to_string(),
        }
    }

    pub fn from_template(template: &str, vars: HashMap<String, AhkShortcutConfig>) -> Self {
        Self {
            inner: Self::replace_variables(template.to_string(), vars),
        }
    }

    pub fn with_lib(mut self) -> Self {
        self.inner = self
            .inner
            .replace("seelen.lib.ahk", &LIB_AHK_PATH.to_string_lossy());
        self
    }

    pub fn save(&self) -> Result<PathBuf> {
        let script_path = temp_dir().join(format!("slu-{}.ahk", uuid::Uuid::new_v4()));
        std::fs::write(&script_path, &self.inner)?;
        Ok(script_path)
    }

    pub fn execute(&self) -> Result<()> {
        let script_path = self.save()?;

        let handle = get_app_handle();
        let ahk_executable_path = handle
            .path()
            .resolve("static/redis/AutoHotkey.exe", BaseDirectory::Resource)?
            .to_string_lossy()
            .trim_start_matches(r"\\?\")
            .to_owned();

        handle
            .shell()
            .command(ahk_executable_path)
            .arg(script_path.to_string_lossy().to_string())
            .spawn()?;

        Ok(())
    }

    fn replace_variables(template: String, vars: HashMap<String, AhkShortcutConfig>) -> String {
        let mut replaced = template.clone();

        for (key, value) in vars.iter() {
            let pattern = Regex::new(&format!(";{}\\s*x::", key)).unwrap();
            replaced = pattern
                .replace_all(&replaced, format!("{}::", value.ahk))
                .to_string();
        }

        replaced.replace("x::", ";missing_shortcut::")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn replace_variables() {
        let mut vars = HashMap::new();
        vars.insert(
            "test".to_string(),
            AhkShortcutConfig {
                fancy: String::new(),
                ahk: "!b".to_string(),
            },
        );
        vars.insert(
            "test2".to_string(),
            AhkShortcutConfig {
                fancy: String::new(),
                ahk: "!c".to_string(),
            },
        );

        let template = r#"
        ; other comment
        ;test
        x:: anything()
        ;test2
        x:: anything2()
        "#
        .to_owned();

        let expected = r#"
        ; other comment
        !b:: anything()
        !c:: anything2()
        "#;

        assert_eq!(AutoHotKey::replace_variables(template, vars), expected);
    }

    #[test]
    fn comment_missing_shortcuts() {
        let mut vars = HashMap::new();
        vars.insert(
            "test".to_string(),
            AhkShortcutConfig {
                fancy: String::new(),
                ahk: "!b".to_string(),
            },
        );

        let template = r#"
        ; other comment
        ;test
        x:: anything()
        ;test2
        x:: anything2()
        "#
        .to_owned();

        let expected = r#"
        ; other comment
        !b:: anything()
        ;test2
        ;missing_shortcut:: anything2()
        "#;

        assert_eq!(AutoHotKey::replace_variables(template, vars), expected);
    }
}
