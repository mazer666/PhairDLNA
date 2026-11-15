//! The address the server is reachable at.
//!
//! What: the local address the daemon binds its base URL to.
//! Why: the device description and every address in a DIDL-Lite document carry
//!       the address a client has to reach the server at. A wrong address makes
//!       the server invisible to every client (`INT-029`, `UX-010`).
//! Example: `local_ipv4` answers `192.168.1.20` on a laptop at home.
//!
//! **How the address is found.** The daemon asks the routing table: it binds a
//! UDP socket to every address of the machine and "connects" it to a reserved
//! address in the documentation range. A UDP socket sends nothing when it is
//! connected — the connection only chooses the route — so the local address of
//! the socket is the address the operating system would send from. No packet
//! leaves the machine and no name server is asked.

use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

/// The address of the documentation range that is used to ask the routing table.
///
/// `192.0.2.0/24` is reserved for documentation (`RFC 5737`), so it can never
/// belong to a real host, and a UDP socket that is connected to it sends
/// nothing.
const ROUTE_PROBE: SocketAddr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1)), 9);

/// The local address the server is reachable at.
///
/// `None` when the machine has no route to the outside world; the caller then
/// falls back to the loopback address and says so in the log.
pub fn local_ipv4() -> Option<Ipv4Addr> {
    let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, 0)).ok()?;
    socket.connect(ROUTE_PROBE).ok()?;
    let address = socket.local_addr().ok()?;
    match address.ip() {
        IpAddr::V4(address) => Some(address),
        IpAddr::V6(_) => None,
    }
}

/// The address the server is reachable at, with the loopback as the fallback.
///
/// A server without a route is still a server: it answers on the loopback, and
/// the log says that the machine has no address on the network.
pub fn local_address_or_loopback() -> Ipv4Addr {
    local_ipv4().unwrap_or(Ipv4Addr::LOCALHOST)
}

#[cfg(test)]
mod tests {
    use super::{local_address_or_loopback, local_ipv4};
    use std::net::Ipv4Addr;

    /// The address is either one the machine has or the loopback.
    #[test]
    fn the_address_is_answered() {
        let address = local_address_or_loopback();
        assert!(!address.is_unspecified());
        if local_ipv4().is_none() {
            assert_eq!(address, Ipv4Addr::LOCALHOST);
        }
    }

    /// The address is never the unspecified address.
    #[test]
    fn the_address_is_never_unspecified() {
        if let Some(address) = local_ipv4() {
            assert!(!address.is_unspecified());
        }
    }
}
