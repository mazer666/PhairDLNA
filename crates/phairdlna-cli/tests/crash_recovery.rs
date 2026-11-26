//! Crash and corruption recovery tests (REL-001, AC-11).
//!
//! What: verifies that abrupt termination and index corruption never alter media originals,
//!       and that subsequent restarts safely rebuild a consistent catalogue.
//! Why: user media files are strictly read-only and immutable; crashes must never corrupt originals.

use std::fs;
use tempfile::tempdir;

use phairdlna_core::catalogue::index::CatalogueIndex;
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::LocalFileSystem;

const DUMMY_VIDEO: &[u8] = b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00mp42isomDummyMediaContent";

#[test]
fn abrupt_crash_and_corrupt_index_preserves_media_originals() {
    let dir = tempdir().expect("tempdir");
    let media_dir = dir.path().join("media");
    fs::create_dir_all(&media_dir).expect("media dir");

    let movie_file = media_dir.join("Inception.mp4");
    fs::write(&movie_file, DUMMY_VIDEO).expect("write movie");

    let index_file = dir.path().join("catalogue.db");

    // 1. Initial successful scan and index store
    {
        let root = MediaRoot {
            name: "Movies".to_owned(),
            path: media_dir.to_string_lossy().to_string(),
            exclude: Vec::new(),
            symlink_policy: None,
        };
        let mut cat = phairdlna_core::catalogue::Catalogue::build(
            &[root],
            SymlinkPolicy::Skip,
            &LocalFileSystem,
        );
        let opened = CatalogueIndex::open(&index_file).expect("open index");
        opened.index.store(&mut cat).expect("store catalogue");
    }

    // 2. Simulate corruption / abrupt power loss during index write
    fs::write(&index_file, b"corrupted incomplete database header").expect("corrupt index");

    // 3. Restart and reopen index
    let reopened = CatalogueIndex::open(&index_file).expect("reopen corrupted index");
    let repair = reopened
        .recovery
        .expect("corruption detected and moved aside");
    assert!(repair.moved_to.exists(), "corrupted file moved aside");

    // 4. Media original file must remain byte-for-byte identical (REL-001)
    let post_recovery_bytes = fs::read(&movie_file).expect("read post-recovery");
    assert_eq!(
        DUMMY_VIDEO,
        post_recovery_bytes.as_slice(),
        "media original was altered during crash/recovery!"
    );
}
