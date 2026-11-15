//! The device description as a control point sees it.
//!
//! What: the acceptance criteria of the feature specification F-02 that can be
//!       checked without a socket — the structure of the document at
//!       `/description.xml`.
//! Why: the description is generated from a pure function over the
//!       configuration, so it can be compared structurally. The criteria that
//!       need a running server — the response headers and the reachability of
//!       the advertised URLs — are checked by the protocol conformance test.
//! Example: `cargo test --package phairdlna-core --test device_description`.

use phairdlna_core::upnp::{BaseUrl, DeviceDescription, DeviceInfo};

mod support;

use support::assert_well_formed;

/// The origin the description is served from in the tests.
const ORIGIN: &str = "http://192.168.1.20:8200";

/// The unique device name used in the tests.
const UDN: &str = "11111111-2222-3333-4444-555555555555";

/// The description under test.
fn description() -> DeviceDescription {
    let base = BaseUrl::new(ORIGIN).expect("a valid base URL");
    DeviceDescription::new(&base, UDN, DeviceInfo::new("Living room"))
        .expect("a complete device description")
}

/// The description of a device with a chosen name.
fn description_named(name: &str) -> DeviceDescription {
    let base = BaseUrl::new(ORIGIN).expect("a valid base URL");
    DeviceDescription::new(&base, UDN, DeviceInfo::new(name)).expect("a complete description")
}

