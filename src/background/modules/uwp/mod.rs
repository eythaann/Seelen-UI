// unused/deprecated code but could be useful for understanding how uwp packing works
pub mod domain;

use domain::{ManifestApplication, PackageManifest};
use lazy_static::lazy_static;
use std::path::{Path, PathBuf};
use windows::ApplicationModel::{AppInfo, Package};

use crate::error_handler::Result;

pub static UWP_LIGHTUNPLATED_POSTFIX: &str = "_altform-lightunplated";
#[allow(dead_code)]
pub static UWP_UNPLATED_POSTFIX: &str = "_altform-unplated";

lazy_static! {
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

pub fn get_hightest_quality_posible(icon_path: &Path) -> Option<PathBuf> {
    let filename = icon_path.file_stem()?.to_str()?;
    let extension = icon_path.extension()?.to_str()?;

    let size_postfixes = (*UWP_TARGET_SIZE_POSTFIXES)
        .iter()
        .chain((*UWP_SCALE_POSTFIXES).iter());

    for size_postfix in size_postfixes {
        let maybe_icon_path = icon_path.with_file_name(format!(
            "{}{}{}.{}",
            filename, size_postfix, UWP_LIGHTUNPLATED_POSTFIX, extension
        ));
        if maybe_icon_path.exists() {
            return Some(maybe_icon_path);
        }

        let maybe_icon_path =
            icon_path.with_file_name(format!("{}{}.{}", filename, size_postfix, extension));
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

impl PackageManifest {
    pub fn get_app(&self, id: &str) -> Option<&ManifestApplication> {
        let apps = self.applications.as_ref()?;
        apps.application.iter().find(|app| app.id == id)
    }
}

pub struct UwpManager;

impl UwpManager {
    pub fn manifest_from_package(package: &Package) -> Result<PackageManifest> {
        let package_path = PathBuf::from(package.InstalledPath()?.to_os_string());
        let manifest_path = package_path.join("AppxManifest.xml");

        let file = std::fs::File::open(&manifest_path)?;
        let mut reader = std::io::BufReader::new(file);

        Ok(quick_xml::de::from_reader(&mut reader)?)
    }

    pub fn get_app_path(app_umid: &str) -> Result<PathBuf> {
        let app_info = AppInfo::GetFromAppUserModelId(&app_umid.into())?;
        let package = app_info.Package()?;
        let manifest = Self::manifest_from_package(&package)?;
        let apps = manifest
            .applications
            .map(|apps| apps.application)
            .unwrap_or_default();
        for app in apps {
            if app_umid != app.id {
                continue;
            }
            if let Some(executable) = app.executable {
                let package_path = PathBuf::from(package.InstalledPath()?.to_os_string());
                return Ok(package_path.join(executable));
            }
        }
        Err(format!("App path not found for {app_umid}").into())
    }

    pub fn get_high_quality_icon_path(app_umid: &str) -> Result<PathBuf> {
        let app_info = AppInfo::GetFromAppUserModelId(&app_umid.into())?;
        let package = app_info.Package()?;
        let manifest = Self::manifest_from_package(&package)?;

        let package_path = PathBuf::from(package.InstalledPath()?.to_os_string());
        let store_logo = package_path.join(&manifest.properties.logo);

        // if package does't have the app but it is still part of the package then use the package logo
        let app_manifest = match manifest.get_app(&app_info.Id()?.to_string_lossy()) {
            Some(app) => app,
            None => {
                return get_hightest_quality_posible(&store_logo)
                    .ok_or("Could not find package logo path".into())
            }
        };

        let app_logo_44 = package_path.join(&app_manifest.visual_elements.logo_44);
        let app_logo_150 = package_path.join(&app_manifest.visual_elements.logo_150);

        get_hightest_quality_posible(&app_logo_44)
            .or_else(|| get_hightest_quality_posible(&app_logo_150))
            .or_else(|| get_hightest_quality_posible(&store_logo))
            .ok_or_else(|| format!("App icon not found for {app_umid}").into())
    }
}
