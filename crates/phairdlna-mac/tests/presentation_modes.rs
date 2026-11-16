//! The acceptance criteria of the two views of the menu.
//!
//! What: the tests that check the two presentation modes — that switching the
//!       view changes nothing but the view, that a warning of the server
//!       stays visible in the short view, and that both views read the same
//!       state.
//! Why: the mode is a preference of the view and not a setting of the server;
//!      a mode that changed a setting or interrupted a transfer would make
//!      the server and the application disagree (`UX-015`, `UX-016`,
//!      `MB-001`).
//! Example: a server with a running transfer shows the same transfer before
//!          and after the user switches the view.

use std::path::PathBuf;

use phairdlna_mac::actions::MenuAction;
use phairdlna_mac::app::App;
use phairdlna_mac::menu::MenuEntry;
use phairdlna_mac::model::{PresentationMode, ServerSnapshot, ServerState, TransferInfo};
use phairdlna_mac::preferences::Preferences;
use phairdlna_mac::text::Strings;

/// The texts of the language of the tests.
fn texts() -> Strings {
    Strings::load("en-US").expect("the language is shipped")
}

/// The application of the tests, with its preferences in a known file.
fn application(folder: &std::path::Path) -> App {
    let preferences = Preferences::at(folder.join("presentation.mode"));
    App::with_preferences(
        texts(),
        PathBuf::from("/nowhere/config.toml"),
        "http://127.0.0.1:8210".to_owned(),
        preferences,
        PresentationMode::Simple,
    )
}

/// The state of a server that serves a client.
fn serving() -> ServerSnapshot {
    ServerSnapshot {
        state: ServerState::Running,
        active_streams: 1,
        catalogue_objects: 20,
        transfers: vec![TransferInfo {
            client: "the television".to_owned(),
            title: "a film".to_owned(),
        }],
        ..ServerSnapshot::default()
    }
}

/// The lines of text of the menu.
fn lines_of(app: &App) -> Vec<String> {
    app.entries()
        .into_iter()
        .filter_map(|entry| match entry {
            MenuEntry::Status { title } => Some(title),
            _ => None,
        })
        .collect()
}

/// Switching the view changes the view and nothing else.
#[test]
fn switching_the_view_changes_nothing_but_the_view() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let mut app = application(folder.path());
    app.take_state(serving());

    app.perform(MenuAction::TogglePresentation);

    assert_eq!(app.mode(), PresentationMode::Advanced);
    assert_eq!(app.snapshot(), &serving(), "the state must not change");
    assert_eq!(app.snapshot().active_streams, 1);
    assert_eq!(app.snapshot().transfers.len(), 1);
}

/// The mode is written in the file of the preferences and not in the
/// configuration of the server.
#[test]
fn the_mode_lives_in_the_preferences_of_the_application() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let mut app = application(folder.path());

    app.perform(MenuAction::TogglePresentation);

    let preferences = Preferences::at(folder.path().join("presentation.mode"));
    assert_eq!(preferences.mode(), PresentationMode::Advanced);
    assert!(
        !PathBuf::from("/nowhere/config.toml").exists(),
        "the configuration of the server must not be written"
    );
}

/// A warning of the server stays visible in the short view.
#[test]
fn a_warning_stays_visible_in_the_short_view() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let mut app = application(folder.path());
    let warning = "the firewall may block the connections".to_owned();
    app.take_state(ServerSnapshot {
        state: ServerState::Running,
        warnings: vec![warning.clone()],
        ..serving()
    });

    app.perform(MenuAction::TogglePresentation);
    let detailed = lines_of(&app);
    app.perform(MenuAction::TogglePresentation);
    let short = lines_of(&app);

    assert!(detailed.contains(&warning));
    assert!(short.contains(&warning));
}

/// Both views show the same state of the server.
#[test]
fn both_views_show_the_same_state() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let mut app = application(folder.path());
    app.take_state(serving());

    let short = lines_of(&app);
    app.perform(MenuAction::TogglePresentation);
    let detailed = lines_of(&app);

    assert!(short.iter().any(|line| line.contains("1 active streams")));
    assert!(detailed
        .iter()
        .any(|line| line.contains("1 active streams")));
    assert!(detailed.len() > short.len(), "the detailed view shows more");
}

/// The action of the view says what it does.
#[test]
fn the_action_of_the_view_says_what_it_does() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let mut app = application(folder.path());

    let short = title_of(&app, MenuAction::TogglePresentation);
    app.perform(MenuAction::TogglePresentation);
    let detailed = title_of(&app, MenuAction::TogglePresentation);

    assert_eq!(short, texts().text("menu.action.mode_advanced"));
    assert_eq!(detailed, texts().text("menu.action.mode_simple"));
}

/// The text of an action of the menu.
fn title_of(app: &App, action: MenuAction) -> String {
    app.entries()
        .into_iter()
        .find_map(|entry| match entry {
            MenuEntry::Action {
                title,
                action: chosen,
                ..
            } if chosen == action => Some(title),
            _ => None,
        })
        .expect("the action is in the menu")
}
