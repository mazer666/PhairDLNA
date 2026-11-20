//! Native save and alert dialogs for macOS.
//!
//! What: wrappers for `NSSavePanel` and `NSAlert`.
//! Why: diagnostics export and recovery guidance use familiar macOS panels;
//!      media-folder selection remains unavailable until its access model is approved.

use std::path::PathBuf;

use objc2::MainThreadMarker;
use objc2_app_kit::{NSAlert, NSAlertStyle, NSModalResponseOK, NSOpenPanel, NSSavePanel};
use objc2_foundation::NSString;

/// Prompts the user to select one or more media folders using a native `NSOpenPanel`.
pub fn pick_media_folders(mtm: MainThreadMarker, prompt_title: &str) -> Vec<PathBuf> {
    let panel = NSOpenPanel::openPanel(mtm);
    let title = NSString::from_str(prompt_title);
    panel.setMessage(Some(&title));
    panel.setCanChooseDirectories(true);
    panel.setCanChooseFiles(false);
    panel.setAllowsMultipleSelection(true);
    panel.setResolvesAliases(true);

    if panel.runModal() != NSModalResponseOK {
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

/// Shows a native confirmation dialog with confirm and cancel buttons.
///
/// Returns `true` if the user clicked the confirm button (first button).
pub fn ask_confirmation(
    mtm: MainThreadMarker,
    title: &str,
    message: &str,
    confirm_label: &str,
    cancel_label: &str,
) -> bool {
    let alert = NSAlert::new(mtm);
    let title_str = NSString::from_str(title);
    let message_str = NSString::from_str(message);
    let confirm_str = NSString::from_str(confirm_label);
    let cancel_str = NSString::from_str(cancel_label);

    alert.setMessageText(&title_str);
    alert.setInformativeText(&message_str);
    alert.setAlertStyle(NSAlertStyle::Warning);
    alert.addButtonWithTitle(&confirm_str);
    alert.addButtonWithTitle(&cancel_str);

    // NSAlertFirstButtonReturn in AppKit is 1000
    alert.runModal() == 1000
}
