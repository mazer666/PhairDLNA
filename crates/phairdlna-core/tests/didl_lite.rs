//! The DIDL-Lite document as a control point reads it.
//!
//! What: the acceptance criteria of the feature specification F-05 that the
//!       document can answer without a socket: the shape of the document, the
//!       class of an object, the length policy, the artwork address and the
//!       metadata elements. The resource of an item and the two modes of the
//!       `protocolInfo` have their own file, `didl_resource.rs`.
//! Why: the document is what a television shows in its list of files. A
//!       document that breaks the schema is rejected by the client, and a
//!       document that lies about a format makes the client fetch bytes the
//!       server cannot deliver.
//! Example: `cargo test --package phairdlna-core --test didl_lite`.

mod support;

use phairdlna_core::catalogue::{ContentItem, MediaKind, MediaMetadata};
use phairdlna_core::didl::{
    item_from_catalogue, render, Container, DidlDocument, ProtocolPolicy, MAX_VALUE_BYTES,
    TRUNCATION_MARKER,
};
use support::catalogue::{movie, music_track, photo, BASE_URL};
use support::didl::{attribute, declared_prefixes, parse, used_prefixes};

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

/// AC-01 — the root element is `DIDL-Lite` in the namespace of the schema,
/// every prefix the document uses is declared on it, and every object carries
/// the attributes the schema requires.
#[test]
fn the_document_has_the_shape_the_schema_asks_for() {
    let container = Container::new("1", "0", "Music", 12);
    let xml = render(&DidlDocument::with_containers(vec![container]), None);
    support::assert_well_formed(&xml);

    assert!(xml.starts_with("<DIDL-Lite"));
    assert!(xml.contains("xmlns=\"urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/\""));
    for prefix in used_prefixes(&xml) {
        assert!(
            declared_prefixes(&xml).contains(&prefix),
            "the prefix `{prefix}` is used but not declared"
        );
    }
    assert!(xml.contains("<container id=\"1\" parentID=\"0\" restricted=\"1\""));
    assert!(xml.contains("childCount=\"12\""));

    // The document is read with the parser of the project: the root element,
    // its namespace, the namespace of a property and the attributes of the
    // container are the ones the schema asks for.
    let root = parse(&xml);
    assert_eq!(root.name, "DIDL-Lite");
    assert_eq!(
        root.namespace,
        "urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/"
    );

    let container = root.child("container").expect("the container is there");
    assert_eq!(attribute(container, "id").as_deref(), Some("1"));
    assert_eq!(attribute(container, "parentID").as_deref(), Some("0"));
    assert_eq!(attribute(container, "restricted").as_deref(), Some("1"));
    assert_eq!(attribute(container, "childCount").as_deref(), Some("12"));

    let title = container.child("title").expect("the title is there");
    assert_eq!(title.namespace, "http://purl.org/dc/elements/1.1/");
    assert_eq!(title.text, "Music");
    let class = container.child("class").expect("the class is there");
    assert_eq!(class.namespace, "urn:schemas-upnp-org:metadata-1-0/upnp/");
}

/// AC-01 — an item carries `id`, `parentID` and `restricted`.
#[test]
fn an_item_carries_the_required_attributes() {
    let xml = document_of(vec![music_track()]);
    support::assert_well_formed(&xml);
    assert!(xml.contains("<item id=\"a3f1\" parentID=\"1\" restricted=\"1\">"));

    // The two mandatory properties are on the item as well (`INT-066`).
    let root = parse(&xml);
    let item = root.child("item").expect("the item is there");
    assert_eq!(attribute(item, "id").as_deref(), Some("a3f1"));
    assert_eq!(attribute(item, "parentID").as_deref(), Some("1"));
    assert_eq!(attribute(item, "restricted").as_deref(), Some("1"));
    assert_eq!(
        item.child_text("title"),
        Some("Nightfall"),
        "the title is mandatory"
    );
    assert_eq!(
        item.child_text("class"),
        Some("object.item.audioItem.musicTrack"),
        "the class is mandatory"
    );
}

