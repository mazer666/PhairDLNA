//! The watch of the drives of the library.
//!
//! What: the task that looks at the roots from time to time and reads the
//!       catalogue again when a drive came, went or was replaced.
//! Why: the library of a person is not a fixed thing. A removable drive is
//!       unplugged, a network share comes back, and the server has to follow
//!       without being restarted (`FUN-017`, `PLT-011`).
//! Example: the watch finds that the drive "Films" is not there any more, reads
//!       the catalogue again and tells the daemon what happened.
//!
//! **The watch asks a cheap question.** It looks at the roots and at nothing
//! else: the question "did anything change at the roots?" is answered without
//! reading a single file of the library. Only when the answer is "yes" is the
//! catalogue read again.
//!
//! **The work of the watch is background work.** It is arbitrated by the
//! scheduler of the daemon, so that the reading of the library waits for the
//! clients of the server and yields while the machine is busy (`RES-001`,
//! `RES-003`).

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use phairdlna_core::catalogue::{RootChange, VolumeChange};
use phairdlna_core::platform::{ChangeWatcher, FsChangeWatcher, LocalFileSystem};
use tokio::task::JoinHandle;

use super::library::Library;
use super::lifecycle::LifecycleController;
use super::log::Logger;
use super::resources::Resources;

/// How often the watch looks at the roots.
///
/// The interval is a constant of the daemon: the configuration has no key for
/// it yet (`FUN-024`).
pub const WATCH_INTERVAL: Duration = Duration::from_secs(5);

/// Starts the watch and hands back the task and the way to stop it.
pub fn spawn(
    library: Library,
    logger: Logger,
    resources: Resources,
    lifecycle: LifecycleController,
    schedule: phairdlna_core::config::ScanSchedule,
) -> (tokio::sync::watch::Sender<bool>, JoinHandle<()>) {
    let interval = match schedule {
        phairdlna_core::config::ScanSchedule::Interval(secs) => Duration::from_secs(secs),
        phairdlna_core::config::ScanSchedule::ManualOnly => WATCH_INTERVAL,
    };
    spawn_scheduled(
        library,
        logger,
        resources,
        lifecycle,
        interval,
        schedule.is_automatic(),
    )
}

/// Builds the change watcher for the media roots of the library.
fn build_fs_watcher(library: &Library, logger: &Logger) -> Option<Arc<FsChangeWatcher>> {
    let roots: Vec<PathBuf> = library
        .roots()
        .iter()
        .map(|r| PathBuf::from(&r.path))
        .collect();
    match FsChangeWatcher::new(&roots) {
        Ok(w) => Some(Arc::new(w)),
        Err(e) => {
            logger.warn(
                "filesystem change watcher unavailable",
                &[("reason", &e.to_string())],
            );
            None
        }
    }
}

/// Starts the watch with a given interval and hands back the task and the way
/// to stop it.
///
/// The interval is a parameter so that a test may watch a drive without
/// waiting for the interval of the daemon.
pub fn spawn_with_interval(
    library: Library,
    logger: Logger,
    resources: Resources,
    lifecycle: LifecycleController,
    interval: Duration,
) -> (tokio::sync::watch::Sender<bool>, JoinHandle<()>) {
    spawn_scheduled(library, logger, resources, lifecycle, interval, true)
}

/// Starts the watch with explicit interval and auto-scan control (FUN-024).
pub fn spawn_scheduled(
    library: Library,
    logger: Logger,
    resources: Resources,
    lifecycle: LifecycleController,
    interval: Duration,
    auto_rescan: bool,
) -> (tokio::sync::watch::Sender<bool>, JoinHandle<()>) {
    let fs_watcher = build_fs_watcher(&library, &logger);
    let (stop, mut stopped) = tokio::sync::watch::channel(false);
    let task = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if *stopped.borrow() {
                        break;
                    }
                    let library = library.clone();
                    let logger = logger.clone();
                    let resources = resources.clone();
                    let lifecycle = lifecycle.clone();
                    let fs_watcher = fs_watcher.clone();
                    if let Err(error) = resources.submit_background(move || {
                        watch_once(&library, &logger, &lifecycle, fs_watcher.as_deref(), auto_rescan);
                    }) {
                        resources.explain(&error);
                    }
                }
                _ = stopped.changed() => break,
            }
        }
    });
    (stop, task)
}

/// Looks at the roots once and reads the catalogue again when something moved.
fn watch_once(
    library: &Library,
    logger: &Logger,
    lifecycle: &LifecycleController,
    fs_watcher: Option<&FsChangeWatcher>,
    auto_rescan: bool,
) {
    let roots_changed = library.changed(&LocalFileSystem);
    let fs_changed = fs_watcher
        .and_then(|w| w.poll_changes().ok())
        .map(|changes| !changes.is_empty())
        .unwrap_or(false);

    if !roots_changed && (!auto_rescan || !fs_changed) {
        return;
    }
    if fs_changed {
        logger.info("filesystem change detected, refreshing catalogue", &[]);
    }
    let report = library.refresh(&LocalFileSystem);
    handle_volume_changes(&report.changes, logger, lifecycle);
}

/// Applies volume changes to server lifecycle and structured logs.
fn handle_volume_changes(changes: &[RootChange], logger: &Logger, lifecycle: &LifecycleController) {
    let mut any_lost = false;
    let mut any_returned = false;
    for change in changes {
        if matches!(change.change, VolumeChange::First | VolumeChange::Unchanged) {
            continue;
        }
        if matches!(change.change, VolumeChange::Lost) {
            any_lost = true;
        } else if matches!(
            change.change,
            VolumeChange::Returned | VolumeChange::Remounted
        ) {
            any_returned = true;
        }
        let sentence = sentence_of(&change.change);
        logger.info(
            "the library changed",
            &[("root", &change.name), ("reason", &sentence)],
        );
    }

    if any_lost {
        lifecycle.transition(
            phairdlna_core::lifecycle::LifecycleState::Degraded,
            "one or more media roots are unavailable",
        );
    } else if any_returned
        && lifecycle.state() == phairdlna_core::lifecycle::LifecycleState::Degraded
    {
        lifecycle.transition(
            phairdlna_core::lifecycle::LifecycleState::Ready,
            "all media roots have returned",
        );
    }
}

