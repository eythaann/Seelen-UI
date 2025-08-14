use seelen_core::state::UpdateChannel;
use slu_ipc::messages::SvcAction;
use tauri_plugin_updater::{Update, UpdaterExt};

use crate::{
    app::get_app_handle, cli::ServicePipe, error_handler::Result, state::application::FULL_STATE,
};

use super::is_running_as_appx;

pub async fn check_for_updates() -> Result<Option<Update>> {
    if tauri::is_dev() || is_running_as_appx() {
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
            ])?
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
            ])?
            .build()?;
        update = updater.check().await?;
    }

    Ok(update)
}

pub async fn trace_update_intallation(update: Update) -> Result<()> {
    log::trace!("Update: downloading");
    let bytes = update
        .download(
            |_chunk_length, _content_length| {},
            || log::trace!("Update: download finished"),
        )
        .await?;
    ServicePipe::request(SvcAction::Stop)?;
    update.install(bytes)?;
    log::trace!("Update: intallation finished");
    Ok(())
}
