//! The rules of the values a document carries.
//!
//! What: the length policy of a value, the format of a duration, the format
//!       of a resolution and the check of a date.
//! Why: every value in the document comes from a file name or a tag, and a
//!       tag is untrusted input (`SEC-007`). A value is therefore bounded
//!       before it reaches the document (`INT-072`), and a value the server
//!       cannot format is left out rather than guessed (`INT-068`).
//! Example: `bounded` cuts a long title so that its escaped form fits into
//!       the budget.

use std::time::Duration;

use crate::xml::{escaped_character_len, escaped_text_len};

/// The largest number of bytes a value may have in its XML-escaped form.
///
/// The DLNA client implementation guide asks clients to support values of at
/// most 8192 bytes in their escaped form (`INT-072`).
pub const MAX_VALUE_BYTES: usize = 8192;

/// The marker that tells the user that a value was cut.
pub const TRUNCATION_MARKER: char = '…';

/// Bounds a value so that its XML-escaped form fits into the budget.
///
/// The cut happens on the raw value and on a character boundary, so that an
/// entity is never cut in half; the escaped form of the result is therefore
/// never longer than the budget. The marker at the end tells the user that
/// the value is shorter than the file says (`INT-072`, `UX-004`).
pub fn bounded(value: &str) -> String {
    if escaped_text_len(value) <= MAX_VALUE_BYTES {
        return value.to_owned();
    }
    let budget = MAX_VALUE_BYTES - TRUNCATION_MARKER.len_utf8();
    let mut bounded = String::new();
    let mut used = 0usize;
    for character in value.chars() {
        let length = escaped_character_len(character);
        if used + length > budget {
            break;
        }
        bounded.push(character);
        used += length;
    }
    bounded.push(TRUNCATION_MARKER);
    bounded
}

/// The running time in the format the schema asks for (`INT-063`).
///
/// The format is `H*:MM:SS.F*`: any number of hour digits, two minute and
/// second digits and three fraction digits.
pub fn format_duration(duration: Duration) -> String {
    let seconds = duration.as_secs();
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let rest = seconds % 60;
    format!(
        "{hours}:{minutes:02}:{rest:02}.{:03}",
        duration.subsec_millis()
    )
}

/// The pixel dimensions in the format the schema asks for (`INT-063`).
///
/// The format is `<width>x<height>`.
pub fn format_resolution(width: u32, height: u32) -> String {
    format!("{width}x{height}")
}

/// Whether a date is an ISO 8601 date or date-time (`INT-068`).
///
/// The two accepted forms are `YYYY-MM-DD` and `YYYY-MM-DDThh:mm:ss`. A
/// value in another form — a time zone, for example — is not accepted: the
/// catalogue normalises the time it reads, and the server does not guess.
pub fn is_iso_date(value: &str) -> bool {
    match value.split_once('T') {
        None => is_iso_day(value),
        Some((day, time)) => is_iso_day(day) && is_iso_time(time),
    }
}

/// Whether a value is a day of the ISO 8601 calendar.
fn is_iso_day(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return false;
    }
    if !digits(&bytes[0..4]) || !digits(&bytes[5..7]) || !digits(&bytes[8..10]) {
        return false;
    }
    let month = number(&bytes[5..7]);
    let day = number(&bytes[8..10]);
    (1..=12).contains(&month) && (1..=31).contains(&day)
}

/// Whether a value is a time of day of the ISO 8601 calendar.
fn is_iso_time(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 8 || bytes[2] != b':' || bytes[5] != b':' {
        return false;
    }
    if !digits(&bytes[0..2]) || !digits(&bytes[3..5]) || !digits(&bytes[6..8]) {
        return false;
    }
    let hour = number(&bytes[0..2]);
    let minute = number(&bytes[3..5]);
    let second = number(&bytes[6..8]);
    hour <= 23 && minute <= 59 && second <= 59
}

/// Whether every byte of a slice is a decimal digit.
fn digits(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| byte.is_ascii_digit())
}

/// The number a slice of decimal digits spells.
fn number(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .fold(0u32, |value, byte| value * 10 + u32::from(byte - b'0'))
}

#[cfg(test)]
mod tests {
    use super::{bounded, format_duration, format_resolution, is_iso_date, MAX_VALUE_BYTES};
    use crate::xml::escaped_text_len;
    use std::time::Duration;

    /// A short value is kept as it is.
    #[test]
    fn a_short_value_is_kept() {
        assert_eq!(bounded("Nightfall"), "Nightfall");
        assert_eq!(bounded(""), "");
    }

    /// A long value is cut, and the cut is marked (`INT-072`).
    #[test]
    fn a_long_value_is_cut_and_marked() {
        let title = "a".repeat(20_000);
        let bounded = bounded(&title);
        assert!(bounded.ends_with('…'));
        assert!(escaped_text_len(&bounded) <= MAX_VALUE_BYTES);
    }

    /// A value that grows when it is escaped is cut so that the escaped form
    /// fits, not the raw form.
    #[test]
    fn the_escaped_form_is_what_fits() {
        let title = "&".repeat(20_000);
        let bounded = bounded(&title);
        assert!(escaped_text_len(&bounded) <= MAX_VALUE_BYTES);
        assert!(bounded.ends_with('…'));
    }

    /// The cut happens on a character boundary, so that the value stays valid
    /// UTF-8 and the marker is the last character.
    #[test]
    fn the_cut_happens_on_a_character_boundary() {
        let title = "ä".repeat(20_000);
        let bounded = bounded(&title);
        assert!(bounded.ends_with('…'));
        assert!(bounded
            .chars()
            .all(|character| character == 'ä' || character == '…'));
    }

    /// A value of exactly the budget is not cut.
    #[test]
    fn a_value_at_the_budget_is_not_cut() {
        let value = "a".repeat(MAX_VALUE_BYTES);
        assert_eq!(bounded(&value), value);
    }

    /// The duration has the format `H*:MM:SS.F*`.
    #[test]
    fn the_duration_has_the_format_of_the_schema() {
        assert_eq!(format_duration(Duration::from_secs(232)), "0:03:52.000");
        assert_eq!(format_duration(Duration::from_secs(0)), "0:00:00.000");
        assert_eq!(
            format_duration(Duration::from_secs(3600 * 2 + 61)),
            "2:01:01.000"
        );
    }

    /// The resolution has the format `<width>x<height>`.
    #[test]
    fn the_resolution_has_the_format_of_the_schema() {
        assert_eq!(format_resolution(1920, 1080), "1920x1080");
        assert_eq!(format_resolution(0, 0), "0x0");
    }

    /// The two accepted date forms are accepted, and the others are not.
    #[test]
    fn the_two_date_forms_are_accepted() {
        assert!(is_iso_date("2024-05-28"));
        assert!(is_iso_date("2024-05-28T02:28:47"));
        assert!(!is_iso_date("2024-05-28T02:28:47Z"));
        assert!(!is_iso_date("2024-05-28T02:28:47+0000"));
        assert!(!is_iso_date("28.05.2024"));
        assert!(!is_iso_date("2024-13-28"));
        assert!(!is_iso_date("2024-05-32"));
        assert!(!is_iso_date("2024-05-28T24:00:00"));
        assert!(!is_iso_date("2024-5-28"));
        assert!(!is_iso_date(""));
    }
}
