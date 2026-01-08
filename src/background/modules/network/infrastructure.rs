use std::sync::Once;

use seelen_core::{handlers::SeelenEvent, system_state::WlanProfile};
use tauri_plugin_shell::ShellExt;
use windows::Win32::Networking::NetworkListManager::{
    INetworkListManager, NetworkListManager, NLM_CONNECTIVITY_IPV4_INTERNET,
    NLM_CONNECTIVITY_IPV6_INTERNET,
};

use crate::{
    app::{emit_to_webviews, get_app_handle},
    error::Result,
    utils::sleep_millis,
    windows_api::Com,
};

use super::application::{get_local_ip_address, NetworkManager, NetworkManagerEvent};

fn get_network_manager() -> &'static NetworkManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        NetworkManager::subscribe(|event| match event {
            NetworkManagerEvent::ConnectivityChanged { connectivity, ip } => {
                log::trace!(target: "network", "Connectivity changed: {connectivity:?}");
                if let Ok(adapters) = NetworkManager::get_adapters() {
                    let has_internet_ipv4 = connectivity.0 & NLM_CONNECTIVITY_IPV4_INTERNET.0
                        == NLM_CONNECTIVITY_IPV4_INTERNET.0;
                    let has_internet_ipv6 = connectivity.0 & NLM_CONNECTIVITY_IPV6_INTERNET.0
                        == NLM_CONNECTIVITY_IPV6_INTERNET.0;

                    emit_to_webviews(SeelenEvent::NetworkDefaultLocalIp, ip);
                    emit_to_webviews(SeelenEvent::NetworkAdapters, adapters);
                    emit_to_webviews(
                        SeelenEvent::NetworkInternetConnection,
                        has_internet_ipv4 || has_internet_ipv6,
                    );
                }
            }
        });
    });
    NetworkManager::instance()
}

async fn try_connect_to_profile(ssid: &str) -> Result<bool> {
    let handle = get_app_handle();
    let output = handle
        .shell()
        .command("netsh")
        .args(["wlan", "connect", &format!("name={ssid}")])
        .output()
        .await?;

    if output.status.success() {
        // wait to ensure connection
        let mut attempts = 0;
        while !NetworkManager::is_connected_to(ssid)? && attempts < 10 {
            attempts += 1;
            sleep_millis(1000);
        }
        Ok(attempts < 10)
    } else {
        Err(output.into())
    }
}

#[tauri::command(async)]
pub fn wlan_start_scanning() {
    get_network_manager();
    log::trace!("Start scanning networks");
    NetworkManager::start_scanning(|list| {
        emit_to_webviews(SeelenEvent::NetworkWlanScanned, &list);
    });
}

#[tauri::command(async)]
pub fn wlan_stop_scanning() {
    get_network_manager();
    log::trace!("Stop scanning networks");
    NetworkManager::stop_scanning();
}

#[tauri::command(async)]
pub async fn wlan_get_profiles() -> Result<Vec<WlanProfile>> {
    get_network_manager();
    NetworkManager::get_wifi_profiles().await
}

#[tauri::command(async)]
pub async fn wlan_connect(ssid: String, password: Option<String>, hidden: bool) -> Result<bool> {
    get_network_manager();
    if let Some(passphrase) = password {
        NetworkManager::add_profile(&ssid, &passphrase, hidden).await?;
    } else {
        let passphrase = String::new();
        NetworkManager::add_profile(&ssid, &passphrase, hidden).await?;
    }

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

#[tauri::command(async)]
pub async fn wlan_disconnect() -> Result<()> {
    get_network_manager();
    NetworkManager::disconnect_all()
}

#[tauri::command(async)]
pub fn get_network_default_local_ip() -> Result<String> {
    get_network_manager();
    get_local_ip_address()
}

#[tauri::command(async)]
pub fn get_network_adapters() -> Result<Vec<seelen_core::system_state::NetworkAdapter>> {
    get_network_manager();
    NetworkManager::get_adapters()
}

#[tauri::command(async)]
pub fn get_network_internet_connection() -> Result<bool> {
    get_network_manager();
    Com::run_with_context(|| {
        let list_manager: INetworkListManager = Com::create_instance(&NetworkListManager)?;
        let connectivity = unsafe { list_manager.GetConnectivity()? };

        let has_internet_ipv4 =
            connectivity.0 & NLM_CONNECTIVITY_IPV4_INTERNET.0 == NLM_CONNECTIVITY_IPV4_INTERNET.0;
        let has_internet_ipv6 =
            connectivity.0 & NLM_CONNECTIVITY_IPV6_INTERNET.0 == NLM_CONNECTIVITY_IPV6_INTERNET.0;

        Ok(has_internet_ipv4 || has_internet_ipv6)
    })
}
