//! Configuration-backed folder actions for the native application.
//!
//! What: read and open the folders shared by the server.
//! Why: the app must not change service configuration before its access and activation model is approved.
//! Example: choosing Open folders reads configured paths without writing settings.

use std::path::{Path, PathBuf};

use phairdlna_core::config::{validate, Config};

use crate::app::settings::SettingsError;
use crate::app::{ActionOutcome, App, DesktopRequest};

impl App {
    /// Returns the configuration file path.
    pub fn config_path(&self) -> &Path {
        &self.config_path
    }

    /// Whether app-driven folder changes can be saved.
    pub fn can_add_media_roots(&self) -> bool {
        Config::load(&self.config_path).is_ok()
    }

    /// Adds new media roots and saves the updated configuration.
    pub fn add_media_roots(&mut self, paths: &[PathBuf]) -> Result<usize, String> {
        let mut config = Config::load(&self.config_path).map_err(|e| e.to_string())?;
        let mut added = 0;
        for path in paths {
            let path_str = path.to_string_lossy().to_string();
            if !config.library.roots.iter().any(|r| r.path == path_str) {
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Media")
                    .to_owned();
                config
                    .library
                    .roots
                    .push(phairdlna_core::config::MediaRoot {
                        name,
                        path: path_str,
                        exclude: Vec::new(),
                        symlink_policy: None,
                    });
                added += 1;
            }
        }
        if added > 0 {
            config.save(&self.config_path).map_err(|e| e.to_string())?;
        }
        Ok(added)
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

    /// Removes one shared folder and saves the updated configuration.
    ///
    /// The folder is looked up by its path, because the path is the only part
    /// of a root that identifies it. A path that is not configured changes
    /// nothing and is not an error: the configuration of another program must
    /// not be rewritten because of a stale choice.
    pub fn remove_media_root(&mut self, path: &str) -> Result<bool, SettingsError> {
        let mut config = Config::load(&self.config_path).map_err(|_| SettingsError::Unreadable)?;
        let before = config.library.roots.len();
        config.library.roots.retain(|root| root.path != path);
        if config.library.roots.len() == before {
            return Ok(false);
        }
        validate(&config).map_err(|_| SettingsError::Invalid)?;
        config
            .save(&self.config_path)
            .map_err(|_| SettingsError::Unwritable)?;
        Ok(true)
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
