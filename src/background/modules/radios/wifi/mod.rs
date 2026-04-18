pub mod adapter;
pub mod handlers;

use std::{sync::LazyLock, time::Duration};

use seelen_core::system_state::WlanBssEntry;
use windows::{
    Devices::WiFi::{WiFiAccessStatus, WiFiAdapter, WiFiConnectionStatus, WiFiReconnectionKind},
    Security::Credentials::PasswordCredential,
    Win32::{
        Foundation::HANDLE,
        NetworkManagement::WiFi::{
            WlanCloseHandle, WlanDeleteProfile, WlanEnumInterfaces, WlanOpenHandle, WlanScan,
            DOT11_SSID, WLAN_API_VERSION_2_0, WLAN_INTERFACE_INFO, WLAN_INTERFACE_INFO_LIST,
        },
    },
};
use windows_core::HSTRING;

use crate::{
    error::{Result, ResultLogExt},
    event_manager, log_error,
    utils::lock_free::SyncHashMap,
    windows_api::{string_utils::WindowsString, DeviceEnumerator, DeviceEvent, DeviceId},
};

use adapter::{wifi_known_profiles, SluWifiAdapter};

// ── Win32 WLAN handle (only used for directed probe on hidden networks) ──────

struct WlanHandle(HANDLE);

impl Drop for WlanHandle {
    fn drop(&mut self) {
        unsafe { WlanCloseHandle(self.0, None) };
    }
}

impl std::ops::Deref for WlanHandle {
    type Target = HANDLE;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

fn open_wlan() -> Result<WlanHandle> {
    let mut handle = HANDLE::default();
    let mut version = 0u32;
    let err = unsafe { WlanOpenHandle(WLAN_API_VERSION_2_0, None, &mut version, &mut handle) };
    if err != 0 {
        return Err(format!("WlanOpenHandle failed: {err}").into());
    }
    Ok(WlanHandle(handle))
}

fn get_wlan_interfaces(client: HANDLE) -> Result<Vec<WLAN_INTERFACE_INFO>> {
    unsafe {
        let mut ptr: *mut WLAN_INTERFACE_INFO_LIST = std::ptr::null_mut();
        let err = WlanEnumInterfaces(client, None, &mut ptr);
        if err != 0 || ptr.is_null() {
            return Err(format!("WlanEnumInterfaces failed: {err}").into());
        }
        let list = &*ptr;
        Ok(
            std::slice::from_raw_parts(list.InterfaceInfo.as_ptr(), list.dwNumberOfItems as usize)
                .to_vec(),
        )
    }
}

// ── Async helpers ─────────────────────────────────────────────────────────────

fn join_timed<T>(
    op: impl FnOnce() -> windows_core::Result<T> + Send + 'static,
    timeout: Duration,
) -> Result<Option<T>>
where
    T: Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let _ = tx.send(op());
    });
    match rx.recv_timeout(timeout) {
        Ok(Ok(v)) => Ok(Some(v)),
        Ok(Err(e)) => Err(e.into()),
        Err(_) => Ok(None),
    }
}

const CONNECT_TIMEOUT: Duration = Duration::from_secs(15);

// ── WifiManager ──────────────────────────────────────────────────────────────

#[derive(Debug, Clone)]
pub enum WifiManagerEvent {
    NetworksChanged,
}

pub struct WifiManager {
    pub adapters: SyncHashMap<DeviceId, SluWifiAdapter>,
    device_enumerator: Option<DeviceEnumerator>,
}

unsafe impl Send for WifiManager {}
unsafe impl Sync for WifiManager {}

event_manager!(WifiManager, WifiManagerEvent);

impl WifiManager {
    fn new() -> Self {
        Self {
            adapters: SyncHashMap::new(),
            device_enumerator: None,
        }
    }

    fn init(&mut self) -> Result<()> {
        let selector = WiFiAdapter::GetDeviceSelector()?.to_string();

        let enumerator = DeviceEnumerator::new(selector, |event| {
            log_error!(WifiManager::instance().on_device_event(event));
        })?;

        let devices = enumerator.start_blocking()?;

        let adapters: std::collections::HashMap<DeviceId, SluWifiAdapter> = devices
            .iter()
            .filter_map(|info| {
                let id = info.Id().ok()?.to_string();
                SluWifiAdapter::create(&id)
                    .map_err(|e| log::error!("Failed to create SluWifiAdapter {id}: {e}"))
                    .ok()
                    .map(|a| (id, a))
            })
            .collect();

        self.adapters = SyncHashMap::from(adapters);
        self.device_enumerator = Some(enumerator);
        Ok(())
    }

