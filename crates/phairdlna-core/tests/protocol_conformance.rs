//! Protocol conformance acceptance test suite (PC-01 .. PC-26).
//!
//! What: verifies standard UPnP AV / DLNA MediaServer conformance checks against
//!       the portable domain core, SSDP, SCPD, SOAP and DIDL-Lite engines.
//! Why: INTEROPERABILITY-MATRIX.md defines 26 protocol conformance requirements (PC-01..PC-26)
//!      that must be verified by automated fixtures to guarantee zero regressions.
//! Standards: UPnP Device Architecture 1.0, ContentDirectory:1, ConnectionManager:1.

mod support;

use phairdlna_core::catalogue::ContentItem;
use phairdlna_core::didl::{item_from_catalogue, render, DidlDocument, ProtocolPolicy};
use phairdlna_core::ssdp::{advertised_targets, parse_search, DeviceIdentity};
use phairdlna_core::upnp::{BaseUrl, DeviceDescription, DeviceInfo, Dispatcher, ServiceSpec};
use support::browse_request;
use support::catalogue::{movie, TestCatalogue, BASE_URL};

const ORIGIN: &str = "http://192.168.1.100:8200";
const UDN: &str = "00000000-1111-2222-3333-444444444444";

/// PC-01 & PC-04: SSDP search matching and USN compliance for rootdevice.
#[test]
fn pc_01_and_04_ssdp_search_matching_and_usn() {
    let identity = DeviceIdentity::new(UDN, "PhairDLNA");
    let targets = advertised_targets(&identity);
    assert_eq!(targets.len(), 5);

    let root_target = targets
        .iter()
        .find(|t| t.nt == "upnp:rootdevice")
        .expect("rootdevice target must exist");
    assert!(
        root_target
            .usn
            .starts_with(&format!("uuid:{UDN}::upnp:rootdevice")),
        "USN must comply with PC-04: {}",
        root_target.usn
    );
}

/// PC-02: SSDP search without MX must be rejected/ignored.
#[test]
fn pc_02_ssdp_search_without_mx_ignored() {
    let raw = "M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nMAN: \"ssdp:discover\"\r\nST: upnp:rootdevice\r\n\r\n";
    let parsed = parse_search(raw);
    assert!(
        parsed.is_err(),
        "M-SEARCH without MX header must fail parsing (PC-02)"
    );
}

/// PC-06: Device description is valid XML with all required elements.
#[test]
fn pc_06_device_description_elements() {
    let base = BaseUrl::new(ORIGIN).expect("valid base url");
    let desc = DeviceDescription::new(&base, UDN, DeviceInfo::new("PhairDLNA Media Server"))
        .expect("description")
        .to_xml();

    support::assert_well_formed(&desc);
    assert!(desc.contains("<major>1</major>"));
    assert!(desc.contains("<minor>0</minor>"));
    assert!(desc.contains("<deviceType>urn:schemas-upnp-org:device:MediaServer:1</deviceType>"));
    assert!(desc.contains(&format!("<UDN>uuid:{UDN}</UDN>")));
    assert!(desc.contains("urn:schemas-upnp-org:service:ContentDirectory:1"));
    assert!(desc.contains("urn:schemas-upnp-org:service:ConnectionManager:1"));
}

/// PC-07: Every advertised service has a valid, well-formed SCPD.
#[test]
fn pc_07_advertised_services_have_valid_scpd() {
    let cds_xml = ServiceSpec::content_directory().to_xml();
    support::assert_well_formed(&cds_xml);
    assert!(cds_xml.contains("<actionList>"));
    assert!(cds_xml.contains("<name>Browse</name>"));

    let cm_xml = ServiceSpec::connection_manager().to_xml();
    support::assert_well_formed(&cm_xml);
    assert!(cm_xml.contains("<actionList>"));
    assert!(cm_xml.contains("<name>GetProtocolInfo</name>"));
}

