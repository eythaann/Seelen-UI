use lazy_static::lazy_static;
use parking_lot::Mutex;
use serde::Deserialize;
use std::{
    path::{Path, PathBuf},
    sync::Arc,
};
use tauri::Manager;

use crate::{
    error_handler::Result,
    seelen::get_app_handle,
    trace_lock,
    utils::{pwsh::PwshScript, PERFORMANCE_HELPER},
};

pub static UWP_LIGHTUNPLATED_POSTFIX: &str = "_altform-lightunplated";
#[allow(dead_code)]
pub static UWP_UNPLATED_POSTFIX: &str = "_altform-unplated";

lazy_static! {
    pub static ref UWP_MANAGER: Arc<Mutex<WindowsAppsManager>> = Arc::new(Mutex::new({
        let mut manager = WindowsAppsManager::default();
        manager.refresh().expect("Failed to refresh UWP manager");
        manager
    }));
    pub static ref UWP_TARGET_SIZE_POSTFIXES: Vec<&'static str> = vec![
        ".targetsize-256",
        ".targetsize-96",
        ".targetsize-64",
        ".targetsize-48",
        ".targetsize-32",
    ];
    pub static ref UWP_SCALE_POSTFIXES: Vec<&'static str> = vec![
        ".scale-400",
        ".scale-200",
        ".scale-150",
        ".scale-125",
        ".scale-100",
    ];
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
#[allow(dead_code)]
pub struct UWPPackage {
    name: String,
    version: String,
    publisher_id: String,
    package_full_name: String,
    install_location: PathBuf,
    store_logo: Option<String>,
    applications: Vec<UWPApplication>,
}

#[derive(Deserialize, Debug, Default)]
#[serde(rename_all = "PascalCase")]
pub struct UWPApplication {
    app_id: String,
    /// subpath from UWPPackage.install_location
    executable: String,
    alias: Option<String>,
    /// subpath from UWPPackage.install_location
    square150x150_logo: Option<String>,
    // subpath from UWPPackage.install_location
    square44x44_logo: Option<String>,
}

impl UWPApplication {
    pub fn get_44_icon(&self) -> Option<&String> {
        self.square44x44_logo.as_ref()
    }

    pub fn get_150_icon(&self) -> Option<&String> {
        self.square150x150_logo.as_ref()
    }
}

impl UWPPackage {
    pub fn get_store_logo(&self) -> Option<&String> {
        self.store_logo.as_ref()
    }

    pub fn get_app(&self, exe: &str) -> Option<&UWPApplication> {
        self.applications
            .iter()
            .find(|app| app.executable.ends_with(exe) || app.alias.as_deref() == Some(exe))
    }

    pub fn get_light_icon_path(icon_path: &Path) -> Option<PathBuf> {
        let filename = icon_path.file_stem()?.to_str()?;
        let extension = icon_path.extension()?.to_str()?;

        let postfixes = (*UWP_TARGET_SIZE_POSTFIXES)
            .iter()
            .chain((*UWP_SCALE_POSTFIXES).iter());

        for postfix in postfixes {
            let maybe_icon_path = icon_path.with_file_name(format!(
                "{}{}{}.{}",
                filename, postfix, UWP_LIGHTUNPLATED_POSTFIX, extension
            ));
            if maybe_icon_path.exists() {
                return Some(maybe_icon_path);
            }

            let maybe_icon_path =
                icon_path.with_file_name(format!("{}{}.{}", filename, postfix, extension));
            if maybe_icon_path.exists() {
                return Some(maybe_icon_path);
            }
        }

        // Some apps only adds one icon and without any postfix
        // but we prefer the light/dark specific icon
        if icon_path.exists() {
            return Some(icon_path.to_path_buf());
        }

        None
    }

    pub fn get_light_icon(&self, exe: &str) -> Option<PathBuf> {
        let app = self.get_app(exe)?;

        app.get_44_icon()
            .and_then(|sub_path| Self::get_light_icon_path(&self.install_location.join(sub_path)))
            .or_else(|| {
                app.get_150_icon().and_then(|sub_path| {
                    Self::get_light_icon_path(&self.install_location.join(sub_path))
                })
            })
            .or_else(|| {
                self.get_store_logo().and_then(|sub_path| {
                    Self::get_light_icon_path(&self.install_location.join(sub_path))
                })
            })
    }

    pub fn get_shell_path(&self, exe: &str) -> Option<String> {
        let app = self.get_app(exe)?;
        Some(format!(
            "shell:AppsFolder\\{}_{}!{}",
            self.name, self.publisher_id, app.app_id
        ))
    }
}

#[derive(Debug, Default)]
pub struct WindowsAppsManager {
    packages: Vec<UWPPackage>,
}

impl WindowsAppsManager {
    fn get_save_path() -> Result<std::path::PathBuf> {
        Ok(get_app_handle()
            .path()
            .app_data_dir()?
            .join("uwp_manifests.json"))
    }

    pub fn refresh(&mut self) -> Result<()> {
        log::trace!("Loading UWP packages");
        trace_lock!(PERFORMANCE_HELPER).start("uwp");
        let script = PwshScript::new(include_str!("load_uwp_apps.ps1"));
        let contents = tauri::async_runtime::block_on(script.execute())?;
        self.packages = serde_json::from_str(&contents)?;
        std::fs::write(Self::get_save_path()?, &contents)?;
        log::trace!(
            "UWP packages loaded in: {:.2}s",
            trace_lock!(PERFORMANCE_HELPER).elapsed("uwp").as_secs_f64()
        );
        Ok(())
    }

    pub fn get_from_path(&self, exe_path: &Path) -> Option<&UWPPackage> {
        let exe = exe_path.file_name()?.to_string_lossy().to_string();
        self.packages.iter().find(|p| {
            exe_path.starts_with(&p.install_location)
                && p.applications
                    .iter()
                    .any(|app| app.executable.ends_with(&exe) || app.alias.as_deref() == Some(&exe))
        })
    }
}
