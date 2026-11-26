//! Build matrix and platform support acceptance tests (PLT-002, F-22 AC-02).
//!
//! What: verifies that the build matrix workflow exists, targets all required
//!       operating systems and architectures (macOS arm64/x86_64, Linux x86_64,
//!       Windows x86_64, and MSRV), and complies with local compute policy.
//! Why: PLT-002 requires a build matrix covering macOS (arm64, x86_64), Linux (x86_64)
//!      and Windows (x86_64). F-22 AC-02 specifies this build matrix in
//!      `.github/workflows/ci-matrix.yml`.

use std::path::Path;

#[test]
fn the_build_matrix_covers_all_required_platforms() {
    let workflow_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join(".github/workflows/ci-matrix.yml"))
        .expect("workflow path resolution");

    assert!(
        workflow_path.exists(),
        "build matrix workflow must exist at .github/workflows/ci-matrix.yml (PLT-002, AC-02)"
    );

    let content = std::fs::read_to_string(&workflow_path).expect("read ci-matrix.yml content");

    // Must cover macOS ARM64 (Apple Silicon)
    assert!(
        content.contains("macOS (stable, Apple Silicon)") && content.contains("ARM64"),
        "build matrix must cover macOS Apple Silicon (ARM64)"
    );

    // Must cover macOS Intel (x86_64)
    assert!(
        content.contains("macOS (stable, Intel)") && content.contains("x64"),
        "build matrix must cover macOS Intel (x86_64)"
    );

    // Must cover Linux x86_64
    assert!(
        content.contains("Linux (stable)") && content.contains("linux"),
        "build matrix must cover Linux x86_64"
    );

    // Must cover Windows x86_64
    assert!(
        content.contains("Windows (stable)") && content.contains("windows"),
        "build matrix must cover Windows x86_64"
    );

    // Must cover MSRV
    assert!(
        content.contains("Linux (MSRV"),
        "build matrix must cover the MSRV target"
    );

    // Project and user rules: must never use GitHub-hosted cloud runners
    assert!(
        !content.contains("ubuntu-latest")
            && !content.contains("macos-latest")
            && !content.contains("windows-latest"),
        "build matrix must never use GitHub-hosted cloud runner labels"
    );
    assert!(
        content.contains("self-hosted"),
        "build matrix jobs must be routed to self-hosted runners"
    );
}
