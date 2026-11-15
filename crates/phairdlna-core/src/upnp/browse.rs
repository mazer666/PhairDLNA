//! The arguments of a ContentDirectory request.
//!
//! What: the reading of the arguments of `Browse` and `Search` — the object
//!       identifier, the browse flag, the filter, the page, the order and the
//!       search expression.
//! Why: every argument comes from the network and is therefore untrusted
//!       (`SEC-018`). An argument that is missing or malformed is the fault
//!       `402 Invalid Args` (`INT-025`), and an argument the server does not
//!       understand is ignored rather than refused, so that a client of a
//!       newer ContentDirectory version still gets an answer (`INT-034`).
//! Example: `read_browse_arguments` reads the six arguments of a browse.

use crate::upnp::content_directory::{
    KNOWN_PROPERTIES, SEARCHABLE_PROPERTIES, SORTABLE_PROPERTIES,
};
use crate::upnp::error::UpnpError;
use crate::upnp::soap::SoapRequest;

/// What a browse asks for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowseFlag {
    /// The metadata of the container itself.
    Metadata,
    /// The children of the container.
    DirectChildren,
}

/// How the answer is ordered.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SortCriteria {
    /// The property to order by, if the client asked for an order.
    pub property: Option<String>,
    /// Whether the order is descending.
    pub descending: bool,
}

/// The arguments of a browse request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BrowseArguments {
    /// The container the client wants to see.
    pub object_id: String,
    /// Whether the client wants the container or its children.
    pub flag: BrowseFlag,
    /// The properties the client asked for; `None` is the wildcard.
    pub filter: Option<Vec<String>>,
    /// The first child the client wants.
    pub starting_index: u32,
    /// The number of children the client wants, where `0` means all of them.
    pub requested_count: u32,
    /// The order the client asked for.
    pub sort: SortCriteria,
}

/// The expression of a search request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SearchCriteria {
    /// The property the client searches in.
    pub property: String,
    /// The text the client looks for.
    pub text: String,
}

/// Reads an unsigned argument of four bytes.
pub(super) fn read_ui4(value: &str, argument: &str) -> Result<u32, UpnpError> {
    value
        .trim()
        .parse::<u32>()
        .map_err(|_| UpnpError::InvalidArgument {
            argument: argument.to_owned(),
        })
}

/// Reads the arguments of a browse request.
///
/// An argument that is missing or malformed is the fault `402 Invalid Args`
/// (`INT-025`). A `SearchCriteria` element is accepted and ignored, because it
/// belongs to `Search` and to ContentDirectory:2 (`INT-030`).
pub fn read_browse_arguments(request: &SoapRequest) -> Result<BrowseArguments, UpnpError> {
    let object_id = request.required_argument("ObjectID")?.trim().to_owned();
    if object_id.is_empty() {
        return Err(UpnpError::InvalidArgument {
            argument: "ObjectID".to_owned(),
        });
    }
    Ok(BrowseArguments {
        object_id,
        flag: read_browse_flag(request.required_argument("BrowseFlag")?)?,
        filter: read_filter(request.required_argument("Filter")?),
        starting_index: read_ui4(request.required_argument("StartingIndex")?, "StartingIndex")?,
        requested_count: read_ui4(
            request.required_argument("RequestedCount")?,
            "RequestedCount",
        )?,
        sort: read_sort(request.required_argument("SortCriteria")?)?,
    })
}

/// Reads the browse flag.
///
/// The flag is a bit field, but most clients send the name of the flag. Both
/// spellings are understood; anything else is `402 Invalid Args`.
fn read_browse_flag(value: &str) -> Result<BrowseFlag, UpnpError> {
    match value.trim() {
        "BrowseMetadata" | "0" => Ok(BrowseFlag::Metadata),
        "BrowseDirectChildren" | "1" => Ok(BrowseFlag::DirectChildren),
        _ => Err(UpnpError::InvalidArgument {
            argument: "BrowseFlag".to_owned(),
        }),
    }
}

/// Reads the filter.
///
/// The wildcard `*` means every property and is `None`. An explicit list is
/// the list of the properties the client asked for, and a property the server
/// does not know is dropped rather than refused, so that a client of a newer
/// ContentDirectory version still gets an answer (`INT-034`). A list that
/// names no known property stays an empty list, and the item then carries
/// only its mandatory properties — it does **not** become the wildcard.
pub(super) fn read_filter(value: &str) -> Option<Vec<String>> {
    let trimmed = value.trim();
    if trimmed == "*" {
        return None;
    }
    Some(
        trimmed
            .split(',')
            .map(|property| property.trim())
            .filter(|property| KNOWN_PROPERTIES.contains(property))
            .map(|property| property.to_owned())
            .collect(),
    )
}

