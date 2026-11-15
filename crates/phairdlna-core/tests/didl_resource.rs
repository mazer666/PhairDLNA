//! The resource of an item and the protocol it is delivered with.
//!
//! What: the acceptance criteria of F-05 that concern the `res` element and
//!       its `protocolInfo`: the attributes of the resource, the format of the
//!       duration and of the resolution, the two signalling modes and the
//!       claims the server does not make.
//! Why: the resource is what the client fetches. A resource that names a
//!       format the server cannot deliver, or that promises an operation the
//!       server does not support, makes the client fail in a way the user
//!       cannot repair.
//! Example: `cargo test --package phairdlna-core --test didl_resource`.

mod support;

use phairdlna_core::catalogue::ContentItem;
use phairdlna_core::didl::{item_from_catalogue, render, DidlDocument, DlnaFlags, ProtocolPolicy};
use support::catalogue::{hostile_item, movie, music_track, photo, BASE_URL};

/// The plain mode, which is the mode the server starts in.
fn plain() -> ProtocolPolicy {
    ProtocolPolicy::plain()
}

/// The document of the given items.
fn document_of(items: Vec<ContentItem>) -> String {
    let mapped: Vec<_> = items
        .iter()
        .map(|item| item_from_catalogue(item, BASE_URL, &plain()))
        .collect();
    render(&DidlDocument::with_items(mapped), None)
}

/// AC-03 — the resource of an item carries the attributes of the schema, and
/// the protocol names a format the server really serves.
#[test]
fn the_resource_carries_the_attributes_of_the_schema() {
    let xml = document_of(vec![music_track()]);
    assert!(xml.contains("protocolInfo=\"http-get:*:audio/mpeg:*\""));
    assert!(xml.contains("size=\"7340032\""));
    assert!(xml.contains("duration=\"0:03:52.000\""));
    assert!(xml.contains("bitrate=\"31744\""));
    assert!(xml.contains("sampleFrequency=\"44100\""));
    assert!(xml.contains("nrAudioChannels=\"2\""));
    assert!(xml.contains("bitsPerSample=\"16\""));
    assert!(xml.contains(">http://192.168.1.20:8200/media/a3f1</res>"));
}

/// AC-03 — the duration has the format `H*:MM:SS.F*` and the resolution the
/// format `<width>x<height>`.
#[test]
fn the_duration_and_the_resolution_have_the_format_of_the_schema() {
    let xml = document_of(vec![music_track(), photo()]);
    assert!(xml.contains("duration=\"0:03:52.000\""));
    assert!(xml.contains("resolution=\"1920x1080\""));

    let duration = xml
        .split("duration=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .unwrap_or_default();
    let parts: Vec<&str> = duration.split(':').collect();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[1].len(), 2, "the minutes have two digits");
    assert_eq!(
        parts[2].len(),
        6,
        "the seconds have two digits and a fraction"
    );
}

/// AC-03 — the third field of the protocol names a format of the registry.
#[test]
fn the_protocol_names_a_format_the_server_serves() {
    let xml = document_of(vec![music_track(), movie(), photo()]);
    for mime_type in ["audio/mpeg", "video/mp4", "image/jpeg"] {
        assert!(
            xml.contains(&format!("protocolInfo=\"http-get:*:{mime_type}:*\"")),
            "{mime_type} is missing"
        );
    }
    for field in xml
        .split("protocolInfo=\"")
        .skip(1)
        .map(|rest| rest.split('"').next().unwrap_or_default())
    {
        assert_eq!(field.split(':').count(), 4, "{field} has not four fields");
    }
}

/// AC-08 — the plain mode and the DLNA mode describe the same file, and the
/// DLNA mode carries the four tokens.
#[test]
fn the_two_modes_describe_the_same_file() {
    let mut item = music_track();
    item.metadata.dlna_profile = Some("MP3".to_owned());
    let plain = document_of(vec![item.clone()]);
    let policy = ProtocolPolicy::dlna(true, DlnaFlags::BYTE_BASED_SEEK);
    let dlna = render(
        &DidlDocument::with_items(vec![item_from_catalogue(&item, BASE_URL, &policy)]),
        None,
    );

    assert!(plain.contains("protocolInfo=\"http-get:*:audio/mpeg:*\""));
    assert!(!plain.contains("DLNA."));

    assert!(dlna.contains("DLNA.ORG_PN="));
    assert!(dlna.contains("DLNA.ORG_OP=01"));
    assert!(dlna.contains("DLNA.ORG_CI=0"));
    assert!(dlna.contains("DLNA.ORG_FLAGS="));
    support::assert_well_formed(&dlna);
}

/// AC-09 — no profile is claimed without one, and the byte-range operation
/// only when the server can seek by byte range.
#[test]
fn nothing_is_claimed_that_the_server_cannot_prove() {
    let mut item = music_track();
    item.metadata.dlna_profile = None;

    let without_profile = ProtocolPolicy::dlna(true, DlnaFlags::NONE);
    let xml = render(
        &DidlDocument::with_items(vec![item_from_catalogue(&item, BASE_URL, &without_profile)]),
        None,
    );
    assert!(!xml.contains("DLNA.ORG_PN"));
    assert!(xml.contains("DLNA.ORG_OP=01"));

    let without_seek = ProtocolPolicy::dlna(false, DlnaFlags::NONE);
    let xml = render(
        &DidlDocument::with_items(vec![item_from_catalogue(&item, BASE_URL, &without_seek)]),
        None,
    );
    assert!(!xml.contains("DLNA.ORG_OP=01"));

    assert!(!document_of(vec![item]).contains("writeStatus"));
}

/// AC-09 — the document never carries a `writeStatus` attribute.
#[test]
fn the_document_never_carries_a_write_status() {
    let xml = document_of(vec![music_track(), movie(), photo(), hostile_item()]);
    assert!(!xml.contains("writeStatus"));
}
