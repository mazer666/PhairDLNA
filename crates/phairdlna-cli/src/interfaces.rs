//! The interfaces command (`OPS-004`).
//!
//! What: lists known network interfaces and evaluates which ones would be used
//!       for SSDP discovery, used for media or excluded, naming reasons for any exclusion.
//! Why: lets users and scripts verify and troubleshoot network exposure without starting
//!      the server daemon.

use std::net::IpAddr;
use std::path::Path;

use phairdlna_core::config::Config;
use phairdlna_core::platform::{
    select_interfaces, InterfaceDecision, LocalNetworkStatus, NetworkInterface,
};

use crate::serve::address;
use crate::{resolve, Outcome};

/// Discovers known platform interfaces without external dependencies.
pub fn detect_known_interfaces() -> Vec<NetworkInterface> {
    let mut interfaces = Vec::new();

    interfaces.push(NetworkInterface {
        name: "lo0".to_owned(),
        addresses: vec![IpAddr::V4(std::net::Ipv4Addr::LOCALHOST)],
        supports_multicast: false,
        is_local: true,
    });

    if let Some(ip) = address::local_ipv4() {
        if !ip.is_loopback() {
            let is_local = ip.is_private() || ip.is_link_local();
            interfaces.push(NetworkInterface {
                name: "default".to_owned(),
                addresses: vec![IpAddr::V4(ip)],
                supports_multicast: true,
                is_local,
            });
        }
    }

    interfaces
}

/// Formats the interface decisions as JSON.
fn render_json(decisions: &[InterfaceDecision]) -> String {
    let mut report = String::from("[\n");
    for (i, d) in decisions.iter().enumerate() {
        let addrs: Vec<String> = d
            .interface
            .addresses
            .iter()
            .map(|a| format!("\"{a}\""))
            .collect();
        let reasons: Vec<String> = d
            .exclusion_reasons
            .iter()
            .map(|r| format!("\"{r}\""))
            .collect();
        report.push_str(&format!(
            "  {{\n    \"name\": \"{}\",\n    \"addresses\": [{}],\n    \"supports_multicast\": {},\n    \"is_local\": {},\n    \"used_for_discovery\": {},\n    \"used_for_media\": {},\n    \"exclusion_reasons\": [{}]\n  }}",
            d.interface.name,
            addrs.join(", "),
            d.interface.supports_multicast,
            d.interface.is_local,
            d.used_for_discovery,
            d.used_for_media,
            reasons.join(", ")
        ));
        if i + 1 < decisions.len() {
            report.push_str(",\n");
        } else {
            report.push('\n');
        }
    }
    report.push_str("]\n");
    report
}

/// Formats a single interface decision as text.
fn render_text_entry(out: &mut String, d: &InterfaceDecision) {
    let role = match (d.used_for_discovery, d.used_for_media) {
        (true, true) => "used for discovery and media",
        (true, false) => "used for discovery",
        (false, true) => "used for media",
        (false, false) => "excluded",
    };
    let addrs: Vec<String> = d
        .interface
        .addresses
        .iter()
        .map(|a| a.to_string())
        .collect();
    out.push_str(&format!(
        "Interface \"{}\":\n  Addresses:          {}\n  Multicast:          {}\n  Local:              {}\n  Role:               {}\n",
        d.interface.name,
        addrs.join(", "),
        if d.interface.supports_multicast { "yes" } else { "no" },
        if d.interface.is_local { "yes" } else { "no" },
        role
    ));
    if !d.exclusion_reasons.is_empty() {
        out.push_str("  Exclusion Reasons:\n");
        for reason in &d.exclusion_reasons {
            out.push_str(&format!("    - {reason}\n"));
        }
    }
    out.push('\n');
}

/// Lists known network interfaces and their role decisions.
pub fn interfaces(path: Option<&Path>, json: bool) -> Outcome {
    let path = resolve(path);
    let config = match Config::load(&path) {
        Ok(config) => config,
        Err(error) => {
            return Outcome::failure(format!(
                "the configuration \"{}\" is not usable:\n{}\n",
                path.display(),
                error.message()
            ))
        }
    };

    let known = detect_known_interfaces();
    let decisions = select_interfaces(&known, &config, LocalNetworkStatus::Unknown);

    if json {
        return Outcome::ok(render_json(&decisions));
    }

    let mut out = String::from("Network Interfaces Evaluation (OPS-004):\n\n");
    for d in &decisions {
        render_text_entry(&mut out, d);
    }

    Outcome::ok(out)
}
