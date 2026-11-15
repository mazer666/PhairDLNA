//! The mapping from the catalogue to the document.
//!
//! What: the translation of the catalogue objects into the objects of the
//!       document — the class of an item, the address of its bytes and the
//!       address of its artwork.
//! Why: the catalogue knows about files and the document knows about clients,
//!       and the two must not be mixed: the catalogue never builds XML and the
//!       document never reaches the filesystem. The mapping is also where the
//!       path stops: the document carries the opaque identifier of a file and
//!       the address of the endpoint that serves it, never the place the file
//!       lives (`INT-062`, `PLT-008`, `SEC-093`).
//! Example: `item_from_catalogue` builds the item of a file with its
//!       resource.

use crate::catalogue::mime::is_served;
use crate::catalogue::{ContentContainer, ContentItem, MediaKind};
use crate::didl::model::{Container, Item, ObjectClass, Resource};
use crate::didl::policy::ProtocolPolicy;
use crate::didl::uri::{artwork_uri, media_uri};
use crate::didl::value::{format_duration, format_resolution, is_iso_date};

/// The class of a catalogue item (`INT-061`).
///
/// A file the catalogue cannot describe further gets the generic class of the
/// schema; the five named classes are the ones of the requirement.
pub fn class_of(kind: MediaKind) -> ObjectClass {
    match kind {
        MediaKind::MusicTrack => ObjectClass::MusicTrack,
        MediaKind::Movie => ObjectClass::Movie,
        MediaKind::VideoBroadcast => ObjectClass::VideoBroadcast,
        MediaKind::Photo => ObjectClass::Photo,
        MediaKind::Other => ObjectClass::Item,
    }
}

/// Builds the item of a catalogue item.
///
/// The catalogue decides which items carry artwork: the address is built here,
/// from the opaque identifier the catalogue knows (`INT-067`).
pub fn item_from_catalogue(item: &ContentItem, base_url: &str, policy: &ProtocolPolicy) -> Item {
    let metadata = &item.metadata;
    let mut mapped = Item {
        id: item.id.clone(),
        parent_id: item.parent_id.clone(),
        restricted: true,
        class: class_of(item.kind),
        title: item.title.clone(),
        creator: metadata.artist.clone(),
        artist: metadata.artist.clone(),
        album: metadata.album.clone(),
        genre: metadata.genre.clone(),
        track_number: metadata.track_number,
        date: metadata.date.clone().filter(|date| is_iso_date(date)),
        album_art_uri: metadata
            .album_art_id
            .as_ref()
            .map(|id| artwork_uri(base_url, id)),
        resources: Vec::new(),
    };
    if let Some(resource) = resource_from_catalogue(item, base_url, policy) {
        mapped.resources.push(resource);
    }
    mapped
}

/// Builds the container of a catalogue container.
pub fn container_from_catalogue(container: &ContentContainer) -> Container {
    Container {
        id: container.id.clone(),
        parent_id: container.parent_id.clone(),
        restricted: true,
        searchable: true,
        class: ObjectClass::StorageFolder,
        title: container.title.clone(),
        child_count: container.total.unwrap_or(container.children.len() as u32),
        creator: None,
        date: None,
    }
}

/// Builds the resource of a catalogue item.
///
/// An item whose format the server cannot deliver gets no resource at all:
/// the document must not name a format the server does not serve (`INT-064`).
/// A profile that cannot be written does not take the resource with it: the
/// server claims no profile and the file stays playable (`INT-079`).
fn resource_from_catalogue(
    item: &ContentItem,
    base_url: &str,
    policy: &ProtocolPolicy,
) -> Option<Resource> {
    let metadata = &item.metadata;
    let mime_type = metadata.mime_type.as_ref()?;
    if !is_served(mime_type) {
        return None;
    }
    let protocol_info = policy.protocol_info(mime_type, metadata.dlna_profile.as_deref())?;
    let mut resource = Resource::new(media_uri(base_url, &item.id), protocol_info);
    resource.size = metadata.size;
    resource.duration = metadata.duration.map(format_duration);
    resource.resolution = metadata
        .resolution
        .map(|(width, height)| format_resolution(width, height));
    resource.bitrate = metadata.bitrate;
    resource.sample_frequency = metadata.sample_frequency;
    resource.audio_channels = metadata.audio_channels;
    resource.bits_per_sample = metadata.bits_per_sample;
    Some(resource)
}

#[cfg(test)]
mod tests {
    use super::{class_of, container_from_catalogue, item_from_catalogue};
    use crate::catalogue::{ContentContainer, ContentItem, MediaKind, MediaMetadata};
    use crate::didl::model::ObjectClass;
    use crate::didl::policy::ProtocolPolicy;

    /// The five kinds of the catalogue get their class (`INT-061`).
    #[test]
    fn the_kinds_get_their_class() {
        assert_eq!(class_of(MediaKind::MusicTrack), ObjectClass::MusicTrack);
        assert_eq!(class_of(MediaKind::Movie), ObjectClass::Movie);
        assert_eq!(
            class_of(MediaKind::VideoBroadcast),
            ObjectClass::VideoBroadcast
        );
        assert_eq!(class_of(MediaKind::Photo), ObjectClass::Photo);
        assert_eq!(class_of(MediaKind::Other), ObjectClass::Item);
    }

