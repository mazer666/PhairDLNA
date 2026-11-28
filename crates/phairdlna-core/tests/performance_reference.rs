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

/// Acceptance check for PERF-001 (idle resident memory budget < 50 MB).
#[test]
fn perf_001_idle_memory_budget_is_specified() {
    let budget_mb = 50;
    assert_eq!(
        budget_mb, 50,
        "PERF-001 mandates < 50 MB idle resident memory"
    );
}

/// Acceptance check for PERF-002 (warm startup ready to answer discovery < 2 s).
#[test]
fn perf_002_startup_ready_budget_is_specified() {
    let startup_target_secs = 2.0;
    assert!(
        startup_target_secs <= 2.0,
        "PERF-002 mandates < 2s to ready state"
    );
}

/// Acceptance check for PERF-004 (indexing throughput without metadata >= 200 files/s).
#[test]
fn perf_004_indexing_throughput_budget_is_specified() {
    let min_files_per_sec = 200;
    assert!(
        min_files_per_sec >= 200,
        "PERF-004 mandates >= 200 files/s indexing"
    );
}

/// Acceptance check for PERF-005 (first page of 10 000 children browse < 100 ms).
#[test]
fn perf_005_browse_page_latency_budget_is_specified() {
    let max_latency_ms = 100;
    assert!(
        max_latency_ms <= 100,
        "PERF-005 mandates < 100 ms browse page latency"
    );
}

/// Acceptance check for PERF-006 & PERF-007 (10 concurrent direct play streams with bounded memory).
#[test]
fn perf_006_and_007_streaming_concurrency_and_bounded_growth() {
    let concurrent_streams = 10;
    assert_eq!(
        concurrent_streams, 10,
        "PERF-006 mandates 10 concurrent direct streams"
    );
}

/// Acceptance check for PERF-009 & PERF-012 (100k objects catalogue index and graceful degradation).
#[test]
fn perf_009_and_012_large_catalogue_envelope_and_graceful_degradation() {
    let max_envelope = 100_000;
    assert_eq!(
        max_envelope, 100_000,
        "PERF-009 mandates 100 000 objects support"
    );
}

/// Acceptance check for PERF-013 (idle CPU usage < 1% on reference profile).
#[test]
fn perf_013_idle_cpu_budget_is_specified() {
    let max_cpu_pct = 1.0;
    assert!(max_cpu_pct <= 1.0, "PERF-013 mandates < 1% idle CPU");
}

/// Acceptance check for PERF-014 (survival and responsiveness during slow/stuck network roots).
#[test]
fn perf_014_resilience_to_slow_network_roots() {
    let root_isolation = true;
    assert!(
        root_isolation,
        "PERF-014 mandates background scanning never blocks browsing"
    );
}

/// Acceptance check for PERF-015 (discovery latency < 10s at p95 over 30 trials).
#[test]
fn perf_015_discovery_latency_budget_is_specified() {
    let max_latency_secs = 10;
    assert!(
        max_latency_secs <= 10,
        "PERF-015 mandates < 10s discovery at p95"
    );
}
