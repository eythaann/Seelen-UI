//! Persistence of the tiling layout, only to survive app restarts: the file is scoped to the OS
//! logon session (see [`crate::platform::session::logon_key`]), so a new session never finds the
//! previous one and starts from zero. It lives in the temp folder, so no cleanup is needed.

use std::{collections::HashMap, path::PathBuf, sync::LazyLock};

use seelen_core::{
    resource::PluginId,
    state::{TwmRuntimeTree, WorkspaceId},
    system_state::WindowIdentity,
};
use serde::{Deserialize, Serialize};
use slu_utils::{Debounce, debounce};

use crate::{
    error::{Result, ResultLogExt},
    platform::session,
    utils::{atomic_write_file, constants::SEELEN_COMMON},
};

use super::state_v2::WM_STATE;

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TwmStoredState {
    pub workspaces: HashMap<WorkspaceId, TwmStoredWorkspace>,
    pub monocle: HashMap<WorkspaceId, bool>,
    pub paused: bool,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TwmStoredWorkspace {
    /// Layout of the workspace when saved, the tree is discarded if it changed meanwhile.
    pub layout: PluginId,
    /// Only the nodes are serialized, the runtime indexes have to be rebuilt on load.
    pub tree: TwmRuntimeTree,
    /// Floating windows are not part of the nodes, so they are stored apart.
    #[serde(default)]
    pub floating: Vec<WindowIdentity>,
}

pub fn load_stored() -> Result<TwmStoredState> {
    let file = std::fs::File::open(file_path()?)?;
    file.lock()?;
    Ok(serde_json::from_reader(file)?)
}

fn save_state() -> Result<()> {
    // the snapshot is taken and the lock released before touching the disk
    let state = WM_STATE.lock().to_stored();
    atomic_write_file(&file_path()?, &serde_json::to_vec(&state)?)?;
    log::trace!("tiling window manager state successfully saved");
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

fn file_path() -> Result<PathBuf> {
    Ok(SEELEN_COMMON
        .app_temp_dir()
        .join(format!("twm_{}.json", session::logon_key()?)))
}
