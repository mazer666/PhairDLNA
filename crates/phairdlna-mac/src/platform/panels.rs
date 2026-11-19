//! Native save and alert dialogs for macOS.
//!
//! What: wrappers for `NSSavePanel` and `NSAlert`.
//! Why: diagnostics export and recovery guidance use familiar macOS panels;
//!      media-folder selection remains unavailable until its access model is approved.

use std::path::PathBuf;

use objc2::MainThreadMarker;
use objc2_app_kit::{NSAlert, NSAlertStyle, NSModalResponseOK, NSSavePanel};
use objc2_foundation::NSString;

/// Prompts the user to save a file with a suggested filename using `NSSavePanel`.
pub fn pick_save_file(
    mtm: MainThreadMarker,
    prompt_title: &str,
    default_name: &str,
) -> Option<PathBuf> {
    let panel = NSSavePanel::savePanel(mtm);
    let title = NSString::from_str(prompt_title);
    let name = NSString::from_str(default_name);
    panel.setMessage(Some(&title));
    panel.setNameFieldStringValue(&name);
    panel.setCanCreateDirectories(true);

    if panel.runModal() != NSModalResponseOK {
        return None;
    }
    panel
        .URL()
        .and_then(|url| url.path())
        .map(|p| PathBuf::from(p.to_string()))
}

/// Shows a native informational or alert dialog.
pub fn show_alert(mtm: MainThreadMarker, title: &str, message: &str, style: NSAlertStyle) {
    let alert = NSAlert::new(mtm);
    let title = NSString::from_str(title);
    let message = NSString::from_str(message);
    alert.setMessageText(&title);
    alert.setInformativeText(&message);
    alert.setAlertStyle(style);
    alert.runModal();
}
