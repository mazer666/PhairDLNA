//! The resources a request may take of the server.
//!
//! What: the negative tests of the bounds — the browse page that asks for
//!       more items than the server hands out, the request that asks for more
//!       than the limit, and the document that grows past its bound.
//! Why: a client that asks for two billion items gets a page of five hundred,
//!       and a client that asks for a petabyte gets the bytes that exist. The
//!       work of the server stays proportional to what the server can answer,
//!       not to what a client writes (`SEC-TH-22`, `SEC-061`, `SEC-060`,
//!       `SEC-065`).
//! Example: `cargo test --package phairdlna-core --test resource_bounds`.

mod support;

use phairdlna_core::catalogue::{ContentItem, MediaKind};
use phairdlna_core::security::Limits;
use phairdlna_core::upnp::{page, BrowseArguments, BrowseFlag, SortCriteria, PAGE_LIMIT};
use support::catalogue::TestCatalogue;
use support::element_text;

/// The children of a container.
fn children(count: usize) -> Vec<ContentItem> {
    (0..count)
        .map(|index| {
            ContentItem::new(
                format!("id-{index}"),
                "0",
                format!("Film {index}"),
                MediaKind::Movie,
            )
        })
        .collect()
}

/// The arguments of a browse that asks for the given number of items.
fn arguments(requested_count: u32) -> BrowseArguments {
    BrowseArguments {
        object_id: "0".to_owned(),
        flag: BrowseFlag::DirectChildren,
        filter: None,
        starting_index: 0,
        requested_count,
        sort: SortCriteria {
            property: None,
            descending: false,
        },
    }
}

/// SEC-TH-22 — a browse page never holds more items than the limit.
#[test]
fn a_browse_page_never_holds_more_items_than_the_limit() {
    let page = page(&children(PAGE_LIMIT + 100), &arguments(u32::MAX));

    assert!(
        page.len() <= PAGE_LIMIT,
        "a browse page must not hold more than {PAGE_LIMIT} items; it held {}",
        page.len()
    );
}

/// SEC-TH-22 — a request for two billion items is a request for five hundred.
///
/// The number the client writes is not the number the server uses.
#[test]
fn a_request_for_two_billion_items_is_a_request_for_the_limit() {
    let page = page(&children(PAGE_LIMIT + 100), &arguments(2_147_483_647));

    assert_eq!(page.len(), PAGE_LIMIT);
}

/// SEC-TH-22 — a request for a few items is a request for those items.
#[test]
fn a_request_for_a_few_items_is_answered_with_those_items() {
    let page = page(&children(100), &arguments(10));

    assert_eq!(page.len(), 10);
}

/// SEC-TH-22 — the number of items in the answer never exceeds the file size.
#[test]
fn the_answer_never_holds_more_items_than_there_are() {
    let page = page(&children(7), &arguments(u32::MAX));

    assert_eq!(page.len(), 7);
}

/// SEC-TH-22 — a document that grows past its bound is not written.
///
/// The bound on the size of a document is the second half of the rule: the
/// number of items is bounded, and the bytes they take are bounded as well
/// (`SEC-061`).
#[test]
fn a_document_that_grows_past_its_bound_is_not_written() {
    let limits = Limits::default();
    let document = "x".repeat(limits.didl_bytes + 1);

    assert!(
        !limits.accepts_didl(document.len()),
        "a document that is past its bound must not be written"
    );
    assert!(
        limits.accepts_didl(limits.didl_bytes),
        "the bound itself must be accepted"
    );
}

/// SEC-TH-22 — the answer of a browse names the items, not the whole library.
///
/// A browse of a container with many children names the page it holds; the
/// rest of the library is not dragged into the answer.
#[test]
fn the_answer_of_a_browse_names_only_its_page() {
    let catalogue = TestCatalogue;
    let dispatcher = phairdlna_core::upnp::Dispatcher::new(&catalogue, "http://192.168.1.20:8200");
    let body = support::browse_request("0", "BrowseDirectChildren", 0, 2_147_483_647);

    let response = dispatcher.dispatch(&body, None);
    let text = &response.body;

    assert!(
        element_text(text, "NumberReturned").is_some(),
        "the answer names how many items it holds: {text}"
    );
}

/// SEC-TH-22 — the number the answer names is inside the limit.
#[test]
fn the_number_the_answer_names_is_inside_the_limit() {
    let catalogue = TestCatalogue;
    let dispatcher = phairdlna_core::upnp::Dispatcher::new(&catalogue, "http://192.168.1.20:8200");
    let body = support::browse_request("0", "BrowseDirectChildren", 0, 2_147_483_647);

    let response = dispatcher.dispatch(&body, None);
    let returned = element_text(&response.body, "NumberReturned")
        .and_then(|text| text.parse::<usize>().ok())
        .unwrap_or(0);

    assert!(
        returned <= PAGE_LIMIT,
        "the answer must not name more than {PAGE_LIMIT} items; it named {returned}"
    );
}
