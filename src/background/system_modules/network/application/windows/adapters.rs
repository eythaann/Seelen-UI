use std::net::{Ipv4Addr, Ipv6Addr};

use super::interface_type::InterfaceType;
use itertools::Itertools;
use seelen_core::system_state::{AdapterStatus, NetworkAdapter};
use windows::Win32::{
    Foundation::NO_ERROR,
    NetworkManagement::{
        IpHelper::{GetIfEntry2, IP_ADAPTER_ADDRESSES_LH, MIB_IF_ROW2},
        Ndis::{IfOperStatusUp, NdisPhysicalMediumBluetooth},
    },
    Networking::WinSock::{
        AF_INET, AF_INET6, IpSuffixOriginRandom, SOCKADDR_IN, SOCKADDR_IN6, SOCKET_ADDRESS,
    },
};

use crate::{error::Result, windows_api::string_utils::WindowsString};

/// `MIB_IF_ROW2.InterfaceAndOperStatusFlags` bits.
const IF_FLAG_HARDWARE_INTERFACE: u8 = 1 << 0;
const IF_FLAG_CONNECTOR_PRESENT: u8 = 1 << 2;

pub fn adapter_to_slu_net_adapter(
    adapter: &IP_ADAPTER_ADDRESSES_LH,
    is_default: bool,
) -> Result<NetworkAdapter> {
    unsafe {
        let mac_address = adapter
            .PhysicalAddress
            .iter()
            .take(adapter.PhysicalAddressLength as usize)
            .map(|b| format!("{b:02x}"))
            .join(":");

        let status = if adapter.OperStatus == IfOperStatusUp {
            AdapterStatus::Up
        } else {
            AdapterStatus::Down
        };

        Ok(NetworkAdapter {
            dns_suffix: WindowsString::from(adapter.DnsSuffix).to_string(),
            name: WindowsString::from(adapter.FriendlyName).to_string(),
            description: WindowsString::from(adapter.Description).to_string(),
            mac: mac_address,
            status,
            ipv4: get_ipv4(adapter).map(|ip| ip.to_string()),
            gateway: get_gateway(adapter).map(|ip| ip.to_string()),
            ipv6: get_ipv6(adapter).map(|ip| ip.to_string()),
            is_default,
            is_physical: is_physical(adapter),
            interface_type: InterfaceType::from(adapter.IfType)
                .to_string()
                .replace("IF_TYPE_", ""),
        })
    }
}

/// Hardware NIC with a physical connector. Virtual adapters (Hyper-V/WSL switches, VMware,
/// VirtualBox, VPN/TAP, tunnels) report neither flag. Bluetooth PAN is excluded explicitly
/// as it is exposed as an Ethernet interface.
fn is_physical(adapter: &IP_ADAPTER_ADDRESSES_LH) -> bool {
    let mut row = MIB_IF_ROW2 {
        InterfaceLuid: adapter.Luid,
        ..Default::default()
    };
    if unsafe { GetIfEntry2(&mut row) } != NO_ERROR {
        return false;
    }
    let flags = row.InterfaceAndOperStatusFlags._bitfield;
    let physical = flags & IF_FLAG_HARDWARE_INTERFACE != 0
        && flags & IF_FLAG_CONNECTOR_PRESENT != 0
        && row.PhysicalMediumType != NdisPhysicalMediumBluetooth;
    log::debug!(
        "network adapter {:?}: flags={flags:#010b} medium={} physical={physical}",
        WindowsString::from(adapter.FriendlyName).to_string(),
        row.PhysicalMediumType.0,
    );
    physical
}

unsafe fn sockaddr_to_ipv4(address: &SOCKET_ADDRESS) -> Option<Ipv4Addr> {
    if address.lpSockaddr.is_null() {
        return None;
    }
    let sockaddr = unsafe { &*(address.lpSockaddr as *const SOCKADDR_IN) };
    if sockaddr.sin_family != AF_INET {
        return None;
    }
    // S_addr is stored in network byte order
    Some(Ipv4Addr::from(unsafe {
        sockaddr.sin_addr.S_un.S_addr.to_ne_bytes()
    }))
}

unsafe fn sockaddr_to_ipv6(address: &SOCKET_ADDRESS) -> Option<Ipv6Addr> {
    if address.lpSockaddr.is_null() {
        return None;
    }
    let sockaddr = unsafe { &*(address.lpSockaddr as *const SOCKADDR_IN6) };
    if sockaddr.sin6_family != AF_INET6 {
        return None;
    }
    Some(Ipv6Addr::from(unsafe { sockaddr.sin6_addr.u.Byte }))
}

unsafe fn get_gateway(adapter: &IP_ADAPTER_ADDRESSES_LH) -> Option<Ipv4Addr> {
    let mut ptr = adapter.FirstGatewayAddress;
    while !ptr.is_null() {
        let gateway = unsafe { &*ptr };
        if let Some(ip) = unsafe { sockaddr_to_ipv4(&gateway.Address) } {
            return Some(ip);
        }
        ptr = gateway.Next;
    }
    None
}

unsafe fn get_ipv4(adapter: &IP_ADAPTER_ADDRESSES_LH) -> Option<Ipv4Addr> {
    let mut ptr = adapter.FirstUnicastAddress;
    while !ptr.is_null() {
        let unicast = unsafe { &*ptr };
        if let Some(ip) = unsafe { sockaddr_to_ipv4(&unicast.Address) } {
            return Some(ip);
        }
        ptr = unicast.Next;
    }
    None
}

/// Most meaningful IPv6 of the adapter: a stable global address, then a temporary (privacy)
/// one, and the link-local (`fe80::/10`) only as last resort.
unsafe fn get_ipv6(adapter: &IP_ADAPTER_ADDRESSES_LH) -> Option<Ipv6Addr> {
    let mut best: Option<(u8, Ipv6Addr)> = None;
    let mut ptr = adapter.FirstUnicastAddress;
    while !ptr.is_null() {
        let unicast = unsafe { &*ptr };
        ptr = unicast.Next;

        let Some(ip) = (unsafe { sockaddr_to_ipv6(&unicast.Address) }) else {
            continue;
        };

        let is_link_local = (ip.segments()[0] & 0xffc0) == 0xfe80;
        let rank = if is_link_local {
            2
        } else if unicast.SuffixOrigin == IpSuffixOriginRandom {
            1
        } else {
            0
        };

        if best.is_none_or(|(best_rank, _)| rank < best_rank) {
            best = Some((rank, ip));
        }
    }
    best.map(|(_, ip)| ip)
}
