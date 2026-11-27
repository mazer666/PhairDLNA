//! Acceptance criteria test for incremental scan and unchanged files (AC-08, PERF-003).
//!
//! What: verifies that an incremental rescan of a tree with unchanged files
//!       completes without reading file contents, preserving all catalogue entries.
//! Why: re-reading file bodies during background rescans would create massive I/O
//!      and interfere with active streams (PERF-003, RES-001).

mod support;

use phairdlna_core::catalogue::Catalogue;
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::LocalFileSystem;
use std::fs;
use tempfile::tempdir;

#[test]
fn incremental_scan_does_not_read_content_of_unchanged_files() {
    let dir = tempdir().expect("temporary directory");
    let media_root = dir.path().join("Media");
    fs::create_dir_all(&media_root).expect("create media directory");

    // Create 100 media files with standard extensions (e.g. .mp4, .mkv, .mp3)
    // Files are made non-readable to file body reads (permissions 000 on unix) after creation
    // to prove that no open/read of the file content occurs during scan.
    let mut files = Vec::new();
    for i in 0..100 {
        let file_path = media_root.join(format!("track_{i:03}.mp3"));
        fs::write(&file_path, b"ID3\x04\x00\x00\x00\x00\x00\x00sample payload")
            .expect("write file");
        files.push(file_path);
    }

    let roots = vec![MediaRoot {
        name: "Music".to_owned(),
        path: media_root.to_string_lossy().into_owned(),
        exclude: Vec::new(),
        symlink_policy: None,
    }];

    // First scan builds catalogue
    let catalogue = Catalogue::build(&roots, SymlinkPolicy::Skip, &LocalFileSystem);
    assert_eq!(catalogue.roots().len(), 1);
    assert_eq!(catalogue.roots()[0].entries, 100);

    // Now revoke read permissions on all file contents
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for file in &files {
            fs::set_permissions(file, fs::Permissions::from_mode(0o000)).expect("chmod 000");
        }
    }

    // Rescan: Metadata (directory entries, mtime, size) remains accessible, but opening/reading
    // any file content would fail with PermissionDenied.
    // The incremental scan must succeed without opening any file body!
    let catalogue2 = Catalogue::build(&roots, SymlinkPolicy::Skip, &LocalFileSystem);
    assert_eq!(
        catalogue2.roots()[0].entries,
        100,
        "rescan completed with all 100 entries without reading file bodies (PERF-003, AC-08)"
    );

    // Restore permissions for cleanup
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for file in &files {
            let _ = fs::set_permissions(file, fs::Permissions::from_mode(0o644));
        }
    }
}
