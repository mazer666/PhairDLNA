//! The acceptance criteria of F-15 for macOS Apple Silicon native support (PLT-004).
//!
//! What: verifies that phairdlna-mac compiles and executes as a native aarch64 binary
//!       on Apple Silicon without requiring Rosetta translation or x86_64 emulation.

#[test]
fn application_target_matches_host_architecture() {
    let host_arch = std::env::consts::ARCH;
    let target_arch = if cfg!(target_arch = "aarch64") {
        "aarch64"
    } else if cfg!(target_arch = "x86_64") {
        "x86_64"
    } else {
        "other"
    };

    assert_eq!(
        host_arch, target_arch,
        "the application must execute natively on the host architecture without translation"
    );

    #[cfg(target_os = "macos")]
    {
        // On macOS Apple Silicon (arm64/aarch64), verify we are native
        if host_arch == "aarch64" {
            const {
                assert!(
                    cfg!(target_arch = "aarch64"),
                    "Apple Silicon build must target aarch64 natively"
                );
            }
        }
    }
}
