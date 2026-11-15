//! The escaping of the document and of its values.
//!
//! What: the acceptance criterion AC-04 of F-05 — the escaping of the five
//!       characters XML reserves, in element text and in attribute values,
//!       and the round trip through the `Result` element of a SOAP answer.
//! Why: every value in the document comes from a file name or a tag, and both
//!       are untrusted input (`SEC-007`). A title of `</dc:title><evil/>`
//!       must not be able to close the element it lives in (`INT-065`), and a
//!       document that is carried as the text of another document must survive
//!       the second escaping unchanged (`INT-073`).
//! Example: `cargo test --package phairdlna-core --test didl_escaping`.

mod support;

use phairdlna_core::catalogue::{ContentItem, MediaKind, MediaMetadata};
use phairdlna_core::didl::{
    item_from_catalogue, render, render_for_soap_result, DidlDocument, ProtocolPolicy,
};
use support::catalogue::{MediaCatalogue, BASE_URL};
use support::didl::parse;
use support::{assert_well_formed, element_text, element_text_unescaped, unescape_once};

/// The title the client reads from a document.
///
/// The reader of the project decodes the entities, so the value it returns is
/// the value the client sees — not the escaped form.
fn title_read_by_a_client(xml: &str) -> String {
    let root = parse(xml);
    let item = root.child("item").expect("the item is there");
    item.child_text("title")
        .expect("the title is there")
        .to_owned()
}

/// The five characters XML reserves.
const RESERVED: [&str; 5] = ["&", "<", ">", "\"", "'"];

/// The characters a random value is built from.
///
/// The pool carries the five reserved characters, letters and two characters
/// outside the basic plane, so that a cut on a character boundary is tested
/// as well.
const ALPHABET: [&str; 12] = ["a", "b", "Z", "0", " ", "&", "<", ">", "\"", "'", "ä", "😀"];

/// A tiny deterministic generator.
///
/// The project has no property-testing dependency, and adding one would cost
/// a licence assessment (qa-performance §2.6). The generator is deterministic
/// instead, so that a failure can be reproduced exactly.
struct Random(u64);

impl Random {
    /// The next number of the sequence.
    fn next(&mut self) -> u64 {
        let mut value = self.0;
        value ^= value << 13;
        value ^= value >> 7;
        value ^= value << 17;
        self.0 = value;
        value
    }

    /// A number below the given limit.
    fn below(&mut self, limit: usize) -> usize {
        (self.next() % limit as u64) as usize
    }
}

/// A random value of the given length.
fn random_value(random: &mut Random, length: usize) -> String {
    let mut value = String::new();
    for _ in 0..length {
        value.push_str(ALPHABET[random.below(ALPHABET.len())]);
    }
    value
}

/// The document of an item with the given title and identifier.
fn document_of(title: &str, id: &str) -> String {
    let mut item = ContentItem::new(id, "1", title, MediaKind::Movie);
    item.metadata = MediaMetadata {
        mime_type: Some("video/mp4".to_owned()),
        ..MediaMetadata::default()
    };
    let mapped = item_from_catalogue(&item, BASE_URL, &ProtocolPolicy::plain());
    render(&DidlDocument::with_items(vec![mapped]), None)
}

/// The text of the title element of a document.
fn title_of(xml: &str) -> String {
    xml.split("<dc:title>")
        .nth(1)
        .and_then(|rest| rest.split("</dc:title>").next())
        .unwrap_or_default()
        .to_owned()
}

/// The value of the identifier attribute of a document.
fn id_of(xml: &str) -> String {
    xml.split("id=\"")
        .nth(1)
        .and_then(|rest| rest.split('"').next())
        .unwrap_or_default()
        .to_owned()
}

/// AC-04 — the reserved characters are escaped in the element text, and the
/// document stays well-formed.
#[test]
fn the_reserved_characters_are_escaped_in_the_text() {
    let title = "a & b < c > d \" e ' f";
    let xml = document_of(title, "a3f1");
    assert_well_formed(&xml);

    let text = title_of(&xml);
    assert!(text.contains("&amp;"));
    assert!(text.contains("&lt;"));
    assert!(text.contains("&gt;"));
    assert!(!text.contains('<'));
    assert_eq!(unescape_once(&text), title);
    assert_eq!(title_read_by_a_client(&xml), title);
}

/// AC-04 — the five reserved characters are escaped in an attribute value.
#[test]
fn the_reserved_characters_are_escaped_in_an_attribute() {
    let identifier = "a&b<c>d\"e'f";
    let xml = document_of("A title", identifier);
    assert_well_formed(&xml);

    let value = id_of(&xml);
    assert!(value.contains("&amp;"));
    assert!(value.contains("&lt;"));
    assert!(value.contains("&gt;"));
    assert!(value.contains("&quot;"));
    assert!(value.contains("&apos;"));
    assert_eq!(unescape_once(&value), identifier);

    let root = parse(&xml);
    let item = root.child("item").expect("the item is there");
    let read = item
        .attributes
        .iter()
        .find(|(name, _)| name == "id")
        .map(|(_, value)| value.clone());
    assert_eq!(read.as_deref(), Some(identifier));
}

