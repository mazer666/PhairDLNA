//! The local timeline of the life of the server.
//!
//! What: a bounded file that remembers the transitions of the server and the
//!       recovery events, with every private value replaced by a pseudonym.
//! Why: the user must be able to look at what happened to the server before
//!      the report leaves the machine. The file is therefore local, redacted
//!      and small — a timeline that grows without end is a file nobody reads
//!      (`LIFE-011`, `SEC-094`).
//! Example: `Timeline::new(path, redactor).append(&transition)` writes one
//!      line, and `read()` gives the lines back for the export.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::config::user_data_dir;
use crate::security::Redactor;

use super::state::Transition;

/// How much the timeline may grow before the oldest lines are dropped.
///
/// The bound is small on purpose: the timeline is a memory of the last
/// events, not an archive of the machine (`LIFE-011`).
pub const DEFAULT_TIMELINE_LIMIT: u64 = 256 * 1024;

/// The permissions of the timeline: the file belongs to the user alone.
#[cfg(unix)]
const TIMELINE_MODE: u32 = 0o600;

/// The timeline of the server.
#[derive(Debug, Clone)]
pub struct Timeline {
    /// Where the timeline is written.
    path: PathBuf,
    /// The pseudonyms that replace the private values.
    redactor: Arc<Redactor>,
    /// How much the file may grow.
    limit: u64,
}

impl Timeline {
    /// Builds a timeline that writes to the given path.
    pub fn new(path: PathBuf, redactor: Arc<Redactor>) -> Self {
        Self {
            path,
            redactor,
            limit: DEFAULT_TIMELINE_LIMIT,
        }
    }

    /// Builds a timeline with another bound.
    pub fn with_limit(path: PathBuf, redactor: Arc<Redactor>, limit: u64) -> Self {
        Self {
            path,
            redactor,
            limit,
        }
    }

    /// The standard place of the timeline of the instance.
    pub fn default_path() -> PathBuf {
        user_data_dir().join("timeline.log")
    }

    /// The path of the timeline.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Adds a transition to the timeline.
    ///
    /// The reason is redacted before it is written: the reason of a transition
    /// may name a path or an address (`SEC-094`).
    ///
    /// A file that cannot be written is not a reason to stop the server: the
    /// error is returned, and the caller decides what to do with it.
    pub fn append(&self, transition: &Transition) -> std::io::Result<()> {
        let reason = self.redactor.value(&transition.reason);
        let line = format!(
            "{} {} reason=\"{}\" {}",
            transition.moment,
            transition.state.as_str(),
            reason,
            transition.explanation
        );
        self.write_line(&line)
    }

    /// Adds a free line to the timeline.
    ///
    /// The line is redacted as well, so that a recovery event that names a path
    /// cannot leak it.
    pub fn append_note(&self, note: &str) -> std::io::Result<()> {
        let line = self.redactor.value(note);
        self.write_line(&line)
    }

    /// The lines of the timeline, the oldest first.
    ///
    /// A file that is not there is an empty timeline, not an error: the server
    /// may never have written one.
    pub fn read(&self) -> std::io::Result<String> {
        match std::fs::read_to_string(&self.path) {
            Ok(text) => Ok(text),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
            Err(error) => Err(error),
        }
    }

    /// Writes one line and keeps the file inside its bound.
    fn write_line(&self, line: &str) -> std::io::Result<()> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = open_for_append(&self.path)?;
        writeln!(file, "{line}")?;
        file.flush()?;
        self.trim()
    }

    /// Drops the oldest lines when the file grew beyond its bound.
    ///
    /// Half of the file is kept, so that the timeline keeps a history and not
    /// only the last event.
    fn trim(&self) -> std::io::Result<()> {
        let Ok(text) = std::fs::read_to_string(&self.path) else {
            return Ok(());
        };
        if text.len() as u64 <= self.limit {
            return Ok(());
        }
        let kept = keep_the_newest(&text, (self.limit / 2) as usize);
        std::fs::write(&self.path, kept)
    }
}

/// The newest lines of a text that fit into a number of bytes.
fn keep_the_newest(text: &str, limit: usize) -> String {
    let mut kept: Vec<&str> = Vec::new();
    let mut size = 0;
    for line in text.lines().rev() {
        if size + line.len() + 1 > limit {
            break;
        }
        size += line.len() + 1;
        kept.push(line);
    }
    kept.reverse();
    let mut result = kept.join("\n");
    if !result.is_empty() {
        result.push('\n');
    }
    result
}

/// Opens a file for appending, with the permissions of the user alone.
#[cfg(unix)]
fn open_for_append(path: &Path) -> std::io::Result<std::fs::File> {
    use std::os::unix::fs::OpenOptionsExt;
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .mode(TIMELINE_MODE)
        .open(path)
}

/// Opens a file for appending, on a platform without file modes.
#[cfg(not(unix))]
fn open_for_append(path: &Path) -> std::io::Result<std::fs::File> {
    std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::{keep_the_newest, Timeline};
    use crate::lifecycle::state::{LifecycleState, Transition};
    use crate::security::Redactor;

    /// The positive case: a transition is written and read back.
    #[test]
    fn a_transition_is_written_and_read_back() {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let timeline = Timeline::new(
            directory.path().join("timeline.log"),
            Arc::new(Redactor::new()),
        );

        let transition = Transition::new(LifecycleState::Ready, "the listeners are open");
        timeline.append(&transition).expect("the line is written");

        let text = timeline.read().expect("the timeline is read");
        assert!(text.contains("ready"), "{text}");
        assert!(text.contains("the listeners are open"), "{text}");
    }

    /// The negative case: a timeline that was never written is empty, not an
    /// error.
    #[test]
    fn a_missing_timeline_is_empty() {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let timeline = Timeline::new(
            directory.path().join("missing.log"),
            Arc::new(Redactor::new()),
        );

        assert_eq!(timeline.read().expect("the timeline is read"), "");
    }

    /// The negative case: a path in the timeline is a pseudonym.
    #[test]
    fn a_path_in_the_timeline_is_hidden() {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let timeline = Timeline::new(
            directory.path().join("timeline.log"),
            Arc::new(Redactor::new()),
        );

        let transition = Transition::new(LifecycleState::Degraded, "/Users/me/Movies vanished");
        timeline.append(&transition).expect("the line is written");

        let text = timeline.read().expect("the timeline is read");
        assert!(!text.contains("Users"), "{text}");
        assert!(text.contains("<path:1>"), "{text}");
    }

    /// The negative case: the timeline does not grow without end.
    #[test]
    fn the_timeline_stays_inside_its_bound() {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let timeline = Timeline::with_limit(
            directory.path().join("timeline.log"),
            Arc::new(Redactor::new()),
            200,
        );

        for index in 0..50 {
            let transition = Transition::new(LifecycleState::Ready, format!("event {index}"));
            timeline.append(&transition).expect("the line is written");
        }

        let text = timeline.read().expect("the timeline is read");
        assert!(
            text.len() <= 200,
            "the timeline is bounded: {} bytes",
            text.len()
        );
        assert!(
            text.contains("event 49"),
            "the newest event is kept: {text}"
        );
    }

    /// The newest lines are the ones that are kept.
    #[test]
    fn the_newest_lines_are_kept() {
        let text = "one\ntwo\nthree\nfour\n";
        let kept = keep_the_newest(text, 12);

        assert!(kept.contains("four"), "{kept}");
        assert!(kept.contains("three"), "{kept}");
        assert!(!kept.contains("one"), "{kept}");
    }
}
