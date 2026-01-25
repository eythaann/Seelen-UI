use std::path::{Path, PathBuf};
use std::sync::{Arc, LazyLock};
use std::time::Duration;

use notify_debouncer_full::{
    new_debouncer,
    notify::{ReadDirectoryChangesWatcher, RecursiveMode, Watcher},
    DebounceEventResult, DebouncedEvent, Debouncer, FileIdMap,
};
use seelen_core::system_state::StartMenuItem;
use windows::{
    ApplicationModel::PackageCatalog,
    Foundation::TypedEventHandler,
    Management::Deployment::PackageManager,
    Win32::UI::Shell::{FOLDERID_CommonPrograms, FOLDERID_Programs},
    UI::StartScreen::StartScreenManager,
};

use crate::{
    error::{Result, ResultLogExt},
    event_manager, log_error,
    utils::{constants::SEELEN_COMMON, lock_free::SyncVec},
    windows_api::WindowsApi,
};

pub struct StartMenuManager {
    pub list: SyncVec<Arc<StartMenuItem>>,
    cache_path: PathBuf,
    _file_watcher: Option<Arc<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>>,
    _package_catalog: Option<PackageCatalog>,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum StartMenuEvent {
    ItemAdded(Arc<StartMenuItem>),
    ItemRemoved(Arc<StartMenuItem>),
}

event_manager!(StartMenuManager, StartMenuEvent);

unsafe impl Send for StartMenuManager {}
unsafe impl Sync for StartMenuManager {}

impl StartMenuManager {
    /// programs shared by all users
    pub fn common_items_path() -> PathBuf {
        WindowsApi::known_folder(FOLDERID_CommonPrograms)
            .expect("Failed to get common programs folder")
    }

    /// programs specific to the current user
    pub fn user_items_path() -> PathBuf {
        WindowsApi::known_folder(FOLDERID_Programs).expect("Failed to get user programs folder")
    }

    fn new() -> StartMenuManager {
        StartMenuManager {
            list: SyncVec::new(),
            cache_path: SEELEN_COMMON.app_cache_dir().join("start_menu_v2.json"),
            _file_watcher: None,
            _package_catalog: None,
        }
    }

    pub fn instance() -> &'static Self {
        static START_MENU_MANAGER: LazyLock<StartMenuManager> = LazyLock::new(|| {
            let mut manager = StartMenuManager::new();
            manager.init().log_error();
            manager
        });
        &START_MENU_MANAGER
    }

    fn init(&mut self) -> Result<()> {
        if self.cache_path.exists() {
            match self.load_cache() {
                Ok(_) => {
                    // refresh without blocking
                    std::thread::spawn(|| {
                        let menu = StartMenuManager::instance();
                        if let Ok(items) = Self::load_start_menu_items() {
                            menu.list.replace(items);
                            menu.store_cache().log_error();
                        };
                    });
                    // Setup listeners after loading cache
                    self.setup_listeners().log_error();
                    return Ok(());
                }
                Err(e) => {
                    log::error!("Failed to load start menu cache: {e}");
                }
            }
        }

        self.list.replace(Self::load_start_menu_items()?);
        self.store_cache()?;
        // Setup listeners after initial load
        self.setup_listeners().log_error();
        Ok(())
    }

    pub fn get_by_target(&self, target: &Path) -> Option<Arc<StartMenuItem>> {
        self.list
            .find(|item| item.target.as_ref().is_some_and(|t| t == target))
    }

    /// https://learn.microsoft.com/en-us/windows/win32/properties/props-system-appusermodel-relaunchiconresource
    pub fn get_by_file_umid(&self, umid: &str) -> Option<Arc<StartMenuItem>> {
        self.list.find(|item| {
            if let Some(item_umid) = &item.umid {
                return item_umid == umid;
            }
            if let Some(target) = &item.target {
                // some apps registered as media player as example use the process name as umid
                return target.ends_with(umid);
            }
            false
        })
    }

