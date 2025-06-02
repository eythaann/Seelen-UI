use windows::{
    Devices::Radios::RadioKind,
    Networking::{
        Connectivity::NetworkInformation,
        NetworkOperators::{
            NetworkOperatorTetheringManager, TetheringOperationStatus,
            TetheringWiFiAuthenticationKind, TetheringWiFiBand,
        },
    },
};

use crate::{
    error_handler::Result,
    modules::{network::domain::Hotspot, shared::radio::RADIO_MANAGER},
    trace_lock,
};

#[derive(Debug)]
#[allow(dead_code)]
pub struct NetworkManagerV2 {}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum NetworkManagerEventV2 {}

// event_manager!(NetworkManagerV2, NetworkManagerEventV2);

#[allow(dead_code)]
impl NetworkManagerV2 {
    pub fn new() -> Self {
        Self {}
    }

    pub fn toggle_wifi(&self, enabled: bool) -> Result<()> {
        let radios = trace_lock!(RADIO_MANAGER);
        if enabled {
            radios.turn_on_radios(RadioKind::WiFi)?;
        } else {
            radios.turn_off_radios(RadioKind::WiFi)?;
        }
        Ok(())
    }

    pub fn hotspot() -> Result<Option<Hotspot>> {
        if let Ok(profile) = NetworkInformation::GetInternetConnectionProfile() {
            let hotspot = NetworkOperatorTetheringManager::CreateFromConnectionProfile(&profile)?;

            let config = hotspot.GetCurrentAccessPointConfiguration()?;
            let band = match config.Band()? {
                TetheringWiFiBand::Auto => "Auto",
                TetheringWiFiBand::TwoPointFourGigahertz => "2.4GHz",
                TetheringWiFiBand::FiveGigahertz => "5GHz",
                TetheringWiFiBand::SixGigahertz => "6GHz",
                _ => "???",
            }
            .to_string();

            let encryption = match config.AuthenticationKind()? {
                TetheringWiFiAuthenticationKind::Wpa2 => "WPA2",
                TetheringWiFiAuthenticationKind::Wpa3 => "WPA3",
                TetheringWiFiAuthenticationKind::Wpa3TransitionMode => "WPA2/WPA3",
                _ => "???",
            }
            .to_string();

            let state = Hotspot {
                clients: hotspot.ClientCount()?,
                max_clients: hotspot.MaxClientCount()?,
                state: hotspot.TetheringOperationalState()?.into(),
                ssid: config.Ssid().ok().map(|s| s.to_string()),
                passphrase: config.Passphrase().ok().map(|s| s.to_string()),
                band,
                encryption,
            };

            return Ok(Some(state));
        }
        Ok(None)
    }

    pub fn toggle_hotspot(enabled: bool) -> Result<()> {
        let hotspot = NetworkOperatorTetheringManager::CreateFromConnectionProfile(
            &NetworkInformation::GetInternetConnectionProfile()?,
        )?;
        let result = if enabled {
            hotspot.StartTetheringAsync()?.get()?
        } else {
            hotspot.StopTetheringAsync()?.get()?
        };
        let status = result.Status()?;
        if status != TetheringOperationStatus::Success {
            return Err(format!(
                "Failed to toggle hotspot, error code: {:?} - {:?}",
                status,
                result.AdditionalErrorMessage()
            )
            .into());
        }
        Ok(())
    }
}
