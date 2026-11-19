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
    assert!(content.contains("lipo \"${MACOS}/phairdlna-mac\" -verify_arch x86_64"));
    assert!(content.contains("codesign --verify --deep --strict"));
}

/// The bundle's legacy package marker is exactly eight bytes, without a newline.
#[test]
fn the_packager_writes_a_well_formed_pkginfo_file() {
    let script_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .map(|path| path.join("scripts/package-macos-app.sh"))
        .expect("script path resolution");
    let content = std::fs::read_to_string(script_path).expect("read packaging script");

    assert!(content.contains("printf 'APPL????' > \"${CONTENTS}/PkgInfo\""));
    assert!(content.contains("PKG_INFO_BYTES"));
}

/// The architecture check must reject a binary that lacks the other Mac slice.
#[cfg(target_os = "macos")]
#[test]
fn a_thin_binary_is_rejected_for_its_missing_architecture() {
    let missing_architecture = if cfg!(target_arch = "aarch64") {
        "x86_64"
    } else {
        "arm64"
    };
    let binary = std::env::current_exe().expect("test binary path");
    let result = std::process::Command::new("lipo")
        .arg(&binary)
        .arg("-verify_arch")
        .arg(missing_architecture)
        .output()
        .expect("lipo is provided by the macOS developer tools");

    assert!(
        !result.status.success(),
        "lipo must reject the missing {missing_architecture} slice"
    );
}

/// A resource changed after signing must invalidate the sealed application bundle.
#[cfg(target_os = "macos")]
#[test]
fn a_changed_bundle_resource_fails_signature_verification() {
    let temporary = tempfile::tempdir().expect("temporary directory");
    let bundle = temporary.path().join("SignatureCheck.app");
    let contents = bundle.join("Contents");
    let executable = contents.join("MacOS/phairdlna-mac");
    let resources = contents.join("Resources");
    std::fs::create_dir_all(executable.parent().expect("executable parent"))
        .expect("create executable directory");
    std::fs::create_dir_all(&resources).expect("create resources directory");
    std::fs::copy(
        std::env::current_exe().expect("test binary path"),
        &executable,
    )
    .expect("copy test executable into bundle");
    std::fs::copy(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("resources/Info.plist"),
        contents.join("Info.plist"),
    )
    .expect("copy bundle metadata");
    std::fs::write(contents.join("PkgInfo"), b"APPL????").expect("write package marker");
    let resource = resources.join("sealed.txt");
    std::fs::write(&resource, b"sealed before signing").expect("write bundle resource");

    assert!(run_codesign(&[
        "--force",
        "--deep",
        "--sign",
        "-",
        path_of(&bundle)
    ]));
    assert!(run_codesign(&[
        "--verify",
        "--deep",
        "--strict",
        path_of(&bundle)
    ]));
    std::fs::write(resource, b"changed after signing").expect("change sealed resource");
    assert!(
        !run_codesign(&["--verify", "--deep", "--strict", path_of(&bundle)]),
        "verification must fail after a sealed resource changes"
    );
}

#[cfg(target_os = "macos")]
fn run_codesign(arguments: &[&str]) -> bool {
    std::process::Command::new("codesign")
        .args(arguments)
        .status()
        .is_ok_and(|status| status.success())
}

#[cfg(target_os = "macos")]
fn path_of(path: &std::path::Path) -> &str {
    path.to_str().expect("temporary path is valid UTF-8")
}
