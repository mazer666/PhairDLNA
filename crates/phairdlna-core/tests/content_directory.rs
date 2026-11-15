//! The ContentDirectory service as a control point sees it.
//!
//! What: the acceptance criteria of the feature specification F-03 that the
//!       dispatcher can answer without a socket.
//! Why: the criteria are written against a catalogue with twelve children, a
//!       catalogue that cannot count its children and a catalogue that
//!       changes. The fixtures in `support` make those cases reproducible.
//! Example: `cargo test --package phairdlna-core --test content_directory`.

mod support;

use phairdlna_core::upnp::{Dispatcher, ServiceSpec};
use support::catalogue::{
    ChangingCatalogue, EmptyUncountableCatalogue, MediaCatalogue, TestCatalogue,
    UncountableCatalogue, BASE_URL, CHILD_COUNT,
};
use support::{assert_well_formed, browse_request, element_text, element_text_unescaped, envelope};

/// The dispatcher of the test catalogue.
fn dispatcher() -> Dispatcher<'static> {
    Dispatcher::new(&TestCatalogue, BASE_URL)
}

/// The dispatcher of the catalogue whose files carry metadata.
fn media_dispatcher() -> Dispatcher<'static> {
    Dispatcher::new(&MediaCatalogue, BASE_URL)
}

/// The answer of a request against the test catalogue.
fn answer(request: &str) -> String {
    dispatcher().dispatch(request, None).body
}

/// The answer of a browse of the container with the twelve children.
fn browse(flag: &str, starting_index: u32, requested_count: u32) -> String {
    answer(&browse_request("1", flag, starting_index, requested_count))
}

/// The browse answer carries the four arguments, each exactly once and in the
/// order of the specification.
#[test]
fn the_browse_answer_carries_the_four_arguments_in_order() {
    let body = browse("BrowseDirectChildren", 0, 0);
    let result = body.find("<Result>").expect("the result is present");
    let number = body.find("<NumberReturned>").expect("the count is present");
    let total = body.find("<TotalMatches>").expect("the total is present");
    let update = body
        .find("<UpdateID>")
        .expect("the update identifier is present");
    assert!(result < number && number < total && total < update);
    assert_eq!(body.matches("<Result>").count(), 1);
    assert_eq!(body.matches("<NumberReturned>").count(), 1);
    assert_eq!(body.matches("<TotalMatches>").count(), 1);
    assert_eq!(body.matches("<UpdateID>").count(), 1);
}

/// A metadata browse describes the container itself.
#[test]
fn a_metadata_browse_describes_the_container() {
    let body = browse("BrowseMetadata", 0, 0);
    assert_eq!(element_text(&body, "NumberReturned").as_deref(), Some("1"));
    assert_eq!(element_text(&body, "TotalMatches").as_deref(), Some("1"));
    let result = element_text_unescaped(&body, "Result").expect("the result is present");
    assert!(result.contains("<container id=\"1\" parentID=\"0\""));
    assert!(result.contains("<dc:title>Content Directory</dc:title>"));
}

/// A browse with an offset returns the rest of the children.
#[test]
fn a_browse_with_an_offset_returns_the_rest() {
    let body = browse("BrowseDirectChildren", 5, 0);
    assert_eq!(element_text(&body, "TotalMatches").as_deref(), Some("12"));
    let number = element_text(&body, "NumberReturned").expect("the count is present");
    let number: u32 = number.parse().expect("the count is a number");
    assert_eq!(number as usize, CHILD_COUNT - 5);
    assert!(number <= 12, "the count does not exceed the total");
}

/// A browse with a limit returns at most that many children.
#[test]
fn a_browse_with_a_limit_returns_at_most_that_many() {
    let body = browse("BrowseDirectChildren", 0, 5);
    assert_eq!(element_text(&body, "NumberReturned").as_deref(), Some("5"));
    assert_eq!(element_text(&body, "TotalMatches").as_deref(), Some("12"));
    let result = element_text_unescaped(&body, "Result").expect("the result is present");
    assert_eq!(result.matches("<item ").count(), 5);
}

