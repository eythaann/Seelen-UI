use std::{
    ffi::OsString,
    path::{MAIN_SEPARATOR, Path},
    sync::Arc,
};

use seelen_core::system_state::{StartMenuItem, StartMenuLayout};

use crate::error::Result;

/// OS contract of the start menu module.
pub trait StartMenuSystem: Sized + Send + Sync {
    /// Builds the native struct. Never fails: no OS calls here, resources are acquired in `init`.
    fn create() -> Self;

    /// Starts listening OS changes, reported through `on_event`.
    /// Called once; the subscriptions live as long as `self`.
    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(StartMenuSysEvent) + Send + Sync + 'static;

    /// Shortcuts of the current user start menu, in folder order.
    fn user_shortcuts(&self) -> Vec<ShortcutEntry>;

    /// Shortcuts of the start menu shared by all users, in folder order.
    fn system_shortcuts(&self) -> Vec<ShortcutEntry>;

    /// Apps installed as packages (no shortcut file).
    fn packaged_apps(&self) -> Result<Vec<Arc<StartMenuItem>>>;

    /// Layout of the native start menu (pinned items), with paths already expanded.
    fn native_layout(&self) -> impl Future<Output = Result<StartMenuLayout>> + Send;
}

/// Raw OS notifications: they only signal what changed, the manager reloads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartMenuSysEvent {
    /// Files changed in any of the start menu folders.
    ShortcutsChanged,
    /// A package is being installed or uninstalled.
    PackagesChanged,
}

#[derive(Debug, Clone)]
pub enum StartMenuEvent {
    ItemsRefreshed,
}

/// A shortcut file and what it launches.
#[derive(Debug, Clone)]
pub struct ShortcutEntry {
    pub item: Arc<StartMenuItem>,
    pub identity: Option<ShortcutIdentity>,
}

/// What a shortcut launches. Paths are compared case insensitive, as the file system does.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ShortcutIdentity {
    target: String,
    arguments: OsString,
    working_dir: String,
}

impl ShortcutIdentity {
    /// Shortcuts without a resolvable target (e.g. advertised MSI shortcuts) can't be compared.
    pub fn new(target: &Path, arguments: &OsString, working_dir: &Path) -> Option<Self> {
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
