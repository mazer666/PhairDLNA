//! The catalogue of the media the server shares.
//!
//! What: the items and the containers of the catalogue, the metadata of a
//!       media file, and the narrow trait the protocol layer reads them
//!       through.
//! Why: the catalogue owns the filesystem, the indexing and the watching of
//!       the folders (F-08, F-09). The protocol layer must not reach the
//!       filesystem, so it sees the catalogue only through this trait — a door
//!       that is narrow on purpose.
//! Example: `ContentSource::container` returns the container a browse asks
//!       for, or `None` when the server does not know it.

use std::time::Duration;

pub mod mime;

/// What kind of thing an item is.
///
/// The catalogue says *what a file is*; the DIDL-Lite document says *what a
/// client calls it*. The mapping between the two lives in the `didl` module
/// (`INT-061`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaKind {
    /// A music track.
    MusicTrack,
    /// A film.
    Movie,
    /// A live or streamed video source.
    VideoBroadcast,
    /// A photo.
    Photo,
    /// A file of a kind the server does not describe further.
    Other,
}

/// The metadata of a media file.
///
/// Every field is optional: a file that carries no tag, no duration or no
/// artwork is still described, only with less in the description (`INT-063`,
/// `INT-069`).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct MediaMetadata {
    /// The MIME type the HTTP layer sends for the file.
    pub mime_type: Option<String>,
    /// The size of the file in bytes.
    pub size: Option<u64>,
    /// The running time of the media.
    pub duration: Option<Duration>,
    /// The pixel dimensions of an image.
    pub resolution: Option<(u32, u32)>,
    /// The bitrate in bytes per second, as the schema defines it (`INT-074`).
    pub bitrate: Option<u32>,
    /// The sample rate of an audio file in Hz.
    pub sample_frequency: Option<u32>,
    /// The number of channels of an audio file.
    pub audio_channels: Option<u16>,
    /// The number of bits per sample of an audio file.
    pub bits_per_sample: Option<u16>,
    /// The artist, from the tags of the file.
    pub artist: Option<String>,
    /// The album, from the tags of the file.
    pub album: Option<String>,
    /// The genre, from the tags of the file.
    pub genre: Option<String>,
    /// The track number, from the tags of the file.
    pub track_number: Option<u32>,
    /// The modification date, as an ISO 8601 date or date-time (`INT-068`).
    pub date: Option<String>,
    /// The opaque identifier of the artwork of the album, if there is one.
    ///
    /// The identifier, not a URL: the address of the artwork is built from the
    /// base URL of the server, so that a tag file cannot make the client
    /// fetch a third-party address (`INT-067`).
    pub album_art_id: Option<String>,
    /// The DLNA media format profile the file conforms to, if it is known.
    ///
    /// The profile is only claimed when the content really conforms to it
    /// (`INT-079`); until the conformance rules exist, the catalogue leaves
    /// this empty and the server stays in the plain mode.
    pub dlna_profile: Option<String>,
}

/// One item of the catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentItem {
    /// The opaque identifier of the item.
    pub id: String,
    /// The identifier of the container the item lives in.
    pub parent_id: String,
    /// The name the client shows.
    pub title: String,
    /// What kind of thing the item is.
    pub kind: MediaKind,
    /// The metadata of the file behind the item.
    pub metadata: MediaMetadata,
}

impl ContentItem {
    /// Builds an item without metadata.
    pub fn new(
        id: impl Into<String>,
        parent_id: impl Into<String>,
        title: impl Into<String>,
        kind: MediaKind,
    ) -> Self {
        Self {
            id: id.into(),
            parent_id: parent_id.into(),
            title: title.into(),
            kind,
            metadata: MediaMetadata::default(),
        }
    }
}

/// A container of the catalogue.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContentContainer {
    /// The opaque identifier of the container.
    pub id: String,
    /// The identifier of the container this one lives in.
    pub parent_id: String,
    /// The name the client shows.
    pub title: String,
    /// The identifier that changes when the content of the container changes.
    pub update_id: u32,
    /// The children of the container, in the order the catalogue returns them.
    pub children: Vec<ContentItem>,
    /// The total number of children, or `None` when it cannot be computed.
    pub total: Option<u32>,
}

/// The read-only view of the catalogue the ContentDirectory service needs.
///
/// The catalogue module owns the implementation (F-08, F-09); this trait is
/// the narrow door it opens for the protocol layer, so that the protocol
/// layer cannot reach the filesystem.
pub trait ContentSource {
    /// The identifier that changes when the catalogue changes.
    fn system_update_id(&self) -> u32;

    /// The container with the given identifier, or `None` when there is none.
    fn container(&self, object_id: &str) -> Option<ContentContainer>;

    /// The items of a container whose property contains the given text.
    ///
    /// The catalogue owns the search (F-09); until it exists, the search runs
    /// over the children the container returned.
    fn search(&self, container_id: &str, property: &str, text: &str) -> Vec<ContentItem> {
        match self.container(container_id) {
            Some(container) => container
                .children
                .into_iter()
                .filter(|item| item.matches(property, text))
                .collect(),
            None => Vec::new(),
        }
    }
}

impl ContentItem {
    /// Whether the item matches a search expression.
    ///
    /// Only `dc:title` is searchable today; the comparison ignores the case,
    /// because a client does not know how the title was written down.
    pub fn matches(&self, property: &str, text: &str) -> bool {
        match property {
            "dc:title" => self.title.to_lowercase().contains(&text.to_lowercase()),
            _ => false,
        }
    }
}
