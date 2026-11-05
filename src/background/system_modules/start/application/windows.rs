use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use notify_debouncer_full::{
    DebounceEventResult, Debouncer, FileIdMap, new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode},
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use seelen_core::system_state::{StartMenuItem, StartMenuLayout, StartMenuLayoutItem};
use windows::{
    ApplicationModel::{Core::AppListEntry, Package, PackageCatalog},
    Foundation::TypedEventHandler,
    Management::Deployment::PackageManager,
    UI::StartScreen::StartScreenManager,
};

use crate::{
    error::{Result, ResultLogExt},
    platform::paths,
    system_modules::start::domain::{
        ShortcutEntry, ShortcutIdentity, StartMenuSysEvent, StartMenuSystem,
    },
    utils::{constants::SEELEN_COMMON, pwsh::PwshScript},
    windows_api::{Com, WindowsApi, string_utils::WindowsString},
};

const FILE_WATCHER_DEBOUNCE: Duration = Duration::from_millis(500);

pub struct WindowsStartMenu {
    file_watcher: Option<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>,
    package_catalog: Option<PackageCatalog>,
    installing_token: Option<i64>,
    uninstalling_token: Option<i64>,
}

// SAFETY: `PackageCatalog` is not marked agile by windows-rs, but it is only used to register and
// remove its event handlers (as the previous manager did, also with `unsafe impl Send/Sync`).
unsafe impl Send for WindowsStartMenu {}
unsafe impl Sync for WindowsStartMenu {}

