use seelen_core::state::UpdateChannel;
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::{error_handler::Result, seelen::get_app_handle, state::application::FULL_STATE};

pub async fn check_for_updates() -> Result<Option<Update>> {
    if tauri::is_dev() || std::env::current_exe()?.starts_with("C:\\Program Files\\WindowsApps") {
        return Ok(None);
    }

    let state = FULL_STATE.load();
    let channel = state.settings.updater.channel;
    let mut update = None;

    if channel == UpdateChannel::Nightly {
        let updater: tauri_plugin_updater::Updater = get_app_handle()
            .updater_builder()
            .endpoints(vec![
                "https://github.com/eythaann/Seelen-UI/releases/download/nightly/latest.json"
                    .try_into()
                    .expect("Failed to parse url"),
            ])
            .build()?;
        update = updater.check().await?;
    }

    // Release Channel
    if update.is_none() {
        let updater: tauri_plugin_updater::Updater = get_app_handle()
            .updater_builder()
            .endpoints(vec![
                "https://github.com/eythaann/Seelen-UI/releases/latest/download/latest.json"
                    .try_into()
                    .expect("Failed to parse url"),
            ])
            .build()?;
        update = updater.check().await?;
    }

    Ok(update)
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
