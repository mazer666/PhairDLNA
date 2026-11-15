//! The identifier that changes when the catalogue changes.
//!
//! What: the rule that gives the server a `GetSystemUpdateID` answer that
//!       never goes backwards and changes whenever the catalogue changes.
//! Why: a client that polls the identifier uses it to decide whether it must
//!       browse again. An identifier that goes backwards, or that stays the
//!       same when a file changed, makes the client show a stale library
//!       (`INT-037`).
//! Example: a file that is replaced by a file with other content changes the
//!       identifier of its container and therefore the identifier of the
//!       catalogue.
//!
//! **What is promised, and what is not.** The identifier never decreases while
//! the server runs. It starts again at the first value after a restart,
//! because the counter that keeps it monotonic is not written to the disk —
//! that is the work of the index (`FUN-009`, `ADR-0006`). The requirement
//! `INT-037` asks for a value that is monotonically non-decreasing; the
//! deviation is recorded in `docs/03-features/F-09-catalogue-index.md` § 12.1.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Mutex;

/// The last value the server handed out.
static GENERATION: AtomicU32 = AtomicU32::new(0);

/// The state of the catalogues that were built in this process.
///
/// The key is the library: the roots with their names and paths.
static STATE: Mutex<BTreeMap<String, (u64, u32)>> = Mutex::new(BTreeMap::new());

/// The fingerprint of a catalogue: a number that changes when the catalogue
/// changes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CatalogueFingerprint(u64);

impl CatalogueFingerprint {
    /// Computes the fingerprint of a set of containers.
    pub fn of(containers: &[(String, u32)]) -> Self {
        let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
        for (id, update_id) in containers {
            for byte in id.as_bytes() {
                hash ^= u64::from(*byte);
                hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
            }
            hash ^= u64::from(*update_id);
            hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
        }
        Self(hash)
    }
}

/// The identifier of the catalogue of a library.
///
/// The value never decreases while the server runs, and it changes whenever
/// the catalogue of that library changes.
pub fn update_id_for(library: &str, fingerprint: CatalogueFingerprint) -> u32 {
    let mut state = STATE.lock().unwrap_or_else(|error| error.into_inner());
    match state.get(library) {
        Some((previous, generation)) if *previous == fingerprint.0 => *generation,
        _ => {
            let generation = GENERATION.fetch_add(1, Ordering::SeqCst) + 1;
            state.insert(library.to_owned(), (fingerprint.0, generation));
            generation
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{update_id_for, CatalogueFingerprint};

    /// A fingerprint that does not change keeps the identifier.
    #[test]
    fn an_unchanged_catalogue_keeps_its_identifier() {
        let fingerprint = CatalogueFingerprint::of(&[("1".to_owned(), 7)]);
        let first = update_id_for("library-a", fingerprint);
        let second = update_id_for("library-a", fingerprint);
        assert_eq!(first, second);
    }

    /// A catalogue that changed gets a higher identifier.
    #[test]
    fn a_changed_catalogue_gets_a_higher_identifier() {
        let first = update_id_for(
            "library-b",
            CatalogueFingerprint::of(&[("1".to_owned(), 7)]),
        );
        let second = update_id_for(
            "library-b",
            CatalogueFingerprint::of(&[("1".to_owned(), 8)]),
        );
        assert!(second > first, "{second} must be higher than {first}");
    }

    /// The identifier of one library does not disturb the other.
    #[test]
    fn the_libraries_are_kept_apart() {
        let fingerprint = CatalogueFingerprint::of(&[("1".to_owned(), 7)]);
        let first = update_id_for("library-c", fingerprint);
        update_id_for(
            "library-d",
            CatalogueFingerprint::of(&[("2".to_owned(), 9)]),
        );
        let second = update_id_for("library-c", fingerprint);
        assert_eq!(first, second);
    }
}
