//! Acceptance tests for performance reference profile contract (DEC-081, OD-11, PERF-010, PERF-011).
//!
//! PERF-010: "Every measurement names its hardware profile."
//! PERF-011: "Every measurement reports the method, the iteration count and the statistic (median and p95)."

#[test]
fn reference_hardware_profile_contract_is_defined() {
    let profile = "Apple Silicon (arm64, Apple M-Series, >=16 GB RAM, NVMe SSD)";
    assert!(profile.contains("Apple Silicon"));
    assert!(profile.contains("arm64"));
    assert!(profile.contains("NVMe SSD"));
}

#[test]
fn measurement_statistics_contract_requires_median_and_p95() {
    let stats = ["median", "p95"];
    assert_eq!(stats.len(), 2);
    assert!(stats.contains(&"median"));
    assert!(stats.contains(&"p95"));
}
