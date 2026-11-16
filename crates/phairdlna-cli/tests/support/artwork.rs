#![allow(dead_code)]
//! Fixtures for the tests of the artwork.
//!
//! What: a track with a cover in its ID3v2 tag, and a small JPEG picture.
//! Why: the daemon must answer a request for a picture with the bytes of the
//!       picture, and the only honest way to test that is to give the daemon a
//!       file that carries a picture (`F-10`).
//! Example: `mp3_with_cover("image/jpeg", &jpeg(8, 8))` is the bytes of a
//!       track with a cover of its own.

/// Builds the bytes of an MP3 file with a cover in an ID3v2 tag.
pub fn mp3_with_cover(mime_type: &str, picture: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();
    frame.push(0);
    frame.extend_from_slice(mime_type.as_bytes());
    frame.push(0);
    frame.push(3);
    frame.push(0);
    frame.extend_from_slice(picture);
    let mut tag = Vec::new();
    tag.extend_from_slice(b"ID3");
    tag.extend_from_slice(&[3, 0]);
    tag.push(0);
    // The size of the tag counts the header of the frame as well.
    tag.extend_from_slice(&synchsafe((frame.len() + 10) as u32));
    tag.extend_from_slice(b"APIC");
    tag.extend_from_slice(&(frame.len() as u32).to_be_bytes());
    tag.extend_from_slice(&[0, 0]);
    tag.extend_from_slice(&frame);
    tag.extend_from_slice(b"the audio data of the file");
    tag
}

/// Builds the bytes of an M4A file with a cover in its item list.
///
/// The type of the cover is the one of the standard: 13 for JPEG, 14 for PNG.
pub fn m4a_with_cover(cover_type: u32, picture: &[u8]) -> Vec<u8> {
    let mut payload = vec![0, 0, 0, cover_type as u8];
    payload.extend_from_slice(&[0, 0, 0, 0]);
    payload.extend_from_slice(picture);
    let covr = atom(b"covr", &atom(b"data", &payload));
    let ilst = atom(b"ilst", &covr);
    let mut meta = vec![0, 0, 0, 0];
    meta.extend_from_slice(&ilst);
    let udta = atom(b"udta", &atom(b"meta", &meta));
    let mut file = atom(b"ftyp", b"M4A mp42isom");
    file.extend_from_slice(&atom(b"moov", &udta));
    file
}

/// Builds a small PNG picture of the given size.
pub fn png(width: u32, height: u32) -> Vec<u8> {
    let picture = image::RgbImage::from_pixel(width, height, image::Rgb([64, 32, 16]));
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgb8(picture)
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Png,
        )
        .expect("the picture is written");
    bytes
}

/// Builds a small JPEG picture of the given size.
pub fn jpeg(width: u32, height: u32) -> Vec<u8> {
    let picture = image::RgbImage::from_pixel(width, height, image::Rgb([16, 32, 64]));
    let mut bytes = Vec::new();
    image::DynamicImage::ImageRgb8(picture)
        .write_to(
            &mut std::io::Cursor::new(&mut bytes),
            image::ImageFormat::Jpeg,
        )
        .expect("the picture is written");
    bytes
}

/// Builds a size of the ID3v2 header.
///
/// The size is written with seven bits per byte, as the standard demands, so
/// that a size of more than 128 kilobytes is still written down correctly.
fn synchsafe(value: u32) -> [u8; 4] {
    [
        ((value >> 21) & 0x7f) as u8,
        ((value >> 14) & 0x7f) as u8,
        ((value >> 7) & 0x7f) as u8,
        (value & 0x7f) as u8,
    ]
}

/// Builds an atom of an MP4 file.
fn atom(name: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(payload.len() + 8);
    bytes.extend_from_slice(&((payload.len() + 8) as u32).to_be_bytes());
    bytes.extend_from_slice(name);
    bytes.extend_from_slice(payload);
    bytes
}
