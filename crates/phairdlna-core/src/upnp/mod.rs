//! The UPnP device description and the service descriptions.
//!
//! What: the two documents a control point reads before it does anything — the
//!       root device description at `/description.xml` and the service
//!       description (SCPD) of every service the server advertises.
//! Why: a control point decides what a server can do by reading these
//!       documents. A description that lies about the server produces failures
//!       that look like client bugs, so both documents are generated from one
//!       table of services and actions, never from a template file on disk
//!       (`INT-020` … `INT-024`).
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

mod base_url;
mod connection_manager;
mod content_directory;
mod device;
mod device_info;
mod error;
mod scpd;
mod services;
mod xml_writer;

pub use base_url::BaseUrl;
pub use device::{DeviceDescription, DEVICE_NAMESPACE};
pub use device_info::{DeviceInfo, DEFAULT_MODEL_DESCRIPTION, DEFAULT_MODEL_NAME, PROJECT_URL};
pub use error::UpnpError;
pub use scpd::{
    ActionSpec, ArgumentSpec, Direction, ServiceSpec, StateVariableSpec, SCPD_NAMESPACE,
};
pub use services::{
    advertised_services, ServiceDefinition, CONNECTION_MANAGER, CONTENT_DIRECTORY,
    SERVICE_ID_CONNECTION_MANAGER, SERVICE_ID_CONTENT_DIRECTORY,
};
pub use xml_writer::{escape_attribute, escape_text, XmlWriter, XML_DECLARATION};
