//! Acceptance criteria for configuration changes from the native application.
//!
//! What: prove that unresolved service settings cannot be changed by app actions.
//! Why: folder selection alone does not establish service access or activation (`OD-14` … `OD-16`).
//! Example: a folder action leaves a valid settings file byte-for-byte unchanged.

use std::path::{Path, PathBuf};

use phairdlna_core::config::Config;
use phairdlna_mac::actions::MenuAction;
use phairdlna_mac::app::{ActionOutcome, App};
use phairdlna_mac::menu::MenuEntry;
use phairdlna_mac::model::PresentationMode;
use phairdlna_mac::preferences::Preferences;
use phairdlna_mac::text::Strings;

fn app(config_path: &Path, preferences_path: &Path) -> App {
    App::with_preferences(
        Strings::load("en-US").expect("the language is shipped"),
        config_path.to_path_buf(),
        "http://127.0.0.1:8210".to_owned(),
        Preferences::at(preferences_path.to_path_buf()),
        PresentationMode::Simple,
    )
}

fn add_folders_enabled(app: &App) -> bool {
    app.entries()
        .into_iter()
        .find_map(|entry| match entry {
            MenuEntry::Action {
                action: MenuAction::AddMediaFolders,
                enabled,
                ..
            } => Some(enabled),
            _ => None,
        })
        .expect("the disabled folder action remains visible")
}

fn paths(folder: &Path) -> (PathBuf, PathBuf) {
    (folder.join("config.toml"), folder.join("preferences.mode"))
}

#[test]
fn folder_changes_are_unavailable_without_valid_configuration() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let (config, preferences) = paths(folder.path());
    let app = app(&config, &preferences);

    assert!(!app.can_add_media_roots());
    assert!(!add_folders_enabled(&app));
}

#[test]
fn folder_changes_are_unavailable_with_malformed_configuration() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let (config, preferences) = paths(folder.path());
    std::fs::write(&config, "not valid TOML = [").expect("write malformed configuration");
    let app = app(&config, &preferences);

    assert!(!app.can_add_media_roots());
    assert!(!add_folders_enabled(&app));
}

#[test]
fn folder_changes_are_unavailable_with_valid_configuration() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let (config, preferences) = paths(folder.path());
    Config::default()
        .save(&config)
        .expect("save a valid configuration");
    let app = app(&config, &preferences);

    assert!(!app.can_add_media_roots());
    assert!(!add_folders_enabled(&app));
}

#[test]
fn a_folder_action_never_requests_a_picker_or_changes_configuration() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let (config, preferences) = paths(folder.path());
    Config::default()
        .save(&config)
        .expect("save a valid configuration");
    let before = std::fs::read(&config).expect("read the initial configuration");
    let mut app = app(&config, &preferences);

    assert_eq!(
        app.perform(MenuAction::AddMediaFolders),
        ActionOutcome::Nothing
    );
    assert_eq!(
        std::fs::read(&config).expect("read the unchanged configuration"),
        before
    );
    assert_eq!(
        app.notice(),
        Some("You can't add folders in the app yet. Ask whoever set up PhairDLNA for help.")
    );
}
