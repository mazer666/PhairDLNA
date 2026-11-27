//! Native filesystem change watcher implementation (FUN-025).
//!
//! What: watches configured media root folders using OS-native notification APIs.
//! Why: automatic catalogue updates when media files are added, modified, or removed.
//! Example: copying a video into a shared folder triggers change detection.
//!
//! **The changes are kept until the burst settles.** The platform delivers a
//! copy operation as a flood of small events, and the daemon would read the
//! catalogue once per event. The watcher therefore holds the events until the
//! disk has been quiet for a moment, and then hands them out — once, and not
//! one event at a time (`FUN-025`).
//!
//! **The events are never lost.** A burst that is still running is held, not
//! dropped: the events of the next poll are added to the ones that wait. Only
//! the settled events are handed out, and they are handed out exactly once.

use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::sync::Mutex;
use std::time::{Duration, Instant};

use notify::{Config, Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use super::change::{ChangeReliability, ChangeWatcher, FileChange};
use super::traits::{PlatformError, PlatformResult};

/// Minimum duration to coalesce / debounce filesystem bursts.
const DEBOUNCE_DURATION: Duration = Duration::from_millis(500);

/// An active filesystem change watcher utilizing platform notification backends.
pub struct FsChangeWatcher {
    /// Holds the active notify watcher instance.
    _watcher: Mutex<Option<RecommendedWatcher>>,
    /// Channel receiver for events delivered by notify.
    receiver: Mutex<Receiver<notify::Result<Event>>>,
    /// The events that wait for the burst to settle.
    pending: Mutex<Pending>,
    /// Watched paths and their recognized reliability.
    roots: Vec<PathBuf>,
}

/// The events that the platform delivered and the daemon has not seen yet.
#[derive(Debug, Default)]
struct Pending {
    /// The events that wait.
    changes: Vec<FileChange>,
    /// When the last event arrived.
    last_event: Option<Instant>,
}

impl Pending {
    /// Adds the changes of one event of the platform.
    ///
    /// An event that the platform could not deliver is a lost event: the root
    /// has to be read again, because the daemon cannot know what it missed
    /// (`FUN-025`, platform-integration §8.2).
    fn record(&mut self, event: notify::Result<Event>, roots: &[PathBuf]) {
        match event {
            Ok(event) => {
                for path in event.paths {
                    self.changes.push(change_of(&event.kind, path));
                }
            }
            Err(_) => {
                let root = roots.first().cloned().unwrap_or_default();
                self.changes.push(FileChange::RescanRequired { root });
            }
        }
    }

    /// The changes that may be handed out now.
    ///
    /// The events are held until the disk has been quiet for
    /// `DEBOUNCE_DURATION`, so that a burst of events is read once. The events
    /// that wait are kept: they are handed out later, and never twice.
    fn settled(&mut self, now: Instant) -> Vec<FileChange> {
        if self.changes.is_empty() {
            self.last_event = None;
            return Vec::new();
        }
        let Some(last) = self.last_event else {
            // The events wait for nothing: they are handed out.
            return std::mem::take(&mut self.changes);
        };
        if now.duration_since(last) < DEBOUNCE_DURATION {
            return Vec::new();
        }
        self.last_event = None;
        std::mem::take(&mut self.changes)
    }
}

/// The change that one event of the platform means.
///
/// Every event that is not a creation or a removal is a modification: the
/// daemon reads the root again in any case, and the difference between the
/// kinds is of no consequence to it.
fn change_of(kind: &EventKind, path: PathBuf) -> FileChange {
    match kind {
        EventKind::Create(_) => FileChange::Created { path },
        EventKind::Remove(_) => FileChange::Removed { path },
        _ => FileChange::Modified { path },
    }
}

impl FsChangeWatcher {
    /// Creates a new watcher for the provided root directories.
    pub fn new(roots: &[PathBuf]) -> PlatformResult<Self> {
        let (tx, rx) = channel();
        let watcher_res = RecommendedWatcher::new(
            move |res| {
                let _ = tx.send(res);
            },
            Config::default(),
        );

        let mut watcher = match watcher_res {
            Ok(w) => w,
            Err(e) => {
                return Err(PlatformError::Io {
                    operation: "initialize filesystem watcher",
                    source: std::io::Error::other(e),
                });
            }
        };

        for root in roots {
            if !root.exists() {
                continue;
            }
            if let Err(e) = watcher.watch(root, RecursiveMode::Recursive) {
                return Err(PlatformError::Io {
                    operation: "watch media root folder",
                    source: std::io::Error::other(e),
                });
            }
        }

        Ok(Self {
            _watcher: Mutex::new(Some(watcher)),
            receiver: Mutex::new(rx),
            pending: Mutex::new(Pending::default()),
            roots: roots.to_vec(),
        })
    }
}

impl ChangeWatcher for FsChangeWatcher {
    fn poll_changes(&self) -> PlatformResult<Vec<FileChange>> {
        let Ok(rx) = self.receiver.lock() else {
            return Ok(Vec::new());
        };
        let mut pending = self.pending.lock().unwrap_or_else(|e| e.into_inner());

        let mut seen = false;
        while let Ok(event) = rx.try_recv() {
            seen = true;
            pending.record(event, &self.roots);
        }
        if seen {
            // The burst is still running: the moment of the last event is the
            // moment the quiet time is counted from.
            pending.last_event = Some(Instant::now());
        }

        Ok(pending.settled(Instant::now()))
    }

    fn reliability(&self, root: &Path) -> ChangeReliability {
        if self.roots.iter().any(|r| r == root) {
            ChangeReliability::Reliable
        } else {
            ChangeReliability::Unavailable
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{change_of, FsChangeWatcher, Pending, DEBOUNCE_DURATION};
    use crate::platform::change::{ChangeReliability, ChangeWatcher, FileChange};
    use notify::event::{CreateKind, DataChange, ModifyKind, RemoveKind};
    use notify::EventKind;
    use std::path::{Path, PathBuf};
    use std::time::{Duration, Instant};

    /// A path of a root.
    fn root() -> PathBuf {
        PathBuf::from("/library")
    }

    /// A change of a file.
    fn created() -> FileChange {
        FileChange::Created {
            path: PathBuf::from("/library/film.mkv"),
        }
    }

    /// The negative case: a burst that is still running hands out nothing.
    #[test]
    fn a_burst_that_is_running_hands_out_nothing() {
        let mut pending = Pending::default();
        pending.changes.push(created());
        pending.last_event = Some(Instant::now());

        assert!(
            pending.settled(Instant::now()).is_empty(),
            "the events wait until the disk is quiet"
        );
    }

    /// The positive case: the events of a burst are handed out once the disk
    /// has been quiet, and they are handed out exactly once.
    #[test]
    fn the_events_of_a_burst_are_handed_out_once() {
        let mut pending = Pending::default();
        pending.changes.push(created());
        pending.last_event = Some(Instant::now());
        let settled = Instant::now() + DEBOUNCE_DURATION;

        let handed_out = pending.settled(settled);
        assert_eq!(handed_out, vec![created()], "the change is handed out");
        assert!(
            pending.settled(settled + Duration::from_secs(1)).is_empty(),
            "the change is not handed out twice"
        );
    }

    /// The negative case: a watcher that has seen nothing hands out nothing.
    #[test]
    fn a_watcher_that_has_seen_nothing_hands_out_nothing() {
        let mut pending = Pending::default();
        assert!(pending.settled(Instant::now()).is_empty());
    }

    /// The events of the platform that could not be delivered ask for a rescan
    /// of the root, because the daemon cannot know what it missed.
    #[test]
    fn the_lost_events_ask_for_a_rescan() {
        let mut pending = Pending::default();
        pending.record(
            Err(notify::Error::generic("the queue overflowed")),
            &[root()],
        );
        pending.last_event = Some(Instant::now());

        let handed_out = pending.settled(Instant::now() + DEBOUNCE_DURATION);
        assert_eq!(
            handed_out,
            vec![FileChange::RescanRequired { root: root() }],
            "the root is named, so that it is read again"
        );
    }

    /// A creation is a creation and a removal is a removal; every other event
    /// is a modification.
    #[test]
    fn the_kind_of_the_event_is_the_change_it_means() {
        let path = PathBuf::from("/library/film.mkv");
        assert_eq!(
            change_of(&EventKind::Create(CreateKind::File), path.clone()),
            FileChange::Created { path: path.clone() }
        );
        assert_eq!(
            change_of(&EventKind::Remove(RemoveKind::File), path.clone()),
            FileChange::Removed { path: path.clone() }
        );
        assert_eq!(
            change_of(
                &EventKind::Modify(ModifyKind::Data(DataChange::Content)),
                path.clone()
            ),
            FileChange::Modified { path: path.clone() }
        );
    }

    /// The watcher promises its events only for the roots it watches.
    #[test]
    fn the_watcher_promises_only_for_the_roots_it_watches() {
        let folder = tempfile::tempdir().expect("a temporary directory");
        let watcher = FsChangeWatcher::new(&[folder.path().to_path_buf()])
            .expect("the watcher of the platform can be built");

        assert_eq!(
            watcher.reliability(folder.path()),
            ChangeReliability::Reliable,
            "a watched root is watched"
        );
        assert_eq!(
            watcher.reliability(Path::new("/somewhere/else")),
            ChangeReliability::Unavailable,
            "a root that is not watched is not promised"
        );
    }

    /// The negative case: a watcher that has seen nothing hands out nothing,
    /// even when the platform is asked.
    #[test]
    fn a_watcher_without_events_hands_out_nothing() {
        let folder = tempfile::tempdir().expect("a temporary directory");
        let watcher = FsChangeWatcher::new(&[folder.path().to_path_buf()])
            .expect("the watcher of the platform can be built");

        assert!(
            watcher
                .poll_changes()
                .expect("the platform answers")
                .is_empty(),
            "nothing happened, so nothing is handed out"
        );
    }

    /// The end-to-end case: a file that appears in a watched root is handed out
    /// by the real watcher of the platform.
    ///
    /// The budget is generous, because the moment at which the platform
    /// delivers an event is the business of the platform (`FUN-025`).
    #[test]
    fn a_file_that_appears_is_handed_out() {
        let folder = tempfile::tempdir().expect("a temporary directory");
        let watcher = FsChangeWatcher::new(&[folder.path().to_path_buf()])
            .expect("the watcher of the platform can be built");

        std::fs::write(folder.path().join("film.mkv"), b"the film").expect("the film is written");

        let deadline = Instant::now() + Duration::from_secs(15);
        let mut handed_out = Vec::new();
        while Instant::now() < deadline && handed_out.is_empty() {
            std::thread::sleep(Duration::from_millis(100));
            handed_out = watcher.poll_changes().expect("the platform answers");
        }

        assert!(
            !handed_out.is_empty(),
            "the file that appeared is handed out by the watcher"
        );
    }
}
