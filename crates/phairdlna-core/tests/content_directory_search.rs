//! The search, the capabilities and the order of a ContentDirectory answer.
//!
//! What: the acceptance criteria of F-03 that concern `Search`, the
//!       capability lists and the sort criteria.
//! Why: a client decides what it can ask for by reading the capability lists,
//!       and a server that advertises a property it cannot search in sends the
//!       client down a path that ends in an error (`INT-035`, `INT-047`).
//! Example: `cargo test --package phairdlna-core --test content_directory_search`.

mod support;

use phairdlna_core::upnp::{Dispatcher, SEARCHABLE_PROPERTIES, SORTABLE_PROPERTIES};
use support::catalogue::TestCatalogue;
use support::{element_text, element_text_unescaped, envelope, CONTENT_DIRECTORY};

/// The answer of a request against the test catalogue.
fn answer(request: &str) -> String {
    Dispatcher::new(&TestCatalogue).dispatch(request, None).body
}

/// A browse request with the given order.
fn browse_request(criteria: &str) -> String {
    envelope(
        CONTENT_DIRECTORY,
        "Browse",
        &[
            ("ObjectID", "1"),
            ("BrowseFlag", "BrowseDirectChildren"),
            ("Filter", "dc:title"),
            ("StartingIndex", "0"),
            ("RequestedCount", "3"),
            ("SortCriteria", criteria),
        ],
    )
}

/// The capabilities are comma-separated lists of the properties the server
/// really searches and sorts by.
#[test]
fn the_capabilities_name_the_properties_the_server_uses() {
    let search = answer(&envelope(
        support::CONTENT_DIRECTORY,
        "GetSearchCapabilities",
        &[],
    ));
    let sort = answer(&envelope(
        support::CONTENT_DIRECTORY,
        "GetSortCapabilities",
        &[],
    ));
    let search_caps = element_text(&search, "SearchCaps").expect("the capabilities are present");
    let sort_caps = element_text(&sort, "SortCaps").expect("the capabilities are present");

    assert!(!search_caps.is_empty());
    assert!(!sort_caps.is_empty());
    assert!(!search_caps.contains(' '), "the list has no spaces");
    assert!(!sort_caps.contains(' '), "the list has no spaces");

    // Every advertised property is one the server really searches or sorts
    // by, and every property it can search or sort by is advertised.
    for property in search_caps.split(',') {
        assert!(
            SEARCHABLE_PROPERTIES.contains(&property),
            "{property} is advertised, but the server cannot search in it"
        );
    }
    for property in sort_caps.split(',') {
        assert!(
            SORTABLE_PROPERTIES.contains(&property),
            "{property} is advertised, but the server cannot sort by it"
        );
    }
    for property in SEARCHABLE_PROPERTIES {
        assert!(
            search_caps
                .split(',')
                .any(|advertised| advertised == property),
            "{property} can be searched in, but it is not advertised"
        );
    }
    for property in SORTABLE_PROPERTIES {
        assert!(
            sort_caps
                .split(',')
                .any(|advertised| advertised == property),
            "{property} can be sorted by, but it is not advertised"
        );
    }
}

/// A search finds the item whose title contains the text.
#[test]
fn a_search_finds_the_matching_item() {
    let body = answer(&envelope(
        support::CONTENT_DIRECTORY,
        "Search",
        &[
            ("ContainerID", "1"),
            ("SearchCriteria", "dc:title contains \"Item 03\""),
            ("Filter", "*"),
            ("StartingIndex", "0"),
            ("RequestedCount", "0"),
            ("SortCriteria", ""),
        ],
    ));
    assert!(
        !body.contains("<errorCode>"),
        "the answer must not be a fault"
    );
    assert_eq!(element_text(&body, "NumberReturned").as_deref(), Some("1"));
    assert_eq!(element_text(&body, "TotalMatches").as_deref(), Some("1"));
    let result = element_text_unescaped(&body, "Result").expect("the result is present");
    assert!(result.contains("<dc:title>Item 03</dc:title>"));
}