/// A browse without a limit returns every child the page limit allows, and the
/// total stays the true total.
#[test]
fn a_browse_without_a_limit_returns_every_child() {
    let body = browse("BrowseDirectChildren", 0, 0);
    assert_eq!(element_text(&body, "NumberReturned").as_deref(), Some("12"));
    assert_eq!(element_text(&body, "TotalMatches").as_deref(), Some("12"));
}

/// The wildcard filter returns the full property set, a known filter only that
/// property, and an unknown filter a complete answer.
///
/// The two mandatory properties — `dc:title` and `upnp:class` — survive every
/// filter, because the schema makes them mandatory (`INT-066`); the filter
/// narrows the other properties.
#[test]
fn the_filter_narrows_the_answer_without_failing() {
    let full = element_text_unescaped(&browse("BrowseDirectChildren", 0, 1), "Result")
        .expect("the result is present");
    assert!(full.contains("<dc:title>"));
    assert!(full.contains("<upnp:class>"));

    let narrow = media_dispatcher()
        .dispatch(
            &envelope(
                support::CONTENT_DIRECTORY,
                "Browse",
                &[
                    ("ObjectID", "1"),
                    ("BrowseFlag", "BrowseDirectChildren"),
                    ("Filter", "dc:title"),
                    ("StartingIndex", "0"),
                    ("RequestedCount", "1"),
                    ("SortCriteria", ""),
                ],
            ),
            None,
        )
        .body;
    let narrow = element_text_unescaped(&narrow, "Result").expect("the result is present");
    assert!(narrow.contains("<dc:title>Nightfall</dc:title>"));
    assert!(
        narrow.contains("<upnp:class>object.item.audioItem.musicTrack</upnp:class>"),
        "the class is mandatory and survives the filter"
    );
    assert!(
        !narrow.contains("<upnp:artist>"),
        "the artist was not asked for"
    );
    assert!(!narrow.contains("<res "), "the resource was not asked for");

    let unknown = answer(&envelope(
        support::CONTENT_DIRECTORY,
        "Browse",
        &[
            ("ObjectID", "1"),
            ("BrowseFlag", "BrowseDirectChildren"),
            ("Filter", "upnp:unknownProperty"),
            ("StartingIndex", "0"),
            ("RequestedCount", "1"),
            ("SortCriteria", ""),
        ],
    ));
    assert!(
        !unknown.contains("<errorCode>"),
        "the answer must not be a fault"
    );
    assert_eq!(
        element_text(&unknown, "NumberReturned").as_deref(),
        Some("1")
    );
}

/// The identifier of the catalogue is the one the client reads, and it grows
/// when the catalogue changes.
#[test]
fn the_system_update_identifier_grows_with_the_catalogue() {
    let before = Dispatcher::new(&ChangingCatalogue::new(5), BASE_URL).dispatch(
        &envelope(support::CONTENT_DIRECTORY, "GetSystemUpdateID", &[]),
        None,
    );
    let after = Dispatcher::new(&ChangingCatalogue::new(6), BASE_URL).dispatch(
        &envelope(support::CONTENT_DIRECTORY, "GetSystemUpdateID", &[]),
        None,
    );
    let before = element_text(&before.body, "Id").expect("the identifier is present");
    let after = element_text(&after.body, "Id").expect("the identifier is present");
    assert_eq!(before, "5");
    assert_eq!(after, "6");
    assert!(after.parse::<u32>().expect("a number") >= before.parse::<u32>().expect("a number"));
}

/// A browse of the root container reports the identifier of the whole
/// catalogue.
#[test]
fn a_browse_of_the_root_reports_the_system_update_identifier() {
    let body = answer(&browse_request("0", "BrowseMetadata", 0, 0));
    assert_eq!(element_text(&body, "UpdateID").as_deref(), Some("42"));
}

