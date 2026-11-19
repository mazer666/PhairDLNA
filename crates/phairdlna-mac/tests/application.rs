//! The acceptance criteria of the application.
//!
//! What: the tests that check the wiring — that the answer of the surface
//!       becomes the state of the application, that a failure says what went
//!       wrong, and that the actions of the menu ask the system for the right
//!       thing.
//! Why: the application holds no state of the server of its own; a state
//!      that the application keeps after the server is gone is a lie the user
//!      reads (`MB-001`, `MB-005`).
//! Example: an answer that says that the server runs becomes the state
//!          "running", and a failure of the network becomes the state
//!          "stopped" with a message that says why.

use std::path::PathBuf;

use phairdlna_mac::actions::MenuAction;
use phairdlna_mac::app::{ActionOutcome, App, DesktopRequest};
use phairdlna_mac::ipc::client::ClientError;
use phairdlna_mac::model::{ServerSnapshot, ServerState};
use phairdlna_mac::text::Strings;

/// The texts of the language of the tests.
fn texts() -> Strings {
    Strings::load("en-US").expect("the language is shipped")
}

/// The application of the tests.
fn application() -> App {
    App::new(
        texts(),
        PathBuf::from("/nowhere/config.toml"),
        "http://127.0.0.1:8210".to_owned(),
    )
}

/// The state of a server that runs.
fn running() -> ServerSnapshot {
    ServerSnapshot {
        state: ServerState::Running,
        active_streams: 1,
        catalogue_objects: 10,
        ..ServerSnapshot::default()
    }
}

/// The answer of the surface replaces the state of the application.
#[test]
fn the_answer_of_the_surface_replaces_the_state() {
    let mut app = application();
    assert_eq!(app.snapshot().state, ServerState::Checking);

    let outcome = app.take_state(running());

    assert_eq!(outcome, phairdlna_mac::app::RefreshOutcome::Changed);
    assert_eq!(app.snapshot().state, ServerState::Running);
    assert_eq!(app.snapshot().active_streams, 1);
}

/// The same answer twice changes nothing the second time.
#[test]
fn the_same_answer_twice_changes_nothing() {
    let mut app = application();
    app.take_state(running());

    let outcome = app.take_state(running());

    assert_eq!(outcome, phairdlna_mac::app::RefreshOutcome::Unchanged);
}

/// The negative case: a surface that is not there is a stopped server and a
/// message that says why.
#[test]
fn a_surface_that_is_not_there_is_a_stopped_server() {
    let mut app = application();
    app.take_state(running());

    let outcome = app.take_failure(&ClientError::Unreachable {
        reason: "no route to host".to_owned(),
    });

    assert_eq!(outcome, phairdlna_mac::app::RefreshOutcome::Changed);
    assert_eq!(app.snapshot().state, ServerState::Stopped);
    let lines = lines_of(&app);
    assert!(
        lines
            .iter()
            .any(|line| line.contains("couldn't connect to the server")),
        "the menu must say why: {lines:?}"
    );
}

/// A new failure notice changes the menu even while the state stays stopped.
#[test]
fn a_changed_failure_message_refreshes_the_menu() {
    let mut app = application();
    app.take_failure(&ClientError::Unreachable {
        reason: "no route to host".to_owned(),
    });

    let outcome = app.take_failure(&ClientError::Unreadable {
        reason: "connection reset".to_owned(),
    });
    assert_eq!(outcome, phairdlna_mac::app::RefreshOutcome::Changed);

    let outcome = app.take_failure(&ClientError::Unreadable {
        reason: "connection reset".to_owned(),
    });
    assert_eq!(outcome, phairdlna_mac::app::RefreshOutcome::Unchanged);
}

/// The negative case: a refused request gets plain-language recovery guidance.
#[test]
fn a_surface_that_refuses_is_not_a_surface_that_is_gone() {
    let mut app = application();

    app.take_failure(&ClientError::Refused { status: 403 });

    assert_eq!(app.snapshot().state, ServerState::Refused);
    let lines = lines_of(&app);
    assert!(
        lines
            .iter()
            .any(|line| line.contains("couldn't complete this request")),
        "the menu must explain the refusal in plain language: {lines:?}"
    );
    assert!(
        lines.iter().all(|line| !line.contains("403")),
        "the menu must not expose the HTTP status code: {lines:?}"
    );
}

