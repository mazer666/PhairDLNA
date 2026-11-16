//! The ContentDirectory service: the actions a control point calls.
//!
//! What: the handlers of `GetSearchCapabilities`, `GetSortCapabilities`,
//!       `GetSystemUpdateID`, `Browse` and `Search`, with the arithmetic of a
//!       browse answer.
//! Why: the handlers are plain functions over typed values, so they can be
//!       exercised without a socket, and they never touch the filesystem — a
//!       browse reads the catalogue and nothing else (`PERF-008`,
//!       ARCHITECTURE §5). Every argument comes from the network and is
//!       therefore treated as untrusted (`SEC-018`). The document the answer
//!       carries is built by the `didl` module, which owns the escaping and
//!       the length policy (`INT-065`, `INT-072`).
//! Example: `browse` answers a browse request with the four arguments the
//!       specification names: `Result`, `NumberReturned`, `TotalMatches` and
//!       `UpdateID`.

use crate::catalogue::{ContentContainer, ContentItem, ContentSource};
use crate::didl::{
    folder_from_catalogue, item_from_catalogue, render_for_soap_result, DidlDocument,
    ProtocolPolicy, SUPPORTED_PROPERTIES,
};
use crate::upnp::browse::{
    read_browse_arguments, read_filter, read_search_criteria, read_sort, read_ui4, BrowseArguments,
    BrowseFlag,
};
use crate::upnp::error::UpnpError;
use crate::upnp::metadata::{item_answer, metadata_answer};
use crate::upnp::page::{page, sort_items};
use crate::upnp::soap::{response_body, response_body_with_result, SoapRequest, SoapResponse};

/// The properties the server can search in.
///
/// The list names only the properties the server really searches in
/// (`INT-035`, `INT-049`); the catalogue module extends it when it can search
/// in more.
pub const SEARCH_CAPABILITIES: &str = "dc:title";

/// The properties the server can sort by.
///
/// The list names only the properties the server really sorts by (`INT-047`,
/// `INT-049`).
pub const SORT_CAPABILITIES: &str = "dc:title,upnp:class";

/// The properties a filter may select.
///
/// A filter that names another property is ignored rather than refused, so
/// that a client of a newer ContentDirectory version still gets an answer
/// (`INT-034`). The list is the one of the document, so that the two cannot
/// drift apart.
pub const KNOWN_PROPERTIES: &[&str] = &SUPPORTED_PROPERTIES;

/// The properties the server can search in.
///
/// The list is the ground truth of the search: a capability that is not in
/// it cannot be searched in (`INT-035`).
pub const SEARCHABLE_PROPERTIES: [&str; 1] = ["dc:title"];

/// The properties the server can sort by.
///
/// The list is the ground truth of the sort (`INT-047`).
pub const SORTABLE_PROPERTIES: [&str; 2] = ["dc:title", "upnp:class"];

/// The identifier of the root container.
pub const ROOT_OBJECT_ID: &str = "0";

/// The identifier of the content directory root.
pub const CONTENT_DIRECTORY_OBJECT_ID: &str = "1";

/// The answer of `GetSearchCapabilities`.
pub fn get_search_capabilities() -> SoapResponse {
    answer(
        "GetSearchCapabilities",
        &[("SearchCaps", SEARCH_CAPABILITIES)],
    )
}

/// The answer of `GetSortCapabilities`.
pub fn get_sort_capabilities() -> SoapResponse {
    answer("GetSortCapabilities", &[("SortCaps", SORT_CAPABILITIES)])
}

/// The answer of `GetSystemUpdateID`.
pub fn get_system_update_id(source: &dyn ContentSource) -> SoapResponse {
    answer(
        "GetSystemUpdateID",
        &[("Id", &source.system_update_id().to_string())],
    )
}

/// The answer of a browse request.
///
/// The four arguments are the ones the specification names, in that order
/// (`INT-031`). An object the server does not know is the fault `701 No Such
/// Object`, never an empty answer that looks like data loss (`INT-039`).
///
/// The base address is the address the document builds its links from, and
/// the policy decides how the `protocolInfo` of a resource is written
/// (`INT-078`).
pub fn browse(
    request: &SoapRequest,
    source: &dyn ContentSource,
    base_url: &str,
    policy: &ProtocolPolicy,
) -> Result<SoapResponse, UpnpError> {
    let arguments = read_browse_arguments(request)?;
    let Some(container) = source.container(&arguments.object_id) else {
        return item_answer(&arguments, source, base_url, policy);
    };
    match arguments.flag {
        BrowseFlag::Metadata => Ok(metadata_answer(
            &container,
            source,
            arguments.filter.as_deref(),
        )),
        BrowseFlag::DirectChildren => {
            children_answer(&container, &arguments, source, base_url, policy)
        }
    }
}