impl StartMenuSystem for WindowsStartMenu {
    fn create() -> Self {
        Self {
            file_watcher: None,
            package_catalog: None,
            installing_token: None,
            uninstalling_token: None,
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(StartMenuSysEvent) + Send + Sync + 'static,
    {
        let on_event = Arc::new(on_event);

        // both are set up even if the other one fails
        let watcher = self.watch_folders(on_event.clone());
        let catalog = self.watch_packages(on_event);
        watcher?;
        catalog
    }

    fn user_shortcuts(&self) -> Vec<ShortcutEntry> {
        load_shortcuts(&user_items_path())
    }

    fn system_shortcuts(&self) -> Vec<ShortcutEntry> {
        load_shortcuts(&common_items_path())
    }

    /// Each package needs several WinRT calls, so they are resolved in parallel. Rayon balances
    /// the work between threads and the order of the packages is preserved.
    fn packaged_apps(&self) -> Result<Vec<Arc<StartMenuItem>>> {
        let pkg_manager = PackageManager::new()?;
        let start_screen = StartScreenManager::GetDefault()?;

        let packages: Vec<Package> = pkg_manager
            .FindPackagesByUserSecurityId(&"".into())?
            .into_iter()
            .collect();

        Ok(packages
            .into_par_iter()
            .flat_map_iter(|package| process_package(&start_screen, &package))
            .collect())
    }

    /// `Export-StartLayout` writes the pinned items to a json file, with the paths of the
    /// desktop apps unexpanded (`%APPDATA%\...`).
    async fn native_layout(&self) -> Result<StartMenuLayout> {
        let output_path = SEELEN_COMMON.app_cache_dir().join("start-layout.json");
        let output_path_str = output_path.to_string_lossy().to_string();

        let script = PwshScript::new(format!("Export-StartLayout -Path '{}'", output_path_str))
            .inline_command();
        script.execute().await?;

        let file = std::fs::File::open(&output_path)?;
        let mut layout: StartMenuLayout = serde_json::from_reader(file)?;

        for item in &mut layout.pinned_list {
            if let StartMenuLayoutItem::DesktopAppLink(path) = item {
                let source = WindowsString::from_str(path);
                let expanded = WindowsApi::resolve_environment_variables(&source)?;
                *item = StartMenuLayoutItem::DesktopAppLink(expanded.to_string());
            }
        }

        Ok(layout)
    }
}

impl WindowsStartMenu {
    fn watch_folders<F>(&mut self, on_event: Arc<F>) -> Result<()>
    where
        F: Fn(StartMenuSysEvent) + Send + Sync + 'static,
    {
        let mut debouncer = new_debouncer(
            FILE_WATCHER_DEBOUNCE,
            None,
            move |result: DebounceEventResult| match result {
                Ok(events) => {
                    log::debug!(
                        "Start menu file watcher detected changes: {} events",
                        events.len()
                    );
                    on_event(StartMenuSysEvent::ShortcutsChanged);
                }
                Err(errors) => {
                    log::error!("Start menu file watcher error: {errors:?}");
                }
            },
        )?;

        debouncer.watch(common_items_path(), RecursiveMode::Recursive)?;
        debouncer.watch(user_items_path(), RecursiveMode::Recursive)?;
        self.file_watcher = Some(debouncer);
        Ok(())
    }

    fn watch_packages<F>(&mut self, on_event: Arc<F>) -> Result<()>
    where
        F: Fn(StartMenuSysEvent) + Send + Sync + 'static,
    {
        let catalog = PackageCatalog::OpenForCurrentUser()?;

        let cb = on_event.clone();
        self.installing_token = Some(catalog.PackageInstalling(&TypedEventHandler::new(
            move |_catalog, _args| {
                log::debug!("Package installing event detected");
                cb(StartMenuSysEvent::PackagesChanged);
                Ok(())
            },
        ))?);

        self.uninstalling_token = Some(catalog.PackageUninstalling(&TypedEventHandler::new(
            move |_catalog, _args| {
                log::debug!("Package uninstalling event detected");
                on_event(StartMenuSysEvent::PackagesChanged);
                Ok(())
            },
        ))?);

        self.package_catalog = Some(catalog);
        Ok(())
    }
}

impl Drop for WindowsStartMenu {
    fn drop(&mut self) {
        let Some(catalog) = &self.package_catalog else {
            return;
        };
        if let Some(token) = self.installing_token.take() {
            catalog.RemovePackageInstalling(token).log_error();
        }
        if let Some(token) = self.uninstalling_token.take() {
            catalog.RemovePackageUninstalling(token).log_error();
        }
    }
}

/// programs shared by all users
fn common_items_path() -> PathBuf {
    paths::system_apps_dir().expect("Failed to get system start menu folder path")
}

/// programs specific to the current user
fn user_items_path() -> PathBuf {
    paths::user_apps_dir().expect("Failed to get user start menu folder path")
}

/// Resolving each shortcut (lnk target, umid, toast activator) hits the shell, so it's done
/// in parallel. The order of the files is preserved.
fn load_shortcuts(folder: &Path) -> Vec<ShortcutEntry> {
    crate::utils::collect_files(folder)
        .into_par_iter()
        .map(process_file)
        .collect()
}

/// Also keeps what the shortcut launches, used to drop duplicated shortcuts.
fn process_file(path: PathBuf) -> ShortcutEntry {
    // The folders also contain files like desktop.ini, .url or .html, they aren't shell
    // links so there is nothing to resolve on them.
    let is_lnk = path
        .extension()
        .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"));

    // one COM context for all the shell calls, instead of tearing it down after each one
    let (resolved, umid, toast_activator) = if is_lnk {
        Com::run_with_context(|| {
            let resolved = WindowsApi::resolve_lnk_target(&path).ok();
            let (umid, toast_activator) = WindowsApi::get_file_umid_and_toast_activator(&path);
            Ok((resolved, umid, toast_activator))
        })
        .unwrap_or_default()
    } else {
        Default::default()
    };

    let identity = resolved
        .as_ref()
        .and_then(|(target, args, working_dir)| ShortcutIdentity::new(target, args, working_dir));
    let target = resolved.map(|(target, ..)| target);

    // Get display name from filename without extension
    let display_name = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("Unknown")
        .to_string();

    let item = Arc::new(StartMenuItem {
        umid,
        toast_activator,
        path,
        target,
        display_name,
    });
    ShortcutEntry { item, identity }
}

fn process_package(
    start_screen: &StartScreenManager,
    package: &Package,
) -> Vec<Arc<StartMenuItem>> {
    let apps = match package.GetAppListEntries() {
        Ok(apps) => apps,
        Err(e) => {
            log::error!("Failed to get app list entries for a package: {e:?}");
            return Vec::new();
        }
    };

    let mut items = Vec::new();
    for app in apps {
        match process_app_list_entry(start_screen, &app) {
            Ok(Some(item)) => items.push(item),
            Ok(None) => {}
            Err(e) => log::error!("Failed to process start menu app entry: {e:?}"),
        }
    }
    items
}

/// https://learn.microsoft.com/en-us/uwp/schemas/appxpackage/uapmanifestschema/element-uap-visualelements
fn process_app_list_entry(
    start_screen: &StartScreenManager,
    app: &AppListEntry,
) -> Result<Option<Arc<StartMenuItem>>> {
    if !start_screen.SupportsAppListEntry(app)? {
        return Ok(None);
    }

    let umid = app.AppUserModelId()?.to_string_lossy();
    let display_name = app
        .DisplayInfo()?
        .DisplayName()?
        .to_string_lossy()
        .to_string();

    Ok(Some(Arc::new(StartMenuItem {
        umid: Some(umid),
        toast_activator: None,
        path: PathBuf::new(),
        target: None,
        display_name,
    })))
}
