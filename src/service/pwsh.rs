use std::{env::temp_dir, process::Command};

use itertools::Itertools;

use crate::error::Result;

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
}

impl PwshScript {
    pub fn new<S: Into<String>>(contents: S) -> Self {
        Self {
            inner: contents.into(),
            args: Vec::new(),
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

    fn build_args(&self, script_path_str: &str) -> Vec<String> {
        let args = PWSH_COMMON_ARGS
            .iter()
            .map(|s| s.to_string())
            .chain(["-File".to_string(), script_path_str.to_string()])
            .chain(self.args.clone())
            .collect_vec();
        args
    }

    pub fn spawn(&self) -> Result<std::process::Child> {
        let script_path = temp_dir().join(format!("slu-{}.ps1", uuid::Uuid::new_v4()));
        std::fs::write(&script_path, &self.inner)?;
        let args = self.build_args(&script_path.to_string_lossy());
        let result = Command::new("powershell").args(args).spawn();
        // std::fs::remove_file(&script_path)?;
        Ok(result?)
    }

    /// returns `Ok(stdout)` or `Err(stderr || stdout)`
    pub fn output(&self) -> Result<String> {
        let script_path = temp_dir().join(format!("slu-{}.ps1", uuid::Uuid::new_v4()));
        std::fs::write(&script_path, &self.inner)?;

        let args = self.build_args(&script_path.to_string_lossy());
        let result = Command::new("powershell").args(args).output();

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
