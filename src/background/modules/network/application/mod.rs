pub mod scanner;

use std::{
    env::temp_dir,
    net::{IpAddr, UdpSocket},
};

use tauri_plugin_shell::ShellExt;
use windows::Win32::{
    NetworkManagement::IpHelper::{
        GetAdaptersAddresses, GAA_FLAG_INCLUDE_GATEWAYS, GAA_FLAG_INCLUDE_PREFIX,
        IP_ADAPTER_ADDRESSES_LH,
    },
    Networking::{
        NetworkListManager::{
            INetworkListManager, NetworkListManager, NLM_CONNECTIVITY,
            NLM_CONNECTIVITY_IPV4_INTERNET, NLM_CONNECTIVITY_IPV6_INTERNET,
        },
        WinSock::AF_UNSPEC,
    },
};

use crate::{
    error_handler::Result,
    seelen::get_app_handle,
    utils::{pwsh::PwshScript, spawn_named_thread},
    windows_api::Com,
};

use super::domain::{NetworkAdapter, WlanProfile};

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

pub struct NetworkManager {}

impl NetworkManager {
    pub fn get_adapters() -> Result<Vec<NetworkAdapter>> {
        let adapters = unsafe {
            let family = AF_UNSPEC.0 as u32;
            let flags = GAA_FLAG_INCLUDE_PREFIX | GAA_FLAG_INCLUDE_GATEWAYS;
            let mut buffer_length = 0_u32;

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
        F: Fn(NLM_CONNECTIVITY, String) + Send + 'static,
    {
        spawn_named_thread("Network Manager", move || {
            let result: Result<()> = Com::run_with_context(|| {
                let list_manager: INetworkListManager = Com::create_instance(&NetworkListManager)?;
                let mut last_state = None;
                let mut last_ip = None;

                loop {
                    let current_state = unsafe { list_manager.GetConnectivity() }.ok();
                    if let (Some(current_state), Some(last_state)) = (current_state, last_state) {
                        if current_state != last_state {
                            last_ip = get_local_ip_address_base().ok();
                            cb(current_state, last_ip.unwrap().to_string());
                        } else if current_state.0 & NLM_CONNECTIVITY_IPV4_INTERNET.0
                            == NLM_CONNECTIVITY_IPV4_INTERNET.0
                            || current_state.0 & NLM_CONNECTIVITY_IPV6_INTERNET.0
                                == NLM_CONNECTIVITY_IPV6_INTERNET.0
                        {
                            let current_ip = get_local_ip_address_base().ok();
                            if let (Some(current_ip), Some(last_ip)) = (current_ip, last_ip) {
                                if current_ip != last_ip {
                                    cb(current_state, current_ip.to_string())
                                }
                            }

                            last_ip = current_ip;
                        }
                    }
                    last_state = current_state;
                    std::thread::sleep(std::time::Duration::from_millis(5000));
                }
            });

            log::warn!("Network loop finished: {:?}", result);
        })
        .expect("Failed to spawn network manager loop");
    }

    pub async fn get_wifi_profiles() -> Result<Vec<WlanProfile>> {
        let path = PwshScript::new(include_str!("profiles.ps1"))
            .execute()
            .await?;
        let contents = std::fs::read_to_string(path)?;
        let profiles: Vec<WlanProfile> = serde_json::from_str(&contents)?;
        Ok(profiles)
    }

    pub async fn add_profile(ssid: &str, password: &str, hidden: bool) -> Result<()> {
        log::trace!("Adding profile {}", ssid);
        let profile_xml = if password.is_empty() {
            // Be sure that xml is using tabs instead of spaces for indentation
            include_str!("passwordless_profile.template.xml")
                .replace("{ssid}", ssid)
                .replace("{hidden}", if hidden { "true" } else { "false" })
        } else {
            // Be sure that xml is using tabs instead of spaces for indentation
            include_str!("profile.template.xml")
                .replace("{ssid}", ssid)
                .replace("{password}", password)
                .replace("{hidden}", if hidden { "true" } else { "false" })
        };

        let profile_path = temp_dir().join(format!("slu-{}-profile.xml", ssid));

        std::fs::write(&profile_path, profile_xml)?;

        let handle = get_app_handle();
        let output = handle
            .shell()
            .command("netsh")
            .args([
                "wlan",
                "add",
                "profile",
                &format!("filename={}", &profile_path.to_string_lossy()),
            ])
            .output()
            .await?;

        let result = if output.status.success() {
            Ok(())
        } else {
            Err(output.into())
        };

        std::fs::remove_file(&profile_path)?;
        result
    }

    pub async fn remove_profile(ssid: &str) -> Result<()> {
        log::trace!("Removing profile {}", ssid);

        let handle = get_app_handle();
        let output = handle
            .shell()
            .command("netsh")
            .args(["wlan", "delete", "profile", &format!("name={}", ssid)])
            .output()
            .await?;

        if output.status.success() {
            Ok(())
        } else {
            Err(output.into())
        }
    }
}

pub fn get_local_ip_address() -> Result<String> {
    Ok(get_local_ip_address_base()?.to_string())
}
fn get_local_ip_address_base() -> Result<IpAddr> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.connect("8.8.8.8:80")?;
    let local_addr = socket.local_addr()?;
    Ok(local_addr.ip())
}
