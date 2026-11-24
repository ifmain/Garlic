//! Address selection for a Garlic exit. A development target is confined to loopback.
use garlic_relay_data::control::valid_connect_authority;
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};

pub fn target_allowed(ip: IpAddr, loopback_fixture: bool) -> bool {
    if loopback_fixture {
        return ip.is_loopback();
    }
    match ip {
        IpAddr::V4(ip) => {
            let [a, b, _, _] = ip.octets();
            !ip.is_private()
                && !ip.is_loopback()
                && !ip.is_link_local()
                && !ip.is_documentation()
                && a != 0
                && a < 224
                && !(a == 100 && (64..128).contains(&b))
                && !(a == 198 && (b == 18 || b == 19))
        }
        IpAddr::V6(ip) => {
            let segments = ip.segments();
            segments[0] & 0xe000 == 0x2000
                && segments[0] != 0x2002
                && !(segments[0] == 0x2001 && (segments[1] < 0x0200 || segments[1] == 0x0db8))
                && !(segments[0] == 0x3fff && segments[1] <= 0x0fff)
        }
    }
}

pub fn select(authority: &str, loopback_fixture: bool) -> Option<SocketAddr> {
    if !valid_connect_authority(authority) {
        return None;
    }
    // Use the resolved address directly when opening the socket.
    let mut addresses: Vec<_> = authority
        .to_socket_addrs()
        .ok()?
        .filter(|addr| target_allowed(addr.ip(), loopback_fixture))
        .collect();
    addresses.sort_by_key(|addr| addr.is_ipv6());
    addresses.into_iter().next()
}
