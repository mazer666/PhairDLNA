//! The acceptance criteria of the network of the application.
//!
//! What: the tests that check that the application opens no connection to
//!       anything but the administration surface on the loopback interface.
//! Why: the application is a front end for a server on the machine of the
//!      user; an application that phones home or that talks to the network
//!      without the user asking breaks the promise of the product
//!      (`SEC-090`, `SEC-091`, `MB-005`).
//! Example: the sources of the crate name no address but the loopback
//!          interface, and the manifest of the crate holds no library that
//!          could open a connection to the network.

use std::path::{Path, PathBuf};

/// The path of the sources of the crate.
fn sources_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// The path of the manifest of the crate.
fn manifest_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml")
}

/// The names of the addresses that a source holds.
fn addresses_of(source: &str) -> Vec<String> {
    let mut addresses = Vec::new();
    for scheme in ["http://", "https://"] {
        let mut rest = source;
        while let Some(start) = rest.find(scheme) {
            rest = &rest[start + scheme.len()..];
            let end = rest
                .find(|character: char| character.is_whitespace() || character == '"')
                .unwrap_or(rest.len());
            addresses.push(rest[..end].to_owned());
        }
    }
    addresses
}

/// No source of the crate names an address but the loopback interface.
#[test]
fn no_source_names_an_address_but_the_loopback() {
    let mut offenders = Vec::new();
    collect(&sources_path(), &mut offenders);
    assert!(
        offenders.is_empty(),
        "the application must talk to the loopback interface only: {offenders:?}"
    );
}

/// Collects the addresses that are not the loopback interface.
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
        for address in addresses_of(&source) {
            if !is_local(&address) {
                found.push(format!("{}: {address}", path.display()));
            }
        }
    }
}

/// Whether an address is the loopback interface of the machine.
///
/// An address that holds a placeholder is filled by the transport, and the
/// transport of the crate is built with the loopback interface.
fn is_local(address: &str) -> bool {
    address.starts_with("127.0.0.1")
        || address.starts_with("localhost")
        || address.starts_with('{')
        || address.starts_with("github.com/mazer666/PhairDLNA")
}

/// The transport of the client is bound to the loopback interface.
#[test]
fn the_transport_is_bound_to_the_loopback_interface() {
    let source =
        std::fs::read_to_string(sources_path().join("ipc").join("http").join("transport.rs"))
            .expect("the transport is read");

    assert!(
        source.contains("Ipv4Addr::LOCALHOST"),
        "the client must talk to the loopback interface"
    );
}

/// The dependencies of the crate, with the section they stand in.
fn dependencies_of(manifest: &str) -> Vec<(String, String)> {
    let mut section = String::new();
    let mut found = Vec::new();
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            section = trimmed.trim_matches(['[', ']']).to_owned();
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((name, _)) = trimmed.split_once('=') else {
            continue;
        };
        let name = name.trim();
        let plain = !name.is_empty()
            && name
                .chars()
                .all(|character| character.is_alphanumeric() || "-_".contains(character));
        if plain {
            found.push((section.clone(), name.to_owned()));
        }
    }
    found
}

/// The manifest of the crate holds no library that could open a connection to
/// the network.
#[test]
fn the_manifest_holds_no_library_of_the_network() {
    let manifest = std::fs::read_to_string(manifest_path()).expect("the manifest is read");
    let forbidden = [
        "reqwest",
        "ureq",
        "hyper",
        "curl",
        "isahc",
        "surf",
        "attohttpc",
        "native-tls",
        "rustls",
        "tokio",
    ];

    for (_, name) in dependencies_of(&manifest) {
        assert!(
            !forbidden.contains(&name.as_str()),
            "the application must not depend on {name}"
        );
    }
}

/// The application depends on the core of the project and on the bindings of
/// the system, and on nothing else.
#[test]
fn the_dependencies_are_the_ones_of_the_project() {
    let manifest = std::fs::read_to_string(manifest_path()).expect("the manifest is read");
    let mut names: Vec<String> = dependencies_of(&manifest)
        .into_iter()
        .filter(|(section, _)| section == "dependencies")
        .map(|(_, name)| name)
        .collect();
    names.sort();

    assert_eq!(names, vec!["phairdlna-core".to_owned()]);
}
