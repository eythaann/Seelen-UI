use std::collections::HashMap;
use std::sync::{Arc, LazyLock, OnceLock};
use std::time::Duration;

use parking_lot::RwLock;
use rand::Rng;
use seelen_core::handlers::SeelenEvent;
use seelen_core::resource::WallpaperId;
use seelen_core::state::{WallpaperCollection, WorkspaceId};
use seelen_core::system_state::MonitorId;
use tauri::Listener;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::app::get_app_handle;
use crate::error::{Result, ResultLogExt};
use crate::get_tokio_handle;
use crate::state::application::FULL_STATE;

use super::events::VirtualDesktopEvent;

/// Tracks the current wallpaper index for each collection
static COLLECTION_INDICES: LazyLock<Arc<RwLock<HashMap<Uuid, usize>>>> =
    LazyLock::new(|| Arc::new(RwLock::new(HashMap::new())));

static MANUAL_CHANGE_SENDER: OnceLock<mpsc::UnboundedSender<ChangeDirection>> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ChangeDirection {
    Next,
    Previous,
}

pub struct WorkspaceWallpapersManager;
impl WorkspaceWallpapersManager {
    pub fn init(manager: &super::SluWorkspacesManager2) {
        log::trace!("Initializing Workspaces Wallpaper Manager");

        // Set initial wallpapers for all workspaces
        Self::update_workspace_wallpapers_internal(manager);

        // Start rotation loop
        let (tx, rx) = mpsc::unbounded_channel();
        MANUAL_CHANGE_SENDER.set(tx).ok();
        get_tokio_handle().spawn(async move {
            Self::rotation_loop(rx).await.log_error();
        });

        // Listen for settings changes
        get_app_handle().listen(SeelenEvent::StateSettingsChanged, |_| {
            Self::on_settings_changed();
        });
    }

    /// Handle settings changes
    fn on_settings_changed() {
        let vd_manager = super::SluWorkspacesManager2::instance();
        Self::update_workspace_wallpapers_internal(vd_manager);
        super::SluWorkspacesManager2::send(VirtualDesktopEvent::StateChanged);
    }

    /// Get the wallpaper collection ID for a given workspace on a monitor
    /// Priority: workspace collection → monitor collection → global collection → None
    fn get_collection_id(monitor_id: &MonitorId, workspace_id: &WorkspaceId) -> Option<Uuid> {
        let state = FULL_STATE.load();

        // Try to get workspace collection
        if let Some(monitor_config) = state.settings.monitors_v3.get(monitor_id) {
            if let Some(workspace_config) = monitor_config.by_workspace.get(workspace_id) {
                if let Some(collection_id) = workspace_config.wallpaper_collection {
                    return Some(collection_id);
                }
            }

            // Try to get monitor collection
            if let Some(collection_id) = monitor_config.wallpaper_collection {
                return Some(collection_id);
            }
        }

        // Try to get global default collection
        state.settings.by_widget.wall.default_collection
    }

    /// Get the collection by UUID
    fn get_collection(collection_id: &Uuid) -> Option<WallpaperCollection> {
        let state = FULL_STATE.load();
        state
            .settings
            .wallpaper_collections
            .iter()
            .find(|c| &c.id == collection_id)
            .cloned()
    }

    /// Get the current wallpaper for a workspace
    pub fn get_current_wallpaper(
        monitor_id: &MonitorId,
        workspace_id: &WorkspaceId,
    ) -> Option<WallpaperId> {
        let collection_id = Self::get_collection_id(monitor_id, workspace_id)?;
        let collection = Self::get_collection(&collection_id)?;

        if collection.wallpapers.is_empty() {
            return None;
        }

        let indices = COLLECTION_INDICES.read();
        let index = indices.get(&collection_id).copied().unwrap_or(0);
        let wallpaper_index = index % collection.wallpapers.len();

        collection.wallpapers.get(wallpaper_index).cloned()
    }

