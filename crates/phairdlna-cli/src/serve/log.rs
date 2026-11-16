//! The log of the daemon.
//!
//! What: the structured records the daemon writes to standard error.
//! Why: the log is the only place a user can see what the server does, and it
//!       must be readable by a person and by a machine at the same time
//!       (`OPS-001`, `OPS-006`).
//! Example: a record reads
//!       `2026-11-15T19:30:00Z info ssdp listening address=192.168.1.20:8200`.
//!
//! The records go to **standard error**; standard output carries the results of
//! the commands, so that `phairdlna config export > config.toml` writes a valid
//! file (`OPS-001`).

use std::collections::VecDeque;
use std::io::Write;
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::SystemTime;

use phairdlna_core::config::LogLevel;
use phairdlna_core::security::Redactor;

/// How many records the daemon keeps for the diagnostics.
///
/// The records are a memory of what happened, not a log file: the number is
/// small and bounded, so that the diagnostics of the server cannot cost the
/// server its memory.
const KEPT_RECORDS: usize = 256;

/// How much the daemon says.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Level {
    /// Something the user wants to know.
    Info,
    /// Something that is not right but does not stop the server.
    Warn,
    /// Something that stops the server.
    Error,
}

impl Level {
    /// The name of the level, as it appears in a record.
    pub fn as_str(&self) -> &'static str {
        match self {
            Level::Info => "info",
            Level::Warn => "warn",
            Level::Error => "error",
        }
    }

    /// The level of the configuration.
    ///
    /// The configuration knows a level *debug* that the daemon does not write:
    /// the daemon says what a user wants to know, and the detail of a request
    /// belongs to a future release (`OPS-006`).
    pub fn from_config(level: LogLevel) -> Self {
        match level {
            LogLevel::Debug => Level::Info,
            LogLevel::Info => Level::Info,
            LogLevel::Warn => Level::Warn,
            LogLevel::Error => Level::Error,
        }
    }
}

/// One record of the log, kept so that the diagnostics can show it.
///
/// The record carries the private values already replaced by their pseudonyms:
/// the diagnostics of the server must not name the files of the user
/// (`SEC-094`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Record {
    /// When the record was written.
    pub time: String,
    /// How serious the record is.
    pub level: String,
    /// Which part of the server wrote the record.
    pub component: String,
    /// What the record says.
    pub message: String,
    /// The fields of the record.
    pub fields: Vec<(String, String)>,
}

/// The log of the daemon.
///
/// The logger writes to standard error and never fails: a log that cannot be
/// written must not stop a server that serves media (`F-13` § 9).
///
/// The logger also hides: a path, an address and the identifier of the device
/// are replaced by a pseudonym, so that the log does not become a list of the
/// files of the user (`SEC-094`). The same path keeps the same pseudonym in
/// every record, so that the log stays useful.
#[derive(Debug, Clone)]
pub struct Logger {
    level: Level,
    component: &'static str,
    redactor: Arc<Redactor>,
    redact: bool,
    /// The records the daemon keeps for the diagnostics.
    records: Arc<Mutex<VecDeque<Record>>>,
}

