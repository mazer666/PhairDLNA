#![allow(dead_code)]
//! Fixtures for the protocol tests.
//!
//! What: a catalogue with twelve children, a catalogue whose child count
//!       cannot be computed, a catalogue whose identifier changes, and a
//!       catalogue whose files carry metadata.
//! Why: the acceptance criteria of F-03 are written against a catalogue with
//!       twelve children, a catalogue that cannot count its children and a
//!       catalogue that changes. The criteria of F-05 are written against
//!       files that carry a duration, a size, dimensions and tags. The
//!       fixtures make those cases reproducible without a filesystem.
//! Example: `TestCatalogue::new().container("1")` returns the container with
//!       the twelve children.

use phairdlna_core::catalogue::{
    ContentContainer, ContentItem, ContentSource, MediaKind, MediaMetadata,
};

/// The number of children of the test catalogue.
pub const CHILD_COUNT: usize = 12;

/// The base address of the test server.
pub const BASE_URL: &str = "http://192.168.1.20:8200";

/// A catalogue with a container of twelve children.
///
/// The children are in the reverse order of their titles, so that a test can
/// tell the order of the catalogue from the order the client asked for.
#[derive(Debug, Clone, Copy)]
pub struct TestCatalogue;

impl TestCatalogue {
    /// The container with the twelve children.
    pub fn children_container(&self) -> ContentContainer {
        let children: Vec<ContentItem> = (0..CHILD_COUNT)
            .map(|index| {
                let number = CHILD_COUNT - index;
                ContentItem::new(
                    format!("{}", 100 + index),
                    "1",
                    format!("Item {number:02}"),
                    MediaKind::Movie,
                )
            })
            .collect();
        ContentContainer {
            id: "1".to_owned(),
            parent_id: "0".to_owned(),
            title: "Content Directory".to_owned(),
            update_id: 42,
            children,
            total: Some(CHILD_COUNT as u32),
        }
    }
}

impl ContentSource for TestCatalogue {
    fn system_update_id(&self) -> u32 {
        42
    }

    fn container(&self, object_id: &str) -> Option<ContentContainer> {
        match object_id {
            "0" => Some(ContentContainer {
                id: "0".to_owned(),
                parent_id: "-1".to_owned(),
                title: "Root".to_owned(),
                update_id: 42,
                children: Vec::new(),
                total: Some(0),
            }),
            "1" => Some(self.children_container()),
            _ => None,
        }
    }
}

/// A catalogue whose number of children cannot be computed.
///
/// The catalogue of a server that cannot count its children answers a browse
/// with a `TotalMatches` of `0`, and with the error `720` when it returns
/// nothing at all (`INT-046`).
#[derive(Debug, Clone, Copy)]
pub struct UncountableCatalogue;

impl ContentSource for UncountableCatalogue {
    fn system_update_id(&self) -> u32 {
        1
    }

    fn container(&self, object_id: &str) -> Option<ContentContainer> {
        if object_id != "1" {
            return None;
        }
        Some(ContentContainer {
            id: "1".to_owned(),
            parent_id: "0".to_owned(),
            title: "Content Directory".to_owned(),
            update_id: 1,
            children: vec![ContentItem::new("1", "1", "Only item", MediaKind::Movie)],
            total: None,
        })
    }
}

/// A catalogue that is empty and whose number of children is unknown.
///
/// A browse of this catalogue returns nothing, so the answer is the error
/// `720` (`INT-046`).
#[derive(Debug, Clone, Copy)]
pub struct EmptyUncountableCatalogue;

impl ContentSource for EmptyUncountableCatalogue {
    fn system_update_id(&self) -> u32 {
        1
    }

    fn container(&self, object_id: &str) -> Option<ContentContainer> {
        if object_id != "1" {
            return None;
        }
        Some(ContentContainer {
            id: "1".to_owned(),
            parent_id: "0".to_owned(),
            title: "Content Directory".to_owned(),
            update_id: 1,
            children: Vec::new(),
            total: None,
        })
    }
}