    pub fn instance() -> &'static Self {
        static INSTANCE: LazyLock<WifiManager> = LazyLock::new(|| {
            let mut m = WifiManager::new();
            m.init().log_error();
            m
        });
        &INSTANCE
    }

    fn on_device_event(&self, event: DeviceEvent) -> Result<()> {
        match event {
            DeviceEvent::Added(id) => {
                let adapter = SluWifiAdapter::create(&id)?;
                self.adapters.upsert(id, adapter);
            }
            DeviceEvent::Removed(id) => {
                self.adapters.remove(&id);
            }
            DeviceEvent::Updated(_) => {}
        }
        Ok(())
    }

    /// Trigger a hardware scan on every cached adapter.
    /// Results are delivered asynchronously via `WifiManagerEvent::NetworksChanged`.
    pub fn scan_networks(&self) {
        self.adapters.for_each(|(_, adapter)| adapter.scan());
    }

    /// Read the current `NetworkReport` from all cached adapters.
    pub fn get_available_networks(&self) -> Result<Vec<WlanBssEntry>> {
        let known = wifi_known_profiles();
        let mut entries = Vec::new();
        self.adapters.for_each(
            |(_, adapter)| match adapter.get_available_networks(&known) {
                Ok(mut list) => entries.append(&mut list),
                Err(e) => log::error!("get_available_networks error: {e}"),
            },
        );
        Ok(entries)
    }

    /// Send a directed 802.11 probe for a specific SSID so a hidden AP responds.
    fn directed_probe(&self, ssid: &str) -> Result<()> {
        let handle = open_wlan()?;
        let ifaces = get_wlan_interfaces(*handle)?;

        let bytes = ssid.as_bytes();
        let len = bytes.len().min(32);
        let mut dot11 = DOT11_SSID {
            uSSIDLength: len as u32,
            ucSSID: [0; 32],
        };
        dot11.ucSSID[..len].copy_from_slice(&bytes[..len]);

        for iface in &ifaces {
            unsafe { WlanScan(*handle, &iface.InterfaceGuid, Some(&dot11), None, None) };
        }
        Ok(())
    }

    pub fn connect(&self, ssid: &str, password: Option<&str>, hidden: bool) -> Result<bool> {
        if hidden {
            let _ = self.directed_probe(ssid);
            std::thread::sleep(Duration::from_secs(2));
        }

        let access = WiFiAdapter::RequestAccessAsync()?.join()?;
        if access != WiFiAccessStatus::Allowed {
            return Err("WiFi access denied by the system".into());
        }

        let adapters = WiFiAdapter::FindAllAdaptersAsync()?.join()?;
        for adapter in &adapters {
            adapter.ScanAsync()?.join()?;

            let networks = adapter.NetworkReport()?.AvailableNetworks()?;
            for net in &networks {
                let net_ssid = net.Ssid()?.to_string();
                let matches = if hidden {
                    net_ssid == ssid || net_ssid.is_empty()
                } else {
                    net_ssid == ssid
                };
                if !matches {
                    continue;
                }

                let op_result = match (hidden, password.filter(|p| !p.is_empty())) {
                    (true, Some(pwd)) => {
                        let cred = PasswordCredential::new()?;
                        cred.SetPassword(&HSTRING::from(pwd))?;
                        let op = adapter.ConnectWithPasswordCredentialAndSsidAsync(
                            &net,
                            WiFiReconnectionKind::Automatic,
                            &cred,
                            &HSTRING::from(ssid),
                        )?;
                        join_timed(move || op.join(), CONNECT_TIMEOUT)?
                    }
                    (true, None) => {
                        let op = adapter.ConnectWithPasswordCredentialAndSsidAsync(
                            &net,
                            WiFiReconnectionKind::Automatic,
                            &PasswordCredential::new()?,
                            &HSTRING::from(ssid),
                        )?;
                        join_timed(move || op.join(), CONNECT_TIMEOUT)?
                    }
                    (false, Some(pwd)) => {
                        let cred = PasswordCredential::new()?;
                        cred.SetPassword(&HSTRING::from(pwd))?;
                        let op = adapter.ConnectWithPasswordCredentialAsync(
                            &net,
                            WiFiReconnectionKind::Automatic,
                            &cred,
                        )?;
                        join_timed(move || op.join(), CONNECT_TIMEOUT)?
                    }
                    (false, None) => {
                        let op = adapter.ConnectAsync(&net, WiFiReconnectionKind::Automatic)?;
                        join_timed(move || op.join(), CONNECT_TIMEOUT)?
                    }
                };

                let Some(result) = op_result else {
                    return Ok(false);
                };

                let status = result.ConnectionStatus()?;
                if status == WiFiConnectionStatus::Success {
                    return Ok(true);
                }
                if hidden && status == WiFiConnectionStatus::InvalidCredential {
                    return Ok(false);
                }
                if !hidden {
                    return Ok(false);
                }
            }
        }

        Ok(false)
    }

    /// Delete the saved WLAN profile named `ssid` from every interface that has it.
    pub fn forget(&self, ssid: &str) -> Result<()> {
        let handle = open_wlan()?;
        let ifaces = get_wlan_interfaces(*handle)?;

        let profile_name = WindowsString::from_str(ssid);

        let mut last_err: Option<u32> = None;
        let mut deleted_any = false;
        for iface in &ifaces {
            let err = unsafe {
                WlanDeleteProfile(
                    *handle,
                    &iface.InterfaceGuid,
                    profile_name.as_pcwstr(),
                    None,
                )
            };
            if err == 0 {
                deleted_any = true;
            } else {
                last_err = Some(err);
            }
        }

        if !deleted_any {
            if let Some(err) = last_err {
                return Err(format!("WlanDeleteProfile failed: {err}").into());
            }
        }
        Ok(())
    }

    pub fn disconnect(&self) -> Result<()> {
        let access = WiFiAdapter::RequestAccessAsync()?.join()?;
        if access != WiFiAccessStatus::Allowed {
            return Ok(());
        }
        let adapters = WiFiAdapter::FindAllAdaptersAsync()?.join()?;
        for adapter in &adapters {
            adapter.Disconnect()?;
        }
        Ok(())
    }
}