/// AC-02 — the classes of the schema are used, and the identifier of an
/// object is opaque and stable.
#[test]
fn the_classes_are_the_ones_of_the_schema() {
    let xml = document_of(vec![music_track(), movie(), photo()]);
    assert!(xml.contains("<upnp:class>object.item.audioItem.musicTrack</upnp:class>"));
    assert!(xml.contains("<upnp:class>object.item.videoItem.movie</upnp:class>"));
    assert!(xml.contains("<upnp:class>object.item.imageItem.photo</upnp:class>"));

    let container = render(
        &DidlDocument::with_containers(vec![Container::new("1", "0", "Music", 0)]),
        None,
    );
    assert!(container.contains("<upnp:class>object.container.storageFolder</upnp:class>"));
}

/// AC-02 — the identifier of an object does not change between two renderings
/// and carries no path separator.
#[test]
fn the_identifier_is_stable_and_carries_no_path() {
    let first = document_of(vec![music_track()]);
    let second = document_of(vec![music_track()]);
    assert_eq!(first, second, "the same file yields the same document");

    let identifier = first
        .split("id=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .unwrap_or_default();
    assert_eq!(identifier, "a3f1");
    assert!(!identifier.contains('/'));
    assert!(!identifier.contains('\\'));
}

/// AC-05 — a title of twenty thousand characters is cut so that the escaped
/// value fits into the budget, and the cut is marked.
#[test]
fn a_very_long_title_is_cut_and_marked() {
    // The title is built from a character that grows when it is escaped, so
    // the cut is proven on the escaped form and not on the raw one.
    let mut item = ContentItem::new("a3f1", "1", "&".repeat(20_000), MediaKind::Movie);
    item.metadata = MediaMetadata {
        mime_type: Some("video/mp4".to_owned()),
        ..MediaMetadata::default()
    };
    let xml = document_of(vec![item]);
    let title = xml
        .split("<dc:title>")
        .nth(1)
        .and_then(|rest| rest.split("</dc:title>").next())
        .unwrap_or_default();
    assert!(
        title.len() <= MAX_VALUE_BYTES,
        "the title is {} bytes",
        title.len()
    );
    assert!(title.ends_with(TRUNCATION_MARKER), "the cut is not marked");

    // The value the client reads is the cut value, and it is still one value.
    let root = parse(&xml);
    let item = root.child("item").expect("the item is there");
    let text = item.child_text("title").expect("the title is there");
    assert!(text.ends_with(TRUNCATION_MARKER));
    assert!(text.len() <= MAX_VALUE_BYTES);
}

/// AC-06 — the artwork address is an address of this server, and it carries
/// neither a `dlna:profileID` attribute nor a `dlna` namespace.
#[test]
fn the_artwork_address_carries_no_dlna_attribute() {
    let xml = document_of(vec![music_track()]);
    assert!(
        xml.contains("<upnp:albumArtURI>http://192.168.1.20:8200/art/art-a3f1</upnp:albumArtURI>")
    );
    assert!(!xml.contains("dlna:profileID"));
    assert!(!xml.contains("xmlns:dlna"));
}

/// AC-07 — the date, the artist, the creator and the track number carry the
/// values of the tags.
#[test]
fn the_metadata_elements_carry_the_values_of_the_tags() {
    let xml = document_of(vec![music_track()]);
    assert!(xml.contains("<dc:date>2024-05-28T02:28:47</dc:date>"));
    assert!(xml.contains("<upnp:artist>Nightfall Orchestra</upnp:artist>"));
    assert!(xml.contains("<dc:creator>Nightfall Orchestra</dc:creator>"));
    assert!(xml.contains("<upnp:album>Long Nights</upnp:album>"));
    assert!(xml.contains("<upnp:genre>Ambient</upnp:genre>"));
    assert!(xml.contains("<upnp:originalTrackNumber>3</upnp:originalTrackNumber>"));
}

/// AC-07 — a date the server cannot read is left out rather than guessed.
#[test]
fn an_unreadable_date_is_left_out() {
    let mut item = music_track();
    item.metadata.date = Some("28.05.2024".to_owned());
    let xml = document_of(vec![item]);
    assert!(!xml.contains("<dc:date>"));
}
