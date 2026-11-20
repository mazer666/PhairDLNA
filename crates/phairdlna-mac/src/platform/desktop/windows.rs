//! Native windows and chooser flows for the macOS application.
//!
//! What: show the overview, preferences and native save or alert panels.
//! Why: AppKit presentation stays in the main-thread desktop adapter.
//! Example: unreadable settings appear as recovery guidance in the overview.

use objc2_app_kit::{NSAlertStyle, NSApplication};

use crate::onboarding::OnboardingDecision;
use crate::platform::desktop::Desktop;

impl Desktop {
    /// Opens several folders in the program of the user.
    pub(super) fn open_paths(&mut self, paths: &[std::path::PathBuf]) {
        for path in paths {
            if !crate::platform::workspace::open_path(path) {
                self.app.set_notice(self.app.cannot_open_text());
            }
        }
    }

    /// Saves a diagnostics report through the native save panel.
    pub(super) fn save_diagnostics(&mut self, json_content: &str) {
        let title = self.app.localized_text("panel.save_diagnostics");
        if let Some(target_path) = crate::platform::panels::pick_save_file(
            self.marker,
            &title,
            "phairdlna-diagnostics.json",
        ) {
            match std::fs::write(&target_path, json_content) {
                Ok(()) => {
                    self.app
                        .set_notice(self.app.localized_text("panel.diagnostics_saved"));
                    self.write();
                }
                Err(_) => {
                    let message = self.app.localized_text("error.diagnostics_save_failed");
                    let error_title = self.app.localized_text("panel.diagnostics_error");
                    crate::platform::panels::show_alert(
                        self.marker,
                        &error_title,
                        &message,
                        NSAlertStyle::Warning,
                    );
                }
            }
        }
    }

    /// Shows the native preferences and configuration window.
    pub(super) fn show_preferences(&mut self) {
        if let Some(ref window) = self.prefs_window {
            window.show(self.marker);
            return;
        }
        let window = crate::platform::preferences_window::PreferencesWindow::new(
            self.marker,
            self.app.config_path(),
            self.app.strings(),
            self.item.menu_target(),
        );
        window.update_text_scale(self.app.text_scale());
        window.show(self.marker);
        self.prefs_window = Some(window);
    }

    /// Opens the overview and refreshes it from the current server snapshot.
    pub(super) fn show_main_window(&mut self) {
        if let Some(ref window) = self.main_window {
            window.update(
                self.app.strings(),
                self.app.snapshot(),
                self.app.text_scale(),
                self.app.notice(),
                self.app.can_add_media_roots(),
            );
            window.show(self.marker);
            return;
        }
        let window = crate::platform::main_window::MainWindow::new(
            self.marker,
            self.app.strings(),
            self.app.snapshot(),
            self.item.menu_target(),
            self.app.text_scale(),
            self.app.notice(),
            self.app.can_add_media_roots(),
        );
        window.show(self.marker);
        self.main_window = Some(window);
    }

    /// Opens the folder picker dialog and adds chosen folders to configuration.
    pub(super) fn pick_media_folders(&mut self) {
        let title = self.app.localized_text("panel.select_folders");
        let picked = crate::platform::panels::pick_media_folders(self.marker, &title);
        if picked.is_empty() {
            return;
        }
        match self.app.add_media_roots(&picked) {
            Ok(count) if count > 0 => {
                self.app
                    .set_notice(self.app.localized_text("notice.folders_updated"));
                self.worker.ask(crate::worker::Command::Rescan);
                self.write();
            }
            Ok(_) => {}
            Err(_) => {
                let error_title = self.app.localized_text("panel.folders_error");
                let message = self.app.localized_text("error.configuration_unreadable");
                crate::platform::panels::show_alert(
                    self.marker,
                    &error_title,
                    &message,
                    NSAlertStyle::Warning,
                );
            }
        }
    }

    /// Leaves the application after asking the user for confirmation.
    pub(super) fn quit(&self) {
        let title = self.app.localized_text("dialog.quit.title");
        let message = self.app.localized_text("dialog.quit.message");
        let confirm = self.app.localized_text("dialog.quit.confirm");
        let cancel = self.app.localized_text("dialog.quit.cancel");

        if crate::platform::panels::ask_confirmation(
            self.marker,
            &title,
            &message,
            &confirm,
            &cancel,
        ) {
            NSApplication::sharedApplication(self.marker).terminate(None);
        }
    }

    /// Explains the current configuration state before opening the overview.
    pub(super) fn show_initial_configuration_guidance(&mut self) {
        let decision = OnboardingDecision::from_media_roots(self.app.media_roots());
        if let Some(key) = decision.notice_key() {
            self.app.set_notice(self.app.localized_text(key));
        }
    }
}
