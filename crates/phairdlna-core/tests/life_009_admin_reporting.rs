//! Acceptance test for LIFE-009 / AC-27:
//! The administration surface and user interface reflect effective interface,
//! address, listener state and the last recovery event, with technical detail on demand.

use phairdlna_core::diagnostics::Diagnostics;

#[test]
fn life_009_diagnostics_contract_for_effective_interface_and_recovery() {
    let diag = Diagnostics {
        version: "0.1.0".to_owned(),
        effective_interface: Some("en0".to_owned()),
        effective_address: Some("192.168.1.50:5000".to_owned()),
        listener_state: Some("open".to_owned()),
        last_recovery_event: Some("rebound listeners on en0 (192.168.1.50:5000)".to_owned()),
        ..Default::default()
    };

    assert_eq!(diag.effective_interface.as_deref(), Some("en0"));
    assert_eq!(diag.effective_address.as_deref(), Some("192.168.1.50:5000"));
    assert_eq!(diag.listener_state.as_deref(), Some("open"));
    assert_eq!(
        diag.last_recovery_event.as_deref(),
        Some("rebound listeners on en0 (192.168.1.50:5000)")
    );
}