/// A search in a property the server does not advertise is the fault `402`.
#[test]
fn a_search_in_an_unknown_property_is_refused() {
    let body = answer(&envelope(
        support::CONTENT_DIRECTORY,
        "Search",
        &[
            ("ContainerID", "1"),
            ("SearchCriteria", "dc:creator contains \"x\""),
            ("Filter", "*"),
            ("StartingIndex", "0"),
            ("RequestedCount", "0"),
            ("SortCriteria", ""),
        ],
    ));
    assert!(body.contains("<errorCode>402</errorCode>"));
}

/// The empty sort criteria returns the order of the catalogue, `+dc:title`
/// sorts ascending and `-dc:title` descending.
#[test]
fn the_sort_criteria_order_the_answer() {
    let request = |criteria: &str| answer(&browse_request(criteria));
    let titles = |body: &str| -> Vec<String> {
        let result = element_text_unescaped(body, "Result").expect("the result is present");
        result
            .split("<dc:title>")
            .skip(1)
            .map(|title| {
                title
                    .split("</dc:title>")
                    .next()
                    .unwrap_or_default()
                    .to_owned()
            })
            .collect()
    };

    let own = titles(&request(""));
    assert_eq!(
        own.first().map(String::as_str),
        Some("Item 12"),
        "the catalogue order"
    );
    let ascending = titles(&request("+dc:title"));
    assert_eq!(
        ascending.first().map(String::as_str),
        Some("Item 01"),
        "ascending"
    );
    let descending = titles(&request("-dc:title"));
    assert_eq!(
        descending.first().map(String::as_str),
        Some("Item 12"),
        "descending"
    );
}

/// A sort criterion without a sign is refused, because the server must not
/// guess an order the client did not ask for.
#[test]
fn a_sort_criterion_without_a_sign_is_refused() {
    let body = answer(&browse_request("dc:title"));
    assert!(body.contains("<errorCode>402</errorCode>"));
    assert!(body.contains("<errorDescription>Invalid Args</errorDescription>"));
}

/// A filter that names only unknown properties does not become the wildcard:
/// the item carries only its mandatory properties.
#[test]
fn an_unknown_filter_does_not_widen_the_answer() {
    let body = answer(&envelope(
        support::CONTENT_DIRECTORY,
        "Browse",
        &[
            ("ObjectID", "1"),
            ("BrowseFlag", "BrowseDirectChildren"),
            ("Filter", "dc:creator"),
            ("StartingIndex", "0"),
            ("RequestedCount", "1"),
            ("SortCriteria", ""),
        ],
    ));
    assert!(
        !body.contains("<errorCode>"),
        "the answer must not be a fault"
    );
    let result = element_text_unescaped(&body, "Result").expect("the result is present");
    assert!(
        !result.contains("<dc:title>"),
        "the title was not asked for"
    );
    assert!(
        !result.contains("<upnp:class>"),
        "the class was not asked for"
    );
    assert!(
        result.contains("<restricted>1</restricted>"),
        "the mandatory property is there"
    );
}

/// A search is paged like a browse, so that the answer stays small
/// (`SEC-061`).
#[test]
fn a_search_returns_at_most_the_page_limit() {
    let body = answer(&envelope(
        support::CONTENT_DIRECTORY,
        "Search",
        &[
            ("ContainerID", "1"),
            ("SearchCriteria", "dc:title contains \"Item\""),
            ("Filter", "*"),
            ("StartingIndex", "0"),
            ("RequestedCount", "2"),
            ("SortCriteria", ""),
        ],
    ));
    assert_eq!(element_text(&body, "NumberReturned").as_deref(), Some("2"));
    let total: u32 = element_text(&body, "TotalMatches")
        .expect("the total is present")
        .parse()
        .expect("the total is a number");
    assert!(total >= 2, "the total is the number of matches");
    let result = element_text_unescaped(&body, "Result").expect("the result is present");
    assert_eq!(result.matches("<item ").count(), 2);
}
