use slu_ipc::messages::{IpcResponse, SvcAction};

use crate::{
    error::Result,
    modules::{
        hotkeys::HotkeysManager, native_wallpaper::NativeWallpaperManager, startup::StartupManager,
        taskbar::TaskbarManager, window_positioning::WindowPositioningManager,
    },
};

async fn _process_action(command: SvcAction) -> Result<()> {
    match command {
        SvcAction::Stop => crate::exit(0),
        // -----------------------------------------------------------------------
        SvcAction::SetStartup(enabled) => StartupManager::instance().set_run_on_logon(enabled)?,
        SvcAction::ShowWindow { window, state } => {
            WindowPositioningManager::instance().set_state(window, state, false)?
        }
        SvcAction::ShowWindowAsync { window, state } => {
            WindowPositioningManager::instance().set_state(window, state, true)?
        }
        SvcAction::SetWindowPosition { window, rect, .. } => {
            WindowPositioningManager::instance().set_position(window, &rect)?
        }
        SvcAction::DeferWindowPositions {
            list,
            animated,
            animation_duration,
            easing,
        } => WindowPositioningManager::instance().place_batch(
            list,
            animated,
            animation_duration,
            &easing,
        )?,
        SvcAction::SetForeground(window) => {
            WindowPositioningManager::instance().set_foreground(window)?
        }
        SvcAction::SetShortcuts(shortcuts) => {
            HotkeysManager::instance().set_shortcuts(shortcuts)?
        }
        SvcAction::StartShortcutRegistration => {
            HotkeysManager::instance().start_recording().await?
        }
        SvcAction::StopShortcutRegistration => HotkeysManager::instance().stop_recording(),
        SvcAction::HideNativeTaskbar => TaskbarManager::instance().hide(),
        SvcAction::RestoreNativeTaskbar => TaskbarManager::instance().restore()?,
        SvcAction::SetNativeWallpaper { path } => {
            tokio::task::spawn_blocking(move || NativeWallpaperManager::instance().set(path))
                .await
                .map_err(|err| err.to_string())??
        }
        SvcAction::RestoreNativeWallpaper => {
            tokio::task::spawn_blocking(|| NativeWallpaperManager::instance().restore())
                .await
                .map_err(|err| err.to_string())??
        }
    }
    Ok(())
}

pub async fn process_action(command: SvcAction) -> IpcResponse {
    log::trace!("Processing action: {:?}", command);
    match _process_action(command).await {
        Ok(()) => IpcResponse::Success,
        Err(err) => IpcResponse::Err(err.to_string()),
    }
}
