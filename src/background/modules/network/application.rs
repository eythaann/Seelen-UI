use std::sync::LazyLock;

use seelen_core::system_state::{Hotspot, HotspotState, NetworkAdapter as SluNetAdapter};
use windows::{
    Networking::{
        Connectivity::{NetworkInformation, NetworkStatusChangedEventHandler},
        NetworkOperators::{
            NetworkOperatorTetheringAccessPointConfiguration, NetworkOperatorTetheringManager,
            TetheringCapability, TetheringOperationStatus, TetheringOperationalState,
            TetheringWiFiAuthenticationKind, TetheringWiFiBand,
        },
    },
    Win32::{
        Foundation::{HANDLE, NO_ERROR},
        NetworkManagement::IpHelper::{
            CancelMibChangeNotify2, GAA_FLAG_INCLUDE_GATEWAYS, GAA_FLAG_INCLUDE_PREFIX,
            GetAdaptersAddresses, GetBestInterfaceEx, IP_ADAPTER_ADDRESSES_LH, MIB_IPINTERFACE_ROW,
            MIB_NOTIFICATION_TYPE, NotifyIpInterfaceChange,
        },
        Networking::{
            NetworkListManager::{INetworkListManager, NLM_CONNECTIVITY, NetworkListManager},
            WinSock::{AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR, SOCKADDR_IN, SOCKADDR_IN6},
        },
    },
};

use crate::{
    error::{Result, ResultLogExt},
    event_manager,
    windows_api::Com,
};

use super::domain::adapter_to_slu_net_adapter;

pub struct NetworkManager {
    status_changed_token: Option<i64>,
    ip_interface_change_handle: Option<HANDLE>,
}

#[derive(Debug, Clone)]
#[allow(clippy::enum_variant_names)]
pub enum NetworkManagerEvent {
    AdaptersChanged,
    ConnectivityChanged(NLM_CONNECTIVITY),
    HotspotChanged(Option<Hotspot>),
}

unsafe impl Send for NetworkManager {}
unsafe impl Sync for NetworkManager {}

event_manager!(NetworkManager, NetworkManagerEvent);

