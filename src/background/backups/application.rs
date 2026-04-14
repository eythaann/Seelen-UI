use std::path::PathBuf;

use seelen_core::system_state::BackupStatus;
use serde::{Deserialize, Serialize};

use crate::{
    session::application::SessionManager, state::application::FULL_STATE,
    utils::constants::SEELEN_COMMON,
};

#[cfg(dev)]
const PRODUCT_BASE_URL: &str = "https://product.staging.seelen.io";
#[cfg(not(dev))]
const PRODUCT_BASE_URL: &str = "https://product.seelen.io";

const BACKUP_NAME: &str = "seelen-ui-settings";

#[derive(Serialize)]
struct BackupPayload {
    name: &'static str,
    data: serde_json::Value,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BackupDocument {
    data: serde_json::Value,
    updated_at: String,
}

fn pending_flag_path() -> PathBuf {
    SEELEN_COMMON.app_data_dir().join(".backup_pending")
}

fn last_sync_path() -> PathBuf {
    SEELEN_COMMON.app_data_dir().join(".backup_last_sync")
}

fn mark_sync_pending() {
    let _ = std::fs::write(pending_flag_path(), b"");
}

fn clear_sync_pending() {
    let _ = std::fs::remove_file(pending_flag_path());
}

fn is_sync_pending() -> bool {
    pending_flag_path().exists()
}

fn write_last_sync_now() {
    use time::{format_description::well_known::Rfc3339, OffsetDateTime};
    let now = OffsetDateTime::now_utc()
        .format(&Rfc3339)
        .unwrap_or_default();
    let _ = std::fs::write(last_sync_path(), now.as_bytes());
}

pub fn read_last_sync() -> Option<String> {
    std::fs::read_to_string(last_sync_path()).ok()
}

fn emit_status_changed() {
    use crate::app::emit_to_webviews;
    use seelen_core::handlers::SeelenEvent;
    let status = BackupStatus {
        last_sync: read_last_sync(),
    };
    emit_to_webviews(SeelenEvent::SeelenBackupStatusChanged, status);
}

pub fn get_backup_status() -> BackupStatus {
    BackupStatus {
        last_sync: read_last_sync(),
    }
}

// ─── Public entry points ──────────────────────────────────────────────────────

fn is_backup_sync_enabled() -> bool {
    FULL_STATE.load().settings.backup_sync_enabled
}

/// Fire-and-forget upload triggered after every local settings save.
pub fn on_settings_saved() {
    crate::get_tokio_handle().spawn(async {
        if !is_backup_sync_enabled() {
            return;
        }
        if !SessionManager::instance().lock().has_cloud_backup_access() {
            return;
        }
        if let Err(e) = upload_settings().await {
            log::warn!("Cloud backup upload failed (will retry on next startup): {e:?}");
            mark_sync_pending();
        } else {
            clear_sync_pending();
            write_last_sync_now();
            emit_status_changed();
        }
    });
}

/// Called at startup and on session-change. Reconciles local settings with the
/// cloud backup and either uploads or downloads depending on which is newer.
pub async fn run_cloud_sync() {
    if !is_backup_sync_enabled() {
        return;
    }
    if !SessionManager::instance().lock().has_cloud_backup_access() {
        return;
    }
    if let Err(e) = reconcile().await {
        log::warn!("Cloud backup sync failed: {e:?}");
    }
}

// ─── Core operations ──────────────────────────────────────────────────────────

/// Uploads the current local settings to the cloud backup.
async fn upload_settings() -> crate::error::Result<()> {
    let data = {
        let state = FULL_STATE.load();
        serde_json::to_value(&state.settings)?
    };
    let url = format!("{PRODUCT_BASE_URL}/backup/");
    let resp = SessionManager::authed_post(&url)
        .json(&BackupPayload {
            name: BACKUP_NAME,
            data,
        })
        .send()
        .await?;
    if !resp.status().is_success() {
        return Err(format!("backup upload failed with status {}", resp.status()).into());
    }
    log::debug!("Cloud backup uploaded successfully");
    Ok(())
}

/// Downloads the cloud backup and applies it to local settings, overwriting
/// the local files and triggering a file-watcher reload.
fn download_and_apply(data: serde_json::Value) -> crate::error::Result<()> {
    use seelen_core::state::Settings;
    let mut settings: Settings = serde_json::from_value(data)?;
    settings.sanitize()?;
    // Saving to disk triggers the existing file-watcher →
    // FULL_STATE reload → StateSettingsChanged emitted to frontend.
    settings.save(SEELEN_COMMON.settings_path())?;
    log::info!("Cloud backup downloaded and applied to local settings");
    Ok(())
}

// ─── Reconciliation ───────────────────────────────────────────────────────────

/// Fetches the cloud backup and decides whether to upload local settings or
/// download the cloud version based on which timestamp is newer.
async fn reconcile() -> crate::error::Result<()> {
    let url = format!("{PRODUCT_BASE_URL}/backup/{BACKUP_NAME}");
    let resp = SessionManager::authed_get(&url).send().await?;

    if resp.status() == 404 {
        log::info!("No cloud backup exists yet; uploading local settings");
        upload_settings().await?;
        clear_sync_pending();
        write_last_sync_now();
        emit_status_changed();
        return Ok(());
    }
    if !resp.status().is_success() {
        return Err(format!("backup fetch failed with status {}", resp.status()).into());
    }

    let doc: BackupDocument = resp.json().await?;
    let cloud_secs = parse_rfc3339_secs(&doc.updated_at);
    let local_secs = local_settings_mtime_secs();

    if is_sync_pending() || local_secs > cloud_secs {
        log::info!("Local settings are newer (or upload was pending); uploading to cloud");
        upload_settings().await?;
        clear_sync_pending();
    } else if cloud_secs > local_secs {
        log::info!("Cloud backup is newer; downloading and applying");
        download_and_apply(doc.data)?;
    }
    write_last_sync_now();
    emit_status_changed();
    Ok(())
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn parse_rfc3339_secs(ts: &str) -> u64 {
    use time::{format_description::well_known::Rfc3339, OffsetDateTime};
    OffsetDateTime::parse(ts, &Rfc3339)
        .map(|dt| dt.unix_timestamp() as u64)
        .unwrap_or(0)
}

fn local_settings_mtime_secs() -> u64 {
    SEELEN_COMMON
        .settings_path()
        .metadata()
        .and_then(|m| m.modified())
        .map(|t| {
            t.duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs()
        })
        .unwrap_or(u64::MAX)
}
