//! Client compatibility profiles acceptance tests (INT-100 .. INT-104, F-07 AC-01..AC-06).
//!
//! What: verifies that client quirk profiles are declarative, enforce mandatory evidence,
//!       can match incoming client User-Agent substrings, and can be disabled via configuration
//!       to restore plain conforming behavior without recompilation.
//! Why: INT-100 requires declarative profiles; INT-101 protects specification conformance;
//!      INT-104 mandates non-code global disable switches.

use phairdlna_core::profile::{ClientProfile, ProfileStore, QuirkAdjustment};

#[test]
fn profile_without_evidence_pointer_is_rejected_by_evidence_gate() {
    let profile = ClientProfile {
        id: "legacy-renderer".to_owned(),
        client_family: "Legacy Renderer".to_owned(),
        user_agent_substring: "LegacyPlayer".to_owned(),
        symptom: "Disconnects on unknown 4th protocolInfo parameter".to_owned(),
        adjustment: QuirkAdjustment::CompactProtocolInfo,
        evidence: String::new(), // Missing evidence
        touches: vec!["INT-050".to_owned()],
    };

    let err = profile
        .validate()
        .expect_err("evidence gate must reject empty evidence (INT-100)");
    assert!(err.contains("missing mandatory evidence"), "{err}");
}

#[test]
fn profile_matching_is_case_sensitive_and_matches_substring() {
    let mut store = ProfileStore::new();
    let profile = ClientProfile {
        id: "smart-tv-2018".to_owned(),
        client_family: "SmartTV 2018".to_owned(),
        user_agent_substring: "SmartTV-2018".to_owned(),
        symptom: "Rejects subtitle sidecars in res tags".to_owned(),
        adjustment: QuirkAdjustment::SuppressSubtitleRes,
        evidence: "qa/device-matrix.toml#row-04".to_owned(),
        touches: vec!["INT-060".to_owned()],
    };
    store.register(profile).expect("register");

    assert!(store
        .match_client(Some("SEC_HHP_TV/1.0 SmartTV-2018/v1"))
        .is_some());
    assert!(store
        .match_client(Some("SEC_HHP_TV/1.0 OtherTV-2020"))
        .is_none());
    assert!(store.match_client(None).is_none());
}

#[test]
fn disabled_profiles_restore_plain_behavior_without_recompilation() {
    let mut store = ProfileStore::new();
    let profile = ClientProfile {
        id: "quirky-client".to_owned(),
        client_family: "Quirky Client".to_owned(),
        user_agent_substring: "QuirkyClient/1".to_owned(),
        symptom: "Expects compact headers".to_owned(),
        adjustment: QuirkAdjustment::CompactProtocolInfo,
        evidence: "qa/device-matrix.toml#row-09".to_owned(),
        touches: vec!["INT-050".to_owned()],
    };
    store.register(profile).expect("register");

    // Initially active
    assert!(store.match_client(Some("QuirkyClient/1.2")).is_some());

    // Disabled via config (INT-104)
    store.disable(&["quirky-client".to_owned()]);
    assert!(
        store.match_client(Some("QuirkyClient/1.2")).is_none(),
        "disabling profile must restore plain behavior (INT-104)"
    );
}
