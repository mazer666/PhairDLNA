//! Configuration-backed folder actions for the native application.
//!
//! What: read and open the folders shared by the server.
//! Why: the app must not change service configuration before its access and activation model is approved.
//! Example: choosing Open folders reads configured paths without writing settings.

use std::path::{Path, PathBuf};

use phairdlna_core::config::Config;

use crate::app::{ActionOutcome, App, DesktopRequest};

impl App {
    /// Returns the configuration file path.
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Whether app-driven folder changes are approved by the service model.
    pub fn can_add_media_roots(&self) -> bool {
        // The configuration bootstrap, service identity and activation contract remain open.
        false
    }

    /// Returns the configured media folders.
    pub fn media_roots(&self) -> Result<Vec<PathBuf>, String> {
        let config = Config::load(&self.config_path).map_err(|error| error.to_string())?;
        Ok(config
            .library
            .roots
            .iter()
            .map(|root| PathBuf::from(&root.path))
            .collect())
    }

    pub(super) fn open_media_folders(&mut self) -> ActionOutcome {
        match self.media_roots() {
            Ok(paths) if paths.is_empty() => {
                self.notice = Some(self.strings.text("menu.status.no_roots").to_owned());
                ActionOutcome::Nothing
            }
            Ok(paths) => ActionOutcome::Request(DesktopRequest::OpenPaths(paths)),
            Err(_) => {
                self.notice = Some(
                    self.strings
                        .text("error.configuration_unreadable")
                        .to_owned(),
                );
                ActionOutcome::Nothing
            }
        }
    }

    pub(super) fn switch_view(&mut self) -> ActionOutcome {
        self.mode = self.mode.other();
        if self.preferences.set_mode(self.mode).is_err() {
            self.notice = Some(self.strings.text("error.preference_failed").to_owned());
        }
        ActionOutcome::Nothing
    }
}
