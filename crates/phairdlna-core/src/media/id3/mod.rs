//! The picture that is embedded in an ID3v2 tag.
//!
//! What: the reading of the cover picture out of the ID3v2 tag of an MP3 file.
//! Why: a control point asks for the cover of a track before it plays it, and
//!      the cover is the part of a file that is broken most often. The parser
//!      therefore never trusts the tag: every size is checked against what is
//!      left of the tag, every read is bounded by `MAX_TAG_BYTES`, and a
//!      broken tag gives no picture instead of an error (`F-10`, `SEC-067`).
//! Example: `picture_info` says which picture a tag holds, `read_picture`
//!       hands out its bytes.

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use self::picture::PictureHead;

/// The largest tag that is read.
///
/// The sizes of a tag are numbers of a file, and a file must not be able to
/// make the server read without end.
pub const MAX_TAG_BYTES: u64 = 16 * 1024 * 1024;

/// The largest part of a picture frame that is read to describe the picture.
const HEAD_LIMIT: u64 = 1024;

/// The picture of an ID3v2 tag, described without reading the picture itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id3PictureInfo {
    /// The type of the picture, as the tag names it (`image/jpeg`, `image/png`).
    pub mime_type: String,
    /// How many bytes the picture has.
    pub length: u64,
}

/// The picture of an ID3v2 tag.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Id3Picture {
    /// The type of the picture, as the tag names it.
    pub mime_type: String,
    /// The picture itself.
    pub bytes: Vec<u8>,
}

/// Describes the picture of an ID3v2 tag without reading the picture.
///
/// Returns `None` for a file that is not an ID3v2 tag, for a tag without a
/// picture, for a broken tag, and for a tag that is larger than
/// [`MAX_TAG_BYTES`].
pub fn picture_info(file: &mut File) -> Option<Id3PictureInfo> {
    let tag = Tag::open(file)?;
    let frame = find_picture(file, &tag)?;
    let head = read_at(file, frame.offset, frame.size.min(HEAD_LIMIT))?;
    let head = desynchronise(&head, frame.unsynchronised);
    let header = PictureHead::parse(&head, tag.major_version)?;
    // What is left of the frame is the picture. For an unsynchronised tag
    // this is an upper bound, because the zeros that the unsynchronisation
    // inserted are not counted here.
    Some(Id3PictureInfo {
        mime_type: header.mime_type,
        length: frame.size.saturating_sub(header.offset as u64),
    })
}

/// Reads the picture of an ID3v2 tag.
///
/// Returns `None` for everything `picture_info` refuses, and for a picture
/// that is larger than `max_bytes`.
pub fn read_picture(file: &mut File, max_bytes: u64) -> Option<Id3Picture> {
    let tag = Tag::open(file)?;
    let frame = find_picture(file, &tag)?;
    let data = read_at(file, frame.offset, frame.size)?;
    let mut data = desynchronise(&data, frame.unsynchronised);
    let header = PictureHead::parse(&data, tag.major_version)?;
    if header.offset > data.len() {
        return None;
    }
    data.drain(..header.offset);
    if data.len() as u64 > max_bytes {
        return None;
    }
    Some(Id3Picture {
        mime_type: header.mime_type,
        bytes: data,
    })
}

/// The header of a tag, as the parser understands it.
struct Tag {
    /// The major version of the tag (2, 3 or 4).
    major_version: u8,
    /// Whether the data of the frames is unsynchronised.
    unsynchronised: bool,
    /// Where the frames begin and where they end.
    frames_start: u64,
    frames_end: u64,
    /// The length of the header of a frame.
    frame_header: u64,
}

impl Tag {
    /// Reads the header of the tag and checks it.
    fn open(file: &mut File) -> Option<Self> {
        file.seek(SeekFrom::Start(0)).ok()?;
        let mut header = [0u8; 10];
        file.read_exact(&mut header).ok()?;
        if &header[0..3] != b"ID3" {
            return None;
        }
        let major_version = header[3];
        if !(2..=4).contains(&major_version) {
            return None;
        }
        let flags = header[5];
        let size = synchsafe(&header[6..10]);
        // A footer of version 2.4 is counted in the size of the tag, but it
        // holds no frame.
        let has_footer = major_version == 4 && flags & 0x10 != 0;
        let footer = if has_footer { 10 } else { 0 };
        if size < footer || 10 + size > MAX_TAG_BYTES {
            return None;
        }
        let frames_end = 10 + size - footer;
        let frames_start = 10 + extended_header_len(file, major_version, flags, frames_end)?;
        Some(Tag {
            major_version,
            unsynchronised: flags & 0x80 != 0,
            frames_start,
            frames_end,
            frame_header: if major_version == 2 { 6 } else { 10 },
        })
    }
}