impl Logger {
    /// A logger for one component.
    pub fn new(level: Level, component: &'static str) -> Self {
        Self {
            level,
            component,
            redactor: Arc::new(Redactor::new()),
            redact: true,
            records: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// A logger that shares the pseudonyms of the other loggers.
    ///
    /// The pseudonyms belong to the daemon, not to a component: a path that
    /// is named in the log of the HTTP server and in the log of the discovery
    /// must carry the same pseudonym (`SEC-094`).
    pub fn with_redactor(level: Level, component: &'static str, redactor: Arc<Redactor>) -> Self {
        Self {
            level,
            component,
            redactor,
            redact: true,
            records: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    /// Whether the values of the records are replaced by pseudonyms.
    ///
    /// The user may turn the redaction off; the default is on, and the log of
    /// a server that says nothing about the files of the user is the safer one
    /// (`SEC-094`).
    pub fn redacting(mut self, redact: bool) -> Self {
        self.redact = redact;
        self
    }

    /// The same logger, for another component.
    ///
    /// The new logger shares the pseudonyms: a path that is named by two
    /// components carries the same pseudonym in both logs (`SEC-094`).
    pub fn for_component(&self, component: &'static str) -> Self {
        Self {
            level: self.level,
            component,
            redactor: Arc::clone(&self.redactor),
            redact: self.redact,
            records: Arc::clone(&self.records),
        }
    }

    /// Writes a record.
    ///
    /// The record is written to standard error and kept in the memory of the
    /// log, so that the diagnostics of the server can show what happened
    /// (`OPS-020`).
    pub fn write(&self, level: Level, message: &str, fields: &[(&str, &str)]) {
        if level < self.level {
            return;
        }
        let line = self.record(level, message, fields);
        self.keep(level, message, fields);
        let mut error = std::io::stderr();
        let _ = error.write_all(line.as_bytes());
        let _ = error.flush();
    }

    /// Keeps a record in the memory of the log.
    ///
    /// The memory is bounded: the oldest record is dropped when the memory is
    /// full, so that a server that runs for weeks does not grow without end.
    fn keep(&self, level: Level, message: &str, fields: &[(&str, &str)]) {
        let record = Record {
            time: timestamp(),
            level: level.as_str().to_owned(),
            component: self.component.to_owned(),
            message: message.to_owned(),
            fields: self.fields_of(fields),
        };
        let mut records = self.records();
        records.push_back(record);
        while records.len() > KEPT_RECORDS {
            records.pop_front();
        }
    }

    /// The records the log keeps, the newest last.
    ///
    /// The number of records is bounded by the caller: a person wants the last
    /// few, not a thousand.
    pub fn recent(&self, limit: usize) -> Vec<Record> {
        let records = self.records();
        let start = records.len().saturating_sub(limit);
        records.iter().skip(start).cloned().collect()
    }

    /// The records of the log.
    fn records(&self) -> MutexGuard<'_, VecDeque<Record>> {
        self.records
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    /// Builds a record without writing it.
    ///
    /// The record is a single line: the moment, the level, the component, the
    /// message and the named fields. A machine reads the fields, a person
    /// reads the line (`OPS-006`).
    pub fn record(&self, level: Level, message: &str, fields: &[(&str, &str)]) -> String {
        let mut line = format!(
            "{} {} {} {}",
            timestamp(),
            level.as_str(),
            self.component,
            message
        );
        for (name, value) in self.fields_of(fields) {
            line.push_str(&format!(" {name}={value}"));
        }
        line.push('\n');
        line
    }

    /// The fields of a record, with the private values replaced.
    fn fields_of(&self, fields: &[(&str, &str)]) -> Vec<(String, String)> {
        if self.redact {
            return self.redactor.fields(fields);
        }
        fields
            .iter()
            .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
            .collect()
    }

    /// Writes a record of level *info*.
    pub fn info(&self, message: &str, fields: &[(&str, &str)]) {
        self.write(Level::Info, message, fields);
    }

    /// Writes a record of level *warn*.
    pub fn warn(&self, message: &str, fields: &[(&str, &str)]) {
        self.write(Level::Warn, message, fields);
    }

    /// Writes a record of level *error*.
    pub fn error(&self, message: &str, fields: &[(&str, &str)]) {
        self.write(Level::Error, message, fields);
    }
}

/// The timestamp of a record, in UTC.
///
/// The format is the one of RFC 3339 without the fraction of a second: it is
/// what every log reader understands.
fn timestamp() -> String {
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default();
    format_epoch_seconds(now.as_secs())
}

/// Formats a number of seconds since the epoch as a UTC timestamp.
///
/// The conversion is the one of the proleptic Gregorian calendar, computed
/// from the days since the epoch; the core has no calendar library, and a
/// daemon that cannot tell the time is not a daemon (`OPS-006`).
fn format_epoch_seconds(seconds: u64) -> String {
    let days = (seconds / 86_400) as i64;
    let time_of_day = seconds % 86_400;
    let (year, month, day) = civil_from_days(days);
    format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}Z",
        year,
        month,
        day,
        time_of_day / 3600,
        (time_of_day % 3600) / 60,
        time_of_day % 60
    )
}

/// The calendar date of a number of days since 1970-01-01.
///
/// The algorithm of Howard Hinnant: the days are shifted so that the year
/// begins on the first of March, and the correction of the leap years is a
/// division.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let shifted = days + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let shifted_month = (5 * day_of_year + 2) / 153;
    let day = (day_of_year - (153 * shifted_month + 2) / 5 + 1) as u32;
    let month = if shifted_month < 10 {
        shifted_month + 3
    } else {
        shifted_month - 9
    } as u32;
    let year = if month <= 2 { year + 1 } else { year };
    (year, month, day)
}

#[cfg(test)]
mod tests {
    use super::{format_epoch_seconds, Level, Logger};

