//! Tests for FUN-012 container metadata extraction.

use std::io::Write;
use std::time::Duration;
use tempfile::NamedTempFile;

use phairdlna_core::media::probe::{probe_file, probe_file_with_timeout, probe_mp4, probe_png};

/// Builds an atom with a normal size.
fn atom(name: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut bytes = (payload.len() as u32 + 8).to_be_bytes().to_vec();
    bytes.extend_from_slice(name);
    bytes.extend_from_slice(payload);
    bytes
}

/// Builds an mvhd payload with timescale and duration (version 0).
fn mvhd_payload(timescale: u32, duration: u32) -> Vec<u8> {
    let mut buf = vec![0u8; 20];
    buf[12..16].copy_from_slice(&timescale.to_be_bytes());
    buf[16..20].copy_from_slice(&duration.to_be_bytes());
    buf
}

#[test]
fn probes_mp4_duration_and_bitrate() {
    let mvhd = atom(b"mvhd", &mvhd_payload(1000, 60_000)); // 60 seconds
    let moov = atom(b"moov", &mvhd);
    let mut file_bytes = atom(b"ftyp", b"mp42isom");
    file_bytes.extend_from_slice(&moov);

    let mut temp = NamedTempFile::new().expect("temp file");
    temp.write_all(&file_bytes).expect("write temp");
    temp.flush().expect("flush temp");

    let mut file = temp.reopen().expect("reopen");
    let probed = probe_mp4(&mut file).expect("probed mp4");
    assert_eq!(probed.duration, Some(Duration::from_secs(60)));
    assert_eq!(probed.codec, Some("mp4".to_owned()));
}

#[test]
fn probes_png_dimensions() {
    let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    let mut ihdr_data = Vec::new();
    ihdr_data.extend_from_slice(&1920u32.to_be_bytes());
    ihdr_data.extend_from_slice(&1080u32.to_be_bytes());
    ihdr_data.extend_from_slice(&[8, 2, 0, 0, 0]); // 8-bit truecolor

    // length (4), type (4), data, crc (4)
    png.extend_from_slice(&(ihdr_data.len() as u32).to_be_bytes());
    png.extend_from_slice(b"IHDR");
    png.extend_from_slice(&ihdr_data);
    png.extend_from_slice(&[0, 0, 0, 0]); // dummy crc

    let mut temp = NamedTempFile::new().expect("temp file");
    temp.write_all(&png).expect("write temp");
    temp.flush().expect("flush temp");

    let mut file = temp.reopen().expect("reopen");
    let probed = probe_png(&mut file).expect("probed png");
    assert_eq!(probed.resolution, Some((1920, 1080)));
    assert_eq!(probed.codec, Some("png".to_owned()));
}

#[test]
fn unsupported_or_broken_file_yields_none() {
    let mut temp = NamedTempFile::new().expect("temp file");
    temp.write_all(b"garbage not a valid media header")
        .expect("write temp");
    temp.flush().expect("flush temp");

    assert!(probe_file(temp.path()).is_none());
}

/// SEC-062 — metadata extraction respects time limit and abandons file if exceeded.
#[test]
fn sec_062_metadata_extraction_time_limit_abandonment() {
    let mvhd = atom(b"mvhd", &mvhd_payload(1000, 60_000));
    let moov = atom(b"moov", &mvhd);
    let mut file_bytes = atom(b"ftyp", b"mp42isom");
    file_bytes.extend_from_slice(&moov);

    let mut temp = NamedTempFile::new().expect("temp file");
    temp.write_all(&file_bytes).expect("write temp");
    temp.flush().expect("flush temp");

    // Zero-duration timeout triggers immediate abandonment
    assert!(probe_file_with_timeout(temp.path(), Duration::ZERO).is_none());
}
