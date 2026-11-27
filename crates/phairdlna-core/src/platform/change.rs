//! The change events of the filesystem.
//!
//! What: the contract between the core and the operating system for the
//!       changes of a watched root — *what* changed, and *how much* the
//!       platform promises about it.
//! Why: a newly added file must be visible in the catalogue within 30 seconds
//!       on a file system that delivers reliable change events, and the
//!       catalogue must fall back to the scheduled rescan on one that does not
//!       (`FUN-025`). The core needs both answers without asking the operating
//!       system directly, or the core stops being portable (`ADR-0003`,
//!       ADR-0014).
//! Example: a watcher that reports `Reliable` for the root "Films" lets the
//!       catalogue act on a single event; a watcher that reports `Unavailable`
//!       makes the catalogue read the root again on the schedule instead.
//!
//! **The trait reads; it never writes.** Like the volume watcher, it is a
//! polling interface: the core asks for the changes since the last call, and
//! the platform answers however it can. The core therefore stays free of every
//! event loop of the operating system, and a platform that cannot watch
//! anything answers "not supported" instead of pretending to watch (`PLT-003`).
//!
//! **A path that arrives here is not trusted.** The events name paths on the
//! disk, and a path is only worth something when it lies inside the root it
//! belongs to. The core checks every path against its root before it uses it
//! (`SEC-001`, `SEC-003`); the watcher does not have to be right for that.

use std::path::{Path, PathBuf};

use super::traits::PlatformResult;

/// What the platform can promise about the change events of a root.
///
/// The answer decides whether the catalogue may act on a single event or
/// whether the root falls back to the scheduled rescan (`FUN-025`, `PLT-019`).
/// A platform that cannot say must not be assumed to be reliable: an unknown
/// answer is treated like a missing one.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ChangeReliability {
    /// Every change of the root reaches the core.
    Reliable,
    /// Most changes of the root reach the core; some may be lost.
    ///
    /// A platform that says this must also say **when** it lost events, so
    /// that the root can be read again (`FileChange::RescanRequired`).
    BestEffort,
    /// No change of the root reaches the core.
    Unavailable,
    /// The platform cannot say.
    #[default]
    Unknown,
}

impl ChangeReliability {
    /// Whether the root has to be read again on the schedule.
    ///
    /// The answer is "yes" for everything but `Reliable`. A root whose events
    /// are unreliable is read again on the schedule, so that the catalogue is
    /// never knowingly stale (`FUN-025`, `FUN-024`).
    pub fn requires_scheduled_rescan(self) -> bool {
        !matches!(self, ChangeReliability::Reliable)
    }
}

/// One change of the filesystem.
///
/// The events name **paths**, not catalogue entries: the core owns the
/// semantics of a change — what it means for the index, whether the entry
/// stays or goes — and the platform owns only the mechanism that reports it
/// (`ADR-0003`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FileChange {
    /// A file or a folder appeared.
    Created {
        /// The path of the new file or folder, on the disk.
        path: PathBuf,
    },
    /// The content of a file changed.
    Modified {
        /// The path of the file that changed, on the disk.
        path: PathBuf,
    },
    /// A file or a folder went away.
    Removed {
        /// The path that is not there any more, on the disk.
        path: PathBuf,
    },
    /// A file or a folder was renamed.
    ///
    /// The two paths are the same file under two names; the core removes the
    /// old entry and adds the new one, so that the identifier of the object
    /// does not change (`FUN-011`).
    Renamed {
        /// The path the file had before.
        from: PathBuf,
        /// The path the file has now.
        to: PathBuf,
    },
    /// The watcher lost events, and the root must be read again.
    ///
    /// A platform that delivers events but loses some of them says so here
    /// instead of leaving the catalogue stale in silence. The `IN_Q_OVERFLOW`
    /// of Linux is the usual reason (platform-integration §8.2).
    RescanRequired {
        /// The root whose events were lost.
        root: PathBuf,
    },
}

impl FileChange {
    /// The path the change is about.
    ///
    /// For a rename that is the **new** path: the path the change leaves the
    /// catalogue with is the old one, and it is asked for separately.
    pub fn path(&self) -> &Path {
        match self {
            FileChange::Created { path }
            | FileChange::Modified { path }
            | FileChange::Removed { path } => path,
            FileChange::RescanRequired { root } => root,
            FileChange::Renamed { to, .. } => to,
        }
    }

