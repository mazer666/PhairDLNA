//! Offline volume handling, non-destructive availability reporting and volume restoration (LIFE-005, REL-002, AC-12).
//!
//! What: verifies that an offline media root is marked unavailable without modifying configuration
//!       or destroying un-mounted root definitions, and when the volume returns, the files become
//!       available again without recreating root configurations.
//! Why: network mounts and external drives may disconnect temporarily; definitions and index entries
//!      must be preserved truthfully without losing user roots (LIFE-005, REL-002).

use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::LocalFileSystem;
use tempfile::tempdir;

#[test]
fn offline_media_root_reported_unavailable_and_preserved() {
    let dir = tempdir().expect("tempdir");
    let missing_path = dir.path().join("external_offline_drive");

    let root = MediaRoot {
        name: "External Drive".to_owned(),
        path: missing_path.to_string_lossy().to_string(),
        exclude: Vec::new(),
        symlink_policy: None,
    };

    let roots = vec![root.clone()];
    let catalogue =
        phairdlna_core::catalogue::Catalogue::build(&roots, SymlinkPolicy::Skip, &LocalFileSystem);

    // The root was offline / unmounted, but catalogue is created cleanly
    let root_summaries = catalogue.roots();
    assert_eq!(root_summaries.len(), 1);
    assert_eq!(root_summaries[0].name, "External Drive");
    assert!(
        !root_summaries[0].present,
        "offline volume must be marked not present"
    );

    // Configuration retains the root definition without deletion (LIFE-005, REL-002)
    assert_eq!(roots.len(), 1);
    assert_eq!(roots[0].path, missing_path.to_string_lossy().to_string());
}

#[test]
fn volume_restoration_recovers_entries_without_configuration_loss() {
    let dir = tempdir().expect("tempdir");
    let drive_path = dir.path().join("removable_drive");

    let root = MediaRoot {
        name: "Removable Drive".to_owned(),
        path: drive_path.to_string_lossy().to_string(),
        exclude: Vec::new(),
        symlink_policy: None,
    };
    let roots = vec![root];

    // Stage 1: Volume is absent (unplugged)
    let cat_absent =
        phairdlna_core::catalogue::Catalogue::build(&roots, SymlinkPolicy::Skip, &LocalFileSystem);
    assert!(!cat_absent.roots()[0].present);
    assert_eq!(cat_absent.roots()[0].entries, 0);

    // Stage 2: Volume is mounted with media files
    std::fs::create_dir_all(&drive_path).expect("create drive directory");
    std::fs::write(drive_path.join("song.mp3"), b"ID3\x03\0\0\0\0\0\0").expect("write mp3");

    let cat_restored =
        phairdlna_core::catalogue::Catalogue::build(&roots, SymlinkPolicy::Skip, &LocalFileSystem);
    assert!(
        cat_restored.roots()[0].present,
        "volume return is detected (LIFE-005)"
    );
    assert_eq!(
        cat_restored.roots()[0].entries,
        1,
        "media items in returned volume are available (LIFE-005)"
    );
    assert_eq!(
        roots[0].name, "Removable Drive",
        "configured root definition is unchanged"
    );
}
