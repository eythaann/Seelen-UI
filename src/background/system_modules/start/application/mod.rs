#[cfg(windows)]
mod windows;

use std::{
    collections::HashSet,
    path::Path,
    sync::{Arc, LazyLock},
    time::Duration,
};

use seelen_core::system_state::{StartMenuItem, StartMenuLayout};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    utils::lock_free::SyncVec,
};

use super::domain::{ShortcutEntry, StartMenuEvent, StartMenuSysEvent, StartMenuSystem};

#[cfg(windows)]
type Platform = windows::WindowsStartMenu;

/// Installing/uninstalling packages are notified before the package catalog reflects them.
const PACKAGES_RELOAD_DELAY: Duration = Duration::from_secs(1);

pub struct StartMenuManager {
    platform: Platform,
    /// win32 unpackaged, from the current user start menu folder. Only refreshed by the file watcher.
    user_menu: SyncVec<ShortcutEntry>,
    /// win32 unpackaged, from the start menu folder shared by all users. Only refreshed by the file watcher.
    system_menu: SyncVec<ShortcutEntry>,
    /// win32/uwp packaged, from the package catalog. Only refreshed by the package events.
    packaged: SyncVec<Arc<StartMenuItem>>,
}

event_manager!(StartMenuManager, StartMenuEvent);

impl StartMenuManager {
    /// Cheap to call from anywhere and never blocks on the items: the lists stay empty until
    /// [`Self::initialize`] finishes.
    pub fn instance() -> &'static Self {
        static MANAGER: LazyLock<StartMenuManager> = LazyLock::new(|| {
            let mut m = StartMenuManager::create();
            m.init().log_error();
            m
        });
        &MANAGER
    }

    fn create() -> Self {
        Self {
            platform: Platform::create(),
            user_menu: SyncVec::new(),
            system_menu: SyncVec::new(),
            packaged: SyncVec::new(),
        }
    }

    /// Only the listeners: the items are loaded by [`Self::initialize`].
    fn init(&mut self) -> Result<()> {
        self.platform.init(Self::on_platform_event)
    }

    /// Loads the start menu items. Should be awaited during the app startup.
    pub fn initialize() -> Result<()> {
        let manager = Self::instance();
        crate::measure!("StartMenu Shortcuts", manager.load_shortcuts());
        crate::measure!("StartMenu Packaged", manager.load_packaged()?);
        Ok(())
    }

    fn on_platform_event(event: StartMenuSysEvent) {
        match event {
            StartMenuSysEvent::ShortcutsChanged => {
                Self::instance().load_shortcuts();
                Self::send(StartMenuEvent::ItemsRefreshed);
            }
            StartMenuSysEvent::PackagesChanged => {
                std::thread::spawn(|| {
                    std::thread::sleep(PACKAGES_RELOAD_DELAY);
                    match Self::instance().load_packaged() {
                        Ok(()) => Self::send(StartMenuEvent::ItemsRefreshed),
                        Err(err) => log::error!("Failed to reload packaged apps: {err}"),
                    }
                });
            }
        }
    }

    /// The user and system folders are independent, so they are loaded at the same time.
    /// Everything is kept, duplicates are only dropped on [`Self::get_all`].
    fn load_shortcuts(&self) {
        let (user, system) = rayon::join(
            || self.platform.user_shortcuts(),
            || self.platform.system_shortcuts(),
        );
        self.user_menu.replace(user);
        self.system_menu.replace(system);
    }

    fn load_packaged(&self) -> Result<()> {
        self.packaged.replace(self.platform.packaged_apps()?);
        Ok(())
    }

    /// User shortcuts, then the system ones, then the packaged apps, without duplicates as the
    /// native start menu shows them. The same app can be in both folders, so an item is dropped
    /// if an earlier one has the same umid or is a shortcut launching the same
    /// target/arguments/working dir. This way user items win over the system ones.
    pub async fn native_layout(&self) -> Result<StartMenuLayout> {
        self.platform.native_layout().await
    }

    pub fn get_all(&self) -> Vec<Arc<StartMenuItem>> {
        let mut seen_umids = HashSet::new();
        let mut seen_identities = HashSet::new();

        let mut is_new = |item: &StartMenuItem, identity: Option<_>| {
            // both are always registered, so later duplicates of any of them are also dropped
            let new_umid = item
                .umid
                .as_ref()
                .is_none_or(|umid| seen_umids.insert(umid.clone()));
            let new_identity = identity.is_none_or(|id| seen_identities.insert(id));
            new_umid && new_identity
        };

        let mut items = Vec::new();
        let shortcuts = self
            .user_menu
            .to_vec()
            .into_iter()
            .chain(self.system_menu.to_vec());
        for entry in shortcuts {
            if is_new(&entry.item, entry.identity) {
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
}
