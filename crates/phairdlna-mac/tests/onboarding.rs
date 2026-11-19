//! The first-run configuration decisions of the native application.
//!
//! What: check how loaded configuration selects a setup branch.
//! Why: unreadable configuration must never be presented as successful setup.
//! Example: a missing file selects the recovery message, not the folder picker.

use std::path::{Path, PathBuf};

use phairdlna_core::config::{Config, MediaRoot};
use phairdlna_mac::app::App;
use phairdlna_mac::model::PresentationMode;
use phairdlna_mac::onboarding::OnboardingDecision;
use phairdlna_mac::preferences::Preferences;
use phairdlna_mac::text::Strings;

fn decision(config_path: &Path, preferences_path: &Path) -> OnboardingDecision {
    let app = App::with_preferences(
        Strings::load("en-US").expect("the language is shipped"),
        config_path.to_path_buf(),
        "http://127.0.0.1:8210".to_owned(),
        Preferences::at(preferences_path.to_path_buf()),
        PresentationMode::Simple,
    );
    OnboardingDecision::from_media_roots(app.media_roots())
}

fn config_path(folder: &Path) -> (PathBuf, PathBuf) {
    (folder.join("config.toml"), folder.join("preferences.mode"))
}

#[test]
fn missing_configuration_selects_recovery_without_a_picker() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let (config, preferences) = config_path(folder.path());

    assert_eq!(
        decision(&config, &preferences),
        OnboardingDecision::ConfigurationUnavailable
    );
    assert_eq!(
        OnboardingDecision::ConfigurationUnavailable.notice_key(),
        Some("error.configuration_unreadable")
    );
}

#[test]
fn malformed_configuration_selects_recovery_without_a_picker() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let (config, preferences) = config_path(folder.path());
    std::fs::write(&config, "not valid TOML = [").expect("write malformed configuration");

    assert_eq!(
        decision(&config, &preferences),
        OnboardingDecision::ConfigurationUnavailable
    );
    assert_eq!(
        OnboardingDecision::ConfigurationUnavailable.notice_key(),
        Some("error.configuration_unreadable")
    );
}

#[test]
fn valid_configuration_without_roots_selects_the_empty_library_branch() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let (config, preferences) = config_path(folder.path());
    Config::default()
        .save(&config)
        .expect("save valid configuration");

    assert_eq!(
        decision(&config, &preferences),
        OnboardingDecision::ValidConfigurationWithoutRoots
    );
    assert_eq!(
        OnboardingDecision::ValidConfigurationWithoutRoots.notice_key(),
        Some("notice.folder_changes_unavailable")
    );
}

#[test]
fn valid_configuration_with_roots_skips_first_run_setup() {
    let folder = tempfile::tempdir().expect("a temporary folder");
    let (config_path, preferences) = config_path(folder.path());
    let mut config = Config::default();
    config.library.roots.push(MediaRoot {
        name: "Media".to_owned(),
        path: folder.path().to_string_lossy().into_owned(),
        exclude: Vec::new(),
        symlink_policy: None,
    });
    config.save(&config_path).expect("save configured roots");

    assert_eq!(
        decision(&config_path, &preferences),
        OnboardingDecision::ConfiguredWithRoots
    );
    assert_eq!(OnboardingDecision::ConfiguredWithRoots.notice_key(), None);
}
