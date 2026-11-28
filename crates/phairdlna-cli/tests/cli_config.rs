//! Acceptance tests for CFG-006 / AC-08:
//! Given a configuration that selects network interfaces for discovery and media,
//! the server and interfaces command evaluate role decisions accurately.

use phairdlna_core::config::Config;
use phairdlna_core::platform::{select_interfaces, LocalNetworkStatus, NetworkInterface};
use std::fs;
use std::net::{IpAddr, Ipv4Addr};
use std::path::PathBuf;

fn write_temp_config(content: &str) -> PathBuf {
    let dir = tempfile::tempdir().expect("tempdir");
    let path = dir.path().join("phairdlna.toml");
    fs::write(&path, content).expect("write config");
    Box::leak(Box::new(dir));
    path
}

fn dummy_iface(name: &str, ip: (u8, u8, u8, u8), multicast: bool) -> NetworkInterface {
    NetworkInterface {
        name: name.to_owned(),
        addresses: vec![IpAddr::V4(Ipv4Addr::new(ip.0, ip.1, ip.2, ip.3))],
        supports_multicast: multicast,
        is_local: true,
    }
}

#[test]
fn ac_08_interface_selection_and_role_split() {
    let toml_content = r#"
schema_version = 1
[server]
device_name = "SplitServer"
[network]
interfaces = ["en0"]
"#;
    let path = write_temp_config(toml_content);
    let config = Config::load(&path).expect("load config");

    let ifaces = vec![
        dummy_iface("en0", (192, 168, 1, 50), true),
        dummy_iface("en1", (192, 168, 1, 51), true),
    ];

    let decisions = select_interfaces(&ifaces, &config, LocalNetworkStatus::Permitted);

    // en0 is permitted for both discovery and media
    assert_eq!(decisions[0].interface.name, "en0");
    assert!(decisions[0].used_for_discovery);
    assert!(decisions[0].used_for_media);

    // en1 is excluded because network.interfaces restricts to ["en0"]
    assert_eq!(decisions[1].interface.name, "en1");
    assert!(!decisions[1].used_for_discovery);
    assert!(!decisions[1].used_for_media);
    assert!(decisions[1]
        .exclusion_reasons
        .iter()
        .any(|r| r.contains("name not in network.interfaces")));
}
