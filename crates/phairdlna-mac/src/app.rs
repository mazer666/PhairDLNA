//! The wiring of the application.
//!
//! What: the part of the application that holds the state and shows it.
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
//! the system — a browser, a file manager — returns a request that the
//! platform carries out, so that the logic of the application can be tested
//! without a window (`MB-004`).

use std::path::PathBuf;
use std::time::{Duration, Instant};

use phairdlna_core::config::Config;

use crate::actions::MenuAction;
use crate::ipc::client::ClientError;
use crate::ipc::snapshots::RescanReport;
use crate::menu::{self, MenuEntry};
use crate::model::{PresentationMode, ServerSnapshot, ServerState};
use crate::preferences::Preferences;
use crate::text::Strings;

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
    /// Open an address in the browser.
    OpenUrl(String),
    /// Open a file or a folder.
    OpenPath(PathBuf),
    /// Open several folders.
    OpenPaths(Vec<PathBuf>),
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
    /// The path to the token file, when available.
    token_file: Option<PathBuf>,
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

/// How long a scan may run before the application stops waiting for it.
///
/// The scan of the library is bounded by the surface; the application does
/// not wait for it for ever, so that the menu never claims that a scan runs
/// when none does (`MB-002`).
const SCAN_DEADLINE: Duration = Duration::from_secs(180);

impl App {
    /// Builds the application.
    ///
    /// The mode of the presentation comes from the preferences of the user,
    /// so that the application shows the view that the user chose the last
    /// time (`UX-015`).
    pub fn new(strings: Strings, config_path: PathBuf, address: String) -> Self {
        let preferences = Preferences::of_the_user();
        let mode = preferences.mode();
        Self::with_preferences_and_token(strings, config_path, address, None, preferences, mode)
    }

    /// Builds the application with a known token file.
    pub fn with_token_file(
        strings: Strings,
        config_path: PathBuf,
        address: String,
        token_file: PathBuf,
    ) -> Self {
        let preferences = Preferences::of_the_user();
        let mode = preferences.mode();
        Self::with_preferences_and_token(
            strings,
            config_path,
            address,
            Some(token_file),
            preferences,
            mode,
        )
    }

    /// Builds the application with known preferences.
    pub fn with_preferences(
        strings: Strings,
        config_path: PathBuf,
        address: String,
        preferences: Preferences,
        mode: PresentationMode,
    ) -> Self {
        Self::with_preferences_and_token(strings, config_path, address, None, preferences, mode)
    }

