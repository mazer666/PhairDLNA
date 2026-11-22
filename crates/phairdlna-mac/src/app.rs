//! The wiring of the application.
//!
//! What: the part of the application that holds state and coordinates its views.
//! Why: the menu, the texts and the client are parts that can be tested on
//!      their own; this module is where they meet (`F-15`).
//! Example: a refresh takes the answer of the surface and turns it into a
//!          snapshot.
//!
//! **The application holds no state of its own.** Everything it shows comes
//! from the administration surface, and a failure to reach the surface changes
//! the state to "stopped" instead of keeping an old answer (`MB-001`).
//!
//! **The main thread never waits for the network.** The application asks the
//! [`crate::worker`] for an answer and takes it when it is there; the request
//! itself runs on another thread (`MB-002`).
//!
//! **The actions do not touch the server themselves.** An action that needs
//! the system — for example, the file manager — returns a request that the
//! platform carries out, so that the logic of the application can be tested
//! without a window (`MB-004`).

use std::path::PathBuf;
use std::time::Instant;

mod actions;
mod folders;
mod settings;
mod state;

use crate::menu::{self, MenuEntry};
use crate::model::{PresentationMode, ServerSnapshot};
use crate::preferences::Preferences;
use crate::text::Strings;

pub use settings::{ConfiguredRoot, ServerSettings, SettingsError};

/// The result of a refresh of the state.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RefreshOutcome {
    /// The state did not change.
    Unchanged,
    /// The state changed; the menu must be rebuilt.
    Changed,
}

/// What the application asks the system to do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DesktopRequest {
    /// Open a file or a folder.
    OpenPath(PathBuf),
    /// Open several folders.
    OpenPaths(Vec<PathBuf>),
    /// Open the native preferences dialog.
    ShowPreferences,
    /// Open the native server overview.
    ShowMainWindow,
    /// Export the diagnostics report.
    ExportDiagnostics,
    /// Pick media folders to share.
    PickMediaFolders,
    /// Start the background server.
    StartServer,
    /// Stop the background server.
    StopServer,
    /// Leave the application; the server keeps running.
    Quit,
}

/// The result of an action of the menu.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActionOutcome {
    /// Nothing is to be done outside of the menu.
    Nothing,
    /// The system is asked to do something.
    Request(DesktopRequest),
}

/// The application of the menu bar.
#[derive(Debug)]
pub struct App {
    /// The texts the user reads.
    strings: Strings,
    /// The file of the configuration of the server.
    config_path: PathBuf,
    /// The address of the administration surface.
    address: String,
    /// The preferences of the application.
    preferences: Preferences,
    /// The mode of the presentation.
    mode: PresentationMode,
    /// The last answer of the surface.
    snapshot: ServerSnapshot,
    /// What happened last, as the user reads it.
    notice: Option<String>,
    /// Whether a scan is running.
    scanning: bool,
    /// When the scan began, so that a scan without an answer can end.
    scanning_since: Option<Instant>,
}

impl App {
    /// Builds the application.
    pub fn new(strings: Strings, config_path: PathBuf, address: String) -> Self {
        let preferences = Preferences::of_the_user();
        let mode = preferences.mode();
        Self::with_preferences(strings, config_path, address, preferences, mode)
    }

    /// Builds the application with known preferences.
    pub fn with_preferences(
        strings: Strings,
        config_path: PathBuf,
        address: String,
        preferences: Preferences,
        mode: PresentationMode,
    ) -> Self {
        Self {
            strings,
            config_path,
            address,
            preferences,
            mode,
            snapshot: ServerSnapshot::checking(),
            notice: None,
            scanning: false,
            scanning_since: None,
        }
    }

    /// The mode of the presentation.
    pub fn mode(&self) -> PresentationMode {
        self.mode
    }

    /// The state of the server as the application knows it.
    pub fn snapshot(&self) -> &ServerSnapshot {
        &self.snapshot
    }

    /// The latest user-facing action result, if one is still relevant.
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    /// The address of the administration surface.
    pub fn address(&self) -> &str {
        &self.address
    }

    /// The entries of the menu.
    pub fn entries(&self) -> Vec<MenuEntry> {
        menu::entries(
            &self.snapshot,
            &self.strings,
            self.notice.as_deref(),
            self.mode,
        )
    }

    /// The text of a file or an address that the system refuses to open.
    pub fn cannot_open_text(&self) -> String {
        self.strings.text("error.cannot_open").to_owned()
    }

    /// The loaded strings resources.
    pub fn strings(&self) -> &Strings {
        &self.strings
    }

    /// The text of a localized key.
    pub fn localized_text(&self, key: &str) -> String {
        self.strings.text(key).to_owned()
    }

    /// The short text of the item in the menu bar.
    pub fn badge(&self) -> String {
        self.strings.text("menu.status.badge").to_owned()
    }

    /// The name and the value that the screen reader reads.
    pub fn accessibility(&self) -> (String, String) {
        let label = self.strings.text("menu.accessibility.label").to_owned();
        let value = menu::accessibility_value(&self.snapshot, &self.strings);
        (label, value)
    }

    /// Puts a message of the platform into the menu.
    ///
    /// The platform says things the surface does not know: that a file cannot
    /// be opened, for example.
    pub fn set_notice(&mut self, notice: String) {
        self.notice = Some(notice);
    }
}
