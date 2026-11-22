//! The settings of the server that the overview may change.
//!
//! What: reads the settings of the server, changes them and writes them back.
//! Why: a change of the configuration must be validated before it is written
//!      and must be written atomically, and a file that cannot be read must
//!      never be replaced by a half-written one (`CFG-002`).
//! Example: saving the server name loads the configuration, changes the name
//!          and the port, validates the document and writes it atomically.
//!
//! **The application writes only what it knows.** Every change starts from
//! the file on the disk, so that a setting the application does not show is
//! never lost.

use phairdlna_core::config::{validate, Config};

use crate::app::App;

/// The settings of the server that the overview shows and changes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ServerSettings {
    /// The name a client sees in its device list.
    pub device_name: String,
    /// The TCP port the media server listens on.
    pub http_port: u16,
    /// The folders the server shares.
    pub roots: Vec<ConfiguredRoot>,
}

/// One folder the server shares.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfiguredRoot {
    /// The name a client sees for this folder.
    pub name: String,
    /// The absolute path of the folder.
    pub path: String,
}

/// Why a change of the settings did not reach the file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingsError {
    /// The configuration file cannot be read.
    Unreadable,
    /// The server name is empty.
    EmptyName,
    /// The port is not a number between 1 and 65535.
    InvalidPort,
    /// The document does not pass the validation of the project.
    Invalid,
    /// The configuration file cannot be written.
    Unwritable,
}

impl SettingsError {
    /// The key of the text that explains the failure to the user.
    ///
    /// The reason is a key and not a sentence, because every text the user
    /// reads lives in a resource file of the language of the user (`UX-020`).
    pub fn notice_key(self) -> &'static str {
        match self {
            Self::Unreadable => "error.configuration_unreadable",
            Self::EmptyName => "error.settings_name_invalid",
            Self::InvalidPort => "error.settings_port_invalid",
            Self::Invalid => "error.settings_invalid",
            Self::Unwritable => "error.settings_not_saved",
        }
    }
}

impl App {
    /// The settings of the server, as the configuration file holds them.
    ///
    /// The answer holds empty values when the configuration cannot be read:
    /// the overview then shows that it does not know the values, and the
    /// button that saves them refuses to write a file it could not read.
    pub fn settings(&self) -> ServerSettings {
        let Ok(config) = Config::load(&self.config_path) else {
            return ServerSettings::default();
        };
        ServerSettings {
            device_name: config.server.device_name,
            http_port: config.server.http_port,
            roots: config
                .library
                .roots
                .iter()
                .map(|root| ConfiguredRoot {
                    name: root.name.clone(),
                    path: root.path.clone(),
                })
                .collect(),
        }
    }

    /// Saves the server name and the port the user typed.
    ///
    /// The configuration is loaded, the two values are changed, the document is
    /// validated and only then written — atomically, so that a server that
    /// reads the file while it is written never sees half a document.
    pub fn save_settings(
        &mut self,
        device_name: &str,
        http_port: &str,
    ) -> Result<(), SettingsError> {
        let mut config = Config::load(&self.config_path).map_err(|_| SettingsError::Unreadable)?;
        let name = device_name.trim();
        if name.is_empty() {
            return Err(SettingsError::EmptyName);
        }
        let port = http_port
            .trim()
            .parse::<u16>()
            .map_err(|_| SettingsError::InvalidPort)?;
        if port == 0 {
            return Err(SettingsError::InvalidPort);
        }
        config.server.device_name = name.to_owned();
        config.server.http_port = port;
        validate(&config).map_err(|_| SettingsError::Invalid)?;
        config
            .save(&self.config_path)
            .map_err(|_| SettingsError::Unwritable)?;
        Ok(())
    }
}
