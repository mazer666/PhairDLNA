//! The page of an answer: the offset, the limit and the order.
//!
//! What: the arithmetic that cuts the page the client asked for out of the
//!       children of a container, and the order of the items on it.
//! Why: a client may ask for one item of a catalogue with ten thousand, and
//!       the answer must stay small (`SEC-061`). The page limit bounds the
//!       answer even when the client asks for all of them, and the order is
//!       applied to the items the catalogue returned, because the catalogue
//!       does not sort yet (`PERF-009`).
//! Example: `page` returns the items from the index the client asked for,
//!       never more than the page limit.

use crate::catalogue::ContentItem;
use crate::didl::class_of;
use crate::upnp::browse::{BrowseArguments, SortCriteria};

/// The largest number of items one answer carries (`SEC-061`).
pub const PAGE_LIMIT: usize = 500;

/// The children the client asked for: the page after the offset and the limit.
///
/// The page never holds more items than the page limit, even when the client
/// asks for all of them (`SEC-061`).
pub fn page(children: &[ContentItem], arguments: &BrowseArguments) -> Vec<ContentItem> {
    let start = (arguments.starting_index as usize).min(children.len());
    let limit = if arguments.requested_count == 0 {
        PAGE_LIMIT
    } else {
        (arguments.requested_count as usize).min(PAGE_LIMIT)
    };
    children.iter().skip(start).take(limit).cloned().collect()
}

/// Sorts the items in the order the client asked for.
///
/// The catalogue sorts in the index (PERF-009); until that exists the order
/// is applied here, over the items the catalogue returned.
pub(super) fn sort_items(items: &mut [ContentItem], sort: &SortCriteria) {
    let Some(property) = sort.property.as_deref() else {
        return;
    };
    items.sort_by(|left, right| {
        let order = property_of(left, property).cmp(&property_of(right, property));
        if sort.descending {
            order.reverse()
        } else {
            order
        }
    });
}

/// The value of a property of an item.
fn property_of(item: &ContentItem, property: &str) -> String {
    match property {
        "dc:title" => item.title.clone(),
        "upnp:class" => class_of(item.kind).as_str().to_owned(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::{page, PAGE_LIMIT};
    use crate::catalogue::{ContentItem, MediaKind};
    use crate::upnp::browse::{BrowseArguments, BrowseFlag, SortCriteria};

    /// A container with the given number of children.
    fn children(count: usize) -> Vec<ContentItem> {
        (0..count)
            .map(|index| {
                ContentItem::new(
                    index.to_string(),
                    "1",
                    format!("Item {index}"),
                    MediaKind::Movie,
                )
            })
            .collect()
    }

    /// The browse arguments of a client that asks for a page.
    fn arguments(starting_index: u32, requested_count: u32) -> BrowseArguments {
        BrowseArguments {
            object_id: "1".to_owned(),
            flag: BrowseFlag::DirectChildren,
            filter: None,
            starting_index,
            requested_count,
            sort: SortCriteria {
                property: None,
                descending: false,
            },
        }
    }

    /// The page starts at the index the client asked for.
    #[test]
    fn the_page_starts_at_the_index() {
        let page = page(&children(12), &arguments(5, 0));
        assert_eq!(page.len(), 7);
        assert_eq!(page[0].id, "5");
    }

    /// The page never holds more items than the client asked for.
    #[test]
    fn the_page_honours_the_limit() {
        assert_eq!(page(&children(12), &arguments(0, 5)).len(), 5);
    }

    /// The page never holds more items than the page limit, even when the
    /// client asks for all of them (`SEC-061`).
    #[test]
    fn the_page_never_exceeds_the_page_limit() {
        let page = page(&children(PAGE_LIMIT + 100), &arguments(0, 0));
        assert_eq!(page.len(), PAGE_LIMIT);
    }

    /// An index beyond the end of the children gives an empty page and not a
    /// panic.
    #[test]
    fn an_index_beyond_the_end_gives_an_empty_page() {
        assert!(page(&children(12), &arguments(99, 0)).is_empty());
    }
}
