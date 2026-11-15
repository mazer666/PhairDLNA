//! The vocabulary of the document.
//!
//! What: the namespaces of the document and the names of the properties it
//!       carries.
//! Why: the names are the contract with the client, and the client is strict
//!       about them: a document with an undeclared prefix is rejected by the
//!       strict parser of Cling (ReadyMedia bug #316), and a document that
//!       uses a property the schema does not define is not a DIDL-Lite
//!       document (`INT-071`).
//! Example: `SUPPORTED_PROPERTIES` is the list a filter is checked against.

/// The namespace of the DIDL-Lite document.
pub const DIDL_LITE_NAMESPACE: &str = "urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/";

/// The namespace of the Dublin Core properties.
pub const DC_NAMESPACE: &str = "http://purl.org/dc/elements/1.1/";

/// The namespace of the UPnP properties.
pub const UPNP_NAMESPACE: &str = "urn:schemas-upnp-org:metadata-1-0/upnp/";

/// The namespace of the DLNA properties.
///
/// The namespace is never declared, because the document never uses a
/// `dlna:`-namespaced element: the DLNA signalling lives in the fourth field
/// of `res@protocolInfo`, and the `dlna:profileID` attribute of
/// `upnp:albumArtURI` is forbidden (`INT-067`). The constant is here so that
/// the reason is written down next to the code.
pub const DLNA_NAMESPACE: &str = "urn:schemas-dlna-org:metadata-1-0/";

/// The title of an object.
pub const TITLE: &str = "dc:title";

/// The class of an object.
pub const CLASS: &str = "upnp:class";

/// The creator of a work.
pub const CREATOR: &str = "dc:creator";

/// The artist of a work.
pub const ARTIST: &str = "upnp:artist";

/// The album of a work.
pub const ALBUM: &str = "upnp:album";

/// The genre of a work.
pub const GENRE: &str = "upnp:genre";

/// The track number of a work.
pub const TRACK_NUMBER: &str = "upnp:originalTrackNumber";

/// The date of a file.
pub const DATE: &str = "dc:date";

/// The address of the artwork of an album.
pub const ALBUM_ART: &str = "upnp:albumArtURI";

/// The resource of an item.
pub const RESOURCE: &str = "res";

/// The properties every object carries, whatever the client asked for.
///
/// The schema makes `dc:title` and `upnp:class` mandatory (standards §4.2),
/// and `INT-066` asks for both on every item and container. A filter narrows
/// the *other* properties; it never removes these two, because a document
/// without them is not a valid DIDL-Lite document.
pub const MANDATORY_PROPERTIES: [&str; 2] = [TITLE, CLASS];

/// The properties a document can carry.
///
/// A filter that names a property outside this list names a property the
/// server does not write, and is therefore not understood (`INT-034`).
pub const SUPPORTED_PROPERTIES: [&str; 10] = [
    TITLE,
    CLASS,
    CREATOR,
    ARTIST,
    ALBUM,
    GENRE,
    TRACK_NUMBER,
    DATE,
    ALBUM_ART,
    RESOURCE,
];
