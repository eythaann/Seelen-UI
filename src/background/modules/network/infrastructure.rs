use std::sync::Once;

use seelen_core::handlers::SeelenEvent;
use windows::Win32::Networking::NetworkListManager::{
    NLM_CONNECTIVITY_IPV4_INTERNET, NLM_CONNECTIVITY_IPV6_INTERNET,
};

use crate::{app::emit_to_webviews, error::Result};

use super::application::{get_local_ip_address, NetworkManager, NetworkManagerEvent};

fn get_network_manager() -> &'static NetworkManager {
    static TAURI_EVENT_REGISTRATION: Once = Once::new();
    TAURI_EVENT_REGISTRATION.call_once(|| {
        NetworkManager::subscribe(|event| match event {
            NetworkManagerEvent::AdaptersChanged => {
                if let Ok(adapters) = NetworkManager::get_adapters() {
                    emit_to_webviews(SeelenEvent::NetworkAdapters, adapters);
                }
            }
            NetworkManagerEvent::ConnectivityChanged { connectivity, ip } => {
                log::trace!(target: "network", "Connectivity changed: {connectivity:?}");
                let has_internet = connectivity.0 & NLM_CONNECTIVITY_IPV4_INTERNET.0
                    == NLM_CONNECTIVITY_IPV4_INTERNET.0
                    || connectivity.0 & NLM_CONNECTIVITY_IPV6_INTERNET.0
                        == NLM_CONNECTIVITY_IPV6_INTERNET.0;
                emit_to_webviews(SeelenEvent::NetworkDefaultLocalIp, ip);
                emit_to_webviews(SeelenEvent::NetworkInternetConnection, has_internet);
            }
        });
    });
    NetworkManager::instance()
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
    use crate::windows_api::Com;
    use windows::Win32::Networking::NetworkListManager::{INetworkListManager, NetworkListManager};
    Com::run_with_context(|| {
        let list_manager: INetworkListManager = Com::create_instance(&NetworkListManager)?;
        let connectivity = unsafe { list_manager.GetConnectivity()? };
        Ok(
            connectivity.0 & NLM_CONNECTIVITY_IPV4_INTERNET.0 == NLM_CONNECTIVITY_IPV4_INTERNET.0
                || connectivity.0 & NLM_CONNECTIVITY_IPV6_INTERNET.0
                    == NLM_CONNECTIVITY_IPV6_INTERNET.0,
        )
    })
}
