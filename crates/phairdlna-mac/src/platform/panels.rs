//! Native file and folder dialogs for macOS.
//!
//! What: wrappers for `NSOpenPanel` and `NSAlert`.
//! Why: lets users choose media folders through standard native macOS dialogs
//!      without typing raw filesystem paths.

use std::path::PathBuf;

use objc2::MainThreadMarker;
use objc2_app_kit::{NSAlert, NSAlertStyle, NSModalResponseOK, NSOpenPanel, NSSavePanel};
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

/// Prompts the user to select one or more media folders using a native `NSOpenPanel`.
pub fn pick_media_folders(mtm: MainThreadMarker, prompt_title: &str) -> Vec<PathBuf> {
    let panel = NSOpenPanel::openPanel(mtm);
    let title = NSString::from_str(prompt_title);
    panel.setMessage(Some(&title));
    panel.setCanChooseDirectories(true);
    panel.setCanChooseFiles(false);
    panel.setAllowsMultipleSelection(true);
    panel.setResolvesAliases(true);

    let response = panel.runModal();
    if response != NSModalResponseOK {
        return Vec::new();
    }

    let urls = panel.URLs();
    let mut selected = Vec::new();
    for url in urls.iter() {
        if let Some(path_str) = url.path() {
            selected.push(PathBuf::from(path_str.to_string()));
        }
    }
    selected
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
