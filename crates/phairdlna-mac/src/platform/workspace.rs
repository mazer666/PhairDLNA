//! The opening of addresses and of files.
//!
//! What: the two things the application asks the system to show — an address
//!       in the browser and a folder in the file manager.
//! Why: the application must not implement a browser and must not walk the
//!      filesystem of the user; the system does both and does them the way
//!      the user expects (`UX-001`).
//! Example: the action "Open the administration page" opens the address of
//!          the surface in the browser of the user.
//!
//! **The thread.** The workspace of the system belongs to the main thread,
//! and the application asks it from there (`MB-002`).

use std::path::Path;

use objc2_app_kit::NSWorkspace;
use objc2_foundation::{NSString, NSURL};

/// Opens an address in the program of the system that shows addresses.
///
/// The answer is `false` when the system refuses the address or when no
/// program of the system can show it; the application then says so instead
/// of pretending that the address opened.
pub fn open_url(address: &str) -> bool {
    let text = NSString::from_str(address);
    let Some(url) = NSURL::URLWithString(&text) else {
        return false;
    };
    NSWorkspace::sharedWorkspace().openURL(&url)
}

/// Opens a file or a folder with the program of the system.
///
/// The answer is `false` when there is no such file or when the system
/// refuses to open it.
pub fn open_path(path: &Path) -> bool {
    let text = NSString::from_str(&path.to_string_lossy());
    let url = NSURL::fileURLWithPath(&text);
    NSWorkspace::sharedWorkspace().openURL(&url)
}
