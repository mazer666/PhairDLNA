//! The cover that is embedded in an MP4 container.
//!
//! What: the reading of the cover picture that an `.m4a` or an `.m4b` file
//!      carries in its metadata.
//! Why: a control point asks for `upnp:albumArtURI`, and the server has to
//!      answer with a picture. The cover lies in a container that a damaged
//!      file can make nonsense of, so the parser never trusts the file: every
//!      size is checked against the file, every read is bounded, and a file
//!      that cannot be understood gives no cover instead of an error
//!      (`SEC-067`, `UX-004`).
//! Example: `picture_info` says whether a file has a cover and how big the
//!       cover is, `read_picture` returns the cover itself.
//!
//! **The way to the cover.** `moov` → `udta` → `meta` → `ilst` → `covr` →
//! `data`; some writers put the `meta` directly under the `moov`, and both
//! ways are walked.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use self::atom::{read_atom, Atom};

/// The type of a JPEG cover.
const JPEG: &str = "image/jpeg";
/// The type of a PNG cover.
const PNG: &str = "image/png";
/// The data type of a JPEG in a `data` atom.
const DATA_JPEG: u32 = 13;
/// The data type of a PNG in a `data` atom.
const DATA_PNG: u32 = 14;
/// The bytes between the header of a full box and its children.
const FULL_BOX_HEADER: u64 = 4;
/// The deepest nesting of atoms that is walked. The way to the cover is short
/// and fixed, so the walk cannot nest deeply anyway; the limit is the second
/// line of defence.
const MAX_DEPTH: usize = 8;

/// The way to the cover when the metadata lies under the user data.
const VIA_UDTA: &[&[u8; 4]] = &[b"moov", b"udta", b"meta", b"ilst", b"covr", b"data"];
/// The way to the cover when the metadata lies directly under the movie box.
const DIRECT: &[&[u8; 4]] = &[b"moov", b"meta", b"ilst", b"covr", b"data"];

/// The cover of an MP4 file, described without reading the cover itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mp4PictureInfo {
    /// The type of the cover, as the container names it (`image/jpeg`, `image/png`).
    pub mime_type: String,
    /// How many bytes the cover has.
    pub length: u64,
}

/// The cover of an MP4 file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mp4Picture {
    /// The type of the cover.
    pub mime_type: String,
    /// The cover itself.
    pub bytes: Vec<u8>,
}

/// Describes the cover of an MP4 file without reading the cover.
///
/// Returns `None` for a file that is not an MP4 container, for a container
/// without a cover, and for a container that is broken.
pub fn picture_info(file: &mut File) -> Option<Mp4PictureInfo> {
    file.seek(SeekFrom::Start(0)).ok()?;
    let cover = locate_cover(file)?;
    Some(Mp4PictureInfo {
        mime_type: cover.mime_type.to_owned(),
        length: cover.length,
    })
}

/// Reads the cover of an MP4 file.
///
/// Returns `None` for everything `picture_info` refuses, and for a cover that
/// is larger than `max_bytes`.
pub fn read_picture(file: &mut File, max_bytes: u64) -> Option<Mp4Picture> {
    file.seek(SeekFrom::Start(0)).ok()?;
    let cover = locate_cover(file)?;
    if cover.length > max_bytes {
        return None;
    }
    let mut bytes = Vec::new();
    file.seek(SeekFrom::Start(cover.start)).ok()?;
    file.take(cover.length).read_to_end(&mut bytes).ok()?;
    // The file must be as long as the container promised.
    if bytes.len() as u64 != cover.length {
        return None;
    }
    Some(Mp4Picture {
        mime_type: cover.mime_type.to_owned(),
        bytes,
    })
}

/// The cover inside the container: its type and where it lies.
struct Cover {
    /// The type of the image.
    mime_type: &'static str,
    /// Where the cover begins in the file.
    start: u64,
    /// How many bytes the cover has.
    length: u64,
}

/// The cover of the file.
///
/// Both ways to the cover are walked, because a file that has no user data
/// may still carry its metadata directly under the movie box.
fn locate_cover(file: &mut File) -> Option<Cover> {
    let length = file.metadata().ok()?.len();
    if let Some(cover) = walk(file, 0, length, VIA_UDTA, 0) {
        return Some(cover);
    }
    walk(file, 0, length, DIRECT, 0)
}

/// Walks the way to the cover.
///
/// Nothing comes back when the file does not hold the way, or when one of the
/// atoms is broken: a broken atom ends the walk, because the rest of the file
/// can no longer be trusted.
fn walk(file: &mut File, start: u64, end: u64, path: &[&[u8; 4]], depth: usize) -> Option<Cover> {
    if depth > MAX_DEPTH {
        return None;
    }
    let (wanted, rest) = path.split_first()?;
    let mut offset = start;
    while offset < end {
        let Some(atom) = read_atom(file, offset, end) else {
            break;
        };
        offset = atom.end;
        if &atom.name != *wanted {
            continue;
        }
        // The metadata is a full box: four bytes of version and flags stand
        // between its header and its children.
        let skip = if &atom.name == b"meta" {
            FULL_BOX_HEADER
        } else {
            0
        };
        let children = atom.payload_start.checked_add(skip)?;
        if children > atom.end {
            continue;
        }
        if rest.is_empty() {
            if let Some(cover) = cover_in_data(file, &atom) {
                return Some(cover);
            }
            continue;
        }
        if let Some(cover) = walk(file, children, atom.end, rest, depth + 1) {
            return Some(cover);
        }
    }
    None
}

/// The cover in a `data` atom.
///
/// The atom starts with four bytes of type and four bytes of locale; the
/// cover is what follows them. A type that is neither JPEG nor PNG is not a
/// cover the server can serve, and the atom is skipped.
fn cover_in_data(file: &mut File, data: &Atom) -> Option<Cover> {
    let start = data.payload_start.checked_add(8)?;
    if start > data.end {
        return None;
    }
    file.seek(SeekFrom::Start(data.payload_start)).ok()?;
    let mut indicator = [0u8; 8];
    file.read_exact(&mut indicator).ok()?;
    let kind = u32::from_be_bytes([indicator[0], indicator[1], indicator[2], indicator[3]]);
    let mime_type = match kind & 0x00ff_ffff {
        DATA_JPEG => JPEG,
        DATA_PNG => PNG,
        _ => return None,
    };
    Some(Cover {
        mime_type,
        start,
        length: data.end - start,
    })
}

mod atom;

#[cfg(test)]
mod tests;