impl NetworkManager {
    pub fn instance() -> &'static Self {
        static NETWORK_MANAGER: LazyLock<NetworkManager> = LazyLock::new(|| {
            let mut m = NetworkManager::new();
            m.init().log_error();
            m
        });
        &NETWORK_MANAGER
    }

    fn new() -> Self {
        Self {
            status_changed_token: None,
            ip_interface_change_handle: None,
        }
    }

    fn init(&mut self) -> Result<()> {
        self.status_changed_token = Some(NetworkInformation::NetworkStatusChanged(
            &NetworkStatusChangedEventHandler::new(|_| {
                Self::emit_connectivity_state().log_error();
                Self::emit_hotspot_state().log_error();
                Ok(())
            }),
        )?);

        let mut handle = HANDLE::default();
        let err = unsafe {
            NotifyIpInterfaceChange(
                AF_UNSPEC,
                Some(on_ip_interface_change),
                None,
                false,
                &mut handle,
            )
        };
        if err == NO_ERROR {
            self.ip_interface_change_handle = Some(handle);
        } else {
            return Err(format!("NotifyIpInterfaceChange failed: {}", err.0).into());
        }
        Ok(())
    }

    fn emit_connectivity_state() -> Result<()> {
        let list_manager: INetworkListManager = Com::create_instance(&NetworkListManager)?;
        let connectivity = unsafe { list_manager.GetConnectivity()? };
        NetworkManager::send(NetworkManagerEvent::ConnectivityChanged(connectivity));
        Ok(())
    }

    fn emit_hotspot_state() -> Result<()> {
        let hotspot = Self::get_hotspot()?;
        NetworkManager::send(NetworkManagerEvent::HotspotChanged(hotspot));
        Ok(())
    }

    pub fn get_adapters() -> Result<Vec<SluNetAdapter>> {
        let adapters = unsafe {
            let family = AF_UNSPEC.0 as u32;
            let flags = GAA_FLAG_INCLUDE_PREFIX | GAA_FLAG_INCLUDE_GATEWAYS;
            let mut buffer_length = 0_u32;
            GetAdaptersAddresses(family, flags, None, None, &mut buffer_length);
            let mut buffer: Vec<u8> = vec![0; buffer_length as usize];
            GetAdaptersAddresses(
                family,
                flags,
                None,
                Some(buffer.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
                &mut buffer_length,
            );
            SluNetAdapter::iter_from_raw(buffer.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH)?
        };
        Ok(adapters)
    }

    /// Returns `None` when hotspot/tethering is not supported or not available
    /// on the current connection (e.g. no active internet connection profile).
    pub fn get_hotspot() -> Result<Option<Hotspot>> {
        let Ok(profile) = NetworkInformation::GetInternetConnectionProfile() else {
            return Ok(None);
        };

        let capability =
            NetworkOperatorTetheringManager::GetTetheringCapabilityFromConnectionProfile(&profile);
        if !matches!(capability, Ok(TetheringCapability::Enabled)) {
            return Ok(None);
        }

        let Ok(tethering) = NetworkOperatorTetheringManager::CreateFromConnectionProfile(&profile)
        else {
            return Ok(None);
        };

        let config = match tethering.GetCurrentAccessPointConfiguration() {
            Ok(config) => config,
            // Windows never persisted an access-point configuration for this profile
            // (e.g. the native Mobile Hotspot feature was never turned on). Tethering
            // is still available, so configure a default access point and retry.
            Err(_) => {
                let default_config = NetworkOperatorTetheringAccessPointConfiguration::new()?;
                let ssid =
                    std::env::var("COMPUTERNAME").unwrap_or_else(|_| "PC Hotspot".to_string());
                default_config.SetSsid(&ssid.into())?;
                default_config.SetPassphrase(&"12345678".into())?;
                tethering
                    .ConfigureAccessPointAsync(&default_config)?
                    .join()?;
                tethering.GetCurrentAccessPointConfiguration()?
            }
        };

        // Band was introduced in windows 10 19041
        let band = match config.Band().unwrap_or_default() {
            TetheringWiFiBand::Auto => "Auto",
            TetheringWiFiBand::TwoPointFourGigahertz => "2.4GHz",
            TetheringWiFiBand::FiveGigahertz => "5GHz",
            TetheringWiFiBand::SixGigahertz => "6GHz",
            _ => "Unknown",
        }
        .to_string();

        // AuthenticationKind was introduced in windows 11 26100
        let encryption = match config.AuthenticationKind().unwrap_or_default() {
            TetheringWiFiAuthenticationKind::Wpa2 => "WPA2",
            TetheringWiFiAuthenticationKind::Wpa3 => "WPA3",
            TetheringWiFiAuthenticationKind::Wpa3TransitionMode => "WPA2/WPA3",
            _ => "Unknown",
        }
        .to_string();

        Ok(Some(Hotspot {
            clients: tethering.ClientCount()?,
            max_clients: tethering.MaxClientCount()?,
            state: hotspot_state_from_tethering(tethering.TetheringOperationalState()?),
            ssid: config.Ssid().ok().map(|s| s.to_string()),
            passphrase: config.Passphrase().ok().map(|s| s.to_string()),
            band,
            encryption,
        }))
    }

    pub fn toggle_hotspot(enabled: bool) -> Result<()> {
        let tethering = NetworkOperatorTetheringManager::CreateFromConnectionProfile(
            &NetworkInformation::GetInternetConnectionProfile()?,
        )?;
        let result = if enabled {
            tethering.StartTetheringAsync()?.join()?
        } else {
            tethering.StopTetheringAsync()?.join()?
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

impl Drop for NetworkManager {
    fn drop(&mut self) {
        if let Some(token) = self.status_changed_token.take() {
            NetworkInformation::RemoveNetworkStatusChanged(token).log_error();
        }

        if let Some(handle) = self.ip_interface_change_handle.take()
            && let Err(err) = unsafe { CancelMibChangeNotify2(handle).ok() }
        {
            log::error!("CancelMibChangeNotify2 failed: {err}");
        }
    }
}

fn hotspot_state_from_tethering(state: TetheringOperationalState) -> HotspotState {
    match state {
        TetheringOperationalState::On => HotspotState::On,
        TetheringOperationalState::Off => HotspotState::Off,
        TetheringOperationalState::InTransition => HotspotState::InTransition,
        _ => HotspotState::Unknown,
    }
}

unsafe extern "system" fn on_ip_interface_change(
    _caller_context: *const std::ffi::c_void,
    _row: *const MIB_IPINTERFACE_ROW,
    _notification_type: MIB_NOTIFICATION_TYPE,
) {
    NetworkManager::send(NetworkManagerEvent::AdaptersChanged);
}

trait IterFromRaw {
    unsafe fn iter_from_raw(raw: *const IP_ADAPTER_ADDRESSES_LH) -> Result<Vec<SluNetAdapter>>;
}

impl IterFromRaw for SluNetAdapter {
    unsafe fn iter_from_raw(raw: *const IP_ADAPTER_ADDRESSES_LH) -> Result<Vec<SluNetAdapter>> {
        unsafe {
            let default_interface = get_default_interface();
            let mut adapters = Vec::new();
            let mut ptr = raw;
            while !ptr.is_null() {
                let adapter = &*ptr;
                let is_default = default_interface.as_ref().is_some_and(|d| {
                    if d.ipv6 {
                        adapter.Ipv6IfIndex == d.index
                    } else {
                        adapter.Anonymous1.Anonymous.IfIndex == d.index
                    }
                });
                adapters.push(adapter_to_slu_net_adapter(adapter, is_default)?);
                ptr = adapter.Next;
            }
            Ok(adapters)
        }
    }
}

struct DefaultInterface {
    index: u32,
    ipv6: bool,
}

/// Interface used by the default route, IPv4 preferred and IPv6 as fallback.
/// Returns `None` when there is no default route (offline), an expected state, not an error.
fn get_default_interface() -> Option<DefaultInterface> {
    // public DNS addresses, only used to ask the routing table, no traffic is sent
    let mut ipv4 = SOCKADDR_IN {
        sin_family: AF_INET,
        ..Default::default()
    };
    ipv4.sin_addr.S_un.S_addr = u32::from_ne_bytes([8, 8, 8, 8]);

    let mut ipv6 = SOCKADDR_IN6 {
        sin6_family: AF_INET6,
        ..Default::default()
    };
    ipv6.sin6_addr.u.Byte = "2001:4860:4860::8888"
        .parse::<std::net::Ipv6Addr>()
        .ok()?
        .octets();

    let mut index = 0u32;
    if unsafe { GetBestInterfaceEx(&ipv4 as *const _ as *const SOCKADDR, &mut index) } == NO_ERROR.0
    {
        return Some(DefaultInterface { index, ipv6: false });
    }
    if unsafe { GetBestInterfaceEx(&ipv6 as *const _ as *const SOCKADDR, &mut index) } == NO_ERROR.0
    {
        return Some(DefaultInterface { index, ipv6: true });
    }
    None
}
