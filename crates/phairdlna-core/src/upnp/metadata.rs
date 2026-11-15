//! The answers that describe one object instead of listing children.
//!
//! What: the answer of a metadata browse of a container and the answer of a
//!       metadata browse of an item.
//! Why: a client asks for the metadata of an object by its identifier, and a
//!       client that listed a container often asks for the metadata of one of
//!       the items it found (`INT-033`).
//! Example: a browse of the identifier of a file answers with that file and
//!       not with `701 No Such Object`.

use crate::catalogue::{ContentContainer, ContentSource};
use crate::didl::{
    container_from_catalogue, item_from_catalogue, render_for_soap_result, DidlDocument,
    ProtocolPolicy,
};
use crate::upnp::browse::{BrowseArguments, BrowseFlag};
use crate::upnp::content_directory::{answer_with_result, update_id_of};
use crate::upnp::error::UpnpError;
use crate::upnp::soap::SoapResponse;

/// The answer of a metadata browse: the container itself (`INT-033`).
///
/// A container carries no file, so the answer needs neither the address of
/// the server nor the protocol policy.
pub(super) fn metadata_answer(
    container: &ContentContainer,
    source: &dyn ContentSource,
    filter: Option<&[String]>,
) -> SoapResponse {
    let document = DidlDocument::with_containers(vec![container_from_catalogue(container)]);
    let result = render_for_soap_result(&document, filter);
    answer_with_result(
        "Browse",
        &result,
        &[
            ("NumberReturned", "1"),
            ("TotalMatches", "1"),
            ("UpdateID", &update_id_of(container, source).to_string()),
        ],
    )
}

/// The answer of a browse that names an item and not a container.
///
/// A client that has listed a container often asks for the metadata of one
/// of the items it found, by the identifier of that item. The catalogue knows
/// the item, so the server answers with the item and not with `701 No Such
/// Object` (`INT-033`).
pub(super) fn item_answer(
    arguments: &BrowseArguments,
    source: &dyn ContentSource,
    base_url: &str,
    policy: &ProtocolPolicy,
) -> Result<SoapResponse, UpnpError> {
    let item = source.item_of(&arguments.object_id);
    match (arguments.flag, item) {
        (BrowseFlag::Metadata, Some(item)) => {
            let document =
                DidlDocument::with_items(vec![item_from_catalogue(&item, base_url, policy)]);
            let result = render_for_soap_result(&document, arguments.filter.as_deref());
            Ok(answer_with_result(
                "Browse",
                &result,
                &[
                    ("NumberReturned", "1"),
                    ("TotalMatches", "1"),
                    ("UpdateID", &source.system_update_id().to_string()),
                ],
            ))
        }
        _ => Err(UpnpError::NoSuchObject {
            object_id: arguments.object_id.clone(),
        }),
    }
}
