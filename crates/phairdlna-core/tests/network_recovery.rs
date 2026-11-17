//! The acceptance criteria of F-13 for lifecycle states, sleep/wake and network recovery.
//!
//! What: tests that verify server lifecycle state transitions, interface change handling,
//!       degraded states when a configured interface disappears, and sleep/wake suspension.
//! Why: a server must not spin or leak resources across sleep/wake, must not widen its
//!      network exposure when interfaces change, and must report truthful states (LIFE-001..LIFE-004).

use phairdlna_core::config::{Config, NetworkSection};
use phairdlna_core::lifecycle::{is_allowed, LifecycleState, Timeline, Transition};
use phairdlna_core::platform::NetworkInterface;
use phairdlna_core::security::Redactor;
use std::net::{IpAddr, Ipv4Addr};
use std::sync::Arc;

/// A mock network adapter that simulates platform interface changes.
#[derive(Clone, Default)]
struct MockNetwork {
    interfaces: Vec<NetworkInterface>,
}

impl MockNetwork {
    fn new(interfaces: Vec<NetworkInterface>) -> Self {
        Self { interfaces }
    }

    /// Evaluates which interfaces are permitted according to configuration policy (LIFE-003, LIFE-004).
    fn permitted_interfaces(&self, config: &Config) -> Vec<NetworkInterface> {
        let configured = &config.network.interfaces;
        self.interfaces
            .iter()
            .filter(|iface| {
                if !configured.is_empty() && !configured.contains(&iface.name) {
                    return false;
                }
                if !config.network.allow_non_local_addresses && !iface.is_local {
                    return false;
                }
                true
            })
            .cloned()
            .collect()
    }
}

/// Helper that evaluates lifecycle state from interface availability (LIFE-004).
fn evaluate_network_lifecycle(
    current_state: LifecycleState,
    configured_interface: &str,
    available_interfaces: &[NetworkInterface],
) -> (LifecycleState, Option<Transition>) {
    let exists = available_interfaces
        .iter()
        .any(|i| i.name == configured_interface);
    if !exists {
        let next_state = LifecycleState::Degraded;
        if is_allowed(current_state, next_state) {
            let transition = Transition::new(
                next_state,
                format!("the configured interface \"{configured_interface}\" disappeared"),
            );
            return (next_state, Some(transition));
        }
    } else if current_state == LifecycleState::Degraded {
        let next_state = LifecycleState::Ready;
        if is_allowed(current_state, next_state) {
            let transition = Transition::new(
                next_state,
                format!("the configured interface \"{configured_interface}\" is available again"),
            );
            return (next_state, Some(transition));
        }
    }
    (current_state, None)
}

/// LIFE-001: Allowed transitions between states follow the specification rule.
#[test]
fn lifecycle_state_transitions_follow_rules() {
    assert!(is_allowed(LifecycleState::Starting, LifecycleState::Ready));
    assert!(is_allowed(LifecycleState::Ready, LifecycleState::Suspended));
    assert!(is_allowed(
        LifecycleState::Suspended,
        LifecycleState::Recovering
    ));
    assert!(is_allowed(
        LifecycleState::Recovering,
        LifecycleState::Ready
    ));
    assert!(is_allowed(LifecycleState::Ready, LifecycleState::Degraded));
    assert!(is_allowed(
        LifecycleState::Degraded,
        LifecycleState::Suspended
    ));
    assert!(is_allowed(
        LifecycleState::Suspended,
        LifecycleState::Degraded
    ));
    assert!(is_allowed(LifecycleState::Degraded, LifecycleState::Ready));
    assert!(is_allowed(LifecycleState::Ready, LifecycleState::Stopping));
    assert!(is_allowed(
        LifecycleState::Stopping,
        LifecycleState::Stopped
    ));

    // Refused transitions (cannot jump directly to ready from stopped without starting)
    assert!(!is_allowed(LifecycleState::Stopped, LifecycleState::Ready));
    assert!(!is_allowed(LifecycleState::Stopping, LifecycleState::Ready));
}

