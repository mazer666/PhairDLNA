//! The opening of files and folders.
//!
//! What: opens a selected media folder in the system file manager.
//! Why: the application must not walk the filesystem of the user; macOS does
//!      that in the way the user expects (`UX-001`).
//! Example: "Open media folders" shows each selected folder in Finder.
//!
//! **The thread.** The workspace of the system belongs to the main thread,
//! and the application asks it from there (`MB-002`).

use std::path::Path;

use objc2_app_kit::NSWorkspace;
use objc2_foundation::{NSString, NSURL};

/// Opens a file or a folder with the program of the system.
///
/// The answer is `false` when there is no such file or when the system
/// refuses to open it.
pub fn open_path(path: &Path) -> bool {
    let text = NSString::from_str(&path.to_string_lossy());
    let url = NSURL::fileURLWithPath(&text);
    NSWorkspace::sharedWorkspace().openURL(&url)
}
