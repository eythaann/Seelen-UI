use tauri::Manager;
use tauri_plugin_shell::ShellExt;
use windows::Win32::Networking::NetworkListManager::{
    NLM_CONNECTIVITY_IPV4_INTERNET, NLM_CONNECTIVITY_IPV6_INTERNET,
};

use crate::{
    error_handler::{log_if_error, Result},
    seelen::get_app_handle,
    utils::sleep_millis,
};

use super::{
    application::{get_local_ip_address, NetworkManager},
    domain::WlanProfile,
};

pub fn register_network_events() -> Result<()> {
    let handle = get_app_handle();

    NetworkManager::register_events(move |connectivity| {
        log::info!(target: "network", "Connectivity changed: {:?}", connectivity);

        let has_internet_ipv4 =
            connectivity.0 & NLM_CONNECTIVITY_IPV4_INTERNET.0 == NLM_CONNECTIVITY_IPV4_INTERNET.0;
        let has_internet_ipv6 =
            connectivity.0 & NLM_CONNECTIVITY_IPV6_INTERNET.0 == NLM_CONNECTIVITY_IPV6_INTERNET.0;

        let has_internet = has_internet_ipv4 || has_internet_ipv6;
        log_if_error(handle.emit("network-internet-connection", has_internet));

        match NetworkManager::get_adapters() {
            Ok(adapters) => log_if_error(handle.emit("network-adapters", adapters)),
            Err(err) => log::error!("Fail on getting network adapters: {}", err),
        }

        match get_local_ip_address() {
            Ok(ip) => log_if_error(handle.emit("network-default-local-ip", ip)),
            Err(err) => log::error!("Fail on getting local ip address: {}", err),
        }
    });

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
        log_if_error(app.emit("wlan-scanned", &list));
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
