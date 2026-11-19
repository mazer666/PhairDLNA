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

#[path = "strings/recovery_copy.rs"]
mod recovery_copy;
mod support;

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use phairdlna_mac::text::{Strings, StringsError, SUPPORTED_LOCALES};
use support::locale_contract::{ALLOWED_LITERALS, EXPECTED_KEYS, TECHNICAL_LITERALS};
use support::{collect_unknown_literals, collect_user_texts, is_unknown_literal, string_literals};

/// The tests that read the environment must not run at the same time: the
/// environment belongs to the whole process, not to a test.
static ENVIRONMENT: Mutex<()> = Mutex::new(());

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

/// The first supported macOS preference wins over the process environment.
#[test]
fn the_preferred_language_wins_over_environment() {
    with_preferred_languages(&["de-AT"], "en-US", "de-DE");
}

/// An unsupported preferred language is skipped for the next supported one.
#[test]
fn the_next_supported_preferred_language_is_selected() {
    with_preferred_languages(&["it-IT", "fr-CA"], "en-GB", "fr-FR");
}

/// The process locale is used when no preferred language is supported.
#[test]
fn preferred_languages_fall_back_to_environment() {
    with_preferred_languages(&["it-IT", "ja-JP"], "en-GB", "en-GB");
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

/// Reads preferred languages while controlling the environment fallback.
fn with_preferred_languages(preferences: &[&str], environment: &str, expected: &str) {
    let _guard = ENVIRONMENT
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    std::env::set_var("LC_ALL", environment);
    std::env::set_var("LC_MESSAGES", "");
    std::env::set_var("LANG", "");
    let preferences: Vec<String> = preferences
        .iter()
        .map(|locale| (*locale).to_owned())
        .collect();
    let strings = Strings::for_preferred_locales(&preferences).expect("the fallback is shipped");
    assert_eq!(strings.locale(), expected);
}

/// The path of the resource file of a language.
fn resource_path(locale: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("resources")
        .join(format!("{locale}.strings"))
}
