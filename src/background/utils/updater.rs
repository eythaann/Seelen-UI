use seelen_core::state::UpdateChannel;
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::{error_handler::Result, seelen::get_app_handle, state::application::FULL_STATE};

fn is_update_valid_on_user_channel(update: &Update) -> bool {
    let state = FULL_STATE.load();
    let channel = state.settings.updater.channel;
    match channel {
        // all updates are available on nightly
        UpdateChannel::Nightly => true,
        // version must not contain `+` that means <build-identifier> used for nightly in this project
        UpdateChannel::Beta => !update.version.contains('+'),
        // version must not contain `+` used for nightly or `-` used for <pre-releases> as `-beta` or `-alpha`
        UpdateChannel::Release => !update.version.contains('+') && !update.version.contains('-'),
    }
}

pub async fn check_for_updates() -> Result<Option<Update>> {
    if tauri::is_dev() || std::env::current_exe()?.starts_with("C:\\Program Files\\WindowsApps") {
        return Ok(None);
    }
    let updater: tauri_plugin_updater::Updater = get_app_handle().updater()?;
    let update = updater.check().await?;
    match update {
        Some(update) if is_update_valid_on_user_channel(&update) => Ok(Some(update)),
        _ => Ok(None),
    }
}

pub async fn trace_update_intallation(update: Update) -> Result<()> {
    log::trace!("Update: downloading");
    update
        .download_and_install(
            |_chunk_length, _content_length| {},
            || log::trace!("Update: download finished"),
        )
        .await?;
    log::trace!("Update: intallation finished");
    Ok(())
}