/// The sentence that tells a person what happened to a drive.
fn sentence_of(change: &VolumeChange) -> String {
    match change {
        VolumeChange::First | VolumeChange::Unchanged => "nothing changed".to_owned(),
        VolumeChange::Lost => "the drive is not connected".to_owned(),
        VolumeChange::Returned => "the drive is connected again".to_owned(),
        VolumeChange::Remounted => "the drive was mounted again".to_owned(),
        VolumeChange::Replaced { .. } => {
            "the drive is now a different volume; its files were given new identifiers".to_owned()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{sentence_of, spawn_with_interval, LifecycleController};
    use crate::serve::library::Library;
    use crate::serve::log::{Level, Logger};
    use crate::serve::resources::Resources;
    use phairdlna_core::catalogue::VolumeChange;
    use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
    use phairdlna_core::platform::LocalFileSystem;
    use phairdlna_core::security::Redactor;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    /// The bytes of a file the sniffer recognises as an MP4.
    const MP4_HEAD: &[u8] = b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00mp42isom";

    /// Every change has a sentence, and the sentence is not empty.
    #[test]
    fn every_change_has_a_sentence() {
        let changes = [
            VolumeChange::First,
            VolumeChange::Unchanged,
            VolumeChange::Lost,
            VolumeChange::Returned,
            VolumeChange::Remounted,
            VolumeChange::Replaced {
                previous: phairdlna_core::platform::VolumeId::new("vol-1"),
            },
        ];
        for change in changes {
            assert!(!sentence_of(&change).is_empty());
        }
    }

    /// The watch reads the catalogue again when a drive goes away.
    #[tokio::test]
    async fn the_watch_reads_the_catalogue_again() {
        let folder = tempfile::tempdir().expect("a temporary directory");
        std::fs::write(folder.path().join("film.mkv"), MP4_HEAD).expect("the film is written");
        let roots = vec![MediaRoot {
            name: "Films".to_owned(),
            path: folder.path().to_string_lossy().into_owned(),
            exclude: Vec::new(),
            symlink_policy: None,
        }];
        let library = Library::build(&roots, SymlinkPolicy::Skip, &LocalFileSystem);
        let redactor = Arc::new(Redactor::new());
        let logger = Logger::with_redactor(Level::Info, "daemon", Arc::clone(&redactor));
        let lifecycle = LifecycleController::new(redactor, logger.clone());
        let resources = Resources::new(&phairdlna_core::config::Config::default(), &logger);
        let (stop, task) = spawn_with_interval(
            library.clone(),
            logger,
            resources,
            lifecycle,
            Duration::from_millis(20),
        );

        // The drive is unplugged while the server runs.
        std::fs::remove_dir_all(folder.path()).expect("the drive is unplugged");

        let noticed = wait_until_absent(&library).await;
        let _ = stop.send(true);
        let _ = task.await;

        assert!(noticed, "the watch noticed that the drive is gone");
        assert!(
            library.with_catalogue(|catalogue| catalogue.roots()[0].entries) == 0,
            "the files of the drive that is gone left the catalogue"
        );
    }

    /// The watch picks up newly created media files automatically via filesystem watcher.
    #[tokio::test]
    async fn the_watch_picks_up_newly_created_media_files() {
        let folder = tempfile::tempdir().expect("a temporary directory");
        let roots = vec![MediaRoot {
            name: "Movies".to_owned(),
            path: folder.path().to_string_lossy().into_owned(),
            exclude: Vec::new(),
            symlink_policy: None,
        }];
        let library = Library::build(&roots, SymlinkPolicy::Skip, &LocalFileSystem);
        let redactor = Arc::new(Redactor::new());
        let logger = Logger::with_redactor(Level::Info, "daemon", Arc::clone(&redactor));
        let lifecycle = LifecycleController::new(redactor, logger.clone());
        let resources = Resources::new(&phairdlna_core::config::Config::default(), &logger);
        let (stop, task) = spawn_with_interval(
            library.clone(),
            logger,
            resources,
            lifecycle,
            Duration::from_millis(50),
        );

        assert_eq!(
            library.with_catalogue(|c| c.roots()[0].entries),
            0,
            "catalogue starts empty"
        );

        // A new film is added to the watched folder.
        std::fs::write(folder.path().join("new_video.mp4"), MP4_HEAD).expect("new film written");

        let found = wait_until_entries(&library, 1).await;
        let _ = stop.send(true);
        let _ = task.await;
        assert!(
            found,
            "the watch detected the new video and refreshed the catalogue"
        );
    }

    /// Waits until the first root of the library has at least `expected` entries.
    async fn wait_until_entries(library: &Library, expected: usize) -> bool {
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            let entries = library.with_catalogue(|c| c.roots()[0].entries);
            if entries >= expected {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
        false
    }

    /// Waits until the root of the library is not there any more.
    ///
    /// The answer is `false` when the budget of the wait is over.
    async fn wait_until_absent(library: &Library) -> bool {
        let deadline = Instant::now() + Duration::from_secs(10);
        while Instant::now() < deadline {
            let present = library.with_catalogue(|catalogue| catalogue.roots()[0].present);
            if !present {
                return true;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        false
    }
}
