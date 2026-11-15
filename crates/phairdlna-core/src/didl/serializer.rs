//! The writer of the DIDL-Lite document.
//!
//! What: the namespaces of the document, the element and attribute names, and
//!       the writer that turns the model into XML.
//! Why: the document is read by clients that are strict about the schema
//!       (standards §8.5), and a document with an undeclared prefix is
//!       rejected by the strict parser of Cling (ReadyMedia bug #316). The
//!       writer therefore declares exactly the prefixes it uses, writes the
//!       attributes the schema requires, and never writes a `writeStatus`
//!       attribute (`INT-077`).
//! Example: `render` writes the document of a page of items.

use crate::didl::model::{Container, DidlDocument, Item, Resource};
use crate::didl::value::bounded;
use crate::didl::vocabulary::{
    ALBUM, ALBUM_ART, ARTIST, CLASS, CREATOR, DATE, DC_NAMESPACE, DIDL_LITE_NAMESPACE, GENRE,
    MANDATORY_PROPERTIES, RESOURCE, TITLE, TRACK_NUMBER, UPNP_NAMESPACE,
};
use crate::xml::{escape_text, XmlWriter};

/// Writes the document.
///
/// The filter names the properties the client asked for; `None` is the
/// wildcard and asks for every property (`INT-034`). The document carries no
/// XML declaration, because it is carried as the text of another document
/// and a declaration is only allowed at the start of a document.
pub fn render(document: &DidlDocument, filter: Option<&[String]>) -> String {
    let mut writer = XmlWriter::new();
    writer.open("DIDL-Lite", &root_attributes());
    for container in &document.containers {
        write_container(&mut writer, container, filter);
    }
    for item in &document.items {
        write_item(&mut writer, item, filter);
    }
    writer.close("DIDL-Lite");
    finish(writer)
}

/// Finishes the document, without the line break in front of the root.
///
/// The writer indents every element, and the root of a document that is
/// carried as the text of another document must not begin with a line break:
/// the client reads the value of the element as it is.
fn finish(writer: XmlWriter) -> String {
    let xml = writer.finish();
    xml.strip_prefix('\n').unwrap_or(&xml).to_owned()
}

/// Writes the document as it is carried in the `Result` element of a body.
///
/// The document is text inside another document, so it is escaped once more
/// (`INT-073`). The caller writes the result verbatim; escaping it a second
/// time would corrupt it (`INT-065`).
pub fn render_for_soap_result(document: &DidlDocument, filter: Option<&[String]>) -> String {
    escape_text(&render(document, filter))
}

/// The namespaces the root element declares.
///
/// The default namespace is the namespace of the document. The `dc` and
/// `upnp` prefixes are always used, because every object carries `dc:title`
/// and `upnp:class` (`INT-066`), and a prefix that is used is always declared
/// (`INT-071`). The `dlna` namespace is not declared: nothing in the document
/// uses it (`INT-067`).
fn root_attributes() -> Vec<(&'static str, &'static str)> {
    vec![
        ("xmlns", DIDL_LITE_NAMESPACE),
        ("xmlns:dc", DC_NAMESPACE),
        ("xmlns:upnp", UPNP_NAMESPACE),
    ]
}

/// Writes a container.
fn write_container(writer: &mut XmlWriter, container: &Container, filter: Option<&[String]>) {
    writer.open(
        "container",
        &[
            ("id", &container.id),
            ("parentID", &container.parent_id),
            ("restricted", flag(container.restricted)),
            ("searchable", flag(container.searchable)),
            ("childCount", &container.child_count.to_string()),
        ],
    );
    writer.text(TITLE, &bounded(&container.title));
    writer.text(CLASS, container.class.as_str());
    write_optional(writer, CREATOR, container.creator.as_deref(), filter);
    write_optional(writer, DATE, container.date.as_deref(), filter);
    writer.close("container");
}

/// Writes an item.
fn write_item(writer: &mut XmlWriter, item: &Item, filter: Option<&[String]>) {
    writer.open(
        "item",
        &[
            ("id", &item.id),
            ("parentID", &item.parent_id),
            ("restricted", flag(item.restricted)),
        ],
    );
    writer.text(TITLE, &bounded(&item.title));
    writer.text(CLASS, item.class.as_str());
    write_optional(writer, CREATOR, item.creator.as_deref(), filter);
    write_optional(writer, ARTIST, item.artist.as_deref(), filter);
    write_optional(writer, ALBUM, item.album.as_deref(), filter);
    write_optional(writer, GENRE, item.genre.as_deref(), filter);
    if wants(filter, TRACK_NUMBER) {
        if let Some(number) = item.track_number {
            writer.text(TRACK_NUMBER, &number.to_string());
        }
    }
    write_optional(writer, DATE, item.date.as_deref(), filter);
    write_optional(writer, ALBUM_ART, item.album_art_uri.as_deref(), filter);
    if wants(filter, RESOURCE) {
        for resource in &item.resources {
            write_resource(writer, resource);
        }
    }
    writer.close("item");
}

/// Writes a resource.
fn write_resource(writer: &mut XmlWriter, resource: &Resource) {
    let mut attributes: Vec<(&str, String)> =
        vec![("protocolInfo", resource.protocol_info.as_string())];
    if let Some(size) = resource.size {
        attributes.push(("size", size.to_string()));
    }
    if let Some(duration) = &resource.duration {
        attributes.push(("duration", duration.clone()));
    }
    if let Some(resolution) = &resource.resolution {
        attributes.push(("resolution", resolution.clone()));
    }
    if let Some(bitrate) = resource.bitrate {
        attributes.push(("bitrate", bitrate.to_string()));
    }
    if let Some(frequency) = resource.sample_frequency {
        attributes.push(("sampleFrequency", frequency.to_string()));
    }
    if let Some(channels) = resource.audio_channels {
        attributes.push(("nrAudioChannels", channels.to_string()));
    }
    if let Some(bits) = resource.bits_per_sample {
        attributes.push(("bitsPerSample", bits.to_string()));
    }
    let pairs: Vec<(&str, &str)> = attributes
        .iter()
        .map(|(name, value)| (*name, value.as_str()))
        .collect();
    writer.element_with_text(RESOURCE, &pairs, &resource.uri);
}