/// AC-04 — a title that tries to close its element cannot do it.
#[test]
fn a_title_cannot_close_its_element() {
    let xml = document_of("</dc:title><evil/>", "a3f1");
    assert_well_formed(&xml);
    assert!(!xml.contains("<evil/>"));
    assert_eq!(title_of(&xml), "&lt;/dc:title&gt;&lt;evil/&gt;");
}

/// AC-04 — the document survives the second escaping unchanged: the answer of
/// a browse carries the document escaped once more, and the client recovers
/// the document by unescaping it.
#[test]
fn the_document_survives_the_second_escaping() {
    let mut item = ContentItem::new("a3f1", "1", "Nightfall & Dawn", MediaKind::Movie);
    item.metadata = MediaMetadata {
        mime_type: Some("video/mp4".to_owned()),
        ..MediaMetadata::default()
    };
    let mapped = item_from_catalogue(&item, BASE_URL, &ProtocolPolicy::plain());
    let document = render(&DidlDocument::with_items(vec![mapped.clone()]), None);
    let result = render_for_soap_result(&DidlDocument::with_items(vec![mapped]), None);
    assert!(result.starts_with("&lt;DIDL-Lite"));
    assert!(!result.contains("<DIDL-Lite"));
    assert_eq!(unescape_once(&result), document);
}

/// AC-04 — the round trip through the whole answer of a browse: the client
/// reads the `Result` element, unescapes it and gets the document back.
#[test]
fn the_document_survives_the_round_trip_through_a_browse() {
    let body = phairdlna_core::upnp::Dispatcher::new(&MediaCatalogue, BASE_URL)
        .dispatch(
            &support::browse_request("1", "BrowseDirectChildren", 0, 1),
            None,
        )
        .body;
    let result = element_text_unescaped(&body, "Result").expect("the result is present");
    assert_well_formed(&result);
    assert!(result.contains("<dc:title>Nightfall</dc:title>"));

    // The raw element carries the document escaped, and the document itself
    // carries the ampersand of a title escaped once more.
    let raw = element_text(&body, "Result").expect("the result is present");
    assert!(raw.contains("&lt;DIDL-Lite"));
}

/// The property test: a random value is rendered, and the value the client
/// reads is the value that was written.
#[test]
fn a_random_value_survives_the_round_trip() {
    let mut random = Random(0x5eed_1234_9abc_def0);
    for _ in 0..300 {
        let title_length = random.below(40);
        let title = random_value(&mut random, title_length);
        let identifier_length = random.below(20);
        let identifier = random_value(&mut random, identifier_length);
        let xml = document_of(&title, &identifier);
        assert_well_formed(&xml);
        assert_eq!(unescape_once(&title_of(&xml)), title);
        assert_eq!(unescape_once(&id_of(&xml)), identifier);
        assert_eq!(title_read_by_a_client(&xml), title);
    }
}

/// The property test: a random value never breaks out of the document.
#[test]
fn a_random_value_cannot_break_out_of_the_document() {
    let mut random = Random(0x1234_5678_9abc_def0);
    for _ in 0..300 {
        let title_length = random.below(60);
        let title = random_value(&mut random, title_length);
        let xml = document_of(&title, "a3f1");
        assert_well_formed(&xml);
        assert!(!xml.contains("<evil"));
        assert_eq!(xml.matches("<item ").count(), 1);
    }
}

/// The property test: a random value never grows the escaped form beyond the
/// budget.
#[test]
fn a_random_value_stays_inside_the_budget() {
    let mut random = Random(0xfeed_face_dead_beef);
    for _ in 0..100 {
        let title = random_value(&mut random, 20_000);
        let xml = document_of(&title, "a3f1");
        let text = title_of(&xml);
        assert!(
            text.len() <= phairdlna_core::didl::MAX_VALUE_BYTES,
            "the title is {} bytes",
            text.len()
        );
        let read = title_read_by_a_client(&xml);
        assert!(
            read.len() <= phairdlna_core::didl::MAX_VALUE_BYTES,
            "the title the client reads is {} bytes",
            read.len()
        );
    }
}

/// Every one of the five reserved characters is escaped, in the text and in
/// the attribute.
#[test]
fn every_reserved_character_is_escaped() {
    for reserved in RESERVED {
        let title = format!("before{reserved}after");
        let xml = document_of(&title, "a3f1");
        assert_well_formed(&xml);
        assert_eq!(unescape_once(&title_of(&xml)), title);
    }
}