/// The root container is `0` and the content directory root is `1`.
#[test]
fn the_root_and_the_content_directory_root_are_known() {
    let root = answer(&browse_request("0", "BrowseMetadata", 0, 0));
    let root = element_text_unescaped(&root, "Result").expect("the result is present");
    assert!(root.contains("<container id=\"0\" parentID=\"-1\""));

    let directory = answer(&browse_request("1", "BrowseMetadata", 0, 0));
    let directory = element_text_unescaped(&directory, "Result").expect("the result is present");
    assert!(directory.contains("<container id=\"1\" parentID=\"0\""));
}

/// An object the server does not know is the fault `701`, never an empty
/// answer.
#[test]
fn an_unknown_object_is_the_fault_701() {
    let body = answer(&browse_request("999", "BrowseDirectChildren", 0, 0));
    assert!(body.contains("<errorCode>701</errorCode>"));
    assert!(body.contains("<errorDescription>No Such Object</errorDescription>"));
}

/// The write actions are absent from the service description, and the
/// dispatcher does not answer them.
#[test]
fn the_write_actions_are_absent_and_not_answered() {
    let scpd = ServiceSpec::content_directory().to_xml();
    for action in [
        "CreateObject",
        "DestroyObject",
        "UpdateObject",
        "MoveObject",
        "ImportResource",
        "ExportResource",
        "DeleteResource",
    ] {
        assert!(!scpd.contains(&format!("<name>{action}</name>")));
        let body = answer(&envelope(support::CONTENT_DIRECTORY, action, &[]));
        assert!(
            body.contains("<errorCode>401</errorCode>"),
            "{action} must be refused"
        );
    }
}

/// The result is a DIDL-Lite document that parses on its own, and the answer
/// carries it escaped.
#[test]
fn the_result_is_a_didl_lite_document_that_parses_on_its_own() {
    let body = browse("BrowseDirectChildren", 0, 1);
    let result = element_text_unescaped(&body, "Result").expect("the result is present");
    assert_well_formed(&result);
    assert!(result.contains("xmlns=\"urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/\""));
    assert!(!result.contains("writeStatus"));
}

/// The raw result element carries the document escaped, not as a literal.
#[test]
fn the_raw_result_element_carries_the_document_escaped() {
    let body = browse("BrowseDirectChildren", 0, 1);
    assert!(body.contains("&lt;DIDL-Lite"));
    assert!(!body.contains("<DIDL-Lite"));
}

/// A container whose count cannot be computed answers with a total of `0` and
/// a count that is not, and with the error `720` when it returns nothing.
#[test]
fn an_uncountable_container_answers_with_zero_or_720() {
    let body = Dispatcher::new(&UncountableCatalogue, BASE_URL)
        .dispatch(&browse_request("1", "BrowseDirectChildren", 0, 0), None);
    let body = body.body;
    assert_eq!(element_text(&body, "TotalMatches").as_deref(), Some("0"));
    assert_eq!(element_text(&body, "NumberReturned").as_deref(), Some("1"));

    let body = Dispatcher::new(&EmptyUncountableCatalogue, BASE_URL)
        .dispatch(&browse_request("1", "BrowseDirectChildren", 0, 0), None);
    let body = body.body;
    assert!(body.contains("<errorCode>720</errorCode>"));
}

/// No non-standard action is advertised, and none of them is answered.
#[test]
fn no_non_standard_action_is_answered() {
    let scpd = ServiceSpec::content_directory().to_xml();
    for action in [
        "GetFreeSpace",
        "GetUpdateCounter",
        "GetObjectIDfromPath",
        "GetParentID",
        "GetTransferTotal",
        "GetShareIndexInBrowse",
    ] {
        assert!(!scpd.contains(&format!("<name>{action}</name>")));
        let body = answer(&envelope(support::CONTENT_DIRECTORY, action, &[]));
        assert!(
            body.contains("<errorCode>401</errorCode>"),
            "{action} must be refused"
        );
    }
}
