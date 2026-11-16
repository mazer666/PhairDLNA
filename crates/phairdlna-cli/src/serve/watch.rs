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

use std::time::Duration;

use phairdlna_core::catalogue::VolumeChange;
use phairdlna_core::platform::LocalFileSystem;
use tokio::task::JoinHandle;

use super::library::Library;
use super::log::Logger;

/// How often the watch looks at the roots.
///
/// The interval is a constant of the daemon: the configuration has no key for
/// it yet (`FUN-024`).
pub const WATCH_INTERVAL: Duration = Duration::from_secs(5);

/// Starts the watch and hands back the task and the way to stop it.
pub fn spawn(
    library: Library,
    logger: Logger,
) -> (tokio::sync::watch::Sender<bool>, JoinHandle<()>) {
    spawn_with_interval(library, logger, WATCH_INTERVAL)
}

/// Starts the watch with a given interval and hands back the task and the way
/// to stop it.
///
/// The interval is a parameter so that a test may watch a drive without
/// waiting for the interval of the daemon.
pub fn spawn_with_interval(
    library: Library,
    logger: Logger,
    interval: Duration,
) -> (tokio::sync::watch::Sender<bool>, JoinHandle<()>) {
    let (stop, mut stopped) = tokio::sync::watch::channel(false);
    let task = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(interval);
        loop {
            tokio::select! {
                _ = ticker.tick() => {
                    if *stopped.borrow() {
                        break;
                    }
                    // The work of the watch reads the disk, and a network share
                    // that does not answer would block the worker of the
                    // runtime for as long as it takes. The work therefore runs
                    // on a thread of its own (`OPS-016`).
                    let library = library.clone();
                    let logger = logger.clone();
                    let _ = tokio::task::spawn_blocking(move || {
                        watch_once(&library, &logger);
                    })
                    .await;
                }
                _ = stopped.changed() => break,
            }
        }
    });
    (stop, task)
}

/// Looks at the roots once and reads the catalogue again when something moved.
fn watch_once(library: &Library, logger: &Logger) {
    if !library.changed(&LocalFileSystem) {
        return;
    }
    let report = library.refresh(&LocalFileSystem);
    for change in &report.changes {
        if matches!(change.change, VolumeChange::First | VolumeChange::Unchanged) {
            continue;
        }
        let sentence = sentence_of(&change.change);
        logger.info(
            "the library changed",
            &[("root", &change.name), ("reason", &sentence)],
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
    use super::{sentence_of, spawn_with_interval};
    use crate::serve::library::Library;
    use crate::serve::log::{Level, Logger};
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
        let logger = Logger::with_redactor(Level::Info, "daemon", Arc::new(Redactor::new()));
        let (stop, task) = spawn_with_interval(library.clone(), logger, Duration::from_millis(20));

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
