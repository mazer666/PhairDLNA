//! Interface routing and rate limiting for SSDP discovery (`INT-006`, `INT-009`, `INT-011`).
//!
//! What: selects which interface responds to a search request, generates interface-bound
//!       `LOCATION` headers, checks exclusion policies, and rate-limits requesters.
//! Why: multi-homed hosts must answer SSDP requests with a `LOCATION` reachable on the
//!      request's receiving interface (`INT-006`), never respond on excluded interfaces (`INT-009`),
//!      and bound responses per source address (`INT-011`).

use std::collections::HashMap;
use std::net::IpAddr;
use std::time::Instant;

use crate::config::Config;
use crate::platform::{select_interfaces, InterfaceDecision, LocalNetworkStatus, NetworkInterface};
use crate::ssdp::location_url;

/// The maximum number of discovery responses permitted per source address within a window (`INT-011`).
pub const RATE_LIMIT_MAX_REQUESTS: usize = 30;

/// The window duration for rate limiting in seconds (`INT-011`).
pub const RATE_LIMIT_WINDOW_SECS: u64 = 5;

/// Rate limiter for discovery requests per source IP (`INT-011`).
#[derive(Debug, Default)]
pub struct DiscoveryRateLimiter {
    history: HashMap<IpAddr, Vec<Instant>>,
}

impl DiscoveryRateLimiter {
    /// Creates a new rate limiter.
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
        }
    }

    /// Checks if a request from the given source IP is permitted.
    pub fn check_and_record(&mut self, source_ip: IpAddr, now: Instant) -> bool {
        let window = std::time::Duration::from_secs(RATE_LIMIT_WINDOW_SECS);
        let timestamps = self.history.entry(source_ip).or_default();
        timestamps.retain(|&t| now.duration_since(t) <= window);

        if timestamps.len() >= RATE_LIMIT_MAX_REQUESTS {
            return false;
        }

        timestamps.push(now);
        true
    }
}

/// Helper that resolves the appropriate location URL for an incoming request on multi-homed hosts.
#[derive(Debug, Clone)]
pub struct InterfaceRouter {
    decisions: Vec<InterfaceDecision>,
    port: u16,
}

impl InterfaceRouter {
    /// Builds an interface router from active interfaces and configuration.
    pub fn new(
        interfaces: &[NetworkInterface],
        config: &Config,
        status: LocalNetworkStatus,
    ) -> Self {
        Self {
            decisions: select_interfaces(interfaces, config, status),
            port: config.server.http_port,
        }
    }

    /// Determines if an incoming request from a local socket address/interface is permitted (`INT-009`).
    pub fn is_interface_permitted(&self, interface_name: &str) -> bool {
        self.decisions
            .iter()
            .any(|d| d.interface.name == interface_name && d.used_for_discovery)
    }

    /// Resolves the absolute LOCATION URL for an interface or IP (`INT-006`).
    pub fn location_for(&self, receiving_ip: &str) -> String {
        location_url(receiving_ip, self.port)
    }

    /// Resolves the decision for an interface name.
    pub fn decision_for(&self, interface_name: &str) -> Option<&InterfaceDecision> {
        self.decisions
            .iter()
            .find(|d| d.interface.name == interface_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::Ipv4Addr;

    fn make_iface(name: &str, ip: Ipv4Addr, multicast: bool, local: bool) -> NetworkInterface {
        NetworkInterface {
            name: name.to_owned(),
            addresses: vec![IpAddr::V4(ip)],
            supports_multicast: multicast,
            is_local: local,
        }
    }

    #[test]
    fn location_is_generated_per_receiving_ip() {
        let mut config = Config::default();
        config.server.http_port = 8200;
        let ifaces = vec![
            make_iface("en0", Ipv4Addr::new(192, 168, 1, 50), true, true),
            make_iface("en1", Ipv4Addr::new(10, 0, 0, 20), true, true),
        ];

        let router = InterfaceRouter::new(&ifaces, &config, LocalNetworkStatus::Permitted);
        assert_eq!(
            router.location_for("192.168.1.50"),
            "http://192.168.1.50:8200/description.xml"
        );
        assert_eq!(
            router.location_for("10.0.0.20"),
            "http://10.0.0.20:8200/description.xml"
        );
    }

    #[test]
    fn excluded_interface_is_rejected() {
        let mut config = Config::default();
        config.network.interfaces = vec!["en0".to_owned()];
        let ifaces = vec![
            make_iface("en0", Ipv4Addr::new(192, 168, 1, 50), true, true),
            make_iface("en1", Ipv4Addr::new(10, 0, 0, 20), true, true),
        ];

        let router = InterfaceRouter::new(&ifaces, &config, LocalNetworkStatus::Permitted);
        assert!(router.is_interface_permitted("en0"));
        assert!(!router.is_interface_permitted("en1"));
    }

    #[test]
    fn rate_limiter_bounds_burst_requests() {
        let mut limiter = DiscoveryRateLimiter::new();
        let ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 100));
        let now = Instant::now();

        for _ in 0..RATE_LIMIT_MAX_REQUESTS {
            assert!(limiter.check_and_record(ip, now));
        }

        // Exceeded bound
        assert!(!limiter.check_and_record(ip, now));
    }
}
