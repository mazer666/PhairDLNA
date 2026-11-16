//! The objects of a DIDL-Lite document.
//!
//! What: the model of what a control point reads — the items, the containers,
//!       the resources of an item and the class of an object.
//! Why: the model is built from untrusted input (file names, tags, `INT-065`),
//!       and it is deliberately free of paths: an object carries an opaque
//!       identifier, never the place the file lives (`INT-062`, `SEC-093`).
//!       The model knows nothing about XML; the writer does (`INT-071`).
//! Example: `Item::new` builds the item of a file with its class.

use crate::didl::protocol_info::ProtocolInfo;

/// The class of an object, as the UPnP metadata schema names it (`INT-061`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectClass {
    /// The generic item class, for a file whose kind is not described further.
    Item,
    /// A music track.
    MusicTrack,
    /// A film.
    Movie,
    /// A live or streamed video source.
    VideoBroadcast,
    /// A photo.
    Photo,
    /// A folder of the catalogue.
    StorageFolder,
    /// A playlist of the catalogue.
    ///
    /// The class exists for exactly this purpose: a playlist is a container
    /// whose children are the tracks it names (`FUN-015`).
    PlaylistContainer,
    /// An item that belongs to a playlist.
    PlaylistItem,
}

impl ObjectClass {
    /// The class as the document spells it.
    pub fn as_str(&self) -> &'static str {
        match self {
            ObjectClass::Item => "object.item",
            ObjectClass::MusicTrack => "object.item.audioItem.musicTrack",
            ObjectClass::Movie => "object.item.videoItem.movie",
            ObjectClass::VideoBroadcast => "object.item.videoItem.videoBroadcast",
            ObjectClass::Photo => "object.item.imageItem.photo",
            ObjectClass::StorageFolder => "object.container.storageFolder",
            ObjectClass::PlaylistContainer => "object.container.playlistContainer",
            ObjectClass::PlaylistItem => "object.item.playlistItem",
        }
    }
}

/// A file the client can fetch: the `res` element of an item.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resource {
    /// The absolute address the client fetches the bytes from.
    pub uri: String,
    /// The four fields that describe how the bytes are delivered (`INT-064`).
    pub protocol_info: ProtocolInfo,
    /// The size of the file in bytes.
    pub size: Option<u64>,
    /// The running time, already in the format the schema asks for.
    pub duration: Option<String>,
    /// The pixel dimensions, already in the format the schema asks for.
    pub resolution: Option<String>,
    /// The bitrate in bytes per second (`INT-074`).
    pub bitrate: Option<u32>,
    /// The sample rate of an audio file in Hz (`INT-075`).
    pub sample_frequency: Option<u32>,
    /// The number of channels of an audio file (`INT-075`).
    pub audio_channels: Option<u16>,
    /// The number of bits per sample of an audio file (`INT-075`).
    pub bits_per_sample: Option<u16>,
}

impl Resource {
    /// Builds a resource with the address and the protocol of its delivery.
    pub fn new(uri: impl Into<String>, protocol_info: ProtocolInfo) -> Self {
        Self {
            uri: uri.into(),
            protocol_info,
            size: None,
            duration: None,
            resolution: None,
            bitrate: None,
            sample_frequency: None,
            audio_channels: None,
            bits_per_sample: None,
        }
    }
}

/// An item of the catalogue: a file the client can play.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    /// The opaque identifier of the item.
    pub id: String,
    /// The identifier of the container the item lives in.
    pub parent_id: String,
    /// Whether the item is restricted (`INT-060`).
    pub restricted: bool,
    /// The class of the item (`INT-061`).
    pub class: ObjectClass,
    /// The title the client shows.
    pub title: String,
    /// The creator of the work, as `dc:creator` (`INT-076`).
    pub creator: Option<String>,
    /// The artist, as `upnp:artist` (`INT-069`).
    pub artist: Option<String>,
    /// The album, as `upnp:album` (`INT-069`).
    pub album: Option<String>,
    /// The genre, as `upnp:genre` (`INT-069`).
    pub genre: Option<String>,
    /// The track number, as `upnp:originalTrackNumber` (`INT-070`).
    pub track_number: Option<u32>,
    /// The modification date, as `dc:date` (`INT-068`).
    pub date: Option<String>,
    /// The address of the artwork, as `upnp:albumArtURI` (`INT-067`).
    pub album_art_uri: Option<String>,
    /// The files of the item; the first one is the one the client plays.
    pub resources: Vec<Resource>,
}

