//! Native configuration dialog and wizard for macOS.
//!
//! What: interactive native macOS dialogs for first-run setup and configuration
//!       management (`CFG-002`, `UX-004`).
//! Why: provides ordinary users with a seamless, guided macOS experience
//!      without requiring terminal commands or web browser redirection.

use std::path::{Path, PathBuf};

use objc2::MainThreadMarker;
use objc2_app_kit::{NSAlert, NSAlertFirstButtonReturn, NSAlertStyle};
use objc2_foundation::NSString;
use phairdlna_core::config::{Config, MediaRoot};

use super::panels;

/// Result of the first-run wizard dialog.
#[derive(Debug, PartialEq, Eq)]
pub enum WizardResult {
    /// User selected media folders to share.
    Configured(Vec<PathBuf>),
    /// User chose to skip or configure later.
    Skipped,
}

/// Saves picked media folders to the given configuration file.
fn save_roots(config_path: &Path, paths: &[PathBuf]) {
    let Ok(mut config) = Config::load(config_path) else {
        return;
    };
    for path in paths {
        let path_str = path.to_string_lossy().to_string();
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Media")
            .to_owned();
        config.library.roots.push(MediaRoot {
            name,
            path: path_str,
            exclude: Vec::new(),
            symlink_policy: None,
        });
    }
    let _ = config.save(config_path);
}

/// Runs the first-run onboarding wizard if no media roots exist.
pub fn run_wizard_if_needed(
    mtm: MainThreadMarker,
    config_path: &Path,
    title: &str,
    message: &str,
    btn_choose: &str,
    btn_later: &str,
    panel_title: &str,
) -> WizardResult {
    let alert = NSAlert::new(mtm);
    let title_ns = NSString::from_str(title);
    let message_ns = NSString::from_str(message);
    let btn_choose_ns = NSString::from_str(btn_choose);
    let btn_later_ns = NSString::from_str(btn_later);

    alert.setMessageText(&title_ns);
    alert.setInformativeText(&message_ns);
    alert.setAlertStyle(NSAlertStyle::Informational);
    alert.addButtonWithTitle(&btn_choose_ns);
    alert.addButtonWithTitle(&btn_later_ns);

    if alert.runModal() != NSAlertFirstButtonReturn {
        return WizardResult::Skipped;
    }

    let picked = panels::pick_media_folders(mtm, panel_title);
    if picked.is_empty() {
        return WizardResult::Skipped;
    }

    save_roots(config_path, &picked);
    WizardResult::Configured(picked)
}

/// Options presented to the user in the native preferences manager.
#[derive(Debug, PartialEq, Eq)]
pub enum PreferencesChoice {
    /// Add more media folders via NSOpenPanel.
    AddFolders,
    /// Open the existing configuration file in the default editor.
    OpenConfigFile,
    /// Dismiss the preferences dialog.
    Done,
}

/// Shows the native preferences manager dialog.
pub fn show_preferences_dialog(
    mtm: MainThreadMarker,
    title: &str,
    summary: &str,
    btn_add: &str,
    btn_open_file: &str,
    btn_done: &str,
) -> PreferencesChoice {
    let alert = NSAlert::new(mtm);
    let title_ns = NSString::from_str(title);
    let summary_ns = NSString::from_str(summary);
    let btn_add_ns = NSString::from_str(btn_add);
    let btn_open_file_ns = NSString::from_str(btn_open_file);
    let btn_done_ns = NSString::from_str(btn_done);

    alert.setMessageText(&title_ns);
    alert.setInformativeText(&summary_ns);
    alert.setAlertStyle(NSAlertStyle::Informational);
    alert.addButtonWithTitle(&btn_add_ns);
    alert.addButtonWithTitle(&btn_open_file_ns);
    alert.addButtonWithTitle(&btn_done_ns);

    let response = alert.runModal();
    if response == NSAlertFirstButtonReturn {
        PreferencesChoice::AddFolders
    } else if response == NSAlertFirstButtonReturn + 1 {
        PreferencesChoice::OpenConfigFile
    } else {
        PreferencesChoice::Done
    }
}
