use tauri::Manager;
use windows::Win32::Networking::NetworkListManager::{
    NLM_CONNECTIVITY_IPV4_INTERNET, NLM_CONNECTIVITY_IPV6_INTERNET,
};

use crate::{
    error_handler::{log_if_error, Result},
    seelen::get_app_handle,
};

use super::application::{get_local_ip_address, NetworkManager};

pub fn register_network_events() -> Result<()> {
    let handle = get_app_handle();

    let adapters = NetworkManager::get_adapters()?;
    handle.emit("network-adapters", adapters)?;

    NetworkManager::register_events(move |connectivity| {
        let has_internet_ipv4 = connectivity.0 & NLM_CONNECTIVITY_IPV4_INTERNET.0 == NLM_CONNECTIVITY_IPV4_INTERNET.0;
        let has_internet_ipv6 = connectivity.0 & NLM_CONNECTIVITY_IPV6_INTERNET.0 == NLM_CONNECTIVITY_IPV6_INTERNET.0;

        let has_internet = has_internet_ipv4 || has_internet_ipv6;

        log_if_error(handle.emit("network-internet-connection", has_internet));

        if has_internet {
            let ip = get_local_ip_address().unwrap_or_default();
            log_if_error(handle.emit("network-default-local-ip", ip));
        }
    });

    Ok(())
}
