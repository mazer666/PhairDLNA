//! The acceptance criteria of the texts of the menu bar application.
//!
//! What: the tests that check the texts the user reads — that every language
//!       carries every text, that a missing key is harmless, that a
//!       placeholder without a value stays visible, that a language that is
//!       not shipped is refused, and that no text of the user interface lives
//!       in the code.
//! Why: a text that is missing is a menu that shows nothing, and a text that
//!      lives in the code is a text that no translator can reach (`UX-020`).
//! Example: the key `menu.status.running` of the language `en-GB` holds
//!          "Server running — {streams} active streams".

mod support;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use phairdlna_mac::text::{Strings, StringsError, SUPPORTED_LOCALES};
use support::{collect_unknown_literals, collect_user_texts, is_unknown_literal, string_literals};

/// The keys the application needs. Every language must carry every one of
/// them, and the test below fails as soon as one is missing.
const EXPECTED_KEYS: &[&str] = &[
    "cli.error.bad_port",
    "cli.error.missing_value",
    "cli.error.no_menu_bar",
    "cli.error.not_macos",
    "cli.error.not_main_thread",
    "cli.error.unknown_option",
    "cli.help.equivalent",
    "cli.help.port",
    "cli.help.token_file",
    "cli.help.usage",
    "error.answer_invalid",
    "error.answer_unexpected",
    "error.busy",
    "error.cannot_open",
    "error.configuration_unreadable",
    "error.preference_failed",
    "error.request_refused",
    "error.scan_silent",
    "error.server_not_started",
    "error.surface_unreachable",
    "error.surface_unreadable",
    "error.token_refused",
    "error.token_unreadable",
    "error.worker_not_started",
    "menu.accessibility.label",
    "menu.accessibility.refused",
    "menu.accessibility.running",
    "menu.accessibility.scanning",
    "menu.accessibility.stopped",
    "menu.action.mode_advanced",
    "menu.action.mode_simple",
    "menu.action.open_admin",
    "menu.action.open_folders",
    "menu.action.add_folders",
    "menu.action.preferences",
    "menu.action.export_diagnostics",
    "menu.action.quit",
    "menu.action.rescan",
    "menu.action.start_server",
    "menu.action.stop_server",
    "menu.status.badge",
    "menu.status.header",
    "menu.status.no_roots",
    "menu.status.refused",
    "menu.status.last_scan",
    "menu.status.objects",
    "menu.status.root",
    "menu.status.root_missing",
    "menu.status.running",
    "menu.status.scanning",
    "menu.status.stopped",
    "menu.status.transfer",
    "menu.status.uptime",
    "menu.tooltip.add_folders",
    "menu.tooltip.export_diagnostics",
    "menu.tooltip.open_folders",
    "menu.tooltip.preferences",
    "menu.tooltip.quit",
    "menu.tooltip.rescan",
    "menu.tooltip.start_server",
    "menu.tooltip.stop_server",
    "notice.folders_updated",
    "notice.rescan_done",
    "notice.rescan_failed",
    "notice.server_control_failed",
    "notice.server_started",
    "notice.server_stopped",
    "panel.folders_error",
    "panel.save_diagnostics",
    "panel.select_folders",
    "prefs.about_desc",
    "prefs.about_license",
    "prefs.about_repo",
    "prefs.about_title",
    "prefs.about_version",
    "prefs.add_folder",
    "prefs.button_add",
    "prefs.button_done",
    "prefs.button_file",
    "prefs.discovery_enabled",
    "prefs.extract_artwork",
    "prefs.http_port",
    "prefs.log_level",
    "prefs.redact_logs",
    "prefs.remove_folder",
    "prefs.roots_edit_hint",
    "prefs.roots_header",
    "prefs.save",
    "prefs.save_failed",
    "prefs.save_success",
    "prefs.server_name",
    "prefs.ssdp_port",
    "prefs.summary",
    "prefs.tab_about",
    "prefs.tab_advanced",
    "prefs.tab_expert",
    "prefs.tab_general",
    "prefs.tab_ultra",
    "prefs.title",
    "tooltip.add_folder",
    "tooltip.artwork",
    "tooltip.discovery",
    "tooltip.editor",
    "tooltip.http_port",
    "tooltip.log_level",
    "tooltip.redact",
    "tooltip.remove_folder",
    "tooltip.roots_list",
    "tooltip.save",
    "tooltip.server_name",
    "tooltip.ssdp_port",
    "wizard.button_choose",
    "wizard.button_later",
    "wizard.message",
    "wizard.title",
];

