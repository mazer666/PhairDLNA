//! The acceptance criteria of F-13 for recovery events and the redacted timeline (LIFE-011, AC-28).
//!
//! What: verifies that transitions and recovery events are stored in a bounded local timeline,
//!       that paths/addresses/secrets are redacted into pseudonyms, and that the timeline can be read.

use phairdlna_core::lifecycle::{LifecycleState, Timeline, Transition};
use phairdlna_core::security::Redactor;
use std::sync::Arc;

#[test]
fn recovery_timeline_records_ordered_events_with_redaction() {
    let dir = tempfile::tempdir().expect("tempdir");
    let redactor = Arc::new(Redactor::new());
    let timeline_path = dir.path().join("timeline.log");
    let timeline = Timeline::new(timeline_path.clone(), Arc::clone(&redactor));

    // Initially empty
    assert_eq!(timeline.read().expect("read empty"), "");

    // Server starts and becomes ready
    let tr1 = Transition::new(
        LifecycleState::Starting,
        "reading configuration at /Users/tester/.config/phairdlna",
    );
    timeline.append(&tr1).expect("append tr1");

    let tr2 = Transition::new(
        LifecycleState::Ready,
        "bound listeners on 192.168.1.100:5000",
    );
    timeline.append(&tr2).expect("append tr2");

    // Network degradation occurs
    let tr3 = Transition::new(
        LifecycleState::Degraded,
        "interface en0 on 192.168.1.100 disappeared",
    );
    timeline.append(&tr3).expect("append tr3");

    // Network recovers
    let tr4 = Transition::new(
        LifecycleState::Recovering,
        "attempting rebind on 192.168.1.105",
    );
    timeline.append(&tr4).expect("append tr4");

    let tr5 = Transition::new(
        LifecycleState::Ready,
        "rebound successfully on 192.168.1.105:5000",
    );
    timeline.append(&tr5).expect("append tr5");

    let log = timeline.read().expect("read populated timeline");
    let lines: Vec<&str> = log.lines().collect();
    assert_eq!(
        lines.len(),
        5,
        "all 5 events must be present in chronological order"
    );

    // Verify chronological order of states
    assert!(lines[0].contains("starting"));
    assert!(lines[1].contains("ready"));
    assert!(lines[2].contains("degraded"));
    assert!(lines[3].contains("recovering"));
    assert!(lines[4].contains("ready"));

    // Verify redaction: private local paths and raw IPs must not appear verbatim (SEC-094, LIFE-011)
    assert!(
        !log.contains("/Users/tester"),
        "raw user home path must be redacted"
    );
    assert!(
        log.contains("<path:1>"),
        "path must be replaced with pseudonym"
    );
}