/// The identifier a browse reports.
///
/// A browse of the root container reports the identifier of the whole
/// catalogue; a browse of another container reports the identifier of that
/// container (`INT-043`).
pub(super) fn update_id_of(container: &ContentContainer, source: &dyn ContentSource) -> u32 {
    if container.id == ROOT_OBJECT_ID {
        source.system_update_id()
    } else {
        container.update_id
    }
}

/// The answer of a browse of the children of a container.
fn children_answer(
    container: &ContentContainer,
    arguments: &BrowseArguments,
    source: &dyn ContentSource,
    base_url: &str,
    policy: &ProtocolPolicy,
) -> Result<SoapResponse, UpnpError> {
    let children = sorted_children(container, arguments);
    let page = page(&children, arguments);
    let total = match container.total {
        Some(total) => total,
        None => {
            if page.is_empty() {
                return Err(UpnpError::CannotComputeTotalMatches);
            }
            0
        }
    };
    let update_id = update_id_of(container, source);
    let document = document_of(&page, base_url, policy);
    let result = render_for_soap_result(&document, arguments.filter.as_deref());
    Ok(answer_with_result(
        "Browse",
        &result,
        &[
            ("NumberReturned", &page.len().to_string()),
            ("TotalMatches", &total.to_string()),
            ("UpdateID", &update_id.to_string()),
        ],
    ))
}

/// The document of a page of items.
///
/// A folder of the library is a container and not an item: the schema says
/// that a client opens a folder by its identifier, and the folder carries the
/// number of children it holds (`INT-042`). A playlist is a container as well
/// (`FUN-015`).
fn document_of(page: &[ContentItem], base_url: &str, policy: &ProtocolPolicy) -> DidlDocument {
    let (folders, files): (Vec<&ContentItem>, Vec<&ContentItem>) =
        page.iter().partition(|item| item.is_container());
    DidlDocument {
        containers: folders.into_iter().map(folder_from_catalogue).collect(),
        items: files
            .into_iter()
            .map(|item| item_from_catalogue(item, base_url, policy))
            .collect(),
    }
}

/// The children in the order the client asked for.
fn sorted_children(container: &ContentContainer, arguments: &BrowseArguments) -> Vec<ContentItem> {
    let mut children = container.children.clone();
    sort_items(&mut children, &arguments.sort);
    children
}

/// The answer of a search request.
///
/// The expression is `property contains "text"`, over the properties the
/// server advertises in `GetSearchCapabilities`. An expression the server does
/// not understand is the fault `402 Invalid Args` (`INT-036`).
pub fn search(
    request: &SoapRequest,
    source: &dyn ContentSource,
    base_url: &str,
    policy: &ProtocolPolicy,
) -> Result<SoapResponse, UpnpError> {
    let criteria = read_search_criteria(request.required_argument("SearchCriteria")?)?;
    let container_id = request.required_argument("ContainerID")?.trim().to_owned();
    let filter = read_filter(request.required_argument("Filter")?);
    let arguments = BrowseArguments {
        object_id: container_id.clone(),
        flag: BrowseFlag::DirectChildren,
        filter: None,
        starting_index: read_ui4(request.required_argument("StartingIndex")?, "StartingIndex")?,
        requested_count: read_ui4(
            request.required_argument("RequestedCount")?,
            "RequestedCount",
        )?,
        sort: read_sort(request.required_argument("SortCriteria")?)?,
    };
    let mut matches = source.search(&container_id, &criteria.property, &criteria.text);
    let total = matches.len() as u32;
    sort_items(&mut matches, &arguments.sort);
    let page = page(&matches, &arguments);
    let result = render_for_soap_result(&document_of(&page, base_url, policy), filter.as_deref());
    Ok(answer_with_result(
        "Search",
        &result,
        &[
            ("NumberReturned", &page.len().to_string()),
            ("TotalMatches", &total.to_string()),
            ("UpdateID", &source.system_update_id().to_string()),
        ],
    ))
}

/// The answer of an action of the ContentDirectory service.
fn answer(action: &str, arguments: &[(&str, &str)]) -> SoapResponse {
    SoapResponse::ok(response_body(
        crate::upnp::services::CONTENT_DIRECTORY.service_type,
        action,
        arguments,
    ))
}

/// The answer of an action that carries a document.
///
/// The document is already escaped for the text of an element, because the
/// `didl` module owns that escaping (`INT-073`).
pub(super) fn answer_with_result(
    action: &str,
    result: &str,
    arguments: &[(&str, &str)],
) -> SoapResponse {
    SoapResponse::ok(response_body_with_result(
        crate::upnp::services::CONTENT_DIRECTORY.service_type,
        action,
        result,
        arguments,
    ))
}
