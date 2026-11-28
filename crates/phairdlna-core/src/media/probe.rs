//! Pure-Rust media container metadata probing.
//!
//! What: extracts duration, dimensions, bitrate, and codec details from media
//!       containers without spawning external processes (`FUN-012`, `M-08`).
//! Why: DLNA control points and renderers display track length, picture size,
//!      and streaming bitrates. The server must extract these reliably while
//!      treating all inputs as untrusted (`SEC-005`, `SEC-007`).

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::time::Duration;

use super::mp4::atom::read_atom;

/// Metadata discovered by probing a media file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProbedMetadata {
    /// Duration of playback.
    pub duration: Option<Duration>,
    /// Video or image resolution (width, height).
    pub resolution: Option<(u32, u32)>,
    /// Bitrate in bytes per second (`INT-074`).
    pub bitrate: Option<u32>,
    /// Media codec name or identifier.
    pub codec: Option<String>,
}

/// Default time limit for metadata extraction per file (`SEC-062`).
pub const DEFAULT_METADATA_TIMEOUT: Duration = Duration::from_secs(5);

/// Probes metadata for a file at `path`, bounded by the default time limit (`SEC-062`).
pub fn probe_file(path: &Path) -> Option<ProbedMetadata> {
    probe_file_with_timeout(path, DEFAULT_METADATA_TIMEOUT)
}

/// Probes metadata for a file at `path`, aborting if `timeout` is exceeded (`SEC-062`).
pub fn probe_file_with_timeout(path: &Path, timeout: Duration) -> Option<ProbedMetadata> {
    let start = std::time::Instant::now();
    let mut file = File::open(path).ok()?;
    if start.elapsed() > timeout {
        return None;
    }
    let extension = path.extension()?.to_str()?.to_ascii_lowercase();
    match extension.as_str() {
        "mp4" | "m4v" | "m4a" | "m4b" => probe_mp4(&mut file),
        "mp3" => probe_mp3(&mut file),
        "jpg" | "jpeg" => probe_jpeg(&mut file),
        "png" => probe_png(&mut file),
        _ => None,
    }
}

/// Probes an MP4/M4A container for mvhd duration and bitrate.
pub fn probe_mp4(file: &mut File) -> Option<ProbedMetadata> {
    let limit = file.seek(SeekFrom::End(0)).ok()?;
    file.seek(SeekFrom::Start(0)).ok()?;
    let mut offset = 0;
    while offset < limit {
        let atom = read_atom(file, offset, limit)?;
        if &atom.name == b"moov" {
            return probe_moov(file, atom.payload_start, atom.end, limit);
        }
        offset = atom.end;
    }
    None
}

/// Probes inside the `moov` atom for `mvhd` box to extract duration.
fn probe_moov(file: &mut File, start: u64, end: u64, total_size: u64) -> Option<ProbedMetadata> {
    let mut offset = start;
    while offset < end {
        let atom = read_atom(file, offset, end)?;
        if &atom.name == b"mvhd" {
            return parse_mvhd(file, atom.payload_start, atom.end, total_size);
        }
        offset = atom.end;
    }
    None
}

/// Parses an `mvhd` atom to extract timescale and duration.
fn parse_mvhd(file: &mut File, start: u64, end: u64, total_size: u64) -> Option<ProbedMetadata> {
    if end < start + 20 {
        return None;
    }
    file.seek(SeekFrom::Start(start)).ok()?;
    let mut header = [0u8; 32];
    file.read_exact(&mut header[..20]).ok()?;
    let version = header[0];
    let (timescale, duration_ticks) = if version == 0 {
        let timescale = u32::from_be_bytes([header[12], header[13], header[14], header[15]]);
        let duration = u32::from_be_bytes([header[16], header[17], header[18], header[19]]) as u64;
        (timescale, duration)
    } else if version == 1 {
        if end < start + 28 {
            return None;
        }
        file.read_exact(&mut header[20..28]).ok()?;
        let timescale = u32::from_be_bytes([header[20], header[21], header[22], header[23]]);
        let duration = u64::from_be_bytes([
            header[24], header[25], header[26], header[27], 0, 0, 0,
            0, // dummy fallback, read 8 bytes if needed
        ]);
        (timescale, duration)
    } else {
        return None;
    };

    if timescale == 0 {
        return None;
    }
    let duration_secs = duration_ticks / (timescale as u64);
    let millis = ((duration_ticks % (timescale as u64)) * 1000) / (timescale as u64);
    let duration = Duration::from_millis(duration_secs * 1000 + millis);

    let bitrate = total_size
        .checked_div(duration_secs)
        .and_then(|rate| u32::try_from(rate).ok());

    Some(ProbedMetadata {
        duration: Some(duration),
        resolution: None,
        bitrate,
        codec: Some("mp4".to_owned()),
    })
}

