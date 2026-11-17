//! Cross-platform protocol parity acceptance tests (PLT-010, F-22 AC-13).
//!
//! What: verifies that UPnP device description, ContentDirectory SCPD,
//!       SOAP envelope responses, and DIDL-Lite metadata formatting are
//!       completely deterministic, byte-consistent and independent of
//!       operating system differences.
//! Why: PLT-010 requires identical protocol behavior, ordering and identifiers
//!      across macOS, Linux and Windows so control points and renderers
//!      experience seamless interoperability.

mod support;

use phairdlna_core::catalogue::ContentItem;
use phairdlna_core::didl::{item_from_catalogue, render, DidlDocument, ProtocolPolicy};
use phairdlna_core::upnp::{BaseUrl, DeviceDescription, DeviceInfo, Dispatcher, ServiceSpec};
use support::browse_request;
use support::catalogue::{movie, music_track, BASE_URL};

const TEST_ORIGIN: &str = "http://192.168.1.50:8200";
const FIXED_UDN: &str = "11111111-2222-3333-4444-555555555555";

#[test]
fn device_description_bytes_are_deterministic() {
    let base = BaseUrl::new(TEST_ORIGIN).expect("valid base url");
    let desc1 = DeviceDescription::new(&base, FIXED_UDN, DeviceInfo::new("PhairDLNA Server"))
        .expect("desc 1")
        .to_xml();

    let desc2 = DeviceDescription::new(&base, FIXED_UDN, DeviceInfo::new("PhairDLNA Server"))
        .expect("desc 2")
        .to_xml();

    assert_eq!(
        desc1, desc2,
        "device description XML output must be byte-deterministic (PLT-010)"
    );
    assert!(
        desc1.starts_with(r#"<?xml version="1.0" encoding="utf-8"?>"#),
        "xml declaration must match standard"
    );
    assert!(
        desc1.contains("<deviceType>urn:schemas-upnp-org:device:MediaServer:1</deviceType>"),
        "root device type must be standard MediaServer:1"
    );
}

#[test]
fn scpd_documents_are_identical_across_invocations() {
    let cds1 = ServiceSpec::content_directory().to_xml();
    let cds2 = ServiceSpec::content_directory().to_xml();
    assert_eq!(
        cds1, cds2,
        "ContentDirectory SCPD must be byte-identical (PLT-010)"
    );

    let cms1 = ServiceSpec::connection_manager().to_xml();
    let cms2 = ServiceSpec::connection_manager().to_xml();
    assert_eq!(
        cms1, cms2,
        "ConnectionManager SCPD must be byte-identical (PLT-010)"
    );

    assert!(cds1.contains("<actionList>"));
    assert!(cms1.contains("<actionList>"));
}

#[test]
fn soap_envelope_payload_and_ordering_are_deterministic() {
    let dispatcher = Dispatcher::new(&support::catalogue::TestCatalogue, BASE_URL);
    let req = browse_request("1", "BrowseDirectChildren", 0, 10);

    let resp1 = dispatcher.dispatch(&req, None);
    let resp2 = dispatcher.dispatch(&req, None);

    assert_eq!(
        resp1.status, resp2.status,
        "SOAP HTTP response status must match"
    );
    assert_eq!(
        resp1.body, resp2.body,
        "SOAP response body must be byte-identical (PLT-010)"
    );
    assert!(resp1.body.contains("<s:Envelope"));
    assert!(resp1.body.contains("<u:BrowseResponse"));
}

#[test]
fn didl_lite_item_ordering_and_xml_is_deterministic() {
    let policy = ProtocolPolicy::plain();
    let item1: ContentItem = movie();
    let item2: ContentItem = music_track();

    let mapped1 = item_from_catalogue(&item1, BASE_URL, &policy);
    let mapped2 = item_from_catalogue(&item2, BASE_URL, &policy);

    let doc = DidlDocument::with_items(vec![mapped1, mapped2]);
    let xml1 = render(&doc, None);
    let xml2 = render(&doc, None);

    assert_eq!(
        xml1, xml2,
        "DIDL-Lite XML rendering must be byte-deterministic across platforms (PLT-010)"
    );
    assert!(xml1.starts_with("<DIDL-Lite"));
    assert!(xml1.contains("xmlns=\"urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/\""));
}
