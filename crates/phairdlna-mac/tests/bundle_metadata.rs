//! The acceptance criteria of the bundle of the application.
//!
//! What: the tests that check the metadata of the bundle — that the
//!       application is a menu bar application and that it says in plain
//!       words why it asks for the permission of the local network.
//! Why: the system of the operating system reads these keys before it shows
//!      the application to the user; an application without them asks for a
//!      permission it cannot explain (`MB-009`, `UX-010`).
//! Example: the bundle carries `LSUIElement`, so that the application has no
//!          icon in the dock, and `NSLocalNetworkUsageDescription`, so that
//!          the question of the permission has an answer.

use std::path::{Path, PathBuf};

/// The path of the file of the metadata of the bundle.
fn plist_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join("Info.plist")
}

/// The text of the file of the metadata of the bundle.
fn plist_source() -> String {
    std::fs::read_to_string(plist_path()).expect("the file of the bundle is read")
}

/// The value of a key of the file of the metadata.
fn value_of(key: &str) -> Option<String> {
    let source = plist_source();
    let start = source.find(&format!("<key>{key}</key>"))?;
    let rest = &source[start..];
    let open = rest.find("<string>")?;
    let close = rest[open..].find("</string>")?;
    Some(rest[open + "<string>".len()..open + close].to_owned())
}

/// The bundle is an application of the menu bar and not an application with a
/// window.
#[test]
fn the_bundle_is_an_application_of_the_menu_bar() {
    let source = plist_source();

    assert!(
        source.contains("<key>LSUIElement</key>"),
        "the bundle must be marked as an application of the menu bar"
    );
    let after = source.split("<key>LSUIElement</key>").nth(1);
    let after = after.expect("the key is there");
    assert!(
        after.trim_start().starts_with("<true/>"),
        "the key must be true"
    );
}

/// The bundle asks for the permission of the local network and says why, in
/// words that a user can read.
#[test]
fn the_bundle_explains_the_permission_of_the_local_network() {
    let description = value_of("NSLocalNetworkUsageDescription").expect("the key is there");

    assert!(description.contains(' '), "the text must be a sentence");
    assert!(!description.contains('/'), "the text must hold no path");
    assert!(!description.starts_with('/'), "the text must hold no path");
}

/// The bundle names the program that it carries.
#[test]
fn the_bundle_names_its_program() {
    assert_eq!(
        value_of("CFBundleExecutable").as_deref(),
        Some("phairdlna-mac")
    );
    assert_eq!(value_of("CFBundlePackageType").as_deref(), Some("APPL"));
}

/// The bundle registers no helper of its own.
///
/// The server is installed as a `launchd` daemon by the command line, and
/// the application registers nothing: no helper of the bundle and no hidden
/// program of the user (`MB-008`, `LIFE-010`).
#[test]
fn the_bundle_registers_no_helper() {
    let source = plist_source();

    assert!(
        !source.contains("SMAppService"),
        "the bundle must not name a service of its own"
    );
    let sources = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    collect(&sources, &mut offenders);
    assert!(
        offenders.is_empty(),
        "the application must not register a helper: {offenders:?}"
    );
}

/// Collects the sources that name a service of the system.
fn collect(directory: &Path, found: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, found);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        if source.contains("SMAppService") || source.contains("SMJobBless") {
            found.push(path.display().to_string());
        }
    }
}