/// The length of the extended header of a tag, or zero when it has none.
fn extended_header_len(file: &mut File, version: u8, flags: u8, frames_end: u64) -> Option<u64> {
    if flags & 0x40 == 0 {
        return Some(0);
    }
    let size = read_at(file, 10, 4)?;
    // Version 2.4 counts the size field itself, the older versions do not.
    let length = if version == 4 {
        synchsafe(&size)
    } else {
        u32::from_be_bytes([size[0], size[1], size[2], size[3]]) as u64 + 4
    };
    if length > frames_end - 10 {
        return None;
    }
    Some(length)
}

/// A frame of a tag.
struct Frame {
    /// The identifier of the frame (`APIC`, `PIC`, ...).
    id: String,
    /// The offset of the data of the frame in the file.
    offset: u64,
    /// The size of the data of the frame.
    size: u64,
    /// Whether the data of the frame is unsynchronised.
    unsynchronised: bool,
}

/// The first picture frame of a tag.
fn find_picture(file: &mut File, tag: &Tag) -> Option<Frame> {
    let picture = if tag.major_version == 2 {
        "PIC"
    } else {
        "APIC"
    };
    let mut pos = tag.frames_start;
    while pos + tag.frame_header <= tag.frames_end {
        let frame = frame_at(file, tag, pos)?;
        pos += tag.frame_header + frame.size;
        if frame.id == picture {
            return Some(frame);
        }
    }
    None
}

/// The frame that begins at `pos`, or nothing when the tag is broken there.
fn frame_at(file: &mut File, tag: &Tag, pos: u64) -> Option<Frame> {
    let header = read_at(file, pos, tag.frame_header)?;
    let (id, mut size, flags) = frame_header(&header, tag.major_version)?;
    // A frame without a size means that the tag ends here.
    if size == 0 {
        return None;
    }
    let data_start = pos + tag.frame_header;
    if size > tag.frames_end - data_start {
        return None;
    }
    let mut offset = data_start;
    let mut unsynchronised = tag.unsynchronised;
    // A compressed, encrypted or grouped frame cannot be read. The walk ends
    // there rather than guessing where such a frame ends.
    if flags & (0x0080 | 0x0040 | 0x0020 | 0x0100) != 0 {
        return None;
    }
    if tag.major_version == 4 && flags & 0x0001 != 0 && size >= 4 {
        // The data length indicator counts the data of the frame; the frame
        // itself is four bytes longer.
        offset += 4;
        size -= 4;
    }
    if tag.major_version == 4 && flags & 0x0200 != 0 {
        // The data of the frame can be unsynchronised on its own.
        unsynchronised = true;
    }
    Some(Frame {
        id,
        offset,
        size,
        unsynchronised,
    })
}

/// The identifier, the size and the flags of the header of a frame.
///
/// Version 2.2 names a frame in three bytes and counts its size in plain
/// bytes, version 2.3 in four bytes and plain bytes, version 2.4 in four
/// bytes and synchsafe bytes.
fn frame_header(header: &[u8], version: u8) -> Option<(String, u64, u16)> {
    let wide = version != 2;
    let id = String::from_utf8(header[0..if wide { 4 } else { 3 }].to_vec()).ok()?;
    let size = match version {
        // The three bytes of version 2.2 are the last three of a big-endian
        // number; the zero stands in front of them.
        2 => u32::from_be_bytes([0, header[3], header[4], header[5]]) as u64,
        3 => u32::from_be_bytes([header[4], header[5], header[6], header[7]]) as u64,
        _ => synchsafe(&header[4..8]),
    };
    let flags = if wide {
        u16::from_be_bytes([header[8], header[9]])
    } else {
        0
    };
    Some((id, size, flags))
}

/// Removes the zeros that the unsynchronisation put after every `0xFF`.
fn desynchronise(data: &[u8], unsynchronised: bool) -> Vec<u8> {
    if !unsynchronised {
        return data.to_vec();
    }
    let mut out: Vec<u8> = Vec::with_capacity(data.len());
    let mut after_ff = false;
    for byte in data {
        // The zero behind a `0xFF` is the one the unsynchronisation inserted.
        if !(after_ff && *byte == 0) {
            out.push(*byte);
        }
        after_ff = *byte == 0xFF;
    }
    out
}

/// Reads a part of a file, or nothing when it cannot be read.
fn read_at(file: &mut File, offset: u64, length: u64) -> Option<Vec<u8>> {
    let length = usize::try_from(length).ok()?;
    file.seek(SeekFrom::Start(offset)).ok()?;
    let mut buffer = vec![0u8; length];
    file.read_exact(&mut buffer).ok()?;
    Some(buffer)
}

/// Reads a synchsafe number: seven bits per byte, the highest bit is zero.
fn synchsafe(bytes: &[u8]) -> u64 {
    bytes
        .iter()
        .take(4)
        .enumerate()
        .fold(0u64, |value, (index, byte)| {
            value | (u64::from(*byte & 0x7F) << (7 * (3 - index)))
        })
}

mod picture;

#[cfg(test)]
mod tests;
