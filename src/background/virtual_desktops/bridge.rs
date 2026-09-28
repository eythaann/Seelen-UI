use std::{
    collections::HashMap,
    io::Write,
    sync::{
        LazyLock,
        atomic::{AtomicU32, Ordering},
    },
};

use crate::{
    error::{Result, ResultLogExt},
    utils::{
        constants::SEELEN_COMMON,
        lock_free::{SyncHashMap, SyncVec},
    },
    virtual_desktops::VdManager,
};
use seelen_core::{
    state::{VirtualDesktopMonitor, VirtualDesktops, WorkspaceId},
    system_state::MonitorId,
};
use slu_utils::{Debounce, debounce};

pub fn get_indexes(
    monitors: &HashMap<MonitorId, VirtualDesktopMonitor>,
) -> (HashMap<WorkspaceId, MonitorId>, HashMap<isize, WorkspaceId>) {
    let mut workspace_index = HashMap::new();
    let mut window_index = HashMap::new();

    for (mid, m) in monitors {
        for row in m.workspaces.rows() {
            for space in row {
                workspace_index.insert(space.id.clone(), mid.clone());

                for window in &space.windows {
                    window_index.insert(*window, space.id.clone());
                }
            }
        }
    }

    (workspace_index, window_index)
}

impl From<VirtualDesktops> for VdManager {
    fn from(value: VirtualDesktops) -> Self {
        let (workspace_index, window_index) = get_indexes(&value.monitors);
        Self {
            monitors: SyncHashMap::from(value.monitors),
            pinned: SyncVec::from(value.pinned),
            switching: AtomicU32::new(0),
            workspace_index: SyncHashMap::from(workspace_index),
            window_index: SyncHashMap::from(window_index),
        }
    }
}

impl From<&VdManager> for VirtualDesktops {
    fn from(value: &VdManager) -> Self {
        Self {
            monitors: value.monitors.to_hash_map(),
            pinned: value.pinned.to_vec(),
            switching: value.switching.load(Ordering::SeqCst) > 0,
        }
    }
}

pub fn load_stored() -> Result<VirtualDesktops> {
    let path = SEELEN_COMMON.app_cache_dir().join("workspaces2.json");
    let file = std::fs::File::open(path)?;
    file.lock()?;
    Ok(serde_json::from_reader(file)?)
}

fn save_state() -> Result<()> {
    let path = SEELEN_COMMON.app_cache_dir().join("workspaces2.json");
    let state: VirtualDesktops = VdManager::instance().into();

    let mut file = std::fs::File::create(path)?;
    serde_json::to_writer(&mut file, &state)?;
    file.flush()?;

    log::trace!("desktop workspaces successfully saved");
    Ok(())
}

pub fn request_save() {
    static SAVE_DEBOUNCER: LazyLock<Debounce<()>> = LazyLock::new(|| {
        debounce(
            |_| {
                save_state().log_error();
            },
            std::time::Duration::from_secs(2),
        )
    });
    SAVE_DEBOUNCER.call(());
}
