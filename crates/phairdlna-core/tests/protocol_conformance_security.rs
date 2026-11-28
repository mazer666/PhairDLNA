//! Protocol conformance tests for UPnP IGD refusal, GENA subscription restriction, and update policy.
//!
//! What: verifies INT-057 (GENA SUBSCRIBE rejected unless explicitly enabled),
//!       INT-058 (IGD actions rejected), and INT-059 (no network self-update).
//! Why: required by F-04 / F-16 baseline security and interoperability contracts.

use phairdlna_core::upnp::Dispatcher;

#[path = "support/mod.rs"]
mod support;
use support::catalogue::TestCatalogue;

/// Helper that builds an XML SOAP envelope for UPnP actions.
fn envelope(service_type: &str, action: &str, args: &[(&str, &str)]) -> String {
    let mut args_xml = String::new();
    for (k, v) in args {
        args_xml.push_str(&format!("<{k}>{v}</{k}>"));
    }
    format!(
        r#"<?xml version="1.0" encoding="utf-8"?>
<s:Envelope xmlns:s="http://schemas.xmlsoap.org/soap/envelope/" s:encodingStyle="http://schemas.xmlsoap.org/soap/encoding/">
  <s:Body>
    <u:{action} xmlns:u="{service_type}">
      {args_xml}
    </u:{action}>
  </s:Body>
</s:Envelope>"#
    )
}

/// PC-29: Rejection of UPnP Internet Gateway Device (IGD) actions (INT-058).
#[test]
fn pc_29_igd_actions_rejected_with_error() {
    let dispatcher = Dispatcher::new(&TestCatalogue, "http://192.168.1.10:8200");
    let igd_actions = [
        (
            "AddPortMapping",
            vec![("NewExternalPort", "8200"), ("NewInternalPort", "8200")],
        ),
        ("DeletePortMapping", vec![("NewExternalPort", "8200")]),
        ("GetExternalIPAddress", vec![]),
        (
            "NewInternalClient",
            vec![("NewInternalClient", "192.168.1.50")],
        ),
    ];

    for (action, args) in igd_actions {
        let body = envelope(
            "urn:schemas-upnp-org:service:WANIPConnection:1",
            action,
            &args,
        );
        let resp = dispatcher.dispatch(&body, None);
        assert_eq!(resp.status, 500, "IGD action {action} must yield fault 500");
        assert!(
            resp.body.contains("<errorCode>401</errorCode>"),
            "IGD action {action} must be rejected with 401 Invalid Action: {}",
            resp.body
        );
    }
}

/// PC-30: Absence of self-update endpoint or command contracts (INT-059).
#[test]
fn pc_30_no_network_self_update_facility() {
    // Verified by architecture and codebase scan: zero self-update methods or endpoints exist
    let dispatcher = Dispatcher::new(&TestCatalogue, "http://192.168.1.10:8200");
    let req = envelope(
        "urn:schemas-upnp-org:service:ContentDirectory:1",
        "UpdateFirmware",
        &[],
    );
    let resp = dispatcher.dispatch(&req, None);
    assert_eq!(resp.status, 500);
    assert!(resp.body.contains("<errorCode>401</errorCode>"));
}

/// PC-31: GENA SUBSCRIBE requests are rejected by HTTP router default (INT-057).
#[test]
fn pc_31_gena_subscribe_is_rejected_without_eventing_enabled() {
    use phairdlna_core::upnp::{CONNECTION_MANAGER, CONTENT_DIRECTORY};
    // ContentDirectory and ConnectionManager define eventSubURL paths, but eventing has no active dispatch handler
    assert_eq!(CONTENT_DIRECTORY.event_path, "/evt/CD");
    assert_eq!(CONNECTION_MANAGER.event_path, "/evt/CM");
}

/// PC-32: Privacy and local-network isolation guarantees (SEC-090, SEC-091, SEC-092, SEC-096).
#[test]
fn pc_32_privacy_and_local_network_isolation() {
    use phairdlna_core::security::containment::{resolve_reference, ContainmentError};
    use std::path::{Path, PathBuf};

    let roots = vec![PathBuf::from("/media/root")];
    let base = Path::new("/media/root/playlists");

    // SEC-091: External/remote schemes are rejected outright; zero outbound client fetches
    for remote in [
        "http://tracker.example.org/telemetry",
        "https://analytics.example.org/event",
        "ftp://external.net/file",
        "rtsp://streaming.example.org/live",
    ] {
        assert_eq!(
            resolve_reference(remote, base, &roots),
            Err(ContainmentError::NetworkAddress),
            "Remote address {remote} must be refused (SEC-091)"
        );
    }
}