    pub fn store_cache(&self) -> Result<()> {
        let file = std::fs::File::create(&self.cache_path)?;
        let writer = std::io::BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.list.to_vec())?;
        Ok(())
    }

    pub fn load_cache(&mut self) -> Result<()> {
        let file = std::fs::File::open(&self.cache_path)?;
        let reader = std::io::BufReader::new(file);
        let items: Vec<StartMenuItem> = serde_json::from_reader(reader)?;
        self.list.replace(items.into_iter().map(Arc::new).collect());
        Ok(())
    }

    fn _get_items(dir: &Path) -> Result<Vec<Arc<StartMenuItem>>> {
        let mut items = Vec::new();
        for entry in std::fs::read_dir(dir)?.flatten() {
            let path = entry.path();
            let file_type = entry.file_type()?;
            if file_type.is_dir() {
                items.extend(Self::_get_items(&path)?);
                continue;
            }
            if file_type.is_file() {
                let target = WindowsApi::resolve_lnk_target(&path).ok().map(|(t, _)| t);
                // Get display name from filename without extension
                let display_name = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("Unknown")
                    .to_string();

                items.push(Arc::new(StartMenuItem {
                    umid: WindowsApi::get_file_umid(&path).ok(),
                    toast_activator: WindowsApi::get_file_toast_activator(&path).ok(),
                    path,
                    target,
                    display_name,
                }))
            }
        }
        Ok(items)
    }

    pub fn load_start_menu_items() -> Result<Vec<Arc<StartMenuItem>>> {
        let mut items = Vec::new();

        // win32 unpackaged
        items.extend(Self::_get_items(&Self::common_items_path())?);
        items.extend(Self::_get_items(&Self::user_items_path())?);

        // win32/uwp packaged
        let pkg_manager = PackageManager::new()?;
        let start_screen = StartScreenManager::GetDefault()?;

        let packages = pkg_manager.FindPackagesByUserSecurityId(&"".into())?;
        for package in packages {
            let apps = package.GetAppListEntries()?;

            for app in apps {
                // https://learn.microsoft.com/en-us/uwp/schemas/appxpackage/uapmanifestschema/element-uap-visualelements
                let is_on_start_screen = start_screen.SupportsAppListEntry(&app)?;

                if is_on_start_screen {
                    let umid = app.AppUserModelId()?.to_string_lossy();

                    // Get display name from DisplayInfo
                    let display_name = app
                        .DisplayInfo()?
                        .DisplayName()?
                        .to_string_lossy()
                        .to_string();

                    items.push(Arc::new(StartMenuItem {
                        umid: Some(umid),
                        toast_activator: None,
                        path: PathBuf::new(),
                        target: None,
                        display_name,
                    }))
                }
            }
        }

        Ok(items)
    }

    fn create_file_watcher(
        &self,
    ) -> Result<Arc<Debouncer<ReadDirectoryChangesWatcher, FileIdMap>>> {
        let mut debouncer = new_debouncer(
            Duration::from_millis(500),
            None,
            |result: DebounceEventResult| match result {
                Ok(events) => {
                    log::debug!(
                        "Start menu file watcher detected changes: {} events",
                        events.len()
                    );
                    log_error!(Self::on_files_changed(events));
                }
                Err(errors) => {
                    log::error!("Start menu file watcher error: {errors:?}");
                }
            },
        )?;

        let watcher = debouncer.watcher();
        watcher.watch(&Self::common_items_path(), RecursiveMode::Recursive)?;
        watcher.watch(&Self::user_items_path(), RecursiveMode::Recursive)?;

        Ok(Arc::new(debouncer))
    }

    fn on_files_changed(_events: Vec<DebouncedEvent>) -> Result<()> {
        let manager = Self::instance();

        // Reload all items when file changes are detected
        let new_items = Self::load_start_menu_items()?;
        let old_items = manager.list.to_vec();

        // Find removed items
        for old_item in &old_items {
            if !new_items
                .iter()
                .any(|new_item| new_item.path == old_item.path && new_item.umid == old_item.umid)
            {
                log::debug!("Start menu item removed: {:?}", old_item.path);
                Self::send(StartMenuEvent::ItemRemoved(old_item.clone()));
            }
        }

        // Find added items
        for new_item in &new_items {
            if !old_items
                .iter()
                .any(|old_item| new_item.path == old_item.path && new_item.umid == old_item.umid)
            {
                log::debug!("Start menu item added: {:?}", new_item.path);
                Self::send(StartMenuEvent::ItemAdded(new_item.clone()));
            }
        }

        manager.list.replace(new_items);
        manager.store_cache().log_error();

        Ok(())
    }

    fn setup_package_catalog_listener(&mut self) -> Result<()> {
        let catalog = PackageCatalog::OpenForCurrentUser()?;

        let handler_installing = TypedEventHandler::new(|_catalog, _args| {
            log::debug!("Package installing event detected");
            // Reload start menu items when package is being installed
            std::thread::spawn(|| {
                std::thread::sleep(Duration::from_millis(1000));
                log_error!(Self::on_package_changed());
            });
            Ok(())
        });

        catalog.PackageInstalling(&handler_installing)?;

        let handler_uninstalling = TypedEventHandler::new(|_catalog, _args| {
            log::debug!("Package uninstalling event detected");
            // Reload start menu items when package is being uninstalled
            std::thread::spawn(|| {
                std::thread::sleep(Duration::from_millis(1000));
                log_error!(Self::on_package_changed());
            });
            Ok(())
        });

        catalog.PackageUninstalling(&handler_uninstalling)?;

        self._package_catalog = Some(catalog);

        Ok(())
    }

    fn on_package_changed() -> Result<()> {
        let manager = Self::instance();

        let new_items = Self::load_start_menu_items()?;
        let old_items = manager.list.to_vec();

        // Find removed items
        for old_item in &old_items {
            if !new_items
                .iter()
                .any(|new_item| new_item.path == old_item.path && new_item.umid == old_item.umid)
            {
                log::debug!("Start menu item removed (package): {:?}", old_item.umid);
                Self::send(StartMenuEvent::ItemRemoved(old_item.clone()));
            }
        }

        // Find added items
        for new_item in &new_items {
            if !old_items
                .iter()
                .any(|old_item| new_item.path == old_item.path && new_item.umid == old_item.umid)
            {
                log::debug!("Start menu item added (package): {:?}", new_item.umid);
                Self::send(StartMenuEvent::ItemAdded(new_item.clone()));
            }
        }

        manager.list.replace(new_items);
        manager.store_cache().log_error();

        Ok(())
    }

    pub fn setup_listeners(&mut self) -> Result<()> {
        // Setup file system watcher
        let watcher = self.create_file_watcher()?;
        self._file_watcher = Some(watcher);

        // Setup package catalog listener
        self.setup_package_catalog_listener().log_error();

        Ok(())
    }
}
