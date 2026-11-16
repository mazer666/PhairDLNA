//! The tests of the parser of the ID3v2 tag.
//!
//! What: the tags of every version of the standard, the tags that are broken
//!       and the tags that lie about their size.
//! Why: the parser reads a file that a person does not control, so every
//!       version and every way of being wrong has a test (`F-10`, `SEC-067`).

use super::{picture_info, read_picture, MAX_TAG_BYTES};
use std::fs::File;
use std::io::Write;

/// Writes bytes into a temporary file and opens it.
fn file_with(bytes: &[u8]) -> (tempfile::NamedTempFile, File) {
    let mut file = tempfile::NamedTempFile::new().expect("a temporary file");
    file.write_all(bytes).expect("the bytes");
    let opened = File::open(file.path()).expect("the temporary file");
    (file, opened)
}

/// The data of a picture frame: the encoding, the type, the description.
///
/// The format is the type of the picture for version 2.3 and 2.4 — the
/// text and its zero byte — and the three bytes of version 2.2.
fn picture(format: &[u8], image: &[u8]) -> Vec<u8> {
    let mut data = vec![0];
    data.extend_from_slice(format);
    data.extend_from_slice(&[3]);
    data.extend_from_slice(b"cover");
    data.push(0);
    data.extend_from_slice(image);
    data
}

/// Builds a tag with the given frames.
fn tag(version: u8, unsynchronised: bool, frames: &[(&str, Vec<u8>)]) -> Vec<u8> {
    let mut body = Vec::new();
    for (id, data) in frames {
        let data = if unsynchronised {
            unsynchronise(data)
        } else {
            data.clone()
        };
        body.extend_from_slice(id.as_bytes());
        let size = data.len() as u32;
        match version {
            // Version 2.2 names a frame in three bytes and counts its
            // size in three bytes; the versions 2.3 and 2.4 name it in
            // four bytes, count it in four bytes and carry two bytes of
            // flags behind the size.
            2 => body.extend_from_slice(&size.to_be_bytes()[1..4]),
            3 => {
                body.extend_from_slice(&size.to_be_bytes());
                body.extend_from_slice(&[0, 0]);
            }
            _ => {
                body.extend_from_slice(&synchsafe_bytes(size));
                body.extend_from_slice(&[0, 0]);
            }
        }
        body.extend_from_slice(&data);
    }
    let mut tag = vec![b'I', b'D', b'3', version, 0];
    tag.push(if unsynchronised { 0x80 } else { 0 });
    tag.extend_from_slice(&synchsafe_bytes(body.len() as u32));
    tag.extend_from_slice(&body);
    tag
}

/// The bytes of a number whose highest bit of every byte is zero.
fn synchsafe_bytes(value: u32) -> [u8; 4] {
    [
        ((value >> 21) & 0x7F) as u8,
        ((value >> 14) & 0x7F) as u8,
        ((value >> 7) & 0x7F) as u8,
        (value & 0x7F) as u8,
    ]
}

/// Puts a zero behind every `0xFF`, as the unsynchronisation does.
fn unsynchronise(data: &[u8]) -> Vec<u8> {
    let mut out = Vec::new();
    for byte in data {
        out.push(*byte);
        if *byte == 0xFF {
            out.push(0);
        }
    }
    out
}

#[test]
fn the_covers_of_every_version_are_read() {
    let image = b"\xFF\xD8\xFF\xE0jpeg";
    let cases = [
        (3u8, "APIC", &b"image/jpg\0"[..], "image/jpeg"),
        (4, "APIC", &b"image/png\0"[..], "image/png"),
        (2, "PIC", &b"JPG"[..], "image/jpeg"),
    ];
    for (version, id, format, mime) in cases {
        let bytes = tag(version, false, &[(id, picture(format, image))]);
        let (_file, mut file) = file_with(&bytes);
        let info = picture_info(&mut file).expect("a picture");
        assert_eq!(info.mime_type, mime);
        assert_eq!(info.length, image.len() as u64);
        let read = read_picture(&mut file, 1024).expect("the picture");
        assert_eq!(read.mime_type, mime);
        assert_eq!(read.bytes, image);
    }
}

#[test]
fn broken_files_have_no_picture() {
    let mut past_the_end = tag(3, false, &[("APIC", picture(b"image/jpeg\0", b"jpeg"))]);
    past_the_end[14..18].copy_from_slice(&u32::MAX.to_be_bytes());
    let mut without_a_size = past_the_end.clone();
    without_a_size[14..18].copy_from_slice(&0u32.to_be_bytes());
    let mut too_large = vec![b'I', b'D', b'3', 4, 0, 0];
    too_large.extend_from_slice(&synchsafe_bytes(MAX_TAG_BYTES as u32));
    let broken = [
        b"not an id3 tag".to_vec(),
        b"ID3\x04\x00".to_vec(),
        tag(3, false, &[("TIT2", b"a title".to_vec())]),
        past_the_end,
        without_a_size,
        too_large,
    ];
    for bytes in broken {
        let (_file, mut file) = file_with(&bytes);
        assert!(picture_info(&mut file).is_none(), "a picture in {bytes:?}");
        assert!(read_picture(&mut file, 1024).is_none());
    }
}

#[test]
fn a_picture_bigger_than_the_limit_is_refused() {
    let image = vec![7u8; 100];
    let bytes = tag(3, false, &[("APIC", picture(b"image/jpeg\0", &image))]);
    let (_file, mut file) = file_with(&bytes);
    assert!(picture_info(&mut file).is_some());
    assert!(read_picture(&mut file, 10).is_none());
    assert_eq!(
        read_picture(&mut file, 100)
            .expect("the picture")
            .bytes
            .len(),
        100
    );
}

#[test]
fn an_unsynchronised_tag_is_read() {
    let image = b"\xFF\xD8\x00\xFFjpeg";
    let bytes = tag(3, true, &[("APIC", picture(b"image/jpeg\0", image))]);
    let (_file, mut file) = file_with(&bytes);
    let read = read_picture(&mut file, 1024).expect("the picture");
    assert_eq!(read.bytes, image);
    assert_eq!(read.mime_type, "image/jpeg");
}

#[test]
fn an_extended_header_is_skipped() {
    let image = b"jpeg";
    let frame = picture(b"image/jpeg\0", image);
    let mut body = Vec::new();
    // The extended header of version 2.3: a size of six bytes that does
    // not count itself.
    body.extend_from_slice(&6u32.to_be_bytes());
    body.extend_from_slice(&[0; 6]);
    body.extend_from_slice(b"APIC");
    body.extend_from_slice(&(frame.len() as u32).to_be_bytes());
    body.extend_from_slice(&[0, 0]);
    body.extend_from_slice(&frame);
    let mut bytes = vec![b'I', b'D', b'3', 3, 0, 0x40];
    bytes.extend_from_slice(&synchsafe_bytes(body.len() as u32));
    bytes.extend_from_slice(&body);
    let (_file, mut file) = file_with(&bytes);
    assert_eq!(
        read_picture(&mut file, 1024).expect("the picture").bytes,
        image
    );
}