    /// The path the change came from, when there is one.
    ///
    /// Only a rename has one: the path the file had before it was renamed.
    pub fn previous_path(&self) -> Option<&Path> {
        match self {
            FileChange::Renamed { from, .. } => Some(from),
            _ => None,
        }
    }
}

/// The change events of a platform.
///
/// The trait answers two questions and asks none: *what changed since the last
/// call*, and *how much can be trusted about it*. The roots that are watched
/// are given to the adapter when it is built — the trait only reads, so that a
/// change of the root list is a new watcher and not a hidden mutation
/// (`ADR-0003`, ADR-0014).
pub trait ChangeWatcher {
    /// The changes seen since the last call.
    ///
    /// The call **drains** the queue: a change that was returned is not
    /// returned again. An empty list means "nothing happened since the last
    /// call", which is not the same as "nothing happened at all" — a platform
    /// that lost events says so with `FileChange::RescanRequired`.
    fn poll_changes(&self) -> PlatformResult<Vec<FileChange>>;

    /// What the platform can promise about the events of a root.
    ///
    /// The answer decides whether the catalogue may act on a single event or
    /// whether the root falls back to the scheduled rescan (`FUN-025`).
    ///
    /// The default is `Unknown`, because a platform that does not say must not
    /// be assumed to deliver every change (`PLT-003`).
    fn reliability(&self, _root: &Path) -> ChangeReliability {
        ChangeReliability::Unknown
    }
}

#[cfg(test)]
mod tests {
    use super::{ChangeReliability, ChangeWatcher, FileChange};
    use crate::platform::traits::PlatformError;
    use std::path::{Path, PathBuf};

    /// A watcher that never sees anything, like a platform that cannot watch.
    #[derive(Debug, Default)]
    struct SilentWatcher;

    impl ChangeWatcher for SilentWatcher {
        fn poll_changes(&self) -> crate::platform::traits::PlatformResult<Vec<FileChange>> {
            Err(PlatformError::unsupported("watching changes"))
        }
    }

    /// Only a reliable watcher saves the root from the scheduled rescan.
    #[test]
    fn only_a_reliable_watcher_saves_the_scheduled_rescan() {
        assert!(!ChangeReliability::Reliable.requires_scheduled_rescan());
        assert!(ChangeReliability::BestEffort.requires_scheduled_rescan());
        assert!(ChangeReliability::Unavailable.requires_scheduled_rescan());
        assert!(ChangeReliability::Unknown.requires_scheduled_rescan());
    }

    /// The unknown answer is the safe one: it is the default.
    #[test]
    fn the_unknown_answer_is_the_default() {
        assert_eq!(ChangeReliability::default(), ChangeReliability::Unknown);
    }

    /// A watcher that says nothing is asked about nothing.
    #[test]
    fn a_watcher_that_says_nothing_is_asked_about_nothing() {
        let watcher = SilentWatcher;
        assert_eq!(
            watcher.reliability(Path::new("/library")),
            ChangeReliability::Unknown
        );
    }

    /// A watcher that cannot watch says so instead of pretending.
    #[test]
    fn a_watcher_that_cannot_watch_says_so() {
        let watcher = SilentWatcher;
        assert!(watcher.poll_changes().is_err());
    }

    /// A change names the path it is about.
    #[test]
    fn a_change_names_the_path_it_is_about() {
        let path = PathBuf::from("/library/film.mkv");
        let created = FileChange::Created { path: path.clone() };
        assert_eq!(created.path(), path);
        assert!(created.previous_path().is_none());
    }

    /// A rename names the new path and the old one.
    #[test]
    fn a_rename_names_both_paths() {
        let rename = FileChange::Renamed {
            from: PathBuf::from("/library/old.mkv"),
            to: PathBuf::from("/library/new.mkv"),
        };
        assert_eq!(rename.path(), Path::new("/library/new.mkv"));
        assert_eq!(rename.previous_path(), Some(Path::new("/library/old.mkv")));
    }

    /// The lost events of a root name the root.
    #[test]
    fn the_lost_events_of_a_root_name_the_root() {
        let lost = FileChange::RescanRequired {
            root: PathBuf::from("/library"),
        };
        assert_eq!(lost.path(), Path::new("/library"));
        assert!(lost.previous_path().is_none());
    }
}
