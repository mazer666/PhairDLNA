//! The ConnectionManager service as a control point sees it.
//!
//! What: the acceptance criteria of the feature specification F-04 that the
//!       dispatcher can answer without a socket.
//! Why: the criteria are about the protocol list the server advertises, the
//!       list of running transfers and the faults of the actions that have
//!       no transfer to report.
//! Example: `cargo test --package phairdlna-core --test connection_manager`.

mod support;

use phairdlna_core::upnp::{Dispatcher, ServiceSpec, SUPPORTED_MIME_TYPES};
use support::{element_text, envelope, CONNECTION_MANAGER};

/// The answer of a request of the ConnectionManager service.
fn answer(action: &str, arguments: &[(&str, &str)]) -> String {
    Dispatcher::new(
        &support::catalogue::TestCatalogue,
        support::catalogue::BASE_URL,
    )
    .dispatch(&envelope(CONNECTION_MANAGER, action, arguments), None)
    .body
}

/// The source list of the protocol information.
fn source_list() -> String {
    let body = answer("GetProtocolInfo", &[]);
    element_text(&body, "Source").expect("the source list is present")
}

/// The source list is not empty, carries an `http-get` entry, and every entry
/// has exactly four fields.
#[test]
fn the_source_list_has_four_fields_per_entry() {
    let list = source_list();
    assert!(!list.is_empty(), "the list is not empty");
    assert!(list.contains("http-get:"));
    for entry in list.split(',') {
        assert_eq!(
            entry.split(':').count(),
            4,
            "the entry `{entry}` does not have four fields"
        );
    }
}

/// The registry holds the formats the specification names.
///
/// The registry is the ground truth of what the server can deliver, so the
/// formats of the specification have to be in it.
#[test]
fn the_registry_holds_the_formats_the_specification_names() {
    for mime_type in ["video/mp4", "audio/mpeg", "image/jpeg", "video/x-matroska"] {
        assert!(
            SUPPORTED_MIME_TYPES.contains(&mime_type),
            "{mime_type} is missing from the registry"
        );
    }
}

/// The `Source` list holds every format of the registry and names nothing
/// else.
#[test]
fn the_source_list_holds_every_format_of_the_registry() {
    let list = source_list();
    for mime_type in SUPPORTED_MIME_TYPES {
        assert!(
            list.contains(&format!("http-get:*:{mime_type}:*")),
            "{mime_type} is missing from the source list"
        );
    }
    assert_eq!(list.split(',').count(), SUPPORTED_MIME_TYPES.len());
    for entry in list.split(',') {
        let mime_type = entry.split(':').nth(2).expect("the entry has a format");
        assert!(
            SUPPORTED_MIME_TYPES.contains(&mime_type),
            "{mime_type} is not in the registry"
        );
    }
}

/// The list of the running transfers is empty.
#[test]
fn the_list_of_the_running_transfers_is_empty() {
    let body = answer("GetCurrentConnectionIDs", &[]);
    assert!(body.contains("<ConnectionIDs></ConnectionIDs>"));
    assert!(!body.contains("<ConnectionIDs>0</ConnectionIDs>"));
}

/// A transfer that was never issued is the fault `701`.
#[test]
fn an_unknown_transfer_is_the_fault_701() {
    let body = answer("GetCurrentConnectionInfo", &[("ConnectionID", "7")]);
    assert!(body.contains("<errorCode>701</errorCode>"));
    assert!(body.contains("<errorDescription>No Such Object</errorDescription>"));
}

/// A preparation creates no transfer, so nothing is ever left pending.
#[test]
fn a_preparation_creates_no_transfer() {
    let body = answer(
        "PrepareForConnection",
        &[
            ("RemoteProtocolInfo", "http-get:*:video/mp4:*"),
            (
                "PeerConnectionManager",
                "uuid:11111111-2222-3333-4444-555555555555",
            ),
            ("PeerConnectionID", "0"),
            ("Direction", "Output"),
        ],
    );
    assert!(
        body.contains("<errorCode>600</errorCode>"),
        "the optional action is refused"
    );
    assert!(body.contains("<errorDescription>Optional Action Not Implemented</errorDescription>"));
    let identifiers = answer("GetCurrentConnectionIDs", &[]);
    assert!(
        identifiers.contains("<ConnectionIDs></ConnectionIDs>"),
        "no transfer was created"
    );
}

/// A direction outside the two of the specification is the fault `402`.
#[test]
fn a_wrong_direction_is_the_fault_402() {
    let body = answer(
        "PrepareForConnection",
        &[
            ("RemoteProtocolInfo", "http-get:*:video/mp4:*"),
            (
                "PeerConnectionManager",
                "uuid:11111111-2222-3333-4444-555555555555",
            ),
            ("PeerConnectionID", "0"),
            ("Direction", "Sideways"),
        ],
    );
    assert!(body.contains("<errorCode>402</errorCode>"));
    assert!(body.contains("<errorDescription>Invalid Args</errorDescription>"));
}

/// A peer that sends kilobytes of text is refused without an unbounded
/// allocation, and the value is not echoed back.
#[test]
fn a_huge_peer_value_is_refused_without_an_unbounded_allocation() {
    let huge = "x".repeat(512 * 1024);
    let body = answer(
        "PrepareForConnection",
        &[
            ("RemoteProtocolInfo", "http-get:*:video/mp4:*"),
            ("PeerConnectionManager", &huge),
            ("PeerConnectionID", "0"),
            ("Direction", "Output"),
        ],
    );
    assert!(body.contains("<errorCode>"));
    assert!(!body.contains(&huge), "the value must not be echoed back");
}

/// The fourth field of every entry is the wildcard, and no entry names a
/// DLNA profile: the server speaks plain UPnP (`INT-055`, `INT-078`).
#[test]
fn every_entry_carries_the_wildcard_and_names_no_profile() {
    let list = source_list();
    for entry in list.split(',') {
        let fields: Vec<&str> = entry.split(':').collect();
        assert_eq!(fields[3], "*", "the fourth field is the wildcard");
    }
    assert!(!list.contains("DLNA.ORG_"));
}

/// No entry uses the push protocol.
#[test]
fn no_entry_uses_the_push_protocol() {
    assert!(!source_list().contains("http-subscribe"));
}

/// The service description carries the three required actions and marks the
/// two optional ones.
#[test]
fn the_service_description_marks_the_optional_actions() {
    let scpd = ServiceSpec::connection_manager().to_xml();
    for action in [
        "GetProtocolInfo",
        "GetCurrentConnectionIDs",
        "GetCurrentConnectionInfo",
    ] {
        assert!(
            scpd.contains(&format!("<name>{action}</name>")),
            "{action} is missing"
        );
    }
    assert!(scpd.contains("<!-- OPTIONAL: PrepareForConnection -->"));
    assert!(scpd.contains("<!-- OPTIONAL: ConnectionComplete -->"));
}

/// The sink element is present and empty, and no file name appears in the
/// answer.
#[test]
fn the_sink_is_present_and_empty() {
    let body = answer("GetProtocolInfo", &[]);
    assert!(body.contains("<Sink></Sink>"));
    for forbidden in ["/Users", ".mp4", ".mkv", "folder", "path"] {
        assert!(
            !body.contains(forbidden),
            "the answer must not name `{forbidden}`"
        );
    }
}