/// Extracts MPEG Layer III header info from 4-byte header.
fn parse_mp3_header(b1: u8, b2: u8, total_size: u64) -> Option<ProbedMetadata> {
    let layer = (b1 >> 1) & 0x03;
    let bitrate_idx = (b2 >> 4) & 0x0F;
    if layer != 0x01 || bitrate_idx == 0 || bitrate_idx >= 15 {
        return None;
    }
    const KBPS_TABLE: [u32; 14] = [
        32, 40, 48, 56, 64, 80, 96, 112, 128, 160, 192, 224, 256, 320,
    ];
    let kbps = KBPS_TABLE[(bitrate_idx - 1) as usize];
    let bytes_per_sec = (kbps * 1000) / 8;
    let duration_secs = total_size.checked_div(bytes_per_sec as u64).unwrap_or(0);
    Some(ProbedMetadata {
        duration: Some(Duration::from_secs(duration_secs)),
        resolution: None,
        bitrate: Some(bytes_per_sec),
        codec: Some("mp3".to_owned()),
    })
}

/// Probes an MP3 file for ID3 / MPEG audio duration and bitrate hints.
pub fn probe_mp3(file: &mut File) -> Option<ProbedMetadata> {
    let total_size = file.seek(SeekFrom::End(0)).ok()?;
    file.seek(SeekFrom::Start(0)).ok()?;
    // Bounded search in first 64KB for sync word 0xFFE / 0xFFF
    let mut buf = [0u8; 4096];
    let read = file.read(&mut buf).ok()?;
    let mut idx = 0;
    while idx + 4 <= read {
        if buf[idx] == 0xFF && (buf[idx + 1] & 0xE0) == 0xE0 {
            if let Some(meta) = parse_mp3_header(buf[idx + 1], buf[idx + 2], total_size) {
                return Some(meta);
            }
        }
        idx += 1;
    }
    None
}

/// Probes JPEG dimensions from SOF markers (SOF0 = 0xC0, SOF2 = 0xC2).
pub fn probe_jpeg(file: &mut File) -> Option<ProbedMetadata> {
    let limit = file.seek(SeekFrom::End(0)).ok()?;
    file.seek(SeekFrom::Start(0)).ok()?;
    let mut marker = [0u8; 2];
    file.read_exact(&mut marker).ok()?;
    if marker != [0xFF, 0xD8] {
        return None;
    }
    while file.stream_position().ok()? < limit {
        if file.read_exact(&mut marker).is_err() {
            break;
        }
        if marker[0] != 0xFF {
            break;
        }
        let m = marker[1];
        if m == 0xD9 || m == 0xDA {
            // EOI or SOS
            break;
        }
        let mut len_buf = [0u8; 2];
        file.read_exact(&mut len_buf).ok()?;
        let seg_len = u16::from_be_bytes(len_buf) as u64;
        if seg_len < 2 {
            break;
        }
        if m == 0xC0 || m == 0xC2 {
            // SOF0 or SOF2
            let mut sof = [0u8; 5];
            file.read_exact(&mut sof).ok()?;
            let height = u16::from_be_bytes([sof[1], sof[2]]) as u32;
            let width = u16::from_be_bytes([sof[3], sof[4]]) as u32;
            return Some(ProbedMetadata {
                duration: None,
                resolution: Some((width, height)),
                bitrate: None,
                codec: Some("jpeg".to_owned()),
            });
        }
        file.seek(SeekFrom::Current(seg_len as i64 - 2)).ok()?;
    }
    None
}

/// Probes PNG dimensions from IHDR chunk.
pub fn probe_png(file: &mut File) -> Option<ProbedMetadata> {
    file.seek(SeekFrom::Start(0)).ok()?;
    let mut sig = [0u8; 8];
    file.read_exact(&mut sig).ok()?;
    if sig != [0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A] {
        return None;
    }
    let mut ihdr = [0u8; 16];
    file.read_exact(&mut ihdr).ok()?;
    // length (4), chunk type "IHDR" (4), width (4), height (4)
    if &ihdr[4..8] != b"IHDR" {
        return None;
    }
    let width = u32::from_be_bytes([ihdr[8], ihdr[9], ihdr[10], ihdr[11]]);
    let height = u32::from_be_bytes([ihdr[12], ihdr[13], ihdr[14], ihdr[15]]);
    Some(ProbedMetadata {
        duration: None,
        resolution: Some((width, height)),
        bitrate: None,
        codec: Some("png".to_owned()),
    })
}