/// The tests that read the environment must not run at the same time: the
/// environment belongs to the whole process, not to a test.
static ENVIRONMENT: Mutex<()> = Mutex::new(());

/// The literals that are not texts of the user interface.
///
/// The rule of the test below is narrow enough that the literals of the
/// protocol are not texts, but the ones that name a whole header are
/// listed here so that they do not need a new argument every time.
const TECHNICAL_LITERALS: &[&str] = &[
    "Content-Type: application/json",
    "Content-Type: text/plain",
    "Content-Type: text/html",
    "User-Agent",
    "Authorization",
    "Accept",
    "Bearer",
];

/// The literals of the code that are written down.
///
/// Every literal of the production code that holds a space is on this list
/// or on [`TECHNICAL_LITERALS`]. The list holds the texts that the user never
/// reads: the diagnostics of the reader of the answers, the diagnostics of
/// the reader of the resource files and the parts of a request that are
/// built. A new text of the user interface does not belong here: it belongs
/// in a resource file (`UX-020`).
const ALLOWED_LITERALS: &[&str] = &[
    // The diagnostics of the reader of the answers.
    "the answer carries no number",
    "the answer does not begin with a version of the protocol",
    "the answer has no empty line between the headers and the body",
    "the document ends in the middle of a value",
    "the document holds something after byte {position}",
    "the document is not valid json at byte {position}",
    "the document nests deeper than {depth} levels",
    "the document of {length} bytes is too large",
    "the number at byte {position} cannot be read",
    "the text at byte {position} cannot be read",
    // The diagnostics of the reader of the resource files.
    "the key holds a character that is not a letter, a digit, a dot or an underscore",
    "the language {locale} is not supported; the application ships with {}",
    "the line has no `=` between the key and the value",
    "the line has no key in front of the `=`",
    "the line must hold the number: {line}",
    "the value of the key is empty",
    "line {} of the resource file: the key {key} is written twice",
    "line {} of the resource file: {reason}",
    // The parts of a request that are built.
    ": ",
    ", ",
    "{bearer} {token}",
    "{host}: {value}\r\n",
    "{method} {path} {version}\r\n",
    "{version} 200 {ok}\r\n{length}: 2\r\n\r\n{{}}",
    "{} {}",
    // The name of the defence of the surface.
    "cross-site request forgery",
];

#[test]
fn every_language_carries_every_key() {
    let mut expected = EXPECTED_KEYS.to_vec();
    expected.sort_unstable();
    for locale in SUPPORTED_LOCALES {
        let strings = Strings::load(locale).expect("the language is shipped");
        let mut keys = strings.keys();
        keys.sort_unstable();
        assert_eq!(keys, expected, "the language {locale} must carry every key");
    }
}

/// The two languages carry exactly the same set of keys.
#[test]
fn the_languages_carry_the_same_keys() {
    let american = Strings::load("en-US").expect("the language is shipped");
    let british = Strings::load("en-GB").expect("the language is shipped");
    assert_eq!(american.keys(), british.keys());
    assert!(!american.keys().is_empty(), "the languages carry texts");
}

/// Every key of the application has a text in every language.
#[test]
fn every_key_has_a_text() {
    for locale in SUPPORTED_LOCALES {
        let strings = Strings::load(locale).expect("the language is shipped");
        for key in EXPECTED_KEYS {
            assert!(
                !strings.text(key).is_empty(),
                "the language {locale} has no text for the key {key}"
            );
        }
    }
}

/// A key the language does not carry is an empty text, never a panic.
#[test]
fn a_key_the_language_does_not_carry_is_empty() {
    let strings = Strings::load("en-US").expect("the language is shipped");
    assert_eq!(strings.text("menu.does.not.exist"), "");
    assert_eq!(
        strings.format("menu.does.not.exist", &[("streams", "2")]),
        ""
    );
}

/// A placeholder with an argument is filled in.
#[test]
fn a_placeholder_with_an_argument_is_filled_in() {
    let strings = Strings::load("en-GB").expect("the language is shipped");
    let text = strings.format("menu.status.running", &[("streams", "2")]);
    assert!(text.contains('2'), "the text must hold the number: {text}");
    assert!(
        !text.contains("{streams}"),
        "the placeholder must be gone: {text}"
    );
}

/// A placeholder without an argument stays visible.
#[test]
fn a_placeholder_without_an_argument_stays_visible() {
    let strings = Strings::load("en-US").expect("the language is shipped");
    let text = strings.format("menu.status.running", &[]);
    assert!(
        text.contains("{streams}"),
        "the missing value must stay visible: {text}"
    );
}