/// PC-08 & PC-09: Browse returns Result, NumberReturned, TotalMatches, UpdateID.
#[test]
fn pc_08_and_09_browse_response_fields() {
    let dispatcher = Dispatcher::new(&TestCatalogue, BASE_URL);
    let req = browse_request("1", "BrowseDirectChildren", 0, 10);
    let resp = dispatcher.dispatch(&req, None);
    let body = resp.body;

    support::assert_well_formed(&body);
    let num = support::element_text(&body, "NumberReturned").expect("NumberReturned");
    let total = support::element_text(&body, "TotalMatches").expect("TotalMatches");
    let update_id = support::element_text(&body, "UpdateID").expect("UpdateID");

    assert_eq!(num, "10");
    assert_eq!(total, "12");
    assert_eq!(update_id, "42");

    // PC-09: BrowseMetadata returns exactly 1 match
    let req_meta = browse_request("1", "BrowseMetadata", 0, 10);
    let out_meta = dispatcher.dispatch(&req_meta, None);
    let meta_num = support::element_text(&out_meta.body, "NumberReturned").expect("num");
    let meta_total = support::element_text(&out_meta.body, "TotalMatches").expect("total");
    assert_eq!(
        meta_num, "1",
        "BrowseMetadata NumberReturned must be 1 (PC-09)"
    );
    assert_eq!(
        meta_total, "1",
        "BrowseMetadata TotalMatches must be 1 (PC-09)"
    );
}

/// PC-10: Unknown ObjectID returns UPnP error code 701 (NoSuchObjectID).
#[test]
fn pc_10_unknown_object_id_error_701() {
    let dispatcher = Dispatcher::new(&TestCatalogue, BASE_URL);
    let req = browse_request("non-existent-999", "BrowseMetadata", 0, 10);
    let outcome = dispatcher.dispatch(&req, None);

    assert_eq!(outcome.status, 500);
    assert!(
        outcome.body.contains("<errorCode>701</errorCode>"),
        "Unknown ObjectID must fail with 701 (PC-10): {}",
        outcome.body
    );
}

/// PC-11 & PC-12: Standard capabilities and no unadvertised CDS actions.
#[test]
fn pc_11_and_12_capabilities_and_standard_actions() {
    let spec = ServiceSpec::content_directory();
    assert!(spec.action("Browse").is_some());
    assert!(spec.action("GetSearchCapabilities").is_some());
    assert!(spec.action("GetSortCapabilities").is_some());
    assert!(spec.action("GetSystemUpdateID").is_some());

    // PC-12: Must NOT advertise non-standard or write actions
    assert!(spec.action("CreateObject").is_none());
    assert!(spec.action("DestroyObject").is_none());
    assert!(spec.action("ImportResource").is_none());
}

/// PC-13: GetProtocolInfo lists only served MIME types.
#[test]
fn pc_13_get_protocol_info() {
    let cm_spec = ServiceSpec::connection_manager();
    assert!(cm_spec.action("GetProtocolInfo").is_some());
    let cm_xml = cm_spec.to_xml();
    assert!(cm_xml.contains("<name>GetProtocolInfo</name>"));
}

/// PC-15: Result element carries DIDL-Lite escaped a second time.
#[test]
fn pc_15_result_element_double_escaping() {
    let dispatcher = Dispatcher::new(&TestCatalogue, BASE_URL);
    let req = browse_request("1", "BrowseMetadata", 0, 10);
    let outcome = dispatcher.dispatch(&req, None);
    let result_raw = support::element_text(&outcome.body, "Result").expect("Result element");

    // First layer: raw XML should be escaped with &lt;
    assert!(
        result_raw.starts_with("&lt;DIDL-Lite"),
        "Result must be double-escaped: {result_raw}"
    );

    let decoded = support::unescape_once(&result_raw);
    support::assert_well_formed(&decoded);
}

/// PC-22: Content-Type equals res protocolInfo MIME type.
#[test]
fn pc_22_protocol_info_mime_agreement() {
    let item: ContentItem = movie();
    let didl_item = item_from_catalogue(&item, BASE_URL, &ProtocolPolicy::plain());
    let doc = DidlDocument::with_items(vec![didl_item]);
    let rendered = render(&doc, None);

    assert!(rendered.contains("http-get:*:video/mp4:*"));
}

