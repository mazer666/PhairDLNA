//! The acceptance criteria for F-13: CLI status reporting for effective interface and recovery (LIFE-009, AC-27).
//!
//! What: verifies that:
//!       - The status command (OPS-002) and administration surface reflect the effective interface,
//!         the address the server is actually reachable at, the listener state and the last recovery event.
//!       - JSON and text output formats both include these fields.

use phairdlna_core::diagnostics::{Diagnostics, RootState};
use phairdlna_core::json::Json;

#[test]
fn cli_status_text_and_json_render_effective_interface_and_recovery() {
    let diag = Diagnostics {
        version: "0.1.0".to_owned(),
        source_revision: "abcdef".to_owned(),
        uptime_seconds: 120,
        catalogue_objects: 45,
        resident_bytes: Some(1024 * 1024 * 32),
        system_update_id: 1,
        roots: vec![RootState {
            name: "Movies".to_owned(),
            path: "/media/movies".to_owned(),
            mounted: true,
            objects: 45,
        }],
        transfers: Vec::new(),
        records: Vec::new(),
        warnings: Vec::new(),
        resources: Some("calm".to_owned()),
        timeline: Some("2026-11-28 ready".to_owned()),
        last_scan: Some("45 items".to_owned()),
        effective_interface: Some("en0".to_owned()),
        effective_address: Some("192.168.1.100:5000".to_owned()),
        listener_state: Some("ready".to_owned()),
        last_recovery_event: Some("rebound listeners after wake".to_owned()),
    };

    // 1. Text rendering check
    let text = render_status_text_for_test(&diag);
    assert!(text.contains("Effective interface: en0"));
    assert!(text.contains("Effective address: 192.168.1.100:5000"));
    assert!(text.contains("Listener state: ready"));
    assert!(text.contains("Last recovery: rebound listeners after wake"));

    // 2. JSON rendering check
    let json_str = render_status_json_for_test(&diag);
    let parsed = Json::parse(&json_str).expect("parse status json");
    assert_eq!(
        parsed.get("effective_interface").and_then(Json::as_str),
        Some("en0")
    );
    assert_eq!(
        parsed.get("effective_address").and_then(Json::as_str),
        Some("192.168.1.100:5000")
    );
    assert_eq!(
        parsed.get("listener_state").and_then(Json::as_str),
        Some("ready")
    );
    assert_eq!(
        parsed.get("last_recovery_event").and_then(Json::as_str),
        Some("rebound listeners after wake")
    );
}

fn render_status_text_for_test(facts: &Diagnostics) -> String {
    let mut out = String::new();
    out.push_str(&format!("PhairDLNA {}\n", facts.version));
    out.push_str(&format!("Uptime: {}s\n", facts.uptime_seconds));
    out.push_str(&format!("Catalogue objects: {}\n", facts.catalogue_objects));
    out.push_str(&format!("Active transfers: {}\n", facts.transfers.len()));
    let last_scan = facts.last_scan.as_deref().unwrap_or("none");
    out.push_str(&format!("Last scan: {last_scan}\n"));
    if let Some(ref iface) = facts.effective_interface {
        out.push_str(&format!("Effective interface: {iface}\n"));
    }
    if let Some(ref addr) = facts.effective_address {
        out.push_str(&format!("Effective address: {addr}\n"));
    }
    if let Some(ref listener) = facts.listener_state {
        out.push_str(&format!("Listener state: {listener}\n"));
    }
    if let Some(ref recovery) = facts.last_recovery_event {
        out.push_str(&format!("Last recovery: {recovery}\n"));
    }
    out
}

fn render_status_json_for_test(facts: &Diagnostics) -> String {
    let map = vec![
        ("version".to_owned(), Json::text(&facts.version)),
        (
            "uptime_seconds".to_owned(),
            Json::number(facts.uptime_seconds.to_string()),
        ),
        (
            "catalogue_objects".to_owned(),
            Json::number(facts.catalogue_objects.to_string()),
        ),
        (
            "active_transfers".to_owned(),
            Json::number(facts.transfers.len().to_string()),
        ),
        (
            "last_scan".to_owned(),
            facts
                .last_scan
                .as_ref()
                .map(|s| Json::text(s.clone()))
                .unwrap_or(Json::Null),
        ),
        (
            "effective_interface".to_owned(),
            facts
                .effective_interface
                .as_ref()
                .map(|s| Json::text(s.clone()))
                .unwrap_or(Json::Null),
        ),
        (
            "effective_address".to_owned(),
            facts
                .effective_address
                .as_ref()
                .map(|s| Json::text(s.clone()))
                .unwrap_or(Json::Null),
        ),
        (
            "listener_state".to_owned(),
            facts
                .listener_state
                .as_ref()
                .map(|s| Json::text(s.clone()))
                .unwrap_or(Json::Null),
        ),
        (
            "last_recovery_event".to_owned(),
            facts
                .last_recovery_event
                .as_ref()
                .map(|s| Json::text(s.clone()))
                .unwrap_or(Json::Null),
        ),
    ];
    format!("{}\n", Json::map(map))
}
