use std::collections::HashSet;
use std::ffi::OsString;
use std::path::{MAIN_SEPARATOR, Path, PathBuf};
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
    /// win32 unpackaged, from the current user start menu folder. Only refreshed by the file watcher.
    user_menu: SyncVec<ShortcutEntry>,
    /// win32 unpackaged, from the start menu folder shared by all users. Only refreshed by the file watcher.
    system_menu: SyncVec<ShortcutEntry>,
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
            user_menu: SyncVec::new(),
            system_menu: SyncVec::new(),
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
        crate::measure!("StartMenu Shortcuts", manager.load_shortcuts());
        manager.packaged.replace(crate::measure!(
            "StartMenu Packaged",
            Self::load_packaged_items()?
        ));
        manager.setup_listeners();
        Ok(())
    }

    /// User shortcuts, then the system ones, then the packaged apps, without duplicates as the
    /// native start menu shows them. The same app can be in both folders, so an item is dropped
    /// if an earlier one has the same umid or is a shortcut launching the same
    /// target/arguments/working dir. This way user items win over the system ones.
    pub fn get_all(&self) -> Vec<Arc<StartMenuItem>> {
        let mut seen_umids = HashSet::new();
        let mut seen_identities = HashSet::new();

        let mut is_new = |item: &StartMenuItem, identity: Option<&ShortcutIdentity>| {
            // both are always registered, so later duplicates of any of them are also dropped
            let new_umid = item
                .umid
                .as_ref()
                .is_none_or(|umid| seen_umids.insert(umid.clone()));
            let new_identity = identity.is_none_or(|id| seen_identities.insert(id.clone()));
            new_umid && new_identity
        };

        let mut items = Vec::new();
        let shortcuts = self
            .user_menu
            .to_vec()
            .into_iter()
            .chain(self.system_menu.to_vec());
        for entry in shortcuts {
            if is_new(&entry.item, entry.identity.as_ref()) {
                items.push(entry.item);
            }
        }
        for item in self.packaged.to_vec() {
            if is_new(&item, None) {
                items.push(item);
            }
        }
        items
    }

    /// User shortcuts are checked first, as they win over the system ones.
    fn find_shortcut(&self, f: impl Fn(&StartMenuItem) -> bool) -> Option<Arc<StartMenuItem>> {
        self.user_menu
            .find_and_clone(|entry| f(&entry.item))
            .or_else(|| self.system_menu.find_and_clone(|entry| f(&entry.item)))
            .map(|entry| entry.item)
    }

    /// Only shortcuts have a target, packaged apps don't.
    pub fn get_by_target(&self, target: &Path) -> Option<Arc<StartMenuItem>> {
        self.find_shortcut(|item| item.target.as_ref().is_some_and(|t| t == target))
    }

    pub fn has_shortcut_with_umid(&self, umid: &str) -> bool {
        let matches = |entry: &ShortcutEntry| entry.item.umid.as_deref() == Some(umid);
        self.user_menu.any(matches) || self.system_menu.any(matches)
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchiconresource
    pub fn get_by_file_umid(&self, umid: &str) -> Option<Arc<StartMenuItem>> {
        let matches = |item: &StartMenuItem| {
            if let Some(item_umid) = &item.umid {
                return item_umid == umid;
            }
            if let Some(target) = &item.target {
                // some apps registered as media player as example use the process name as umid
                return target.ends_with(umid);
            }
            false
        };
        self.find_shortcut(matches)
            .or_else(|| self.packaged.find_and_clone(|item| matches(item)))
    }

    /// Resolving each shortcut (lnk target, umid, toast activator) hits the shell, so it's done
    /// in parallel. The order of the given paths is preserved.
    ///
    /// Also keeps what the shortcut launches, used to drop duplicated shortcuts.
    fn _process_file(path: PathBuf) -> ShortcutEntry {
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

        let identity = resolved.as_ref().and_then(|(target, args, working_dir)| {
            ShortcutIdentity::new(target, args, working_dir)
        });
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

    /// win32 unpackaged
    ///
    /// The common and user folders are independent, so they are scanned at the same time and
    /// each one starts resolving its shortcuts without waiting for the other one.
    ///
    /// Everything is kept, duplicates are only dropped on [`Self::get_all`].
    fn load_shortcuts(&self) {
        let (user, system) = rayon::join(
            || Self::load_shortcut_items(&Self::user_items_path()),
            || Self::load_shortcut_items(&Self::common_items_path()),
        );
        self.user_menu.replace(user);
        self.system_menu.replace(system);
    }

    fn load_shortcut_items(folder: &Path) -> Vec<ShortcutEntry> {
        crate::utils::collect_files(folder)
            .into_par_iter()
            .map(Self::_process_file)
            .collect()
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
        Self::instance().load_shortcuts();
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

/// A shortcut file and what it launches.
#[derive(Debug, Clone)]
struct ShortcutEntry {
    item: Arc<StartMenuItem>,
    identity: Option<ShortcutIdentity>,
}

/// What a shortcut launches. Paths are compared case insensitive, as the file system does.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct ShortcutIdentity {
    target: String,
    arguments: OsString,
    working_dir: String,
}

impl ShortcutIdentity {
    /// Shortcuts without a resolvable target (e.g. advertised MSI shortcuts) can't be compared.
    fn new(target: &Path, arguments: &OsString, working_dir: &Path) -> Option<Self> {
        if target.as_os_str().is_empty() {
            return None;
        }
        Some(Self {
            target: target.to_string_lossy().to_lowercase(),
            arguments: arguments.clone(),
            working_dir: working_dir
                .to_string_lossy()
                .trim_end_matches(MAIN_SEPARATOR)
                .to_lowercase(),
        })
    }
}
