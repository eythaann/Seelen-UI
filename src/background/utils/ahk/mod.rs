use std::{collections::HashMap, env::temp_dir, path::PathBuf};

use regex::Regex;
use tauri::{path::BaseDirectory, Manager};
use tauri_plugin_shell::ShellExt;
use windows::Win32::UI::Shell::FOLDERID_LocalAppData;

use crate::{
    error_handler::Result, seelen::get_app_handle, state::domain::AhkVar, windows_api::WindowsApi,
};

use super::was_installed_using_msix;

pub struct AutoHotKey {
    inner: String,
    name: Option<String>,
}

impl AutoHotKey {
    pub fn new(contents: &str) -> Self {
        let app_path = if was_installed_using_msix() {
            WindowsApi::known_folder(FOLDERID_LocalAppData)
                .expect("Failed to get known folder")
                .join("Microsoft\\WindowsApps\\seelen-ui.exe")
        } else {
            std::env::current_exe().expect("Failed to get current exe path")
        };
        Self {
            name: None,
            inner: contents.replace("SEELEN_UI_EXE_PATH", app_path.to_string_lossy().as_ref()),
        }
    }

    pub fn from_template(template: &str, vars: &HashMap<String, AhkVar>) -> Self {
        Self {
            name: None,
            inner: Self::replace_variables(template.to_string(), vars),
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn save(&self) -> Result<PathBuf> {
        let script_path = if let Some(name) = &self.name {
            let handle = get_app_handle();
            handle.path().app_local_data_dir()?.join(name)
        } else {
            temp_dir().join(format!("slu-{}.ahk", uuid::Uuid::new_v4()))
        };
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

    fn replace_variables(template: String, vars: &HashMap<String, AhkVar>) -> String {
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
            AhkVar {
                fancy: String::new(),
                ahk: "!b".to_string(),
                readonly: false,
                devonly: false,
            },
        );
        vars.insert(
            "test2".to_string(),
            AhkVar {
                fancy: String::new(),
                ahk: "!c".to_string(),
                readonly: false,
                devonly: false,
            },
        );

        let template = r#"
        ; other comment
        ;test
        x:: anything()
        ;test2
        x:: {
          anything2()
        }
        "#
        .to_owned();

        let expected = r#"
        ; other comment
        !b:: anything()
        !c:: {
          anything2()
        }
        "#;

        assert_eq!(AutoHotKey::replace_variables(template, &vars), expected);
    }

    #[test]
    fn comment_missing_shortcuts() {
        let mut vars = HashMap::new();
        vars.insert(
            "test".to_string(),
            AhkVar {
                fancy: String::new(),
                ahk: "!b".to_string(),
                readonly: false,
                devonly: false,
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

        assert_eq!(AutoHotKey::replace_variables(template, &vars), expected);
    }
}
