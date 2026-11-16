//! The tests of the parser of the MP4 container.
//!
//! What: the containers that carry a cover, the containers that carry none and
//!       the containers whose atoms lie about their size.
//! Why: the parser walks a file that a person does not control, so every way
//!       of being wrong has a test (`F-10`, `SEC-067`).

use super::{picture_info, read_picture, DATA_JPEG, DATA_PNG};
use std::fs::File;
use std::io::{Seek, Write};
use tempfile::NamedTempFile;

/// The bytes of a cover; no other part of a test file holds them.
const COVER: [u8; 4] = [0xa1, 0xb2, 0xc3, 0xd4];

/// Builds an atom with a normal size.
fn atom(name: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut bytes = (payload.len() as u32 + 8).to_be_bytes().to_vec();
    bytes.extend_from_slice(name);
    bytes.extend_from_slice(payload);
    bytes
}

/// Builds the `data` atom of a cover.
fn data_atom(kind: u32, cover: &[u8]) -> Vec<u8> {
    let mut payload = kind.to_be_bytes().to_vec();
    payload.extend_from_slice(&0u32.to_be_bytes());
    payload.extend_from_slice(cover);
    atom(b"data", &payload)
}

/// Builds an MP4 file that carries a cover.
fn m4a(kind: u32, cover: &[u8], under_moov: bool) -> Vec<u8> {
    let mut meta = vec![0u8; 4];
    meta.extend_from_slice(&atom(b"ilst", &atom(b"covr", &data_atom(kind, cover))));
    let meta = atom(b"meta", &meta);
    let moov = if under_moov {
        meta
    } else {
        atom(b"udta", &meta)
    };
    let mut file = atom(b"ftyp", b"M4A mp42isom");
    file.extend_from_slice(&atom(b"moov", &moov));
    file
}

/// Writes the bytes into a temporary file and opens it for reading.
fn open(bytes: &[u8]) -> File {
    let mut file = NamedTempFile::new().expect("a temporary file");
    file.write_all(bytes).expect("a written file");
    file.flush().expect("a flushed file");
    file.reopen().expect("an opened file")
}

/// The place of the movie box in a file.
fn moov_of(bytes: &[u8]) -> usize {
    bytes
        .windows(4)
        .position(|part| part == b"moov")
        .expect("the movie box")
}

#[test]
fn the_cover_is_found_and_read() {
    let mut file = open(&m4a(DATA_JPEG, &COVER, false));
    let info = picture_info(&mut file).expect("a cover");
    assert_eq!(info.mime_type, "image/jpeg");
    assert_eq!(info.length, COVER.len() as u64);
    let picture = read_picture(&mut file, 1024).expect("the cover");
    assert_eq!(picture.mime_type, "image/jpeg");
    assert_eq!(picture.bytes, COVER);
    let mut file = open(&m4a(DATA_PNG, &COVER, false));
    let info = picture_info(&mut file).expect("a cover");
    assert_eq!(info.mime_type, "image/png");
    assert_eq!(
        read_picture(&mut file, 1024).expect("the cover").bytes,
        COVER
    );
}

#[test]
fn the_metadata_may_lie_directly_under_the_movie_box() {
    let mut file = open(&m4a(DATA_JPEG, &COVER, true));
    let info = picture_info(&mut file).expect("a cover");
    assert_eq!(info.length, COVER.len() as u64);
}

#[test]
fn a_container_without_a_cover_has_none() {
    let mut meta = vec![0u8; 4];
    meta.extend_from_slice(&atom(b"ilst", b""));
    let moov = atom(b"moov", &atom(b"udta", &atom(b"meta", &meta)));
    let mut file = open(&moov);
    assert!(picture_info(&mut file).is_none());
    assert!(read_picture(&mut file, 1024).is_none());
    let mut file = open(&m4a(0, &COVER, false));
    assert!(picture_info(&mut file).is_none());
}

#[test]
fn a_file_that_is_not_an_mp4_has_none() {
    let mut file = open(b"this is not an mp4 container at all");
    assert!(picture_info(&mut file).is_none());
    let bytes = m4a(DATA_JPEG, &COVER, false);
    let mut file = open(&bytes[..bytes.len() / 2]);
    assert!(picture_info(&mut file).is_none());
}

#[test]
fn an_atom_with_a_size_of_zero_reaches_the_end_of_the_file() {
    let mut bytes = m4a(DATA_JPEG, &COVER, false);
    let moov = moov_of(&bytes);
    bytes[moov - 4..moov].copy_from_slice(&0u32.to_be_bytes());
    let mut file = open(&bytes);
    assert!(picture_info(&mut file).is_some());
}

#[test]
fn an_atom_that_is_smaller_than_its_header_ends_the_walk() {
    let mut bytes = m4a(DATA_JPEG, &COVER, false);
    let moov = moov_of(&bytes);
    bytes[moov - 4..moov].copy_from_slice(&4u32.to_be_bytes());
    let mut file = open(&bytes);
    assert!(picture_info(&mut file).is_none());
}

#[test]
fn an_atom_with_a_64_bit_size_is_understood() {
    let mut bytes = m4a(DATA_JPEG, &COVER, false);
    let moov = moov_of(&bytes);
    let size = u32::from_be_bytes(bytes[moov - 4..moov].try_into().expect("the size")) as u64;
    let mut wide = Vec::new();
    wide.extend_from_slice(&1u32.to_be_bytes());
    wide.extend_from_slice(b"moov");
    wide.extend_from_slice(&(size + 8).to_be_bytes());
    bytes.splice(moov - 4..moov + 4, wide);
    let mut file = open(&bytes);
    assert!(picture_info(&mut file).is_some());
}

#[test]
fn a_cover_bigger_than_the_limit_is_refused() {
    let mut file = open(&m4a(DATA_JPEG, &COVER, false));
    assert!(read_picture(&mut file, 1).is_none());
    let info = picture_info(&mut file).expect("a cover");
    assert_eq!(info.length, COVER.len() as u64);
}

#[test]
fn picture_info_does_not_read_the_cover() {
    let bytes = m4a(DATA_JPEG, &COVER, false);
    let start = bytes
        .windows(4)
        .position(|part| part == &COVER[..])
        .expect("the cover");
    let mut file = open(&bytes);
    picture_info(&mut file).expect("a cover");
    assert_eq!(file.stream_position().expect("a position"), start as u64);
}
