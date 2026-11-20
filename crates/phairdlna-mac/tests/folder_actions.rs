//! Acceptance criteria for configuration changes from the native application.
//!
//! What: prove that folder actions validate configuration before allowing updates.
//! Why: folder selection can only modify settings when a valid configuration file exists.
//! Example: adding folders appends new media roots and saves valid configuration atomically.

use std::path::{Path, PathBuf};

use phairdlna_core::config::Config;
use phairdlna_mac::actions::MenuAction;
use phairdlna_mac::app::{ActionOutcome, App, DesktopRequest};
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
        .expect("the folder action is present")
}

fn paths(folder: &Path) -> (PathBuf, PathBuf) {
    (folder.join("config.toml"), folder.join("preferences.mode"))
}

#[test]
fn folder_changes_are_unavailable_without_valid_configuration() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let (config, preferences) = paths(folder.path());
    let mut app = app(&config, &preferences);

    assert!(!app.can_add_media_roots());
    assert_eq!(
        app.perform(MenuAction::AddMediaFolders),
        ActionOutcome::Nothing
    );
    assert_eq!(
        app.notice(),
        Some("You can't add folders in the app yet. Ask whoever set up PhairDLNA for help.")
    );
}

#[test]
fn folder_changes_are_unavailable_with_malformed_configuration() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let (config, preferences) = paths(folder.path());
    std::fs::write(&config, "not valid TOML = [").expect("write malformed configuration");
    let mut app = app(&config, &preferences);

    assert!(!app.can_add_media_roots());
    assert_eq!(
        app.perform(MenuAction::AddMediaFolders),
        ActionOutcome::Nothing
    );
}

#[test]
fn folder_changes_are_available_with_valid_configuration() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let (config, preferences) = paths(folder.path());
    Config::default()
        .save(&config)
        .expect("save a valid configuration");
    let mut app = app(&config, &preferences);

    assert!(app.can_add_media_roots());
    assert!(add_folders_enabled(&app));
    assert_eq!(
        app.perform(MenuAction::AddMediaFolders),
        ActionOutcome::Request(DesktopRequest::PickMediaFolders)
    );
}

#[test]
fn adding_media_roots_updates_config_file() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let (config, preferences) = paths(folder.path());
    Config::default()
        .save(&config)
        .expect("save a valid configuration");
    let mut app = app(&config, &preferences);

    let test_dir = folder.path().join("Music");
    std::fs::create_dir(&test_dir).expect("create music directory");

    let count = app
        .add_media_roots(std::slice::from_ref(&test_dir))
        .expect("add media roots successfully");
    assert_eq!(count, 1);

    let reloaded = Config::load(&config).expect("reload configuration");
    assert_eq!(reloaded.library.roots.len(), 1);
    assert_eq!(
        reloaded.library.roots[0].path,
        test_dir.to_string_lossy().to_string()
    );
}
