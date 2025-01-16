use std::env::temp_dir;

use itertools::Itertools;
use tauri_plugin_shell::ShellExt;

use crate::{error_handler::Result, seelen::get_app_handle, windows_api::WindowsApi};

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
    inner: String,
    args: Vec<String>,
    elevated: bool,
}

impl PwshScript {
    pub fn new<S: Into<String>>(contents: S) -> Self {
        Self {
            inner: contents.into(),
            args: Vec::new(),
            elevated: false,
        }
    }

    pub fn with_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args = args.into_iter().map(|s| s.into()).collect_vec();
        self
    }

    pub fn elevated(mut self) -> Self {
        self.elevated = true;
        self
    }

    fn build_args(&self, script_path_str: &str) -> Vec<String> {
        let mut args = PWSH_COMMON_ARGS
            .iter()
            .map(|s| s.to_string())
            .chain(["-File".to_string(), script_path_str.to_string()])
            .chain(self.args.clone())
            .collect_vec();
        if self.elevated && !WindowsApi::is_elevated().unwrap_or(false) {
            args = PWSH_COMMON_ARGS
            .iter()
            .map(|s| s.to_string())
            .chain([
                "-Command".to_string(),
                format!("Start-Process 'powershell' -Verb runAs -WindowStyle Hidden -Wait -ArgumentList '{}'", args.join(" "))
            ])
            .collect_vec();
        }
        args
    }

    /// returns `Ok(stdout)` or `Err(stderr)`
    ///
    /// if elevated, will run as admin and always will return `Ok("")`
    pub async fn execute(&self) -> Result<String> {
        let script_path = temp_dir().join(format!("slu-{}.ps1", uuid::Uuid::new_v4()));
        std::fs::write(&script_path, &self.inner)?;

        let args = self.build_args(&script_path.to_string_lossy());
        let shell = get_app_handle().shell();
        let result = shell.command("powershell").args(args).output().await;
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
