use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock, OnceLock};
use std::time::Duration;

use notify_debouncer_full::{
    DebounceEventResult, DebouncedEvent, Debouncer, FileIdMap, new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode},
};
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use seelen_core::system_state::StartMenuItem;
use windows::Win32::UI::Shell::{FOLDERID_CommonStartMenu, FOLDERID_StartMenu};
use windows::{
    ApplicationModel::{Core::AppListEntry, Package, PackageCatalog},
    Foundation::TypedEventHandler,
    Management::Deployment::PackageManager,
    UI::StartScreen::StartScreenManager,
};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::lock_free::SyncVec,
    windows_api::{Com, WindowsApi},
};

pub struct StartMenuManager {
    /// win32 unpackaged, from the start menu folders. Only refreshed by the file watcher.
    shortcuts: SyncVec<Arc<StartMenuItem>>,
    /// win32/uwp packaged, from the package catalog. Only refreshed by the package events.
    packaged: SyncVec<Arc<StartMenuItem>>,
    _file_watcher: OnceLock<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>,
    _package_catalog: OnceLock<PackageCatalog>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum StartMenuEvent {
    ItemsRefreshed,
}

event_manager!(StartMenuManager, StartMenuEvent);

unsafe impl Send for StartMenuManager {}
unsafe impl Sync for StartMenuManager {}

impl StartMenuManager {
    /// programs shared by all users
    fn common_items_path() -> PathBuf {
        WindowsApi::known_folder(FOLDERID_CommonStartMenu)
            .expect("Failed to get FOLDERID_CommonStartMenu folder path")
    }

    /// programs specific to the current user
    fn user_items_path() -> PathBuf {
        WindowsApi::known_folder(FOLDERID_StartMenu)
            .expect("Failed to get FOLDERID_StartMenu folder path")
    }

    fn new() -> StartMenuManager {
        StartMenuManager {
            shortcuts: SyncVec::new(),
            packaged: SyncVec::new(),
            _file_watcher: OnceLock::new(),
            _package_catalog: OnceLock::new(),
        }
    }

    /// Cheap to call from anywhere and never blocks, the lists stay empty until
    /// [`Self::initialize`] finishes.
    pub fn instance() -> &'static Self {
        static START_MENU_MANAGER: LazyLock<StartMenuManager> =
            LazyLock::new(StartMenuManager::new);
        &START_MENU_MANAGER
    }

    /// Loads the start menu items and sets up
    /// the listeners that keep them updated. Should be awaited during the app startup.
    pub fn initialize() -> Result<()> {
        let manager = Self::instance();
        manager.shortcuts.replace(crate::measure!(
            "StartMenu Shortcuts",
            Self::load_shortcut_items()
        ));
        manager.packaged.replace(crate::measure!(
            "StartMenu Packaged",
            Self::load_packaged_items()?
        ));
        manager.setup_listeners();
        Ok(())
    }

    /// Shortcuts followed by the packaged apps.
    pub fn get_all(&self) -> Vec<Arc<StartMenuItem>> {
        let mut items = self.shortcuts.to_vec();
        items.extend(self.packaged.to_vec());
        items
    }

    /// Only shortcuts have a target, packaged apps don't.
    pub fn get_by_target(&self, target: &Path) -> Option<Arc<StartMenuItem>> {
        self.shortcuts
            .find_and_clone(|item| item.target.as_ref().is_some_and(|t| t == target))
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchiconresource
    pub fn get_by_file_umid(&self, umid: &str) -> Option<Arc<StartMenuItem>> {
        let matches = |item: &Arc<StartMenuItem>| {
            if let Some(item_umid) = &item.umid {
                return item_umid == umid;
            }
            if let Some(target) = &item.target {
                // some apps registered as media player as example use the process name as umid
                return target.ends_with(umid);
            }
            false
        };
        self.shortcuts
            .find_and_clone(matches)
            .or_else(|| self.packaged.find_and_clone(matches))
    }

    /// Resolving each shortcut (lnk target, umid, toast activator) hits the shell, so it's done
    /// in parallel. The order of the given paths is preserved.
    fn _process_file(path: PathBuf) -> Arc<StartMenuItem> {
        // The folders also contain files like desktop.ini, .url or .html, they aren't shell
        // links so there is nothing to resolve on them.
        let is_lnk = path
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("lnk"));

        // one COM context for all the shell calls, instead of tearing it down after each one
        let (target, umid, toast_activator) = if is_lnk {
            Com::run_with_context(|| {
                let target = WindowsApi::resolve_lnk_target(&path).ok().map(|(t, ..)| t);
                let (umid, toast_activator) = WindowsApi::get_file_umid_and_toast_activator(&path);
                Ok((target, umid, toast_activator))
            })
            .unwrap_or_default()
        } else {
            Default::default()
        };

        // Get display name from filename without extension
        let display_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        Arc::new(StartMenuItem {
            umid,
            toast_activator,
            path,
            target,
            display_name,
        })
    }

