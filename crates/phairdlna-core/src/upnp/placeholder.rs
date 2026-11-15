//! The placeholders of the features that do not exist yet.
//!
//! What: the DIDL-Lite document that a browse answer carries, and the
//!       catalogue that answers a browse until the real catalogue exists.
//! Why: the DIDL-Lite document is the job of F-05, which owns the element set,
//!       the `res` element and the escaping (`INT-073`), and the catalogue is
//!       the job of F-08 and F-09. Neither exists yet, so this file carries
//!       the part the protocol layer needs to answer a browse honestly: the
//!       root element with the namespace of DIDL-Lite, one element per item,
//!       and a catalogue with the two root containers and nothing in them.
//!       **Both are placeholders**, and both features replace them; nothing
//!       here is final.
//! Example: `items_document` writes the document of a page of children.

use crate::upnp::content_model::{Container, ContentItem, ContentSource};
use crate::upnp::xml_writer::XmlWriter;

/// The namespace of the DIDL-Lite document.
pub const DIDL_LITE_NAMESPACE: &str = "urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/";

/// The namespace of the Dublin Core properties.
pub const DC_NAMESPACE: &str = "http://purl.org/dc/elements/1.1/";

/// The namespace of the UPnP properties.
pub const UPNP_NAMESPACE: &str = "urn:schemas-upnp-org:metadata-1-0/upnp/";

/// The document that describes a container itself.
pub fn container_document(container: &Container) -> String {
    let mut writer = XmlWriter::new();
    writer.declaration();
    writer.open(
        "DIDL-Lite",
        &[
            ("xmlns", DIDL_LITE_NAMESPACE),
            ("xmlns:dc", DC_NAMESPACE),
            ("xmlns:upnp", UPNP_NAMESPACE),
        ],
    );
    writer.open(
        "container",
        &[
            ("id", &container.id),
            ("parentID", &container.parent_id),
            ("restricted", "1"),
            ("searchable", "1"),
        ],
    );
    writer.text("dc:title", &container.title);
    writer.text("upnp:class", &container.class);
    writer.text("restricted", "1");
    writer.close("container");
    writer.close("DIDL-Lite");
    writer.finish()
}

/// The document that describes a page of items.
///
/// The filter is applied: `None` is the wildcard and asks for every property,
/// and a list asks for exactly those properties the server knows (`INT-034`).
/// The property `restricted` is required in every item and is written
/// whatever the filter says.
pub fn items_document(items: &[ContentItem], filter: Option<&[String]>) -> String {
    let mut writer = XmlWriter::new();
    writer.declaration();
    writer.open(
        "DIDL-Lite",
        &[
            ("xmlns", DIDL_LITE_NAMESPACE),
            ("xmlns:dc", DC_NAMESPACE),
            ("xmlns:upnp", UPNP_NAMESPACE),
        ],
    );
    for item in items {
        writer.open(
            "item",
            &[
                ("id", &item.id),
                ("parentID", &item.parent_id),
                ("restricted", "1"),
            ],
        );
        if wants(filter, "dc:title") {
            writer.text("dc:title", &item.title);
        }
        if wants(filter, "upnp:class") {
            writer.text("upnp:class", &item.class);
        }
        writer.text("restricted", "1");
        writer.close("item");
    }
    writer.close("DIDL-Lite");
    writer.finish()
}

/// Whether the client asked for a property.
///
/// A wildcard asks for every property; an explicit list asks for the
/// properties it names.
fn wants(filter: Option<&[String]>, property: &str) -> bool {
    match filter {
        None => true,
        Some(list) => list.iter().any(|asked| asked == property),
    }
}

#[cfg(test)]
mod tests {
    use super::{container_document, items_document};
    use crate::upnp::content_model::ContentItem;

    /// The document of a container names the container and its title.
    #[test]
    fn the_document_of_a_container_names_it() {
        let container = crate::upnp::content_model::Container {
            id: "0".to_owned(),
            parent_id: "-1".to_owned(),
            title: "Root".to_owned(),
            class: "object.container".to_owned(),
            update_id: 7,
            children: Vec::new(),
            total: Some(0),
        };
        let document = container_document(&container);
        assert!(document.contains("xmlns=\"urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/\""));
        assert!(document.contains("<container id=\"0\" parentID=\"-1\""));
        assert!(document.contains("<dc:title>Root</dc:title>"));
        assert!(document.contains("<upnp:class>object.container</upnp:class>"));
    }

    /// A wildcard filter keeps every property, a named filter keeps only that
    /// one.
    #[test]
    fn the_filter_narrows_the_document() {
        let items = vec![ContentItem::new(
            "2",
            "0",
            "A film",
            "object.item.videoItem",
        )];
        let all = items_document(&items, None);
        assert!(all.contains("<dc:title>A film</dc:title>"));
        assert!(all.contains("<upnp:class>object.item.videoItem</upnp:class>"));

        let narrow = items_document(&items, Some(&["dc:title".to_owned()]));
        assert!(narrow.contains("<dc:title>A film</dc:title>"));
        assert!(!narrow.contains("<upnp:class>"));
    }
}

/// The catalogue that exists until the catalogue module does.
///
/// It knows the two root containers of ContentDirectory:1 — the root `0` and
/// the content directory `1` — and neither of them has a child. A browse of
/// any other identifier is the fault `701 No Such Object`, because the server
/// really does not know it (`INT-039`).
#[derive(Debug, Default, Clone, Copy)]
pub struct PlaceholderCatalogue;

impl ContentSource for PlaceholderCatalogue {
    fn system_update_id(&self) -> u32 {
        0
    }

    fn container(&self, object_id: &str) -> Option<Container> {
        let (id, parent_id, title) = match object_id {
            "0" => ("0", "-1", "Root"),
            "1" => ("1", "0", "Content Directory"),
            _ => return None,
        };
        Some(Container {
            id: id.to_owned(),
            parent_id: parent_id.to_owned(),
            title: title.to_owned(),
            class: "object.container".to_owned(),
            update_id: 0,
            children: Vec::new(),
            total: Some(0),
        })
    }
}