    /// An item of a served format carries a resource that names the format
    /// (`INT-064`).
    #[test]
    fn an_item_of_a_served_format_carries_a_resource() {
        let mut item = ContentItem::new("a3f1", "1", "Nightfall", MediaKind::MusicTrack);
        item.metadata = MediaMetadata {
            mime_type: Some("audio/mpeg".to_owned()),
            size: Some(7_340_032),
            ..MediaMetadata::default()
        };
        let mapped =
            item_from_catalogue(&item, "http://192.168.1.20:8200", &ProtocolPolicy::plain());
        assert_eq!(mapped.resources.len(), 1);
        assert_eq!(
            mapped.resources[0].protocol_info.as_string(),
            "http-get:*:audio/mpeg:*"
        );
        assert_eq!(
            mapped.resources[0].uri,
            "http://192.168.1.20:8200/media/a3f1"
        );
    }

    /// An item of a format the server cannot deliver carries no resource.
    #[test]
    fn an_item_of_an_unserved_format_carries_no_resource() {
        let mut item = ContentItem::new("a3f1", "1", "Nightfall", MediaKind::MusicTrack);
        item.metadata = MediaMetadata {
            mime_type: Some("video/x-msvideo".to_owned()),
            ..MediaMetadata::default()
        };
        let mapped =
            item_from_catalogue(&item, "http://192.168.1.20:8200", &ProtocolPolicy::plain());
        assert!(mapped.resources.is_empty());
    }

    /// An item without a known format carries no resource either.
    #[test]
    fn an_item_without_a_format_carries_no_resource() {
        let item = ContentItem::new("a3f1", "1", "Nightfall", MediaKind::MusicTrack);
        let mapped =
            item_from_catalogue(&item, "http://192.168.1.20:8200", &ProtocolPolicy::plain());
        assert!(mapped.resources.is_empty());
    }

    /// A date the server cannot read is left out rather than guessed
    /// (`INT-068`).
    #[test]
    fn an_unreadable_date_is_left_out() {
        let mut item = ContentItem::new("a3f1", "1", "Nightfall", MediaKind::MusicTrack);
        item.metadata.date = Some("28.05.2024".to_owned());
        let mapped =
            item_from_catalogue(&item, "http://192.168.1.20:8200", &ProtocolPolicy::plain());
        assert!(mapped.date.is_none());

        item.metadata.date = Some("2024-05-28T02:28:47".to_owned());
        let mapped =
            item_from_catalogue(&item, "http://192.168.1.20:8200", &ProtocolPolicy::plain());
        assert_eq!(mapped.date.as_deref(), Some("2024-05-28T02:28:47"));
    }

    /// The artist is written under both names (`INT-076`).
    #[test]
    fn the_artist_is_written_under_both_names() {
        let mut item = ContentItem::new("a3f1", "1", "Nightfall", MediaKind::MusicTrack);
        item.metadata.artist = Some("Nightfall Orchestra".to_owned());
        let mapped =
            item_from_catalogue(&item, "http://192.168.1.20:8200", &ProtocolPolicy::plain());
        assert_eq!(mapped.artist.as_deref(), Some("Nightfall Orchestra"));
        assert_eq!(mapped.creator.as_deref(), Some("Nightfall Orchestra"));
    }

    /// A profile that cannot be written does not take the resource with it:
    /// the server claims no profile and the file stays playable (`INT-079`).
    #[test]
    fn an_unwritable_profile_keeps_the_resource() {
        let mut item = ContentItem::new("a3f1", "1", "Nightfall", MediaKind::MusicTrack);
        item.metadata = MediaMetadata {
            mime_type: Some("audio/mpeg".to_owned()),
            dlna_profile: Some("a:b".to_owned()),
            ..MediaMetadata::default()
        };
        let policy = crate::didl::policy::ProtocolPolicy::dlna(false, crate::didl::DlnaFlags::NONE);
        let mapped = item_from_catalogue(&item, "http://192.168.1.20:8200", &policy);
        assert_eq!(mapped.resources.len(), 1);
        let entry = mapped.resources[0].protocol_info.as_string();
        assert!(!entry.contains("DLNA.ORG_PN"));
        assert_eq!(entry.split(':').count(), 4);
    }

    /// The bitrate is carried in bytes per second, the unit of `INT-074`.
    #[test]
    fn the_bitrate_is_carried_in_bytes_per_second() {
        let mut item = ContentItem::new("a3f1", "1", "Nightfall", MediaKind::MusicTrack);
        item.metadata = MediaMetadata {
            mime_type: Some("audio/mpeg".to_owned()),
            bitrate: Some(31_744),
            ..MediaMetadata::default()
        };
        let mapped =
            item_from_catalogue(&item, "http://192.168.1.20:8200", &ProtocolPolicy::plain());
        assert_eq!(mapped.resources[0].bitrate, Some(31_744));
    }

    /// A container carries the number of children it holds (`INT-060`).
    #[test]
    fn a_container_carries_its_child_count() {
        let container = ContentContainer {
            id: "1".to_owned(),
            parent_id: "0".to_owned(),
            title: "Music".to_owned(),
            update_id: 3,
            children: Vec::new(),
            total: Some(12),
        };
        let mapped = container_from_catalogue(&container);
        assert_eq!(mapped.child_count, 12);
        assert_eq!(mapped.class, ObjectClass::StorageFolder);
    }

    /// A container whose count cannot be computed carries the number of
    /// children the catalogue returned.
    #[test]
    fn an_uncountable_container_carries_what_it_knows() {
        let container = ContentContainer {
            id: "1".to_owned(),
            parent_id: "0".to_owned(),
            title: "Music".to_owned(),
            update_id: 3,
            children: vec![ContentItem::new("2", "1", "One", MediaKind::Photo)],
            total: None,
        };
        assert_eq!(container_from_catalogue(&container).child_count, 1);
    }
}
