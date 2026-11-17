//! The acceptance test for the macOS universal binary bundle (PLT-014).
//!
//! What: verifies that the packaging script or release binary contains
//!       both Apple Silicon (`arm64`) and Intel (`x86_64`) architectures.
//! Why: PLT-014 requires universal binary packaging on macOS so users on
//!      either Intel or Apple Silicon Macs run natively.

#[test]
fn the_bundle_script_targets_universal_architectures() {
    let script_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .map(|p| p.join("scripts/package-macos-app.sh"))
        .expect("script path resolution");

    assert!(script_path.exists(), "packaging script must exist");
    let content = std::fs::read_to_string(&script_path).expect("read script");

    assert!(
        content.contains("aarch64-apple-darwin"),
        "script must build for aarch64"
    );
    assert!(
        content.contains("x86_64-apple-darwin"),
        "script must build for x86_64"
    );
    assert!(
        content.contains("lipo -create"),
        "script must combine into universal binary with lipo"
    );
}