/// The document declares its encoding and carries no DOCTYPE.
#[test]
fn the_document_declares_its_encoding_and_has_no_doctype() {
    let xml = description().to_xml();
    assert!(xml.starts_with(r#"<?xml version="1.0" encoding="utf-8"?>"#));
    assert!(!xml.contains("<!DOCTYPE"));
    assert!(!xml.to_lowercase().contains("doctype"));
}

/// The document is well-formed: the elements nest and every one of them is
/// closed.
#[test]
fn the_document_is_well_formed() {
    assert_well_formed(&description().to_xml());
}

/// The document uses the namespace of the device architecture.
#[test]
fn the_document_uses_the_device_namespace() {
    let xml = description().to_xml();
    assert!(xml.contains(r#"<root xmlns="urn:schemas-upnp-org:device-1-0">"#));
}

/// The document carries the elements of the media server template.
#[test]
fn the_document_carries_the_template_elements() {
    let xml = description().to_xml();
    for element in [
        "specVersion",
        "device",
        "deviceType",
        "friendlyName",
        "manufacturer",
        "manufacturerURL",
        "modelDescription",
        "modelName",
        "modelNumber",
        "modelURL",
        "serialNumber",
        "UDN",
        "serviceList",
        "service",
        "serviceType",
        "serviceId",
        "SCPDURL",
        "controlURL",
        "eventSubURL",
    ] {
        assert!(
            xml.contains(&format!("<{element}>")),
            "{element} is missing"
        );
    }
}

/// The document holds exactly one device element.
#[test]
fn the_document_holds_exactly_one_device_element() {
    let xml = description().to_xml();
    assert_eq!(xml.matches("<device>").count(), 1);
    assert_eq!(xml.matches("</device>").count(), 1);
    assert_eq!(xml.matches("<root").count(), 1);
}

/// The document declares the specification version 1.0 and no base URL.
#[test]
fn the_document_declares_spec_version_one_and_no_url_base() {
    let xml = description().to_xml();
    assert!(xml.contains("<major>1</major>"));
    assert!(xml.contains("<minor>0</minor>"));
    assert!(!xml.contains("<URLBase"));
}

/// The root device carries the identity the specification names.
#[test]
fn the_root_device_carries_its_identity() {
    let xml = description().to_xml();
    assert!(xml.contains("<deviceType>urn:schemas-upnp-org:device:MediaServer:1</deviceType>"));
    assert!(xml.contains("<UDN>uuid:11111111-2222-3333-4444-555555555555</UDN>"));
    assert!(xml.contains("<friendlyName>Living room</friendlyName>"));
    assert!(xml.contains("<manufacturer>"));
    assert!(xml.contains("<modelName>"));
    assert!(xml.contains("<modelNumber>"));
    assert!(xml.contains("<modelURL>"));
    for element in ["manufacturer", "modelName", "modelNumber", "modelURL"] {
        let open = format!("<{element}>");
        let close = format!("</{element}>");
        let start = xml.find(&open).expect("the element is present") + open.len();
        let end = xml[start..].find(&close).expect("the element is closed") + start;
        assert!(!xml[start..end].is_empty(), "{element} must not be empty");
    }
}

/// The service list names exactly the two mandatory services.
#[test]
fn the_service_list_names_both_mandatory_services() {
    let xml = description().to_xml();
    assert_eq!(xml.matches("<service>").count(), 2);
    assert!(
        xml.contains("<serviceType>urn:schemas-upnp-org:service:ContentDirectory:1</serviceType>")
    );
    assert!(
        xml.contains("<serviceType>urn:schemas-upnp-org:service:ConnectionManager:1</serviceType>")
    );
    assert!(xml.contains("<serviceId>urn:upnp-org:serviceId:ContentDirectory</serviceId>"));
    assert!(xml.contains("<serviceId>urn:upnp-org:serviceId:ConnectionManager</serviceId>"));
}

/// Every service entry carries a service description, a control and an event
/// address, and none of them is empty.
#[test]
fn every_service_entry_carries_its_addresses() {
    let xml = description().to_xml();
    for element in ["SCPDURL", "controlURL", "eventSubURL"] {
        let open = format!("<{element}>");
        let close = format!("</{element}>");
        assert_eq!(xml.matches(&open).count(), 2, "{element} must appear twice");
        for start in xml
            .match_indices(&open)
            .map(|(index, _)| index + open.len())
        {
            let end = xml[start..].find(&close).expect("the element is closed") + start;
            let value = &xml[start..end];
            assert!(!value.is_empty(), "{element} must not be empty");
            assert!(value.starts_with("http://"), "{element} must be absolute");
        }
    }
}

/// No renderer-side service is advertised.
#[test]
fn no_renderer_side_service_is_advertised() {
    let xml = description().to_xml();
    assert!(!xml.contains("urn:schemas-upnp-org:service:RenderingControl:1"));
    assert!(!xml.contains("urn:schemas-upnp-org:service:AVTransport:1"));
}

/// The ContentDirectory service is advertised in version one.
#[test]
fn the_content_directory_is_advertised_in_version_one() {
    let xml = description().to_xml();
    assert!(
        xml.contains("<serviceType>urn:schemas-upnp-org:service:ContentDirectory:1</serviceType>")
    );
    assert!(!xml.contains("urn:schemas-upnp-org:service:ContentDirectory:2"));
}

/// The ConnectionManager is advertised even though the server transcodes
/// nothing.
#[test]
fn the_connection_manager_is_advertised_without_transcoding() {
    let description = description();
    let services = description.services();
    let connection_manager = services
        .iter()
        .find(|service| service.service_type.ends_with("ConnectionManager:1"))
        .expect("the ConnectionManager is advertised");
    let xml = description.to_xml();
    assert!(xml.contains(&format!(
        "<controlURL>{}{}</controlURL>",
        ORIGIN, connection_manager.control_path
    )));
    assert!(xml.contains(&format!(
        "<SCPDURL>{}{}</SCPDURL>",
        ORIGIN, connection_manager.scpd_path
    )));
}

/// A name with the reserved characters is escaped, and no element appears.
#[test]
fn a_name_with_reserved_characters_is_escaped() {
    let xml = description_named("Living & <room>").to_xml();
    assert!(xml.contains("<friendlyName>Living &amp; &lt;room&gt;</friendlyName>"));
    assert!(!xml.contains("<room>"));
    assert_eq!(xml.matches("<friendlyName>").count(), 1);
}

/// A quote in a name cannot break out of the element text.
#[test]
fn a_quote_in_a_name_is_escaped() {
    let xml = description_named("Room \"one\"").to_xml();
    assert!(xml.contains("<friendlyName>Room \"one\"</friendlyName>"));
}

/// The document stays small, so that a client on a slow link fetches it
/// quickly.
#[test]
fn the_document_stays_small() {
    let xml = description().to_xml();
    assert!(
        xml.len() < 16 * 1024,
        "the description is {} bytes, which is too large",
        xml.len()
    );
}
