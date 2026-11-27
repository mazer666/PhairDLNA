//! Network interface decision and selection rules.
//!
//! What: evaluates how network interfaces are used according to configuration
//!       and platform state (`CFG-006`, `INT-009`, `OPS-004`).
//! Why: ensures truthful and consistent interface categorization across the daemon
//!      and the CLI commands without duplicating policy logic.

use crate::config::Config;
use crate::platform::{LocalNetworkStatus, NetworkInterface};

/// The role an interface plays for the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterfaceRole {
    /// Used for SSDP multicast discovery and advertisements.
    Discovery,
    /// Used for HTTP media delivery.
    Media,
    /// Excluded from server operations.
    Excluded,
}

/// The decision made for one interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InterfaceDecision {
    /// The evaluated interface.
    pub interface: NetworkInterface,
    /// Whether the interface is used for SSDP discovery.
    pub used_for_discovery: bool,
    /// Whether the interface is used for HTTP media serving.
    pub used_for_media: bool,
    /// If excluded from any role, the explicit human-readable reasons.
    pub exclusion_reasons: Vec<String>,
}

impl InterfaceDecision {
    /// Evaluates policy for one interface given configuration and local-network status.
    pub fn evaluate(
        interface: &NetworkInterface,
        config: &Config,
        network_status: LocalNetworkStatus,
    ) -> Self {
        let mut exclusion_reasons = Vec::new();
        let configured = &config.network.interfaces;

        if network_status == LocalNetworkStatus::Refused {
            exclusion_reasons.push("local network permission refused".to_owned());
            return Self {
                interface: interface.clone(),
                used_for_discovery: false,
                used_for_media: false,
                exclusion_reasons,
            };
        }

        let name_permitted = configured.is_empty() || configured.contains(&interface.name);
        if !name_permitted {
            exclusion_reasons
                .push("excluded by configuration (name not in network.interfaces)".to_owned());
        }

        let address_permitted = interface.is_local || config.network.allow_non_local_addresses;
        if !address_permitted {
            exclusion_reasons
                .push("not a local address and allow_non_local_addresses is false".to_owned());
        }

        let base_permitted = name_permitted && address_permitted;

        let used_for_discovery = if !config.server.discovery_enabled {
            exclusion_reasons.push("discovery disabled in configuration".to_owned());
            false
        } else if !interface.supports_multicast {
            exclusion_reasons.push("interface does not support multicast".to_owned());
            false
        } else {
            base_permitted
        };

        let used_for_media = base_permitted;

        Self {
            interface: interface.clone(),
            used_for_discovery,
            used_for_media,
            exclusion_reasons,
        }
    }
}

/// Evaluates decisions for a list of interfaces.
pub fn select_interfaces(
    interfaces: &[NetworkInterface],
    config: &Config,
    network_status: LocalNetworkStatus,
) -> Vec<InterfaceDecision> {
    interfaces
        .iter()
        .map(|iface| InterfaceDecision::evaluate(iface, config, network_status))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    fn test_interface(name: &str, is_local: bool, multicast: bool) -> NetworkInterface {
        NetworkInterface {
            name: name.to_owned(),
            addresses: vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, 10))],
            supports_multicast: multicast,
            is_local,
        }
    }

    #[test]
    fn default_config_accepts_local_multicast_interface() {
        let config = Config::default();
        let iface = test_interface("en0", true, true);
        let decision = InterfaceDecision::evaluate(&iface, &config, LocalNetworkStatus::Permitted);
        assert!(decision.used_for_discovery);
        assert!(decision.used_for_media);
        assert!(decision.exclusion_reasons.is_empty());
    }

    #[test]
    fn configured_interface_name_filters_others() {
        let mut config = Config::default();
        config.network.interfaces = vec!["en0".to_owned()];

        let iface1 = test_interface("en0", true, true);
        let iface2 = test_interface("en1", true, true);

        let d1 = InterfaceDecision::evaluate(&iface1, &config, LocalNetworkStatus::Unknown);
        let d2 = InterfaceDecision::evaluate(&iface2, &config, LocalNetworkStatus::Unknown);

        assert!(d1.used_for_discovery);
        assert!(!d2.used_for_discovery);
        assert!(d2
            .exclusion_reasons
            .iter()
            .any(|r| r.contains("name not in network.interfaces")));
    }

    #[test]
    fn non_multicast_interface_excluded_from_discovery_only() {
        let config = Config::default();
        let iface = test_interface("en0", true, false);
        let decision = InterfaceDecision::evaluate(&iface, &config, LocalNetworkStatus::Permitted);
        assert!(!decision.used_for_discovery);
        assert!(decision.used_for_media);
        assert!(decision
            .exclusion_reasons
            .iter()
            .any(|r| r.contains("does not support multicast")));
    }

    #[test]
    fn local_network_status_refused_excludes_all() {
        let config = Config::default();
        let iface = test_interface("en0", true, true);
        let decision = InterfaceDecision::evaluate(&iface, &config, LocalNetworkStatus::Refused);
        assert!(!decision.used_for_discovery);
        assert!(!decision.used_for_media);
        assert!(decision
            .exclusion_reasons
            .iter()
            .any(|r| r.contains("refused")));
    }
}
