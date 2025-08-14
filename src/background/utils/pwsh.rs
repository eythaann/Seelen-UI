use std::env::temp_dir;

use itertools::Itertools;
use tauri_plugin_shell::ShellExt;

use crate::{app::get_app_handle, error::Result, windows_api::WindowsApi};

const PWSH_COMMON_ARGS: [&str; 7] = [
    "-NoLogo",
    "-NoProfile",
    "-NonInteractive",
    "-ExecutionPolicy",
    "Bypass",
    "-WindowStyle",
    "Hidden",
];

pub struct PwshScript {
    mode: PwshExecutionMode,
    inner: String,
    elevated: bool,
}

pub enum PwshExecutionMode {
    ScriptFile(Vec<String>),
    Command,
}

static POWERSHELL_PATH: &str = "C:\\Windows\\System32\\WindowsPowerShell\\v1.0\\powershell.exe";

impl PwshScript {
    pub fn new<S: Into<String>>(contents: S) -> Self {
        Self {
            inner: contents.into(),
            mode: PwshExecutionMode::ScriptFile(Vec::new()),
            elevated: false,
        }
    }

    pub fn inline_command(mut self) -> Self {
        self.mode = PwshExecutionMode::Command;
        self
    }

    /// ignored if `mode` is other than `PwshExecutionMode::ScriptFile`
    pub fn elevated(mut self) -> Self {
        self.elevated = true;
        self
    }

    fn build_args(&self, script_path_str: &str) -> Vec<String> {
        match &self.mode {
            PwshExecutionMode::ScriptFile(args) => {
                let mut args = PWSH_COMMON_ARGS
                    .iter()
                    .map(|s| s.to_string())
                    .chain(["-File".to_string(), script_path_str.to_string()])
                    .chain(args.clone())
                    .collect_vec();
                if self.elevated && !WindowsApi::is_elevated().unwrap_or(false) {
                    args = PWSH_COMMON_ARGS
                    .iter()
                    .map(|s| s.to_string())
                    .chain([
                        "-Command".to_string(),
                        format!("Start-Process '{}' -Verb runAs -WindowStyle Hidden -Wait -ArgumentList '{}'", POWERSHELL_PATH, args.join(" "))
                    ])
                    .collect_vec();
                }
                args
            }
            PwshExecutionMode::Command => PWSH_COMMON_ARGS
                .iter()
                .map(|s| s.to_string())
                .chain(["-Command".to_string(), self.inner.to_string()])
                .collect_vec(),
        }
    }

    /// returns `Ok(stdout)` or `Err(stderr)`
    ///
    /// if elevated, will run as admin and always will return `Ok("")`
    pub async fn execute(&self) -> Result<String> {
        let script_path = temp_dir().join(format!("slu-{}.ps1", uuid::Uuid::new_v4()));
        std::fs::write(&script_path, &self.inner)?;

        let args = self.build_args(&script_path.to_string_lossy());
        let shell = get_app_handle().shell();
        let result = shell.command(POWERSHELL_PATH).args(args).output().await;
        // delete script before check output
        std::fs::remove_file(&script_path)?;
        let output = result?;

        if output.status.success() {
            let (cow, _used, _has_errors) = encoding_rs::GBK.decode(&output.stdout);
            Ok(cow.trim().to_string())
        } else {
            Err(output.into())
        }
    }
}
