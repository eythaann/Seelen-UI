use std::net::UdpSocket;

use windows::Win32::{
    NetworkManagement::IpHelper::{
        GetAdaptersAddresses, GAA_FLAG_INCLUDE_GATEWAYS, GAA_FLAG_INCLUDE_PREFIX,
        IP_ADAPTER_ADDRESSES_LH,
    },
    Networking::{
        NetworkListManager::{INetworkListManager, NetworkListManager, NLM_CONNECTIVITY},
        WinSock::AF_UNSPEC,
    },
    System::Com::{
        CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, COINIT_APARTMENTTHREADED,
    },
};

use crate::error_handler::Result;

use super::domain::NetworkAdapter;

impl NetworkAdapter {
    pub unsafe fn iter_from_raw(
        raw: *const IP_ADAPTER_ADDRESSES_LH,
    ) -> Result<Vec<NetworkAdapter>> {
        let mut adapters = Vec::new();

        let mut raw_adapter = raw;
        while !raw_adapter.is_null() {
            let adapter = &*raw_adapter;
            adapters.push(adapter.try_into()?);
            raw_adapter = adapter.Next;
        }

        Ok(adapters)
    }
}

/* #[implement(INetworkListManagerEvents)]
struct NetworkListManagerEvents {}

impl INetworkListManagerEvents_Impl for NetworkListManagerEvents {
    fn ConnectivityChanged(
        &self,
        _new_connectivity: NLM_CONNECTIVITY,
    ) -> Result<(), windows::core::Error> {
        log::debug!("Connectivity changed! {:?}", _new_connectivity);
        log_if_error(emit_network_events());
        Ok(())
    }
}
 */

pub struct NetworkManager {}

impl NetworkManager {
    pub fn get_adapters() -> Result<Vec<NetworkAdapter>> {
        let adapters = unsafe {
            let family = AF_UNSPEC.0 as u32;
            let flags = GAA_FLAG_INCLUDE_PREFIX | GAA_FLAG_INCLUDE_GATEWAYS;
            let mut buffer_length = 0 as u32;

            // first call to get the buffer size
            GetAdaptersAddresses(family, flags, None, None, &mut buffer_length);

            let mut adapters_addresses: Vec<u8> = vec![0; buffer_length as usize];
            GetAdaptersAddresses(
                family,
                flags,
                None,
                Some(adapters_addresses.as_mut_ptr() as *mut IP_ADAPTER_ADDRESSES_LH),
                &mut buffer_length,
            );

            let raw_adapter = adapters_addresses.as_ptr() as *const IP_ADAPTER_ADDRESSES_LH;
            NetworkAdapter::iter_from_raw(raw_adapter)?
        };

        Ok(adapters)
    }

    /// emit connectivity changes, always will emit the current state on registration
    pub fn register_events<F>(cb: F)
    where
        F: Fn(NLM_CONNECTIVITY) + Send + 'static,
    {
        std::thread::spawn(move || -> Result<()> {
            unsafe {
                CoInitializeEx(None, COINIT_APARTMENTTHREADED)?;

                let list_manager: INetworkListManager =
                    CoCreateInstance(&NetworkListManager, None, CLSCTX_ALL)?;

                CoUninitialize();

                let mut last = None;
                loop {
                    match last {
                        Some(last_state) => {
                            let current_state = list_manager.GetConnectivity()?;
                            if last_state != current_state {
                                last = Some(current_state);
                                cb(current_state);
                            }
                        }
                        None => {
                            let state = list_manager.GetConnectivity()?;
                            last = Some(state);
                            cb(state);
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(1000));
                }
            }
        });
    }
}

pub fn get_local_ip_address() -> Result<String> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip().to_string())
}
