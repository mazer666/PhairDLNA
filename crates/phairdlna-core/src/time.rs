//! The time of the server.
//!
//! What: the current moment, written as a UTC timestamp.
//! Why: the log records, the lifecycle transitions and the recovery timeline
//!      all need a moment, and they must all write the same moment in the same
//!      format — a timestamp that two modules format differently is a timeline
//!      nobody can read (`OPS-006`, `LIFE-001`, `LIFE-011`).
//! Example: `now_utc()` returns `2026-11-16T18:34:00Z`.

use std::time::{SystemTime, UNIX_EPOCH};

/// The current moment, as a UTC timestamp.
///
/// The moment is the one of the system clock. A clock that stands still gives a
/// server a timestamp that stands still too; the server does not pretend to know
/// better.
pub fn now_utc() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format_epoch_seconds(now.as_secs())
}

/// Formats a number of seconds since the epoch as a UTC timestamp.
///
/// The format is the one of RFC 3339 without the fraction of a second: it is
/// what every log reader understands.
///
/// The conversion is the one of the proleptic Gregorian calendar, computed from
/// the days since the epoch; the core has no calendar library, and a daemon
/// that cannot tell the time is not a daemon (`OPS-006`).
pub fn format_epoch_seconds(seconds: u64) -> String {
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
/// The algorithm is the one of Howard Hinnant: the days are shifted so that the
/// year begins on the first of March, and the correction of the leap years is a
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
    use super::{format_epoch_seconds, now_utc};

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

    /// The current moment is a timestamp of the same shape.
    #[test]
    fn the_current_moment_is_a_timestamp() {
        let moment = now_utc();
        assert_eq!(moment.len(), 20, "the moment: {moment}");
        assert!(moment.ends_with('Z'), "the moment: {moment}");
        assert!(moment.contains('T'), "the moment: {moment}");
    }
}
