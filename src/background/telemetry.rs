use std::time::Duration;

use serde::Serialize;
use uuid::Uuid;
use winreg::{
    enums::{HKEY_CURRENT_USER, KEY_ALL_ACCESS, KEY_READ},
    RegKey,
};

const REGISTRY_SUBKEY: &str = "Software\\Seelen UI\\Analytics";
const INSTALL_ID_VALUE: &str = "InstallId";

#[cfg(dev)]
const TELEMETRY_ENDPOINT: &str = "https://telemetry.staging.seelen.io/events";
#[cfg(not(dev))]
const TELEMETRY_ENDPOINT: &str = "https://telemetry.seelen.io/events";

/// Mirrors `TelemetryEventContext` from the sl-telemetry server.
/// Uses `tag = "eventName"` and `rename_all = "camelCase"` to match the server's serde config.
#[derive(Debug, Serialize)]
#[serde(rename_all_fields = "camelCase", tag = "eventName")]
pub enum TelemetryEvent {
    SeelenUIUsage {
        install_id: uuid::Uuid,
        os_version: String,
        app_version: String,
    },
}

/// Returns the persisted install ID from the registry, creating it if absent.
///
/// Stored at `HKCU\Software\Seelen\Analytics` → `InstallId` (REG_SZ).
/// This key is NOT declared in the MSIX package manifest, so it survives
/// across reinstalls and package updates.
fn get_or_create_install_id() -> crate::error::Result<Uuid> {
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    if let Ok(key) = hkcu.open_subkey_with_flags(REGISTRY_SUBKEY, KEY_READ) {
        if let Ok(id_str) = key.get_value::<String, _>(INSTALL_ID_VALUE) {
            if let Ok(id) = Uuid::parse_str(&id_str) {
                return Ok(id);
            }
        }
    }

    let new_id = Uuid::new_v4();
    let (key, _) = hkcu.create_subkey_with_flags(REGISTRY_SUBKEY, KEY_ALL_ACCESS)?;
    key.set_value(INSTALL_ID_VALUE, &new_id.to_string())?;
    log::debug!("Telemetry: generated new install_id {new_id}");
    Ok(new_id)
}

fn os_version_string() -> String {
    let info = os_info::get();
    format!("{} {}", info.os_type(), info.version())
}

async fn send_event(install_id: Uuid) {
    let event = TelemetryEvent::SeelenUIUsage {
        install_id,
        os_version: os_version_string(),
        app_version: env!("CARGO_PKG_VERSION").to_string(),
    };

    let body = match serde_json::to_string(&event) {
        Ok(b) => b,
        Err(err) => {
            log::debug!("Telemetry: failed to serialize event: {err:?}");
            return;
        }
    };

    let client = reqwest::Client::new();
    match client
        .post(TELEMETRY_ENDPOINT)
        .header("Content-Type", "application/json")
        .body(body)
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(resp) if resp.status().is_success() => {
            log::debug!("Telemetry: event sent (status {})", resp.status());
        }
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            log::debug!("Telemetry: server responded with {status}: {body}");
        }
        Err(err) => {
            log::debug!("Telemetry: request failed: {err:?}");
        }
    }
}

/// Spawns a background task that sends a `SeelenUIUsage` telemetry event
/// immediately on startup and then every 10 minutes.
///
/// Errors obtaining the install ID are logged but do not affect the rest of the app.
pub fn start_telemetry() {
    let install_id = match get_or_create_install_id() {
        Ok(id) => id,
        Err(err) => {
            log::warn!("Telemetry: could not obtain install_id, skipping: {err:?}");
            return;
        }
    };

    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10 * 60));
        loop {
            interval.tick().await; // first tick fires immediately
            send_event(install_id).await;
        }
    });
}