    /// The epoch is the first of January 1970.
    #[test]
    fn the_epoch_is_read_correctly() {
        assert_eq!(format_epoch_seconds(0), "1970-01-01T00:00:00Z");
    }

    /// A known moment is read correctly.
    #[test]
    fn a_known_moment_is_read_correctly() {
        // 2026-11-15T19:30:00Z
        assert_eq!(format_epoch_seconds(1_794_771_000), "2026-11-15T19:30:00Z");
    }

    /// A leap day is read correctly.
    #[test]
    fn a_leap_day_is_read_correctly() {
        // 2024-02-29T12:00:00Z
        assert_eq!(format_epoch_seconds(1_709_208_000), "2024-02-29T12:00:00Z");
    }

    /// The level has the name the specification names.
    #[test]
    fn the_level_has_a_name() {
        assert_eq!(Level::Info.as_str(), "info");
        assert_eq!(Level::Warn.as_str(), "warn");
        assert_eq!(Level::Error.as_str(), "error");
    }

    /// The positive case: a record carries the moment, the level, the
    /// component, the message and the named fields.
    #[test]
    fn a_record_carries_everything_it_must_carry() {
        let logger = Logger::new(Level::Info, "ssdp");
        let record = logger.record(Level::Info, "listening", &[("port", "1900")]);

        let mut parts = record.split_whitespace();
        let moment = parts.next().expect("the record has a moment");
        assert!(
            moment.ends_with('Z') && moment.contains('-') && moment.contains(':'),
            "the moment is a timestamp: {moment}"
        );
        assert_eq!(parts.next(), Some("info"), "the level follows");
        assert_eq!(parts.next(), Some("ssdp"), "the component follows");
        assert_eq!(parts.next(), Some("listening"), "the message follows");
        assert_eq!(parts.next(), Some("port=1900"), "the field follows");
        assert!(record.ends_with('\n'), "the record is a line");
    }

    /// The negative case: a value that is not a number cannot be mistaken for
    /// a field of the record.
    #[test]
    fn a_field_is_written_as_a_pair() {
        let logger = Logger::new(Level::Warn, "daemon");
        let record = logger.record(Level::Warn, "the port is taken", &[("reason", "in use")]);

        assert!(
            record.contains(
                " reason=in use
"
            ),
            "{record}"
        );
    }

    /// A logger of level *error* says nothing about an *info* record.
    #[test]
    fn a_quiet_logger_stays_quiet() {
        let logger = Logger::new(Level::Error, "test");
        // The record is not written, and the call returns without a panic.
        logger.info("nothing to see", &[("key", "value")]);
    }

    /// The negative case: a path in a record is replaced by a pseudonym, so
    /// that the log does not name the files of the user (`SEC-094`).
    #[test]
    fn a_path_in_a_record_is_hidden() {
        let logger = Logger::new(Level::Warn, "daemon");
        let record = logger.record(
            Level::Warn,
            "a file was refused",
            &[("path", "/Users/me/Movies/film.mkv")],
        );

        assert!(!record.contains("Users"), "the path is gone: {record}");
        assert!(
            record.contains("<path:1>"),
            "the pseudonym is there: {record}"
        );
    }

    /// The same path carries the same pseudonym in every record.
    #[test]
    fn a_path_keeps_its_pseudonym() {
        let logger = Logger::new(Level::Warn, "daemon");
        let first = logger.record(
            Level::Warn,
            "a file was refused",
            &[("path", "/Users/me/a.mkv")],
        );
        let second = logger.record(
            Level::Warn,
            "a file was read",
            &[("path", "/Users/me/a.mkv")],
        );

        assert!(first.contains("<path:1>"), "{first}");
        assert!(second.contains("<path:1>"), "{second}");
    }

    /// The user may turn the redaction off.
    #[test]
    fn the_redaction_may_be_turned_off() {
        let logger = Logger::new(Level::Warn, "daemon").redacting(false);
        let record = logger.record(
            Level::Warn,
            "a file was refused",
            &[("path", "/Users/me/Movies/film.mkv")],
        );

        assert!(record.contains("/Users/me/Movies/film.mkv"), "{record}");
    }
}
