#![allow(dead_code)]
//! Fixtures for the protocol tests.
//!
//! What: a catalogue with twelve children, a catalogue whose child count
//!       cannot be computed, and a catalogue whose identifier changes.
//! Why: the acceptance criteria of F-03 are written against a catalogue with
//!       twelve children, a catalogue that cannot count its children and a
//!       catalogue that changes. The fixtures make those cases reproducible
//!       without a filesystem.
//! Example: `TestCatalogue::new().container("1")` returns the container with
//!       the twelve children.

use phairdlna_core::upnp::{Container, ContentItem, ContentSource};

/// The number of children of the test catalogue.
pub const CHILD_COUNT: usize = 12;

/// A catalogue with a container of twelve children.
///
/// The children are in the reverse order of their titles, so that a test can
/// tell the order of the catalogue from the order the client asked for.
#[derive(Debug, Clone, Copy)]
pub struct TestCatalogue;

impl TestCatalogue {
    /// The container with the twelve children.
    pub fn children_container(&self) -> Container {
        let children: Vec<ContentItem> = (0..CHILD_COUNT)
            .map(|index| {
                let number = CHILD_COUNT - index;
                ContentItem::new(
                    format!("{}", 100 + index),
                    "1",
                    format!("Item {number:02}"),
                    "object.item.videoItem",
                )
            })
            .collect();
        Container {
            id: "1".to_owned(),
            parent_id: "0".to_owned(),
            title: "Content Directory".to_owned(),
            class: "object.container".to_owned(),
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

    fn container(&self, object_id: &str) -> Option<Container> {
        match object_id {
            "0" => Some(Container {
                id: "0".to_owned(),
                parent_id: "-1".to_owned(),
                title: "Root".to_owned(),
                class: "object.container".to_owned(),
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

    fn container(&self, object_id: &str) -> Option<Container> {
        if object_id != "1" {
            return None;
        }
        Some(Container {
            id: "1".to_owned(),
            parent_id: "0".to_owned(),
            title: "Content Directory".to_owned(),
            class: "object.container".to_owned(),
            update_id: 1,
            children: vec![ContentItem::new(
                "1",
                "1",
                "Only item",
                "object.item.videoItem",
            )],
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

    fn container(&self, object_id: &str) -> Option<Container> {
        if object_id != "1" {
            return None;
        }
        Some(Container {
            id: "1".to_owned(),
            parent_id: "0".to_owned(),
            title: "Content Directory".to_owned(),
            class: "object.container".to_owned(),
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

    fn container(&self, _object_id: &str) -> Option<Container> {
        None
    }
}
