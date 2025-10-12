use std::{
    path::{Path, PathBuf},
    sync::LazyLock,
};

use windows::ApplicationModel::{AppInfo, Package};

use crate::{error::Result, log_error, modules::apps::application::msix_manifest::PackageManifest};

static UWP_LIGHTUNPLATED_POSTFIX: &str = "_altform-lightunplated";
static UWP_UNPLATED_POSTFIX: &str = "_altform-unplated";
static UWP_TARGET_SIZE_POSTFIXES: &[&str] = &[
    ".targetsize-256",
    ".targetsize-96",
    ".targetsize-64",
    ".targetsize-48",
    ".targetsize-32",
];
static UWP_SCALE_POSTFIXES: &[&str] = &[
    ".scale-400",
    ".scale-200",
    ".scale-150",
    ".scale-125",
    ".scale-100",
];

pub struct MsixAppsManager {
    // key: package family name, value: app manifest
    // packages: scc::HashMap<String, PackageManifest>, // too slow
}

impl MsixAppsManager {
    fn new() -> Self {
        Self {
            // packages: scc::HashMap::new(),
        }
    }

    pub fn instance() -> &'static Self {
        static MSIX_APPS_MANAGER: LazyLock<MsixAppsManager> = LazyLock::new(|| {
            let m = MsixAppsManager::new();
            log_error!(m.enumerate_all_apps());
            m
        });
        &MSIX_APPS_MANAGER
    }

    fn enumerate_all_apps(&self) -> Result<()> {
        /* let m = PackageManager::new()?;
        let packages = m.FindPackagesByUserSecurityId(&"".into())?; // error access denied

        for pack in packages {
            if let Ok(manifest) = PackageManifest::try_read_for(&pack) {
                self.packages
                    .upsert(pack.Id()?.FamilyName()?.to_string_lossy(), manifest);
            }
        } */

        // debug save on json on debug mode
        /* #[cfg(debug_assertions)]
        {
            let mut data = vec![];
            self.packages.scan(|_key, value| data.push(value.clone()));
            let json = serde_json::to_vec_pretty(&data)?;
            std::fs::write("./msix_apps.json", json)?;
        } */

        Ok(())
    }

    /// Some apps like PWA on edge can be stored as UWP apps and don't have an executable path,
    /// so in that cases the function will return None.
    ///
    /// This function will fail if the umid provided is not of type msix/appx.
    pub fn get_app_path(&self, app_umid: &str) -> Result<Option<PathBuf>> {
        let app_info = AppInfo::GetFromAppUserModelId(&app_umid.into())?;
        let package = app_info.Package()?;
        let package_family_name = package.Id()?.FamilyName()?.to_string_lossy();

        let manifest = PackageManifest::try_read_for(&package)?;

        let apps = &manifest.applications.application;
        for app in apps {
            if format!("{package_family_name}!{}", app.id) != app_umid {
                continue;
            }

            if let Some(executable) = &app.executable {
                let package_path = PathBuf::from(package.InstalledPath()?.to_os_string());
                return Ok(Some(package_path.join(executable)));
            }
        }

        // println!("Manifest {manifest:#?}, package_family_name {package_family_name}");
        Ok(None)
    }

    /// ### Returns:
    /// light and dark icons
    pub fn get_app_icon_path(&self, app_umid: &str) -> Result<(PathBuf, PathBuf)> {
        let app_info = AppInfo::GetFromAppUserModelId(&app_umid.into())?;
        let package = app_info.Package()?;

        let manifest = PackageManifest::try_read_for(&package)?;

        let package_path = PathBuf::from(package.InstalledPath()?.to_os_string());
        let store_logo = package_path.join(&manifest.properties.logo);

        // if package does't have the app but it is still part of the package then use the package logo
        let app_manifest = match manifest.get_app(&app_info.Id()?.to_string_lossy()) {
            Some(app) => app,
            None => {
                return get_hightest_quality_posible_for_uwp_image(&store_logo)
                    .ok_or("Could not find package logo path".into())
            }
        };

        let app_logo_44 = package_path.join(&app_manifest.visual_elements.logo_44);
        let app_logo_150 = package_path.join(&app_manifest.visual_elements.logo_150);

        get_hightest_quality_posible_for_uwp_image(&app_logo_44)
            .or_else(|| get_hightest_quality_posible_for_uwp_image(&app_logo_150))
            .or_else(|| get_hightest_quality_posible_for_uwp_image(&store_logo))
            .ok_or_else(|| format!("App icon not found for {app_umid}").into())
    }
}

impl PackageManifest {
    fn try_read_for(package: &Package) -> Result<Self> {
        let package_path = PathBuf::from(package.InstalledPath()?.to_os_string());
        let manifest_path = package_path.join("AppxManifest.xml");

        let file = std::fs::File::open(&manifest_path)?;
        let mut reader = std::io::BufReader::new(file);

        Ok(quick_xml::de::from_reader(&mut reader)?)
    }
}

// returns light and dark icons
pub fn get_hightest_quality_posible_for_uwp_image(icon_path: &Path) -> Option<(PathBuf, PathBuf)> {
    let filename = icon_path.file_stem()?.to_str()?;
    let extension = icon_path.extension()?.to_str()?;

    let size_postfixes = (*UWP_TARGET_SIZE_POSTFIXES)
        .iter()
        .chain((*UWP_SCALE_POSTFIXES).iter());

    for size_postfix in size_postfixes {
        let light_icon = icon_path.with_file_name(format!(
            "{filename}{size_postfix}{UWP_LIGHTUNPLATED_POSTFIX}.{extension}"
        ));

        let dark_icon = icon_path.with_file_name(format!(
            "{filename}{size_postfix}{UWP_UNPLATED_POSTFIX}.{extension}"
        ));

        let unthemed_icon =
            icon_path.with_file_name(format!("{filename}{size_postfix}.{extension}"));

        match (
            light_icon.exists(),
            dark_icon.exists(),
            unthemed_icon.exists(),
        ) {
            (true, true, _) => return Some((light_icon, dark_icon)),
            (true, false, _) => return Some((light_icon.clone(), light_icon)),
            (false, true, true) => return Some((unthemed_icon, dark_icon)),
            (false, false, true) => return Some((unthemed_icon.clone(), unthemed_icon)),
            _ => {}
        }
    }

    // Some apps only adds one icon and without any postfix
    // but we prefer the light/dark specific icon
    if icon_path.exists() {
        return Some((icon_path.to_path_buf(), icon_path.to_path_buf()));
    }

    None
}