/// Writes an optional property, when the document has one and the client
/// asked for it.
fn write_optional(
    writer: &mut XmlWriter,
    name: &str,
    value: Option<&str>,
    filter: Option<&[String]>,
) {
    let Some(value) = value else {
        return;
    };
    if !wants(filter, name) {
        return;
    }
    writer.text(name, &bounded(value));
}

/// Whether the client asked for a property.
///
/// A wildcard asks for every property; an explicit list asks for the
/// properties it names.
fn wants(filter: Option<&[String]>, property: &str) -> bool {
    if MANDATORY_PROPERTIES.contains(&property) {
        return true;
    }
    match filter {
        None => true,
        Some(list) => list.iter().any(|asked| asked == property),
    }
}

/// The value of a boolean attribute.
fn flag(value: bool) -> &'static str {
    if value {
        "1"
    } else {
        "0"
    }
}

#[cfg(test)]
mod tests {
    use super::{render, render_for_soap_result, MANDATORY_PROPERTIES};
    use crate::didl::model::{Container, DidlDocument, Item, ObjectClass, Resource};
    use crate::didl::protocol_info::ProtocolInfo;

    /// An item with the attributes the schema requires.
    fn item() -> Item {
        Item::new("a3f1", "1", "Nightfall", ObjectClass::MusicTrack)
    }

    /// The root element declares the namespace of the document (`INT-071`).
    #[test]
    fn the_root_declares_the_namespaces() {
        let document = DidlDocument::with_items(vec![item()]);
        let xml = render(&document, None);
        assert!(xml.contains("xmlns=\"urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/\""));
        assert!(xml.contains("xmlns:dc=\"http://purl.org/dc/elements/1.1/\""));
        assert!(xml.contains("xmlns:upnp=\"urn:schemas-upnp-org:metadata-1-0/upnp/\""));
    }

    /// The document never declares the DLNA namespace, because it never uses
    /// it (`INT-067`).
    #[test]
    fn the_dlna_namespace_is_never_declared() {
        let document = DidlDocument::with_items(vec![item()]);
        let xml = render(&document, None);
        assert!(!xml.contains("xmlns:dlna"));
    }

    /// An item carries `id`, `parentID` and `restricted` (`INT-060`).
    #[test]
    fn an_item_carries_the_required_attributes() {
        let document = DidlDocument::with_items(vec![item()]);
        let xml = render(&document, None);
        assert!(xml.contains("<item id=\"a3f1\" parentID=\"1\" restricted=\"1\">"));
    }

    /// A container carries `id`, `parentID`, `restricted` and `childCount`
    /// (`INT-060`).
    #[test]
    fn a_container_carries_the_required_attributes() {
        let document = DidlDocument::with_containers(vec![Container::new("1", "0", "Music", 12)]);
        let xml = render(&document, None);
        assert!(xml.contains(
            "<container id=\"1\" parentID=\"0\" restricted=\"1\" searchable=\"1\" childCount=\"12\">"
        ));
    }

    /// The two mandatory properties are written even when the filter asks for
    /// something else (`INT-066`).
    #[test]
    fn the_mandatory_properties_are_always_written() {
        let document = DidlDocument::with_items(vec![item()]);
        let xml = render(&document, Some(&["upnp:album".to_owned()]));
        assert!(xml.contains("<dc:title>Nightfall</dc:title>"));
        assert!(xml.contains("<upnp:class>object.item.audioItem.musicTrack</upnp:class>"));
        assert!(!xml.contains("<upnp:album>"));
    }

    /// The mandatory properties are the two of the schema.
    #[test]
    fn the_mandatory_properties_are_the_two_of_the_schema() {
        assert_eq!(MANDATORY_PROPERTIES, ["dc:title", "upnp:class"]);
    }

    /// The result of a body carries the document escaped once more
    /// (`INT-073`).
    #[test]
    fn the_result_carries_the_document_escaped() {
        let document = DidlDocument::with_items(vec![item()]);
        let result = render_for_soap_result(&document, None);
        assert!(result.starts_with("&lt;DIDL-Lite"));
        assert!(!result.contains("<DIDL-Lite"));
    }

    /// A resource carries the protocol and the address of the bytes.
    #[test]
    fn a_resource_carries_its_protocol_and_its_address() {
        let mut item = item();
        let mut resource = Resource::new(
            "http://192.168.1.20:8200/media/a3f1",
            ProtocolInfo::http_get("audio/mpeg").expect("a valid entry"),
        );
        resource.size = Some(7_340_032);
        resource.duration = Some("0:03:52.000".to_owned());
        item.resources.push(resource);
        let xml = render(&DidlDocument::with_items(vec![item]), None);
        assert!(xml.contains("protocolInfo=\"http-get:*:audio/mpeg:*\""));
        assert!(xml.contains("size=\"7340032\""));
        assert!(xml.contains("duration=\"0:03:52.000\""));
        assert!(xml.contains(">http://192.168.1.20:8200/media/a3f1</res>"));
    }

    /// The document never carries a `writeStatus` attribute (`INT-077`).
    #[test]
    fn the_document_never_carries_a_write_status() {
        let document = DidlDocument::with_items(vec![item()]);
        assert!(!render(&document, None).contains("writeStatus"));
    }
}
