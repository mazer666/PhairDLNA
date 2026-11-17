//! The priority classes of the server.
//!
//! What: the four kinds of work the server arbitrates between.
//! Why: `RES-001` asks for **one** scheduler with explicit priority classes.
//!      The classes live in their own module, so that the order of the
//!      importance is written down in one place and cannot drift between the
//!      places that use it.
//! Example: `Priority::Streaming` is the class of the work a client waits for
//!       while a film is playing; `Priority::Background` is the class of the
//!       scan of the library.
//!
//! **The order of the variants is the order of the importance.** The smaller
//! the variant, the more important the work: `Streaming` is the smallest and
//! `Background` the largest. The `Ord` implementation of the enum therefore
//! *is* the priority order, and no second comparison function exists.

/// What kind of work a task is.
///
/// The variants are ordered by importance, from the most important to the
/// least important.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Priority {
    /// The transfer of a media file to a client.
    ///
    /// A film that stops is a film that the user notices. The work of this
    /// class therefore takes precedence over everything else (`RES-001`).
    Streaming,
    /// The discovery and the control plane: the SOAP requests of a client.
    ///
    /// A client that waits for a browse waits for an answer, and a television
    /// that waits too long gives up (`PERF-015`).
    ControlPlane,
    /// The reading of the metadata and the production of the thumbnails.
    ///
    /// The work is worth doing, but a client may wait a moment for a cover.
    Media,
    /// The scan of the library and the maintenance of the index.
    ///
    /// The work is invisible to the user and may wait: it is the work that
    /// yields when the machine is busy (`RES-003`).
    Background,
}

impl Priority {
    /// Every class, from the most important to the least important.
    pub const ALL: [Priority; 4] = [
        Priority::Streaming,
        Priority::ControlPlane,
        Priority::Media,
        Priority::Background,
    ];

    /// The place of the class in the order.
    ///
    /// The place is the index of the queue of the class and the index of its
    /// counters; it is stable, because the order of the variants is.
    pub fn index(self) -> usize {
        self as usize
    }

    /// The name of the class, as it appears in a report and in a log.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Streaming => "streaming",
            Self::ControlPlane => "control_plane",
            Self::Media => "media",
            Self::Background => "background",
        }
    }

    /// Whether the work of this class may be shed when the machine is busy.
    ///
    /// Only the background work is shed: the indexing yields when the machine
    /// is busy (`RES-003`), while a stream, a control request and a thumbnail
    /// wait their turn instead of being refused (`RES-004`).
    pub fn is_sheddable(self) -> bool {
        matches!(self, Self::Background)
    }
}

#[cfg(test)]
mod tests {
    use super::Priority;

    /// The classes are ordered by importance: streaming first, background last.
    #[test]
    fn the_classes_are_ordered_by_importance() {
        assert!(Priority::Streaming < Priority::ControlPlane);
        assert!(Priority::ControlPlane < Priority::Media);
        assert!(Priority::Media < Priority::Background);
    }

    /// Every class has a name of its own.
    #[test]
    fn every_class_has_a_name() {
        let mut names: Vec<&str> = Priority::ALL.iter().map(|class| class.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), Priority::ALL.len());
    }

    /// Only the background work is shed; the rest waits its turn.
    #[test]
    fn only_the_background_work_is_sheddable() {
        assert!(Priority::Background.is_sheddable());
        assert!(!Priority::Streaming.is_sheddable());
        assert!(!Priority::ControlPlane.is_sheddable());
        assert!(!Priority::Media.is_sheddable());
    }

    /// The place of a class is the index of its queue.
    #[test]
    fn the_places_are_stable() {
        assert_eq!(Priority::Streaming.index(), 0);
        assert_eq!(Priority::ControlPlane.index(), 1);
        assert_eq!(Priority::Media.index(), 2);
        assert_eq!(Priority::Background.index(), 3);
        assert_eq!(Priority::ALL.len(), 4);
    }
}
