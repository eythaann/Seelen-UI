use std::env::temp_dir;

use itertools::Itertools;
use tauri_plugin_shell::ShellExt;

use crate::{error_handler::Result, seelen::get_app_handle};

pub struct PwshScript {
    inner: String,
    args: Vec<String>,
}

impl PwshScript {
    pub fn new<S: Into<String>>(contents: S) -> Self {
        Self {
            inner: contents.into(),
            args: Vec::new(),
        }
    }

    pub fn with_args<I, S>(&mut self, args: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.args = args.into_iter().map(|s| s.into()).collect_vec();
    }

    /// returns `Ok(stdout)` or `Err(stderr)`
    pub async fn execute(self) -> Result<String> {
        let script_path = temp_dir().join(format!("slu-{}.ps1", uuid::Uuid::new_v4()));
        let script_path_str = script_path.to_string_lossy();

        let args = [
            "-ExecutionPolicy",
            "Bypass",
            "-NoProfile",
            "-File",
            &script_path_str,
        ]
        .iter()
        .map(|s| s.to_string())
        .chain(self.args.clone())
        .collect_vec();

        std::fs::write(&script_path, &self.inner)?;

        let handle = get_app_handle();
        let output = handle
            .shell()
            .command("powershell")
            .args(args)
            .output()
            .await?;

        std::fs::remove_file(&script_path)?;

        if output.status.success() {
            let (cow, _used, _has_errors) = encoding_rs::GBK.decode(&output.stdout);
            Ok(cow.trim().to_string())
        } else {
            Err(output.into())
        }
    }
}
