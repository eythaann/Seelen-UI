pub mod v2;

use std::{
    net::{IpAddr, UdpSocket},
    sync::LazyLock,
};

use seelen_core::system_state::NetworkAdapter as SluNetAdapter;
use windows::{
    Networking::Connectivity::{NetworkInformation, NetworkStatusChangedEventHandler},
    Win32::{
        Foundation::{HANDLE, NO_ERROR},
        NetworkManagement::IpHelper::{
            CancelMibChangeNotify2, GetAdaptersAddresses, NotifyIpInterfaceChange,
            GAA_FLAG_INCLUDE_GATEWAYS, GAA_FLAG_INCLUDE_PREFIX, IP_ADAPTER_ADDRESSES_LH,
            MIB_IPINTERFACE_ROW, MIB_NOTIFICATION_TYPE,
        },
        Networking::{
            NetworkListManager::{INetworkListManager, NetworkListManager, NLM_CONNECTIVITY},
            WinSock::AF_UNSPEC,
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
pub enum NetworkManagerEvent {
    AdaptersChanged,
    ConnectivityChanged {
        connectivity: NLM_CONNECTIVITY,
        ip: String,
    },
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
        let ip = get_local_ip_address_base()?;
        NetworkManager::send(NetworkManagerEvent::ConnectivityChanged {
            connectivity,
            ip: ip.to_string(),
        });
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
}

impl Drop for NetworkManager {
    fn drop(&mut self) {
        if let Some(token) = self.status_changed_token.take() {
            NetworkInformation::RemoveNetworkStatusChanged(token).log_error();
        }

        if let Some(handle) = self.ip_interface_change_handle.take() {
            if let Err(err) = unsafe { CancelMibChangeNotify2(handle).ok() } {
                log::error!("CancelMibChangeNotify2 failed: {err}");
            }
        }
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
        let mut adapters = Vec::new();
        let mut ptr = raw;
        while !ptr.is_null() {
            let adapter = &*ptr;
            adapters.push(adapter_to_slu_net_adapter(adapter)?);
            ptr = adapter.Next;
        }
        Ok(adapters)
    }
}

pub fn get_local_ip_address() -> Result<String> {
    Ok(get_local_ip_address_base()?.to_string())
}

fn get_local_ip_address_base() -> Result<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    Ok(socket.local_addr()?.ip())
}