    /// Builds the application with known preferences and token file.
    pub fn with_preferences_and_token(
        strings: Strings,
        config_path: PathBuf,
        address: String,
        token_file: Option<PathBuf>,
        preferences: Preferences,
        mode: PresentationMode,
    ) -> Self {
        Self {
            strings,
            config_path,
            address,
            token_file,
            preferences,
            mode,
            snapshot: ServerSnapshot::stopped(),
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

    /// Whether a scan is running.
    pub fn is_scanning(&self) -> bool {
        self.scanning
    }

    /// Ends a scan that nobody answered for.
    ///
    /// The answer is `true` when the scan ended: the menu then shows that no
    /// scan runs, and the application does not claim that a scan runs when
    /// none does.
    pub fn end_of_a_silent_scan(&mut self) -> bool {
        let Some(since) = self.scanning_since else {
            return false;
        };
        if since.elapsed() < SCAN_DEADLINE {
            return false;
        }
        self.scanning = false;
        self.scanning_since = None;
        self.snapshot.state = ServerState::Running;
        self.notice = Some(self.strings.text("error.scan_silent").to_owned());
        true
    }

    /// Takes the answer of the surface.
    ///
    /// The answer replaces the last one. The answer is a change when the
    /// state of the server is not the state the application showed.
    pub fn take_state(&mut self, snapshot: ServerSnapshot) -> RefreshOutcome {
        let before = self.snapshot.clone();
        let mut snapshot = snapshot;
        if self.scanning {
            // A scan that runs keeps the state of the menu, so that the user
            // sees that the library is read again.
            snapshot.state = ServerState::Scanning;
        }
        self.snapshot = snapshot;
        if matches!(before.state, ServerState::Stopped | ServerState::Refused)
            && self.snapshot.state == ServerState::Running
        {
            // The message of the failure is not true any more.
            self.notice = None;
        }
        if before == self.snapshot {
            RefreshOutcome::Unchanged
        } else {
            RefreshOutcome::Changed
        }
    }

    /// Takes the failure to reach the surface.
    ///
    /// A surface that refuses the requests of the application is not a
    /// surface that is gone: the two are shown differently, so that the user
    /// knows whether to start the server or to look at the token.
    ///
    /// **Nothing of the last answer stays.** The menu shows the state of the
    /// server and not the state it had: a server that is gone has no folders,
    /// no objects and no warnings (`AC-02`).
    pub fn take_failure(&mut self, error: &ClientError) -> RefreshOutcome {
        let before = self.snapshot.clone();
        self.snapshot = ServerSnapshot {
            state: match error {
                ClientError::Unauthorised | ClientError::Refused { .. } => ServerState::Refused,
                _ => ServerState::Stopped,
            },
            ..ServerSnapshot::default()
        };
        self.scanning = false;
        self.scanning_since = None;
        self.notice = Some(self.message_of(error));
        if before == self.snapshot {
            RefreshOutcome::Unchanged
        } else {
            RefreshOutcome::Changed
        }
    }

    /// Takes the result of a scan of the library.
    pub fn take_rescan(&mut self, report: &RescanReport) -> RefreshOutcome {
        self.scanning = false;
        self.scanning_since = None;
        self.snapshot.catalogue_objects = report.objects;
        let objects = report.objects.to_string();
        self.notice = Some(
            self.strings
                .format("notice.rescan_done", &[("objects", &objects)]),
        );
        RefreshOutcome::Changed
    }

    /// Takes the failure of a scan of the library.
    pub fn take_failed_rescan(&mut self, error: &ClientError) -> RefreshOutcome {
        self.scanning = false;
        self.scanning_since = None;
        self.notice = Some(self.message_of(error));
        RefreshOutcome::Changed
    }

    /// Puts a message of the platform into the menu.
    ///
    /// The platform says things the surface does not know: that a file cannot
    /// be opened, for example.
    pub fn set_notice(&mut self, notice: String) {
        self.notice = Some(notice);
    }

    /// Carries out an action of the menu.
    pub fn perform(&mut self, action: MenuAction) -> ActionOutcome {
        match action {
            MenuAction::OpenAdmin => self.open_admin(),
            MenuAction::OpenMediaFolders => self.open_media_folders(),
            MenuAction::RescanLibrary => {
                if matches!(
                    self.snapshot.state,
                    ServerState::Running | ServerState::Scanning
                ) {
                    self.scanning = true;
                    self.snapshot.state = ServerState::Scanning;
                }
                ActionOutcome::Nothing
            }
            MenuAction::OpenPreferences => {
                ActionOutcome::Request(DesktopRequest::OpenPath(self.config_path.clone()))
            }
            MenuAction::ExportDiagnostics => {
                let diag_url = format!("{}/api/diagnostics", self.address.trim_end_matches('/'));
                ActionOutcome::Request(DesktopRequest::OpenUrl(diag_url))
            }
            MenuAction::TogglePresentation => self.switch_view(),
            MenuAction::Quit => ActionOutcome::Request(DesktopRequest::Quit),
        }
    }

    /// Computes the administration URL, attaching the token parameter if available.
    fn open_admin(&self) -> ActionOutcome {
        let mut url = self.address.clone();
        if let Some(token) = self.token_value() {
            let sep = if url.contains('?') { '&' } else { '?' };
            url.push_str(&format!("{sep}token={token}"));
        }
        ActionOutcome::Request(DesktopRequest::OpenUrl(url))
    }

    /// Reads the token value if a token file is configured.
    fn token_value(&self) -> Option<String> {
        let path = self.token_file.as_deref()?;
        crate::ipc::client::read_token(path).ok()
    }

    /// Switches between the short view and the detailed view.
    ///
    /// The switch changes no setting of the server and starts no work: it
    /// writes a preference of the application and nothing else (`UX-015`,
    /// `MB-004`).
    fn switch_view(&mut self) -> ActionOutcome {
        self.mode = self.mode.other();
        if let Err(error) = self.preferences.set_mode(self.mode) {
            let reason = error.reason.clone();
            self.notice = Some(
                self.strings
                    .format("error.preference_failed", &[("reason", &reason)]),
            );
        }
        ActionOutcome::Nothing
    }

    /// The text that tells the user what went wrong.
    ///
    /// The text names three things: what happened, why it happened and what
    /// the user can do next (`UX-004`). Every text comes from the resource
    /// file of the language of the user (`UX-020`).
    fn message_of(&self, error: &ClientError) -> String {
        match error {
            ClientError::Unreachable { reason } => {
                self.reason_text("error.surface_unreachable", reason)
            }
            ClientError::Unreadable { reason } => {
                self.reason_text("error.surface_unreadable", reason)
            }
            ClientError::Invalid { reason } => self.reason_text("error.answer_invalid", reason),
            ClientError::Unexpected { .. } => {
                self.strings.text("error.answer_unexpected").to_owned()
            }
            ClientError::Unauthorised => self.strings.text("error.token_refused").to_owned(),
            ClientError::Refused { status } => {
                let status = status.to_string();
                self.strings
                    .format("error.request_refused", &[("status", &status)])
            }
            ClientError::Token { .. } => self.strings.text("error.token_unreadable").to_owned(),
            ClientError::NoToken => self.strings.text("error.server_not_started").to_owned(),
            ClientError::Busy => self.strings.text("error.busy").to_owned(),
        }
    }

    /// The text of a failure, with the words of the system.
    ///
    /// The words of the system are only added when they say something: an
    /// empty reason would leave a hole in the sentence of the user.
    fn reason_text(&self, key: &str, reason: &str) -> String {
        if reason.is_empty() {
            return self.strings.text(key).to_owned();
        }
        self.strings.format(key, &[("reason", reason)])
    }

    /// Opens the media folders of the configuration.
    ///
    /// The configuration of the user is read and not the catalogue of the
    /// server: the application never reads the database of the server
    /// (`MB-005`).
    fn open_media_folders(&mut self) -> ActionOutcome {
        match self.media_roots() {
            Ok(paths) if paths.is_empty() => {
                self.notice = Some(self.strings.text("menu.status.no_roots").to_owned());
                ActionOutcome::Nothing
            }
            Ok(paths) => ActionOutcome::Request(DesktopRequest::OpenPaths(paths)),
            Err(reason) => {
                let reason = reason.to_owned();
                self.notice = Some(
                    self.strings
                        .format("error.configuration_unreadable", &[("reason", &reason)]),
                );
                ActionOutcome::Nothing
            }
        }
    }

    /// The paths of the media folders of the configuration.
    fn media_roots(&self) -> Result<Vec<PathBuf>, String> {
        let config = Config::load(&self.config_path).map_err(|error| error.to_string())?;
        Ok(config
            .library
            .roots
            .iter()
            .map(|root| PathBuf::from(&root.path))
            .collect())
    }
}