    /// win32 unpackaged
    ///
    /// The common and user folders are independent, so they are scanned at the same time and
    /// each one starts resolving its shortcuts without waiting for the other one.
    fn load_shortcut_items() -> Vec<Arc<StartMenuItem>> {
        let (a, b) = rayon::join(
            || crate::utils::collect_files(&Self::common_items_path()),
            || crate::utils::collect_files(&Self::user_items_path()),
        );

        a.into_par_iter()
            .chain(b.into_par_iter())
            .map(Self::_process_file)
            .collect::<Vec<_>>()
    }

    /// Each package needs several WinRT calls, so they are resolved in parallel. Rayon balances
    /// the work between threads and the order of the packages is preserved.
    fn load_packaged_items() -> Result<Vec<Arc<StartMenuItem>>> {
        let pkg_manager = PackageManager::new()?;
        let start_screen = StartScreenManager::GetDefault()?;

        let packages: Vec<Package> = pkg_manager
            .FindPackagesByUserSecurityId(&"".into())?
            .into_iter()
            .collect();

        Ok(packages
            .into_par_iter()
            .flat_map_iter(|package| Self::process_package(&start_screen, &package))
            .collect())
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
            match Self::_process_app_list_entry(start_screen, &app) {
                Ok(Some(item)) => items.push(item),
                Ok(None) => {}
                Err(e) => log::error!("Failed to process start menu app entry: {e:?}"),
            }
        }
        items
    }

    /// https://learn.microsoft.com/en-us/uwp/schemas/appxpackage/uapmanifestschema/element-uap-visualelements
    fn _process_app_list_entry(
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

    fn setup_file_watcher(&self) -> Result<()> {
        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            |result: DebounceEventResult| match result {
                Ok(events) => {
                    log::debug!(
                        "Start menu file watcher detected changes: {} events",
                        events.len()
                    );
                    Self::on_files_changed(events);
                }
                Err(errors) => {
                    log::error!("Start menu file watcher error: {errors:?}");
                }
            },
        )?;

        debouncer.watch(Self::common_items_path(), RecursiveMode::Recursive)?;
        debouncer.watch(Self::user_items_path(), RecursiveMode::Recursive)?;

        let _ = self._file_watcher.set(debouncer);
        Ok(())
    }

    fn reload_shortcuts() {
        Self::instance()
            .shortcuts
            .replace(Self::load_shortcut_items());
        Self::send(StartMenuEvent::ItemsRefreshed);
    }

    fn reload_packaged() -> Result<()> {
        Self::instance()
            .packaged
            .replace(Self::load_packaged_items()?);
        Self::send(StartMenuEvent::ItemsRefreshed);
        Ok(())
    }

    fn on_files_changed(_events: Vec<DebouncedEvent>) {
        Self::reload_shortcuts();
    }

    fn setup_package_catalog_listener(&self) -> Result<()> {
        let catalog = PackageCatalog::OpenForCurrentUser()?;

        catalog.PackageInstalling(&TypedEventHandler::new(|_catalog, _args| {
            log::debug!("Package installing event detected");
            std::thread::spawn(|| {
                std::thread::sleep(Duration::from_millis(1000));
                Self::reload_packaged().log_error();
            });
            Ok(())
        }))?;

        catalog.PackageUninstalling(&TypedEventHandler::new(|_catalog, _args| {
            log::debug!("Package uninstalling event detected");
            std::thread::spawn(|| {
                std::thread::sleep(Duration::from_millis(1000));
                Self::reload_packaged().log_error();
            });
            Ok(())
        }))?;

        let _ = self._package_catalog.set(catalog);
        Ok(())
    }

    pub fn setup_listeners(&self) {
        // Setup file system watcher
        self.setup_file_watcher().log_error();
        // Setup package catalog listener
        self.setup_package_catalog_listener().log_error();
    }
}