    /// Increment the index for a collection
    fn increment_collection_index(collection_id: &Uuid, direction: ChangeDirection) {
        let collection = match Self::get_collection(collection_id) {
            Some(c) => c,
            None => return,
        };

        if collection.wallpapers.is_empty() {
            return;
        }

        let state = FULL_STATE.load();
        // Randomize only makes sense if there are more than 2 wallpapers
        let randomize = state.settings.by_widget.wall.randomize && collection.wallpapers.len() > 2;

        let mut indices = COLLECTION_INDICES.write();
        let current_index = indices.get(collection_id).copied().unwrap_or(0);

        let new_index = if randomize {
            let mut rng = rand::rng();
            loop {
                let random_index = rng.random_range(0..collection.wallpapers.len());
                if random_index != current_index {
                    break random_index;
                }
            }
        } else {
            match direction {
                ChangeDirection::Next => (current_index + 1) % collection.wallpapers.len(),
                ChangeDirection::Previous => {
                    if current_index == 0 {
                        collection.wallpapers.len() - 1
                    } else {
                        current_index - 1
                    }
                }
            }
        };

        indices.insert(*collection_id, new_index);
    }

    /// Update wallpapers for all workspaces
    fn update_all_wallpapers(direction: ChangeDirection) {
        let state = FULL_STATE.load();

        // Collect all unique collection IDs being used
        let mut active_collections = std::collections::HashSet::new();

        for monitor in state.settings.monitors_v3.values() {
            // Add monitor-level collection if it exists
            if let Some(collection_id) = monitor.wallpaper_collection {
                active_collections.insert(collection_id);
            }

            // Add workspace-level collections
            for workspace_config in monitor.by_workspace.values() {
                if let Some(collection_id) = workspace_config.wallpaper_collection {
                    active_collections.insert(collection_id);
                }
            }
        }

        // Add global default collection if it exists
        if let Some(collection_id) = state.settings.by_widget.wall.default_collection {
            active_collections.insert(collection_id);
        }

        // Increment index for all active collections
        for collection_id in active_collections {
            Self::increment_collection_index(&collection_id, direction);
        }

        {
            let vd_manager = super::SluWorkspacesManager2::instance();
            Self::update_workspace_wallpapers_internal(vd_manager);
            super::SluWorkspacesManager2::send(VirtualDesktopEvent::StateChanged);
        }
    }

    /// Update wallpaper IDs in all workspaces (internal method with manager reference)
    fn update_workspace_wallpapers_internal(vd_manager: &super::SluWorkspacesManager2) {
        vd_manager.monitors.for_each(|(monitor_id, monitor)| {
            for workspace in &mut monitor.workspaces {
                let wallpaper_id = Self::get_current_wallpaper(monitor_id, &workspace.id);
                workspace.wallpaper = wallpaper_id;
            }
        });
    }

    /// Get the interval duration from settings (in seconds)
    fn get_interval_duration() -> Duration {
        let state = FULL_STATE.load();
        let interval_seconds = state.settings.by_widget.wall.interval;
        Duration::from_secs(interval_seconds as u64)
    }

    /// Main rotation loop
    async fn rotation_loop(mut rx: mpsc::UnboundedReceiver<ChangeDirection>) -> Result<()> {
        loop {
            let interval = Self::get_interval_duration();

            // Wait for either the interval to elapse or a manual change to be triggered
            tokio::select! {
                _ = tokio::time::sleep(interval) => {
                    log::trace!("Automatic wallpaper rotation triggered");
                    Self::update_all_wallpapers(ChangeDirection::Next);
                }
                direction = rx.recv() => {
                    if let Some(direction) = direction {
                        log::trace!("Manual wallpaper change triggered: {:?}", direction);
                        Self::update_all_wallpapers(direction);
                    }
                }
            }
        }
    }

    /// Manually advance to next wallpaper
    pub fn next() {
        if let Some(sender) = MANUAL_CHANGE_SENDER.get() {
            if let Err(e) = sender.send(ChangeDirection::Next) {
                log::warn!("Failed to send next wallpaper command: {}", e);
            }
        }
    }

    /// Manually go to previous wallpaper
    pub fn previous() {
        if let Some(sender) = MANUAL_CHANGE_SENDER.get() {
            if let Err(e) = sender.send(ChangeDirection::Previous) {
                log::warn!("Failed to send previous wallpaper command: {}", e);
            }
        }
    }
}