/// A language the application does not ship is refused.
#[test]
fn a_language_that_is_not_shipped_is_refused() {
    let error: StringsError = Strings::load("it-IT").expect_err("the language is not shipped");
    assert!(error.reason.contains("it-IT"), "{}", error.reason);
    assert!(
        !error.to_string().is_empty(),
        "the error must name the reason"
    );
}

/// The language of the environment is read.
#[test]
fn the_language_of_the_environment_is_read() {
    with_environment(
        &[("LC_ALL", ""), ("LC_MESSAGES", ""), ("LANG", "en_GB.UTF-8")],
        "en-GB",
    );
}

/// The first variable of the environment wins.
#[test]
fn the_first_variable_of_the_environment_wins() {
    with_environment(
        &[
            ("LC_ALL", "en-US"),
            ("LC_MESSAGES", "en-GB"),
            ("LANG", "en-GB"),
        ],
        "en-US",
    );
}

/// A language the application does not ship falls back to `en-US`.
#[test]
fn a_language_that_is_not_shipped_falls_back() {
    with_environment(
        &[("LC_ALL", "it_IT.UTF-8"), ("LC_MESSAGES", ""), ("LANG", "")],
        "en-US",
    );
}

/// The resource files on the disk are well formed and end with a newline.
#[test]
fn the_resource_files_are_well_formed() {
    for locale in SUPPORTED_LOCALES {
        let path = resource_path(locale);
        let source = std::fs::read_to_string(&path).expect("the resource file is read");
        assert!(
            source.ends_with('\n'),
            "the file {path:?} must end with a newline"
        );
        for (number, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let (key, value) = trimmed
                .split_once('=')
                .expect("every entry of a resource file has an equal sign");
            assert!(
                !key.trim().is_empty(),
                "the line {} of {path:?} has no key",
                number + 1
            );
            assert!(
                !value.trim().is_empty(),
                "the line {} of {path:?} has no value",
                number + 1
            );
        }
    }
}

/// No text of the user interface lives in the code.
///
/// The rule is checked twice. The first check looks for a literal that looks
/// like a sentence the user could read. The second check looks for *every*
/// literal that holds a space and asks whether it is on the list of the
/// literals the code is allowed to hold: a text that the user never reads
/// must be written down in [`ALLOWED_LITERALS`], so that a new sentence in
/// the code is a decision and not an accident (`UX-020`).
///
/// The modules of the tests are not part of the code of the application: a
/// test may hold the sentence it checks for.
#[test]
fn no_user_facing_text_lives_in_the_code() {
    let sources = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    collect_user_texts(&sources, TECHNICAL_LITERALS, &mut offenders);
    assert!(
        offenders.is_empty(),
        "these texts of the user interface live in the code and must move into a resource file: {offenders:?}"
    );
}

/// Every literal of the code that holds a space is written down.
///
/// The rule makes the first check above able to fail: a literal that is not
/// on the list is reported, whether it is a sentence of the user or a
/// diagnostic of the code.
#[test]
fn every_literal_of_the_code_is_written_down() {
    let sources = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    collect_unknown_literals(
        &sources,
        TECHNICAL_LITERALS,
        ALLOWED_LITERALS,
        &mut offenders,
    );
    assert!(
        offenders.is_empty(),
        "these literals of the code are not written down in ALLOWED_LITERALS: {offenders:?}"
    );
}

/// The check of the literals is able to fail.
///
/// A gate that has never failed proves nothing; this test is the proof that
/// the check above reports a literal that is not written down.
#[test]
fn the_check_of_the_literals_is_able_to_fail() {
    let source = "fn probe() { let text = \"a sentence of the user\"; }";
    let mut offenders = Vec::new();
    for (_, literal) in string_literals(source) {
        if is_unknown_literal(&literal, TECHNICAL_LITERALS, ALLOWED_LITERALS) {
            offenders.push(literal);
        }
    }
    assert_eq!(offenders, vec!["a sentence of the user".to_owned()]);
}

/// Reads the language of the environment and checks the result.
fn with_environment(variables: &[(&str, &str)], expected: &str) {
    let _guard = ENVIRONMENT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    for (name, value) in variables {
        std::env::set_var(name, value);
    }
    let strings = Strings::for_current_locale().expect("the fallback is shipped");
    assert_eq!(strings.locale(), expected);
}

/// The path of the resource file of a language.
fn resource_path(locale: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join(format!("{locale}.strings"))
}
