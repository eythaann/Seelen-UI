mod adapters;
mod interface_type;

use std::{ffi::c_void, sync::Arc};

use seelen_core::system_state::{Hotspot, HotspotState, NetworkAdapter};
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
            NetworkListManager::{
                INetworkListManager, NLM_CONNECTIVITY_IPV4_INTERNET,
                NLM_CONNECTIVITY_IPV6_INTERNET, NetworkListManager,
            },
            WinSock::{AF_INET, AF_INET6, AF_UNSPEC, SOCKADDR, SOCKADDR_IN, SOCKADDR_IN6},
        },
    },
};

use crate::{
    error::{Result, ResultLogExt},
    system_modules::network::domain::{NetworkSysEvent, NetworkSystem},
    windows_api::Com,
};

use self::adapters::adapter_to_slu_net_adapter;

type EventCallback = Box<dyn Fn(NetworkSysEvent) + Send + Sync>;

pub struct WindowsNetwork {
    /// Context of the IP interface change callback (`Arc::as_ptr`). It must outlive the
    /// registration, which `Drop` cancels before the fields are dropped.
    on_event: Option<Arc<EventCallback>>,
    status_changed_token: Option<i64>,
    ip_interface_change_handle: Option<HANDLE>,
}

// SAFETY: the notification handle is only used to cancel the registration.
unsafe impl Send for WindowsNetwork {}
unsafe impl Sync for WindowsNetwork {}

impl NetworkSystem for WindowsNetwork {
    fn create() -> Self {
        Self {
            on_event: None,
            status_changed_token: None,
            ip_interface_change_handle: None,
        }
    }

    fn init<F>(&mut self, on_event: F) -> Result<()>
    where
        F: Fn(NetworkSysEvent) + Send + Sync + 'static,
    {
        let on_event: Arc<EventCallback> = Arc::new(Box::new(on_event));
        let context = Arc::as_ptr(&on_event) as *const c_void;
        self.on_event = Some(on_event.clone());

        self.status_changed_token = Some(NetworkInformation::NetworkStatusChanged(
            &NetworkStatusChangedEventHandler::new(move |_| {
                on_event(NetworkSysEvent::StatusChanged);
                Ok(())
            }),
        )?);

        let mut handle = HANDLE::default();
        let err = unsafe {
            NotifyIpInterfaceChange(
                AF_UNSPEC,
                Some(on_ip_interface_change),
                Some(context),
                false,
                &mut handle,
            )
        };
        if err != NO_ERROR {
            return Err(format!("NotifyIpInterfaceChange failed: {}", err.0).into());
        }
        self.ip_interface_change_handle = Some(handle);
        Ok(())
    }

    fn adapters(&self) -> Result<Vec<NetworkAdapter>> {
        unsafe {
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
            adapters_from_raw(buffer.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH)
        }
    }

    fn has_internet(&self) -> Result<bool> {
        Com::run_with_context(|| {
            let list_manager: INetworkListManager = Com::create_instance(&NetworkListManager)?;
            let connectivity = unsafe { list_manager.GetConnectivity()? };
            Ok(connectivity.0 & NLM_CONNECTIVITY_IPV4_INTERNET.0
                == NLM_CONNECTIVITY_IPV4_INTERNET.0
                || connectivity.0 & NLM_CONNECTIVITY_IPV6_INTERNET.0
                    == NLM_CONNECTIVITY_IPV6_INTERNET.0)
        })
    }

    fn hotspot(&self) -> Result<Option<Hotspot>> {
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
            state: hotspot_state_from_native(tethering.TetheringOperationalState()?),
            ssid: config.Ssid().ok().map(|s| s.to_string()),
            passphrase: config.Passphrase().ok().map(|s| s.to_string()),
            band,
            encryption,
        }))
    }

    fn set_hotspot_enabled(&self, enabled: bool) -> Result<()> {
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

impl Drop for WindowsNetwork {
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

unsafe extern "system" fn on_ip_interface_change(
    context: *const c_void,
    _row: *const MIB_IPINTERFACE_ROW,
    _notification_type: MIB_NOTIFICATION_TYPE,
) {
    // SAFETY: `context` points to the `EventCallback` kept alive by `WindowsNetwork::on_event`.
    let on_event = unsafe { &*(context as *const EventCallback) };
    on_event(NetworkSysEvent::AdaptersChanged);
}

fn hotspot_state_from_native(state: TetheringOperationalState) -> HotspotState {
    match state {
        TetheringOperationalState::On => HotspotState::On,
        TetheringOperationalState::Off => HotspotState::Off,
        TetheringOperationalState::InTransition => HotspotState::InTransition,
        _ => HotspotState::Unknown,
    }
}

unsafe fn adapters_from_raw(raw: *const IP_ADAPTER_ADDRESSES_LH) -> Result<Vec<NetworkAdapter>> {
    unsafe {
        let default_interface = default_interface();
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

struct DefaultInterface {
    index: u32,
    ipv6: bool,
}

/// Interface used by the default route, IPv4 preferred and IPv6 as fallback.
/// Returns `None` when there is no default route (offline), an expected state, not an error.
fn default_interface() -> Option<DefaultInterface> {
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