/// Reads the sort criteria.
///
/// The empty form means the order of the catalogue. `+property` sorts
/// ascending and `-property` descending (`INT-047`). A property without a
/// sign is not one of the three forms, and the server refuses it with the
/// fault `402 Invalid Args` instead of guessing an order the client did not
/// ask for. A property the server cannot sort by is refused as well.
pub(super) fn read_sort(value: &str) -> Result<SortCriteria, UpnpError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(SortCriteria {
            property: None,
            descending: false,
        });
    }
    let (descending, property) = if let Some(property) = trimmed.strip_prefix('+') {
        (false, property)
    } else if let Some(property) = trimmed.strip_prefix('-') {
        (true, property)
    } else {
        return Err(UpnpError::InvalidArgument {
            argument: "SortCriteria".to_owned(),
        });
    };
    let property = property.trim();
    if !SORTABLE_PROPERTIES.contains(&property) {
        return Err(UpnpError::InvalidArgument {
            argument: "SortCriteria".to_owned(),
        });
    }
    Ok(SortCriteria {
        property: Some(property.to_owned()),
        descending,
    })
}

/// Reads the expression of a search request.
pub(super) fn read_search_criteria(value: &str) -> Result<SearchCriteria, UpnpError> {
    let trimmed = value.trim();
    let (property, rest) =
        trimmed
            .split_once(" contains ")
            .ok_or_else(|| UpnpError::InvalidArgument {
                argument: "SearchCriteria".to_owned(),
            })?;
    if !SEARCHABLE_PROPERTIES.contains(&property) {
        return Err(UpnpError::InvalidArgument {
            argument: "SearchCriteria".to_owned(),
        });
    }
    let text = rest
        .trim()
        .strip_prefix('"')
        .and_then(|text| text.strip_suffix('"'))
        .ok_or_else(|| UpnpError::InvalidArgument {
            argument: "SearchCriteria".to_owned(),
        })?;
    Ok(SearchCriteria {
        property: property.to_owned(),
        text: text.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::{read_filter, read_sort};
    /// The wildcard is no list at all, and an explicit list keeps only the
    /// properties the server knows.
    #[test]
    fn the_filter_keeps_the_known_properties() {
        assert_eq!(read_filter("*"), None);
        assert_eq!(read_filter("dc:title"), Some(vec!["dc:title".to_owned()]));
        assert_eq!(
            read_filter("dc:title,upnp:class"),
            Some(vec!["dc:title".to_owned(), "upnp:class".to_owned()])
        );
    }

    /// A list that names no known property is an empty list, not the
    /// wildcard: the client did not ask for every property.
    #[test]
    fn an_unknown_filter_is_not_the_wildcard() {
        assert_eq!(read_filter("upnp:unknown"), Some(Vec::new()));
        assert_eq!(read_filter("upnp:unknown,dc:alsoUnknown"), Some(Vec::new()));
    }

    /// A property the document can carry is understood, and is kept.
    #[test]
    fn a_known_filter_is_kept() {
        assert_eq!(
            read_filter("dc:title,upnp:artist"),
            Some(vec!["dc:title".to_owned(), "upnp:artist".to_owned()])
        );
    }

    /// The three forms of the sort criteria are the empty form, the plus form
    /// and the minus form.
    #[test]
    fn the_sort_criteria_have_three_forms() {
        let empty = read_sort("").expect("the empty form is accepted");
        assert_eq!(empty.property, None);
        let ascending = read_sort("+dc:title").expect("the plus form is accepted");
        assert_eq!(ascending.property.as_deref(), Some("dc:title"));
        assert!(!ascending.descending);
        let descending = read_sort("-dc:title").expect("the minus form is accepted");
        assert!(descending.descending);
    }

    /// A property without a sign is refused, because the server must not guess
    /// an order the client did not ask for.
    #[test]
    fn a_sort_criterion_without_a_sign_is_refused() {
        assert!(read_sort("dc:title").is_err());
    }

    /// A property the server cannot sort by is refused.
    #[test]
    fn an_unsortable_property_is_refused() {
        assert!(read_sort("+dc:creator").is_err());
    }
}