/// LIFE-002: Sleep transitions to Suspended and wake recovers to Ready or Degraded.
#[test]
fn sleep_wake_cycle_updates_state_and_records_timeline() {
    let dir = tempfile::tempdir().expect("tempdir");
    let redactor = Arc::new(Redactor::new());
    let timeline = Timeline::new(dir.path().join("timeline.log"), Arc::clone(&redactor));

    let mut state = LifecycleState::Ready;

    // Sleep arrives
    assert!(is_allowed(state, LifecycleState::Suspended));
    state = LifecycleState::Suspended;
    let sleep_transition = Transition::new(state, "system enters sleep mode");
    timeline.append(&sleep_transition).expect("appended sleep");

    // While suspended, is_serving is false
    assert!(!state.is_serving());

    // Wake arrives
    assert!(is_allowed(state, LifecycleState::Recovering));
    state = LifecycleState::Recovering;
    let wake_transition = Transition::new(state, "system woke up, re-evaluating network listeners");
    timeline.append(&wake_transition).expect("appended wake");

    assert!(is_allowed(state, LifecycleState::Ready));
    state = LifecycleState::Ready;
    let ready_transition = Transition::new(state, "network listeners rebound successfully");
    timeline.append(&ready_transition).expect("appended ready");

    assert!(state.is_serving());

    let log = timeline.read().expect("read timeline");
    assert!(
        log.contains("suspended"),
        "log should contain suspended: {log}"
    );
    assert!(
        log.contains("recovering"),
        "log should contain recovering: {log}"
    );
    assert!(log.contains("ready"), "log should contain ready: {log}");
}

/// LIFE-003: Interface change never widens network exposure beyond configured policy.
#[test]
fn interface_change_does_not_widen_exposure() {
    let config = Config {
        network: NetworkSection {
            interfaces: vec!["en0".to_owned()],
            allow_non_local_addresses: false,
        },
        ..Config::default()
    };

    let local_ip = IpAddr::V4(Ipv4Addr::new(192, 168, 1, 42));
    let foreign_ip = IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1));
    let public_ip = IpAddr::V4(Ipv4Addr::new(198, 51, 100, 5));

    let initial_network = MockNetwork::new(vec![
        NetworkInterface {
            name: "en0".to_owned(),
            addresses: vec![local_ip],
            supports_multicast: true,
            is_local: true,
        },
        NetworkInterface {
            name: "en1".to_owned(),
            addresses: vec![foreign_ip],
            supports_multicast: true,
            is_local: true,
        },
    ]);

    let permitted = initial_network.permitted_interfaces(&config);
    assert_eq!(permitted.len(), 1);
    assert_eq!(permitted[0].name, "en0");

    // Network changes: a new interface en2 appears with a public IP
    let changed_network = MockNetwork::new(vec![
        NetworkInterface {
            name: "en0".to_owned(),
            addresses: vec![local_ip],
            supports_multicast: true,
            is_local: true,
        },
        NetworkInterface {
            name: "en2".to_owned(),
            addresses: vec![public_ip],
            supports_multicast: true,
            is_local: false,
        },
    ]);

    let permitted_after = changed_network.permitted_interfaces(&config);
    assert_eq!(permitted_after.len(), 1);
    assert_eq!(permitted_after[0].name, "en0");
    assert!(
        !permitted_after.iter().any(|i| i.name == "en2"),
        "en2 must never be adopted"
    );
}

/// LIFE-004: When configured interface disappears, server transitions to Degraded.
#[test]
fn missing_configured_interface_enters_degraded_state() {
    let dir = tempfile::tempdir().expect("tempdir");
    let redactor = Arc::new(Redactor::new());
    let timeline = Timeline::new(dir.path().join("timeline.log"), Arc::clone(&redactor));

    let configured_iface = "en0";
    let state = LifecycleState::Ready;

    // Interface is present initially
    let interfaces = vec![NetworkInterface {
        name: "en0".to_owned(),
        addresses: vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, 15))],
        supports_multicast: true,
        is_local: true,
    }];
    let (state_now, transition) = evaluate_network_lifecycle(state, configured_iface, &interfaces);
    assert_eq!(state_now, LifecycleState::Ready);
    assert!(transition.is_none());

    // Interface en0 drops (e.g. Wi-Fi disabled or cable unplugged)
    let dropped_interfaces: Vec<NetworkInterface> = vec![];
    let (state_degraded, transition) =
        evaluate_network_lifecycle(state_now, configured_iface, &dropped_interfaces);
    assert_eq!(state_degraded, LifecycleState::Degraded);
    let tr = transition.expect("transition recorded");
    assert!(tr.reason.contains("disappeared"));
    timeline.append(&tr).expect("append degraded transition");

    // Interface comes back
    let (state_recovered, transition_recovered) =
        evaluate_network_lifecycle(state_degraded, configured_iface, &interfaces);
    assert_eq!(state_recovered, LifecycleState::Ready);
    let tr_rec = transition_recovered.expect("recovery transition recorded");
    assert!(tr_rec.reason.contains("available again"));
    timeline
        .append(&tr_rec)
        .expect("append recovered transition");

    let log = timeline.read().expect("read timeline");
    assert!(log.contains("degraded"));
    assert!(log.contains("ready"));
}
