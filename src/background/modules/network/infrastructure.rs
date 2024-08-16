use std::sync::atomic::{AtomicBool, Ordering};

use tauri::Emitter;
use tauri_plugin_shell::ShellExt;
use windows::Win32::Networking::NetworkListManager::{
    INetworkListManager, NetworkListManager, NLM_CONNECTIVITY_IPV4_INTERNET,
    NLM_CONNECTIVITY_IPV6_INTERNET,
};

use crate::{
    error_handler::Result, log_error, seelen::get_app_handle, utils::sleep_millis, windows_api::Com,
};

use super::{
    application::{get_local_ip_address, NetworkManager},
    domain::{NetworkAdapter, WlanProfile},
};

fn emit_networks(ip: String, adapters: Vec<NetworkAdapter>, has_internet: bool) {
    let handle = get_app_handle();
    log_error!(handle.emit("network-default-local-ip", ip));
    log_error!(handle.emit("network-adapters", adapters));
    log_error!(handle.emit("network-internet-connection", has_internet));
}

static REGISTERED: AtomicBool = AtomicBool::new(false);
pub fn register_network_events() -> Result<()> {
    if !REGISTERED.load(Ordering::Acquire) {
        log::trace!("Registering network events");
        NetworkManager::register_events(move |connectivity| {
            log::trace!(target: "network", "Connectivity changed: {:?}", connectivity);
            if let (Ok(ip), Ok(adapters)) = (get_local_ip_address(), NetworkManager::get_adapters())
            {
                let has_internet_ipv4 = connectivity.0 & NLM_CONNECTIVITY_IPV4_INTERNET.0
                    == NLM_CONNECTIVITY_IPV4_INTERNET.0;
                let has_internet_ipv6 = connectivity.0 & NLM_CONNECTIVITY_IPV6_INTERNET.0
                    == NLM_CONNECTIVITY_IPV6_INTERNET.0;

                emit_networks(ip, adapters, has_internet_ipv4 || has_internet_ipv6);
            }
        });
        REGISTERED.store(true, Ordering::Release);
    }

    if let (Ok(ip), Ok(adapters)) = (get_local_ip_address(), NetworkManager::get_adapters()) {
        let has_internet = Com::run_with_context(|| {
            let list_manager: INetworkListManager = Com::create_instance(&NetworkListManager)?;
            let connectivity = unsafe { list_manager.GetConnectivity()? };

            let has_internet_ipv4 = connectivity.0 & NLM_CONNECTIVITY_IPV4_INTERNET.0
                == NLM_CONNECTIVITY_IPV4_INTERNET.0;
            let has_internet_ipv6 = connectivity.0 & NLM_CONNECTIVITY_IPV6_INTERNET.0
                == NLM_CONNECTIVITY_IPV6_INTERNET.0;

            Ok(has_internet_ipv4 || has_internet_ipv6)
        })?;
        emit_networks(ip, adapters, has_internet);
    }

    Ok(())
}

async fn try_connect_to_profile(ssid: &str) -> Result<bool> {
    let handle = get_app_handle();
    let output = handle
        .shell()
        .command("netsh")
        .args(["wlan", "connect", &format!("name={}", ssid)])
        .output()
        .await?;

    if output.status.success() {
        // wait to ensure connection
        sleep_millis(2000);
        Ok(NetworkManager::is_connected_to(ssid)?)
    } else {
        Err(output.into())
    }
}

#[tauri::command]
pub fn wlan_start_scanning() {
    log::trace!("Start scanning networks");
    NetworkManager::start_scanning(|list| {
        let app = get_app_handle();
        log_error!(app.emit("wlan-scanned", &list));
    });
}

#[tauri::command]
pub fn wlan_stop_scanning() {
    log::trace!("Stop scanning networks");
    NetworkManager::stop_scanning();
}

#[tauri::command]
pub async fn wlan_get_profiles() -> Result<Vec<WlanProfile>> {
    NetworkManager::get_wifi_profiles().await
}

#[tauri::command]
pub async fn wlan_connect(ssid: String, password: String, hidden: bool) -> Result<bool> {
    NetworkManager::add_profile(&ssid, &password, hidden).await?;
    match try_connect_to_profile(&ssid).await {
        Ok(true) => Ok(true),
        Ok(false) => {
            NetworkManager::remove_profile(&ssid).await?;
            Ok(false)
        }
        Err(err) => {
            NetworkManager::remove_profile(&ssid).await?;
            Err(err)
        }
    }
}

#[tauri::command]
pub async fn wlan_disconnect() -> Result<()> {
    let handle = get_app_handle();
    let output = handle
        .shell()
        .command("netsh")
        .args(["wlan", "disconnect"])
        .output()
        .await?;

    if output.status.success() {
        Ok(())
    } else {
        Err(output.into())
    }
}