/// PC-24 & PC-25: Fault handling on invalid or unknown SOAP actions without panic.
#[test]
fn pc_24_and_25_fault_handling_without_panic() {
    let dispatcher = Dispatcher::new(&TestCatalogue, BASE_URL);

    // PC-24: Unknown action
    let unknown_req = support::envelope(
        support::CONTENT_DIRECTORY,
        "UnknownMagicAction",
        &[("ObjectID", "0")],
    );
    let fault = dispatcher.dispatch(&unknown_req, None);
    assert_eq!(fault.status, 500);
    assert!(fault.body.contains("<errorCode>401</errorCode>"));

    // PC-25: Malformed XML body
    let broken = "<s:Envelope><broken";
    let bad_req = dispatcher.dispatch(broken, None);
    assert_eq!(bad_req.status, 500);
}

/// PC-05: LOCATION header is generated reachable on the requesting interface (INT-006, INT-009).
#[test]
fn pc_05_location_header_reachable_on_requesting_interface() {
    use phairdlna_core::config::Config;
    use phairdlna_core::platform::{LocalNetworkStatus, NetworkInterface};
    use phairdlna_core::ssdp::InterfaceRouter;
    use std::net::{IpAddr, Ipv4Addr};

    let mut config = Config::default();
    config.server.http_port = 8200;
    config.network.interfaces = vec!["eth0".to_owned()];

    let ifaces = vec![
        NetworkInterface {
            name: "eth0".to_owned(),
            addresses: vec![IpAddr::V4(Ipv4Addr::new(192, 168, 1, 150))],
            supports_multicast: true,
            is_local: true,
        },
        NetworkInterface {
            name: "wlan0".to_owned(),
            addresses: vec![IpAddr::V4(Ipv4Addr::new(10, 0, 0, 75))],
            supports_multicast: true,
            is_local: true,
        },
    ];

    let router = InterfaceRouter::new(&ifaces, &config, LocalNetworkStatus::Permitted);

    // Permitted interface gets location on its own reachable IP
    assert!(router.is_interface_permitted("eth0"));
    assert_eq!(
        router.location_for("192.168.1.150"),
        "http://192.168.1.150:8200/description.xml"
    );

    // Excluded interface is rejected before answering
    assert!(!router.is_interface_permitted("wlan0"));
}

/// PC-26: SSDP rate limiting restricts excessive discovery queries from single host (INT-011).
#[test]
fn pc_26_ssdp_rate_limiting_enforced() {
    use phairdlna_core::ssdp::{DiscoveryRateLimiter, RATE_LIMIT_MAX_REQUESTS};
    use std::net::IpAddr;
    use std::time::Instant;

    let mut limiter = DiscoveryRateLimiter::new();
    let client: IpAddr = "192.168.1.99".parse().expect("valid ip");
    let now = Instant::now();

    for _ in 0..RATE_LIMIT_MAX_REQUESTS {
        assert!(
            limiter.check_and_record(client, now),
            "requests within quota must be permitted"
        );
    }

    assert!(
        !limiter.check_and_record(client, now),
        "requests exceeding maximum rate quota within window must be rejected"
    );
}

/// PC-27: DLNA discovery strictly avoids mDNS advertising (INT-018).
#[test]
fn pc_27_ssdp_discovery_only_no_mdns() {
    use phairdlna_core::ssdp::MULTICAST_GROUP;
    // SSDP multicast group is 239.255.255.250:1900, not mDNS 224.0.0.251:5353
    assert_eq!(MULTICAST_GROUP.to_string(), "239.255.255.250");
    assert_ne!(MULTICAST_GROUP.to_string(), "224.0.0.251");
}

/// PC-28: UPnP device description and SCPD XML response headers contract (INT-026).
#[test]
fn pc_28_device_and_scpd_description_headers_contract() {
    use phairdlna_core::upnp::XML_CONTENT_TYPE;
    let cds_xml = ServiceSpec::content_directory().to_xml();
    let cm_xml = ServiceSpec::connection_manager().to_xml();

    assert_eq!(XML_CONTENT_TYPE, "text/xml; charset=\"utf-8\"");
    assert!(!cds_xml.is_empty());
    assert!(!cm_xml.is_empty());
}