impl Item {
    /// Builds an item with its identifier, its container and its title.
    pub fn new(
        id: impl Into<String>,
        parent_id: impl Into<String>,
        title: impl Into<String>,
        class: ObjectClass,
    ) -> Self {
        Self {
            id: id.into(),
            parent_id: parent_id.into(),
            restricted: true,
            class,
            title: title.into(),
            creator: None,
            artist: None,
            album: None,
            genre: None,
            track_number: None,
            date: None,
            album_art_uri: None,
            resources: Vec::new(),
        }
    }
}

/// A container of the catalogue: a folder the client can open.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Container {
    /// The opaque identifier of the container.
    pub id: String,
    /// The identifier of the container this one lives in.
    pub parent_id: String,
    /// Whether the container is restricted (`INT-060`).
    pub restricted: bool,
    /// Whether the contents of the container can be searched.
    pub searchable: bool,
    /// The class of the container (`INT-061`).
    pub class: ObjectClass,
    /// The title the client shows.
    pub title: String,
    /// The number of children the container holds (`INT-060`).
    pub child_count: u32,
    /// The creator of the work, as `dc:creator` (`INT-076`).
    pub creator: Option<String>,
    /// The modification date, as `dc:date` (`INT-068`).
    pub date: Option<String>,
}

impl Container {
    /// Builds a container with its identifier, its parent and its title.
    pub fn new(
        id: impl Into<String>,
        parent_id: impl Into<String>,
        title: impl Into<String>,
        child_count: u32,
    ) -> Self {
        Self {
            id: id.into(),
            parent_id: parent_id.into(),
            restricted: true,
            searchable: true,
            class: ObjectClass::StorageFolder,
            title: title.into(),
            child_count,
            creator: None,
            date: None,
        }
    }
}

/// A whole DIDL-Lite document.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DidlDocument {
    /// The containers the document describes.
    pub containers: Vec<Container>,
    /// The items the document describes.
    pub items: Vec<Item>,
}

impl DidlDocument {
    /// Builds a document that describes nothing.
    pub fn new() -> Self {
        Self::default()
    }

    /// The document as XML, with every property.
    pub fn to_xml(&self) -> String {
        crate::didl::serializer::render(self, None)
    }

    /// Builds a document that describes the given items.
    pub fn with_items(items: Vec<Item>) -> Self {
        Self {
            containers: Vec::new(),
            items,
        }
    }

    /// Builds a document that describes the given containers.
    pub fn with_containers(containers: Vec<Container>) -> Self {
        Self {
            containers,
            items: Vec::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Container, DidlDocument, Item, ObjectClass};

    /// The classes are the ones the schema allows (`INT-061`).
    #[test]
    fn the_classes_are_the_ones_of_the_schema() {
        assert_eq!(
            ObjectClass::MusicTrack.as_str(),
            "object.item.audioItem.musicTrack"
        );
        assert_eq!(ObjectClass::Movie.as_str(), "object.item.videoItem.movie");
        assert_eq!(
            ObjectClass::VideoBroadcast.as_str(),
            "object.item.videoItem.videoBroadcast"
        );
        assert_eq!(ObjectClass::Photo.as_str(), "object.item.imageItem.photo");
        assert_eq!(
            ObjectClass::StorageFolder.as_str(),
            "object.container.storageFolder"
        );
    }

    /// An item is restricted and a container is searchable, because the server
    /// does not offer a way to change either.
    #[test]
    fn the_defaults_are_the_safe_ones() {
        let item = Item::new("a3f1", "1", "Nightfall", ObjectClass::MusicTrack);
        assert!(item.restricted);
        assert!(item.resources.is_empty());

        let container = Container::new("1", "0", "Music", 12);
        assert!(container.restricted);
        assert!(container.searchable);
        assert_eq!(container.child_count, 12);
    }

    /// A document that describes nothing is empty, not broken.
    #[test]
    fn an_empty_document_is_empty() {
        let document = DidlDocument::new();
        assert!(document.items.is_empty());
        assert!(document.containers.is_empty());
    }
}
