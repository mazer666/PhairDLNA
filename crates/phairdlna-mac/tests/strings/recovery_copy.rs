//! Recovery copy must describe only setup paths the app actually supports.
//!
//! What: check empty-library, missing-config and server-stopped guidance.
//! Why: users must not be sent to a nonexistent installer or app menu.
//! Example: a missing server response tells the user who can help start it.

use phairdlna_mac::text::{Strings, SUPPORTED_LOCALES};

/// Recovery copy does not promise setup or lifecycle controls that the app lacks.
#[test]
fn recovery_copy_does_not_assume_an_installer_or_app_controls() {
    let outdated_references = [
        "installer",
        "installationshinweise",
        "installateur",
        "instrucciones de instalación",
        "whoever installed",
        "die phairdlna installiert hat",
        "quien instaló",
        "qui a installé phairdlna",
    ];
    let unavailable_controls = ["from its menu", "desde el menú de la aplicación"];
    let keys = [
        "notice.folder_changes_unavailable",
        "menu.status.server_controls_unavailable",
        "menu.tooltip.add_folders",
        "error.server_not_started",
        "error.configuration_unreadable",
    ];
    for locale in SUPPORTED_LOCALES {
        let strings = Strings::load(locale).expect("the language is shipped");
        for key in keys {
            let message = strings.text(key);
            let lowercase = message.to_lowercase();
            assert!(lowercase.contains("phairdlna"), "{locale} {key}: {message}");
            for outdated in outdated_references {
                assert!(!lowercase.contains(outdated), "{locale} {key}: {message}");
            }
            for unavailable in unavailable_controls {
                assert!(
                    !lowercase.contains(unavailable),
                    "{locale} {key}: {message}"
                );
            }
        }
    }
}

/// The setup command is recommended only when the settings file is missing.
#[test]
fn missing_and_invalid_settings_have_different_recovery_steps() {
    let recovery_clauses = [
        (
            "de-DE",
            "Wenn die Einstellungsdatei fehlt",
            "Wenn die Datei bereits vorhanden ist",
            "phairdlna-cli setup",
        ),
        (
            "en-US",
            "If the settings file is missing",
            "If the file already exists",
            "phairdlna-cli setup",
        ),
        (
            "en-GB",
            "If the settings file is missing",
            "If the file already exists",
            "phairdlna-cli setup",
        ),
        (
            "es-ES",
            "Si falta el archivo de ajustes",
            "Si el archivo ya existe",
            "phairdlna-cli setup",
        ),
        (
            "fr-FR",
            "Si le fichier de réglages est absent",
            "Si le fichier existe déjà",
            "phairdlna-cli setup",
        ),
    ];
    for (locale, missing_clause, existing_clause, command) in recovery_clauses {
        let strings = Strings::load(locale).expect("the language is shipped");
        let recovery = strings.text("error.configuration_unreadable");
        assert!(recovery.contains(missing_clause), "{locale}: {recovery}");
        assert!(recovery.contains(existing_clause), "{locale}: {recovery}");
        assert!(recovery.contains(command), "{locale}: {recovery}");
        assert!(
            recovery.find(missing_clause) < recovery.find(command),
            "{locale}: the CLI command must follow the missing-file condition"
        );
        assert!(
            recovery.find(command) < recovery.find(existing_clause),
            "{locale}: existing files must receive a different recovery step"
        );
    }
}
