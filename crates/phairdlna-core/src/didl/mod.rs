//! The DIDL-Lite document: the metadata a control point reads.
//!
//! What: the object model of the document (items, containers and resources),
//!       the rules of the values it carries, the `protocolInfo` builder and
//!       the writer that turns the model into XML.
//! Why: the document is what a television shows in its list of files, and
//!       every value in it comes from a file name or a tag — that is, from
//!       untrusted input (`SEC-007`). The escaping therefore happens in one
//!       place, every value is bounded, and the document never carries a
//!       path (`INT-062`, `INT-065`, `INT-072`).
//! Example:
//!
//! ```
//! use phairdlna_core::didl::{DidlDocument, Item, ObjectClass};
//!
//! let document = DidlDocument::with_items(vec![Item::new(
//!     "a3f1",
//!     "1",
//!     "Nightfall",
//!     ObjectClass::MusicTrack,
//! )]);
//!
//! let xml = document.to_xml();
//! assert!(xml.contains("xmlns=\"urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/\""));
//! assert!(xml.contains("<dc:title>Nightfall</dc:title>"));
//! ```

pub mod mapping;
pub mod model;
pub mod policy;
pub mod protocol_info;
pub mod serializer;
pub mod uri;
pub mod value;
pub mod vocabulary;

pub use mapping::{class_of, container_from_catalogue, folder_from_catalogue, item_from_catalogue};
pub use model::{Container, DidlDocument, Item, ObjectClass, Resource};
pub use policy::{ProtocolMode, ProtocolPolicy};
pub use protocol_info::{
    DlnaFlags, ProtocolInfo, DLNA_CONVERSION, DLNA_CONVERSION_NONE, DLNA_FLAGS, DLNA_OPERATION,
    DLNA_OPERATION_BYTE_RANGE_SEEK, DLNA_PROFILE,
};
pub use serializer::{render, render_for_soap_result};
pub use uri::{artwork_uri, media_uri, ARTWORK_PATH, MEDIA_PATH};
pub use value::{
    bounded, format_duration, format_resolution, is_iso_date, MAX_VALUE_BYTES, TRUNCATION_MARKER,
};
pub use vocabulary::{
    ALBUM, ALBUM_ART, ARTIST, CLASS, CREATOR, DATE, DC_NAMESPACE, DIDL_LITE_NAMESPACE,
    DLNA_NAMESPACE, GENRE, MANDATORY_PROPERTIES, RESOURCE, SUPPORTED_PROPERTIES, TITLE,
    TRACK_NUMBER, UPNP_NAMESPACE,
};
