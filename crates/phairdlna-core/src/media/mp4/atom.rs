//! The atoms of an MP4 container.
//!
//! What: the header of an atom — its name, where its content begins and where
//!       it ends.
//! Why: every atom of a container is written by someone else, and an atom that
//!       lies about its size would send the reader into the wrong part of the
//!       file; the size is therefore checked against the container before the
//!       atom is used (`SEC-067`).

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

/// The size of the header of an atom: four bytes of size, four of name.
const HEADER: u64 = 8;

/// One atom of a container.
pub(super) struct Atom {
    /// The name of the atom.
    pub(super) name: [u8; 4],
    /// Where the content of the atom begins.
    pub(super) payload_start: u64,
    /// Where the atom ends.
    pub(super) end: u64,
}

/// Reads the header of the atom at the given place.
///
/// Nothing comes back when the atom does not fit into the container, or when
/// its size is smaller than its own header: such an atom is broken, and a
/// broken atom ends the walk.
pub(super) fn read_atom(file: &mut File, offset: u64, limit: u64) -> Option<Atom> {
    let header_end = offset.checked_add(HEADER)?;
    if header_end > limit {
        return None;
    }
    file.seek(SeekFrom::Start(offset)).ok()?;
    let mut header = [0u8; 8];
    file.read_exact(&mut header).ok()?;
    let name = [header[4], header[5], header[6], header[7]];
    let declared = u32::from_be_bytes([header[0], header[1], header[2], header[3]]);
    let (size, payload_start) = if declared == 1 {
        // A size of one means that the real size follows the name.
        let mut wide = [0u8; 8];
        file.read_exact(&mut wide).ok()?;
        (u64::from_be_bytes(wide), header_end + 8)
    } else if declared == 0 {
        // A size of zero means that the atom reaches the end of the container.
        (limit - offset, header_end)
    } else {
        (u64::from(declared), header_end)
    };
    if size < payload_start - offset {
        return None;
    }
    let end = offset.checked_add(size)?;
    if end > limit {
        return None;
    }
    Some(Atom {
        name,
        payload_start,
        end,
    })
}
