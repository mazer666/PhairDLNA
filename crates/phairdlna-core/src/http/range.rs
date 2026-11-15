//! The `Range` header of a media request.
//!
//! What: the range a client asks for, the parser of the header and the
//!       resolver of the range against the length of the file.
//! Why: the range is what makes a timeline drag possible, and it is the part
//!       of HTTP a media server gets wrong most often. The server serves one
//!       range and never a `multipart/byteranges` body (INT-085, standards
//!       §5.1 H4), and every offset is 64-bit, because a 32-bit offset stops
//!       at four gigabytes and the client plays the first part of the film
//!       and then stops (FUN-028).
//! Example: `resolve` turns `bytes=100-199` into the range 100 to 199 of a
//!       file of a thousand bytes.

/// A range as the client wrote it.
///
/// The form matters until the range meets the length of the file: a suffix
/// range of a hundred bytes is the last hundred bytes of the file, and not
/// the first hundred.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RangeSpec {
    /// `bytes=<first>-<last>`
    Closed {
        /// The first byte of the range.
        first: u64,
        /// The last byte of the range, included.
        last: u64,
    },
    /// `bytes=<first>-`
    From {
        /// The first byte of the range.
        first: u64,
    },
    /// `bytes=-<length>`
    Suffix {
        /// The number of bytes the range holds, counted from the end.
        length: u64,
    },
}

/// What a `Range` header asks for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RangeRequest {
    /// The client asks for the whole representation.
    Whole,
    /// The client asks for one byte range.
    Single(RangeSpec),
    /// The client asks for several ranges; the server serves the first one
    /// that the representation can satisfy (INT-085).
    Many(Vec<RangeSpec>),
    /// The range is well-formed but the representation cannot satisfy it.
    Unsatisfiable,
    /// The header cannot be parsed, or names a unit the server does not
    /// support. The server ignores it and serves the whole representation
    /// (INT-083, INT-092).
    Ignored,
}

/// The `Range` unit the server supports.
pub const BYTES: &str = "bytes";

/// Reads a `Range` header.
///
/// The header is either `bytes=<first>-<last>`, `bytes=<first>-` or
/// `bytes=-<suffix>`; a comma separates several ranges. A header that names
/// another unit is ignored, because the server does not know it (INT-092).
pub fn parse(header: &str) -> RangeRequest {
    let header = header.trim();
    let Some((unit, specs)) = header.split_once('=') else {
        return RangeRequest::Ignored;
    };
    if !unit.trim().eq_ignore_ascii_case(BYTES) {
        return RangeRequest::Ignored;
    }
    let mut ranges = Vec::new();
    for spec in specs.split(',') {
        match parse_spec(spec.trim()) {
            Some(range) => ranges.push(range),
            None => return RangeRequest::Unsatisfiable,
        }
    }
    match ranges.len() {
        0 => RangeRequest::Ignored,
        1 => RangeRequest::Single(ranges[0]),
        _ => RangeRequest::Many(ranges),
    }
}

/// Reads one range specification.
///
/// The three forms are `first-last`, `first-` and `-suffix`. A specification
/// that is none of them is not a range at all (INT-083).
fn parse_spec(spec: &str) -> Option<RangeSpec> {
    let (first, last) = spec.split_once('-')?;
    let (first, last) = (first.trim(), last.trim());
    if first.is_empty() {
        // The suffix form: the last `last` bytes.
        let length: u64 = last.parse().ok()?;
        if length == 0 {
            return None;
        }
        return Some(RangeSpec::Suffix { length });
    }
    let first: u64 = first.parse().ok()?;
    if last.is_empty() {
        // The open form: from `first` to the end.
        return Some(RangeSpec::From { first });
    }
    let last: u64 = last.parse().ok()?;
    if last < first {
        return None;
    }
    Some(RangeSpec::Closed { first, last })
}

#[cfg(test)]
mod tests {
    use super::{parse, RangeRequest, RangeSpec};

    /// The three forms of a range are read.
    #[test]
    fn the_three_forms_are_read() {
        assert_eq!(
            parse("bytes=100-199"),
            RangeRequest::Single(RangeSpec::Closed {
                first: 100,
                last: 199
            })
        );
        assert_eq!(
            parse("bytes=100-"),
            RangeRequest::Single(RangeSpec::From { first: 100 })
        );
        assert_eq!(
            parse("bytes=-100"),
            RangeRequest::Single(RangeSpec::Suffix { length: 100 })
        );
    }

    /// A header that names another unit is ignored.
    #[test]
    fn another_unit_is_ignored() {
        assert_eq!(parse("items=0-10"), RangeRequest::Ignored);
        assert_eq!(parse("nonsense"), RangeRequest::Ignored);
        assert_eq!(parse(""), RangeRequest::Ignored);
    }

    /// A header that cannot be parsed is not satisfiable.
    #[test]
    fn a_broken_range_is_not_satisfiable() {
        assert_eq!(parse("bytes=abc-def"), RangeRequest::Unsatisfiable);
        assert_eq!(parse("bytes="), RangeRequest::Unsatisfiable);
        assert_eq!(parse("bytes=200-100"), RangeRequest::Unsatisfiable);
        assert_eq!(parse("bytes=-0"), RangeRequest::Unsatisfiable);
        assert_eq!(parse("bytes=1-2-3"), RangeRequest::Unsatisfiable);
    }

    /// Several ranges are read as a set.
    #[test]
    fn several_ranges_are_read_as_a_set() {
        assert!(matches!(parse("bytes=0-1,5-6"), RangeRequest::Many(_)));
    }
}
