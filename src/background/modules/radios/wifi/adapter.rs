use std::collections::HashSet;

use seelen_core::system_state::WlanBssEntry;
use windows::{
    Devices::WiFi::WiFiAdapter,
    Foundation::TypedEventHandler,
    Networking::Connectivity::{NetworkAuthenticationType, NetworkInformation},
};
use windows_core::HSTRING;

use crate::error::{Result, ResultLogExt};

use super::{WifiManager, WifiManagerEvent};

// ── Known profiles ────────────────────────────────────────────────────────────

pub fn wifi_known_profiles() -> HashSet<String> {
    let mut known = HashSet::new();

    let Ok(profiles) = NetworkInformation::GetConnectionProfiles() else {
        return known;
    };

    for i in 0..profiles.Size().unwrap_or(0) {
        let Ok(profile) = profiles.GetAt(i) else {
            continue;
        };
        if !profile.IsWlanConnectionProfile().unwrap_or(false) {
            continue;
        }
        let Ok(name) = profile.ProfileName() else {
            continue;
        };
        let ssid = name.to_string();
        if !ssid.is_empty() {
            known.insert(ssid);
        }
    }

    known
}

// ── Auth label ────────────────────────────────────────────────────────────────

pub fn auth_type_label(auth: NetworkAuthenticationType) -> &'static str {
    if auth == NetworkAuthenticationType::Open80211 {
        "Open"
    } else if auth == NetworkAuthenticationType::SharedKey80211 {
        "WEP"
    } else if auth == NetworkAuthenticationType::WpaPsk {
        "WPA-Personal"
    } else if auth == NetworkAuthenticationType::Wpa {
        "WPA-Enterprise"
    } else if auth == NetworkAuthenticationType::RsnaPsk {
        "WPA2-Personal"
    } else if auth == NetworkAuthenticationType::Rsna {
        "WPA2-Enterprise"
    } else if auth == NetworkAuthenticationType::WpaNone {
        "WPA-None"
    } else {
        "Unknown"
    }
}

// ── SluWifiAdapter ────────────────────────────────────────────────────────────

/// RAII wrapper around a single `WiFiAdapter`.
///
/// Subscribes to `AvailableNetworksChanged` on construction and unregisters on drop.
/// When the event fires it sends `WifiManagerEvent::NetworksChanged` so the
/// infrastructure layer can emit updated network data to webviews.
pub struct SluWifiAdapter {
    #[allow(dead_code)]
    pub id: String,
    pub raw: WiFiAdapter,
    token: i64,
}

unsafe impl Send for SluWifiAdapter {}

impl SluWifiAdapter {
    /// Obtain the adapter by device ID and subscribe to network change events.
    pub fn create(device_id: &str) -> Result<Self> {
        let raw = WiFiAdapter::FromIdAsync(&HSTRING::from(device_id))?.join()?;

        let token = raw.AvailableNetworksChanged(&TypedEventHandler::new(|_, _| {
            WifiManager::send(WifiManagerEvent::NetworksChanged);
            Ok(())
        }))?;

        Ok(Self {
            id: device_id.to_string(),
            raw,
            token,
        })
    }

    /// Trigger a hardware scan. Rate-limited calls (E_ABORT) are silently ignored.
    pub fn scan(&self) {
        let adapter = self.raw.clone();
        std::thread::spawn(move || {
            if let Ok(operation) = adapter.ScanAsync() {
                if operation.join().is_ok() {
                    WifiManager::send(WifiManagerEvent::NetworksChanged);
                }
            }
        });
    }

    fn get_connected_ssid(&self) -> Option<String> {
        let profile = self
            .raw
            .NetworkAdapter()
            .ok()?
            .GetConnectedProfileAsync()
            .ok()?
            .join()
            .ok()?;
        profile
            .ProfileName()
            .ok()
            .map(|s| s.to_string())
            .filter(|s| !s.is_empty())
    }

    /// Read the current `NetworkReport` for this adapter.
    pub fn get_available_networks(&self, known: &HashSet<String>) -> Result<Vec<WlanBssEntry>> {
        let connected_ssid = self.get_connected_ssid();
        let networks = self.raw.NetworkReport()?.AvailableNetworks()?;
        let mut entries = Vec::new();

        for net in &networks {
            let ssid_raw = net.Ssid()?.to_string();
            let ssid = if ssid_raw.is_empty() {
                None
            } else {
                Some(ssid_raw)
            };
            let bssid = net.Bssid()?.to_string();
            let signal = (net.SignalBars()? as u32) * 20;
            let channel_frequency = net.ChannelCenterFrequencyInKilohertz()? as u32;
            let auth = net.SecuritySettings()?.NetworkAuthenticationType()?;
            let secured = auth != NetworkAuthenticationType::Unknown
                && auth != NetworkAuthenticationType::Open80211;
            let known_net = ssid.as_deref().is_some_and(|s| known.contains(s));
            let is_connected = ssid
                .as_deref()
                .is_some_and(|s| connected_ssid.as_deref() == Some(s));

            entries.push(WlanBssEntry {
                ssid,
                bssid,
                channel_frequency,
                signal,
                known: known_net,
                secured,
                auth: auth_type_label(auth).to_string(),
                connected: is_connected,
            });
        }

        Ok(entries)
    }
}

impl Drop for SluWifiAdapter {
    fn drop(&mut self) {
        self.raw
            .RemoveAvailableNetworksChanged(self.token)
            .log_error();
    }
}
