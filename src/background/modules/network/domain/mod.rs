pub mod types;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use types::InterfaceType;
use windows::Win32::{
    NetworkManagement::{IpHelper::IP_ADAPTER_ADDRESSES_LH, Ndis::IfOperStatusUp},
    Networking::WinSock::{inet_ntop, AF_INET, AF_INET6, SOCKADDR_IN, SOCKADDR_IN6},
};

use crate::error_handler::{AppError, Result};

#[derive(Debug, Clone, Serialize)]
pub enum AdapterStatus {
    Up,
    Down,
}

#[derive(Debug, Clone, Serialize)]
pub struct NetworkAdapter {
    // General information
    name: String,
    description: String,
    status: AdapterStatus,
    dns_suffix: String,
    #[serde(rename = "type")]
    interface_type: String,
    // Address information
    ipv6: Option<String>,
    ipv4: Option<String>,
    gateway: Option<String>,
    mac: String,
}

#[derive(PartialEq, Eq)]
enum Address {
    Ipv4,
    Ipv6,
    Gateway,
}

unsafe fn get_gateway(adapter: &IP_ADAPTER_ADDRESSES_LH) -> Option<String> {
    let mut gateway_ptr = adapter.FirstGatewayAddress;
    while !gateway_ptr.is_null() {
        let gateway = &*gateway_ptr;

        if gateway.Address.lpSockaddr.is_null() {
            gateway_ptr = gateway.Next;
            continue;
        }

        let sockaddr = &*(gateway.Address.lpSockaddr as *const SOCKADDR_IN);
        if sockaddr.sin_family == AF_INET {
            let mut string_buffer = [0u8; 16];
            return inet_ntop(
                AF_INET.0 as i32,
                &sockaddr.sin_addr as *const _ as _,
                &mut string_buffer,
            )
            .to_string()
            .ok();
        }

        gateway_ptr = gateway.Next;
    }
    None
}

unsafe fn get_address(adapter: &IP_ADAPTER_ADDRESSES_LH, address: Address) -> Option<String> {
    if address == Address::Gateway {
        return get_gateway(adapter);
    }

    let mut unicast_ptr = adapter.FirstUnicastAddress;

    while !unicast_ptr.is_null() {
        let unicast = &*unicast_ptr;

        if unicast.Address.lpSockaddr.is_null() {
            unicast_ptr = unicast.Next;
            continue;
        }

        let sockaddr = &*(unicast.Address.lpSockaddr as *const SOCKADDR_IN);
        if address == Address::Ipv4 && sockaddr.sin_family == AF_INET {
            let mut string_buffer = [0u8; 16];
            return inet_ntop(
                AF_INET.0 as i32,
                &sockaddr.sin_addr as *const _ as _,
                &mut string_buffer,
            )
            .to_string()
            .ok();
        }

        let sockaddr = &*(unicast.Address.lpSockaddr as *const SOCKADDR_IN6);
        if address == Address::Ipv6 && sockaddr.sin6_family == AF_INET6 {
            let mut string_buffer = [0u8; 46];
            return inet_ntop(
                AF_INET6.0 as i32,
                &sockaddr.sin6_addr as *const _ as _,
                &mut string_buffer,
            )
            .to_string()
            .ok();
        }

        unicast_ptr = unicast.Next;
    }

    None
}

impl TryFrom<&IP_ADAPTER_ADDRESSES_LH> for NetworkAdapter {
    type Error = AppError;
    fn try_from(adapter: &IP_ADAPTER_ADDRESSES_LH) -> Result<Self> {
        unsafe {
            let mac_address = adapter
                .PhysicalAddress
                .iter()
                .map(|b| format!("{:02x}", b))
                .join(":");

            let status = if adapter.OperStatus == IfOperStatusUp {
                AdapterStatus::Up
            } else {
                AdapterStatus::Down
            };

            Ok(Self {
                dns_suffix: adapter.DnsSuffix.to_string()?,
                name: adapter.FriendlyName.to_string()?,
                description: adapter.Description.to_string()?,
                mac: mac_address,
                status,
                ipv4: get_address(adapter, Address::Ipv4),
                gateway: get_address(adapter, Address::Gateway),
                ipv6: get_address(adapter, Address::Ipv6),
                interface_type: InterfaceType::from(adapter.IfType)
                    .to_string()
                    .replace("IF_TYPE_", ""),
            })
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all(serialize = "camelCase", deserialize = "PascalCase"))]
pub struct WlanProfile {
    profile_name: String,
    #[serde(rename(deserialize = "SSID"))]
    ssid: String,
    authentication: String,
    encryption: String,
    password: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WlanBssEntry {
    pub ssid: Option<String>,
    pub bssid: String,
    pub channel_frequency: u32,
    pub signal: u32,
    /// true if the interface is connected to this network
    pub connected: bool,
    /// true if the interface is connected to this network and is using this channel frequency
    pub connected_channel: bool,
}