/// Nothing of the last answer stays when the server is gone.
///
/// The menu shows the state of the server and not the state it had: a server
/// that is gone has no folders, no objects and no warnings (`AC-02`).
#[test]
fn nothing_of_the_last_answer_stays_when_the_server_is_gone() {
    let mut app = application();
    app.take_state(ServerSnapshot {
        state: ServerState::Running,
        catalogue_objects: 20,
        roots: vec![phairdlna_mac::model::RootInfo {
            name: "music".to_owned(),
            mounted: true,
            objects: 20,
        }],
        warnings: vec!["a warning of the server".to_owned()],
        ..ServerSnapshot::default()
    });

    app.take_failure(&ClientError::Unreachable {
        reason: "no route to host".to_owned(),
    });

    assert_eq!(app.snapshot().catalogue_objects, 0);
    assert!(app.snapshot().roots.is_empty());
    assert!(app.snapshot().warnings.is_empty());
}

/// A scan that the worker cannot take ends at once.
///
/// The menu must never claim that a scan runs when none does (`MB-002`).
#[test]
fn a_scan_that_the_worker_cannot_take_ends_at_once() {
    let mut app = application();
    app.take_state(running());
    app.perform(MenuAction::RescanLibrary);
    assert!(app.is_scanning());

    app.take_failed_rescan(&ClientError::Busy);

    assert!(!app.is_scanning());
    let lines = lines_of(&app);
    assert!(
        lines.iter().any(|line| line.contains("busy")),
        "the menu must say why: {lines:?}"
    );
}

/// The scan of the library leaves a message that says what it found.
#[test]
fn a_scan_leaves_a_message() {
    let mut app = application();
    app.take_state(running());

    app.take_rescan(&phairdlna_mac::ipc::snapshots::RescanReport {
        rescanned: true,
        objects: 12,
        duration_seconds: 1,
        message: "ok".to_owned(),
    });

    let lines = lines_of(&app);
    assert!(
        lines.iter().any(|line| line.contains("12")),
        "the message must name the number of the objects: {lines:?}"
    );
}

/// The action that opens the server window opens the native window.
#[test]
fn the_action_that_opens_the_server_window_requests_native_overview() {
    let mut app = application();

    let outcome = app.perform(MenuAction::OpenAdmin);

    assert_eq!(
        outcome,
        ActionOutcome::Request(DesktopRequest::ShowMainWindow)
    );
}

/// The action that opens the configuration asks for the file of the
/// configuration.
#[test]
fn the_action_that_opens_the_configuration_asks_for_the_file() {
    let mut app = application();

    let outcome = app.perform(MenuAction::OpenPreferences);

    assert_eq!(
        outcome,
        ActionOutcome::Request(DesktopRequest::ShowPreferences)
    );
}

/// The positive case: the action that exports diagnostics asks for the diagnostics url.
#[test]
fn the_action_that_exports_diagnostics_asks_for_the_url() {
    let mut app = application();

    let outcome = app.perform(MenuAction::ExportDiagnostics);

    assert_eq!(
        outcome,
        ActionOutcome::Request(DesktopRequest::ExportDiagnostics)
    );
}

/// The negative case: an unreadable config is explained instead of reported as no roots.
#[test]
fn an_unreadable_configuration_shows_a_recovery_message() {
    let mut app = application();

    let outcome = app.perform(MenuAction::OpenMediaFolders);

    assert_eq!(outcome, ActionOutcome::Nothing);
    let lines = lines_of(&app);
    assert!(
        lines
            .iter()
            .any(|line| line.contains("couldn't read its settings")),
        "the menu must explain the config problem: {lines:?}"
    );
}

/// The scan is a command for the worker and not a request of the menu.
#[test]
fn the_scan_is_a_command_for_the_worker() {
    let mut app = application();
    app.take_state(running());

    let outcome = app.perform(MenuAction::RescanLibrary);

    assert_eq!(outcome, ActionOutcome::Nothing);
    assert!(app.is_scanning());
    assert_eq!(app.snapshot().state, ServerState::Scanning);
}

/// The lines of text of the menu of the application.
fn lines_of(app: &App) -> Vec<String> {
    app.entries()
        .into_iter()
        .filter_map(|entry| match entry {
            phairdlna_mac::menu::MenuEntry::Status { title } => Some(title),
            _ => None,
        })
        .collect()
}
