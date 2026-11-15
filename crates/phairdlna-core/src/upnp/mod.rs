//! The UPnP device description, the service descriptions and the control path.
//!
//! What: the two documents a control point reads before it does anything — the
//!       root device description at `/description.xml` and the service
//!       description (SCPD) of every service the server advertises — and the
//!       SOAP dispatcher that answers the control requests.
//! Why: a control point decides what a server can do by reading these
//!       documents, and it calls the actions the documents name. A
//!       description that lies about the server produces failures that look
//!       like client bugs, so both the documents and the answers are generated
//!       from one table of services and actions, never from a template file on
//!       disk (`INT-020` … `INT-025`).
//! Example:
//!
//! ```
//! use phairdlna_core::upnp::{BaseUrl, DeviceDescription, DeviceInfo, ServiceSpec};
//!
//! let base = BaseUrl::new("http://192.168.1.20:8200").expect("a valid base URL");
//! let description = DeviceDescription::new(
//!     &base,
//!     "11111111-2222-3333-4444-555555555555",
//!     DeviceInfo::new("Living room"),
//! )
//! .expect("a complete description");
//!
//! let xml = description.to_xml();
//! assert!(xml.contains("<deviceType>urn:schemas-upnp-org:device:MediaServer:1</deviceType>"));
//! assert!(ServiceSpec::content_directory().to_xml().contains("<name>Browse</name>"));
//! ```

mod browse;
mod connection_manager;
mod content_directory;
mod device;
mod device_model;
mod dispatcher;
mod error;
mod metadata;
mod mime;
mod page;
mod scpd;
mod scpd_connection_manager;
mod scpd_content_directory;
mod services;
mod soap;
mod xml_syntax;

pub mod xml_reader;

pub use crate::catalogue::mime::SUPPORTED_MIME_TYPES;
pub use crate::catalogue::{ContentContainer, ContentItem, ContentSource};
pub use crate::didl::ProtocolInfo;
pub use crate::xml::{escape_attribute, escape_text, XmlWriter, XML_DECLARATION};
pub use browse::{BrowseArguments, BrowseFlag, SortCriteria};
pub use content_directory::{
    browse, get_search_capabilities, get_sort_capabilities, get_system_update_id, search,
    CONTENT_DIRECTORY_OBJECT_ID, KNOWN_PROPERTIES, ROOT_OBJECT_ID, SEARCHABLE_PROPERTIES,
    SEARCH_CAPABILITIES, SORTABLE_PROPERTIES, SORT_CAPABILITIES,
};
pub use device::{DeviceDescription, DEVICE_NAMESPACE};
pub use device_model::{
    BaseUrl, DeviceInfo, DEFAULT_MODEL_DESCRIPTION, DEFAULT_MODEL_NAME, PROJECT_URL,
};
pub use dispatcher::Dispatcher;
pub use error::{fault_description, UpnpError};
pub use mime::{sink_list, source_list, source_protocol_info};
pub use page::PAGE_LIMIT;
pub use scpd::{
    ActionSpec, ArgumentSpec, Direction, ServiceSpec, StateVariableSpec, SCPD_NAMESPACE,
};
pub use services::{
    advertised_services, ServiceDefinition, CONNECTION_MANAGER, CONTENT_DIRECTORY,
    SERVICE_ID_CONNECTION_MANAGER, SERVICE_ID_CONTENT_DIRECTORY,
};
pub use soap::{
    check_soap_action, fault_body, parse_request, response_body, SoapRequest, SoapResponse,
    SOAP_ENCODING_STYLE, SOAP_ENVELOPE_NAMESPACE, UPNP_CONTROL_NAMESPACE, XML_CONTENT_TYPE,
};
