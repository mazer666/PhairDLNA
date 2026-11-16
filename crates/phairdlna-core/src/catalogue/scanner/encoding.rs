//! The character set of a subtitle file.
//!
//! What: the answer to the question "how is this file written?".
//! Why: a client reads the bytes of a subtitle and shows them; a file that is
//!       read as the wrong character set shows mojibake, and the only way to
//!       avoid it is to say what the file is (`FUN-027`).
//! Example: a file that is valid UTF-8 is UTF-8, and a file that is not is
//!       read as Latin-1, which never fails and never loses a byte.

use std::path::Path;

/// The number of bytes that are read to know the character set of a file.
///
/// The probe is short: a subtitle is a text file, and its first kilobytes
/// tell whether the file is UTF-8. The probe is bounded, so that a file that
/// is not a subtitle at all cannot cost the scan an unbounded reading.
const PROBE_LENGTH: usize = 4096;

/// The character set a subtitle file is written in.
///
/// The answer is what the response states, so that the client reads the file
/// the way it is written (`FUN-027`).
pub(super) fn charset_of(path: &Path) -> &'static str {
    let Ok(bytes) = read_prefix(path, PROBE_LENGTH) else {
        return "utf-8";
    };
    match std::str::from_utf8(&bytes) {
        Ok(_) => "utf-8",
        // A sequence that the probe cuts off says nothing about the file: the
        // file may well be UTF-8.
        Err(error) if error.error_len().is_none() => "utf-8",
        Err(_) => "iso-8859-1",
    }
}

/// Reads the beginning of a file, within a bound.
fn read_prefix(path: &Path, length: usize) -> std::io::Result<Vec<u8>> {
    use std::io::Read;
    let file = std::fs::File::open(path)?;
    let mut bytes = Vec::new();
    file.take(length as u64).read_to_end(&mut bytes)?;
    Ok(bytes)
}