/// A catalogue whose identifier changes when the catalogue changes.
///
/// The identifier is the one a client reads with `GetSystemUpdateID`, and it
/// must grow when the catalogue changes (`INT-037`).
#[derive(Debug, Clone, Copy)]
pub struct ChangingCatalogue {
    /// The identifier the catalogue currently has.
    pub update_id: u32,
}

impl ChangingCatalogue {
    /// A catalogue with the given identifier.
    pub fn new(update_id: u32) -> Self {
        Self { update_id }
    }
}

impl ContentSource for ChangingCatalogue {
    fn system_update_id(&self) -> u32 {
        self.update_id
    }

    fn container(&self, _object_id: &str) -> Option<ContentContainer> {
        None
    }
}

/// A music track with every tag a client reads.
///
/// The item is the one the criteria of F-05 are written against: it has a
/// title, an artist, an album, a genre, a track number, a date, an artwork
/// and a file with a duration, a size and a bitrate.
pub fn music_track() -> ContentItem {
    let mut item = ContentItem::new("a3f1", "1", "Nightfall", MediaKind::MusicTrack);
    item.metadata = MediaMetadata {
        mime_type: Some("audio/mpeg".to_owned()),
        size: Some(7_340_032),
        duration: Some(std::time::Duration::from_secs(232)),
        bitrate: Some(31_744),
        resolution: None,
        sample_frequency: Some(44_100),
        audio_channels: Some(2),
        bits_per_sample: Some(16),
        artist: Some("Nightfall Orchestra".to_owned()),
        album: Some("Long Nights".to_owned()),
        genre: Some("Ambient".to_owned()),
        track_number: Some(3),
        date: Some("2024-05-28T02:28:47".to_owned()),
        album_art_id: Some("art-a3f1".to_owned()),
        dlna_profile: None,
    };
    item
}

/// A film with a duration and a size.
pub fn movie() -> ContentItem {
    let mut item = ContentItem::new("b7c2", "1", "The Long Night", MediaKind::Movie);
    item.metadata = MediaMetadata {
        mime_type: Some("video/mp4".to_owned()),
        size: Some(1_073_741_824),
        duration: Some(std::time::Duration::from_secs(6_723)),
        bitrate: Some(524_288),
        date: Some("2023-11-02".to_owned()),
        ..MediaMetadata::default()
    };
    item
}

/// A photo with its dimensions.
pub fn photo() -> ContentItem {
    let mut item = ContentItem::new("c9d4", "1", "Harbour", MediaKind::Photo);
    item.metadata = MediaMetadata {
        mime_type: Some("image/jpeg".to_owned()),
        size: Some(2_411_724),
        resolution: Some((1920, 1080)),
        ..MediaMetadata::default()
    };
    item
}

/// An item whose title tries to break out of the document.
///
/// The title carries the characters XML reserves and a closing tag, because
/// a file name is untrusted input (`SEC-007`, `INT-065`).
pub fn hostile_item() -> ContentItem {
    let mut item = ContentItem::new(
        "d1e2",
        "1",
        "</dc:title><evil/> & \"quoted\" 'single'",
        MediaKind::Movie,
    );
    item.metadata = MediaMetadata {
        mime_type: Some("video/mp4".to_owned()),
        ..MediaMetadata::default()
    };
    item
}

/// A catalogue whose files carry metadata.
#[derive(Debug, Clone, Copy)]
pub struct MediaCatalogue;

impl ContentSource for MediaCatalogue {
    fn system_update_id(&self) -> u32 {
        7
    }

    fn container(&self, object_id: &str) -> Option<ContentContainer> {
        if object_id != "1" {
            return None;
        }
        Some(ContentContainer {
            id: "1".to_owned(),
            parent_id: "0".to_owned(),
            title: "Media".to_owned(),
            update_id: 7,
            children: vec![music_track(), movie(), photo(), hostile_item()],
            total: Some(4),
        })
    }
}
