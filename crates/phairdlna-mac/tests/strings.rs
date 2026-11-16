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

use std::path::{Path, PathBuf};
use std::sync::Mutex;

use phairdlna_mac::text::{Strings, StringsError, SUPPORTED_LOCALES};

/// The keys the application needs. Every language must carry every one of
/// them, and the test below fails as soon as one is missing.
const EXPECTED_KEYS: &[&str] = &[
    "cli.error.bad_port",
    "cli.error.missing_value",
    "cli.error.no_menu_bar",
    "cli.error.not_macos",
    "cli.error.token",
    "cli.error.unknown_option",
    "cli.help.equivalent",
    "cli.help.port",
    "cli.help.token_file",
    "cli.help.usage",
    "error.preference_failed",
    "error.server_refused",
    "error.server_unreachable",
    "menu.accessibility.label",
    "menu.accessibility.refused",
    "menu.accessibility.running",
    "menu.accessibility.scanning",
    "menu.accessibility.stopped",
    "menu.action.mode_advanced",
    "menu.action.mode_simple",
    "menu.action.open_admin",
    "menu.action.open_folders",
    "menu.action.preferences",
    "menu.action.quit",
    "menu.action.rescan",
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
    "notice.rescan_done",
    "notice.rescan_failed",
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

/// Every language carries every key of the application.
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
    let error: StringsError = Strings::load("de-DE").expect_err("the language is not shipped");
    assert!(error.reason.contains("de-DE"), "{}", error.reason);
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
        &[("LC_ALL", "de_DE.UTF-8"), ("LC_MESSAGES", ""), ("LANG", "")],
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
/// The rule is checked by reading the sources of the crate and looking for a
/// literal that looks like a sentence the user could read. A literal of the
/// protocol — "application/json", "User-Agent" — is not such a sentence and
/// is not reported.
#[test]
fn no_user_facing_text_lives_in_the_code() {
    let sources = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut offenders = Vec::new();
    collect_user_texts(&sources, &mut offenders);
    assert!(
        offenders.is_empty(),
        "these texts of the user interface live in the code and must move into a resource file: {offenders:?}"
    );
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

/// Collects the literals of the sources that look like a text of the user
/// interface, with the file and the line they stand in.
fn collect_user_texts(directory: &Path, found: &mut Vec<String>) {
    let Ok(entries) = std::fs::read_dir(directory) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_user_texts(&path, found);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (line, literal) in string_literals(&source) {
            if looks_like_a_user_text(&literal) {
                found.push(format!("{}:{}: {literal}", path.display(), line));
            }
        }
    }
}

/// The string literals of a source, with the line they stand in.
///
/// The reader understands the comments, the texts, the raw texts, the
/// characters and the byte characters of the language of Rust: a text that
/// begins with a quote inside a character is not the beginning of a text.
fn string_literals(source: &str) -> Vec<(usize, String)> {
    let mut literals = Vec::new();
    let mut line = 1usize;
    let mut characters = source.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '\n' => line += 1,
            '/' if characters.peek() == Some(&'/') => skip_line_comment(&mut characters, &mut line),
            '/' if characters.peek() == Some(&'*') => {
                skip_block_comment(&mut characters, &mut line)
            }
            '"' => {
                let start_line = line;
                read_text(&mut characters, &mut line, &mut literals, start_line);
            }
            '\'' => skip_character(&mut characters),
            'r' if characters.peek() == Some(&'"') => {
                skip_raw_text(&mut characters, &mut line);
            }
            _ => {}
        }
    }
    literals
}

/// Reads a text and remembers it.
fn read_text(
    characters: &mut std::iter::Peekable<std::str::Chars>,
    line: &mut usize,
    literals: &mut Vec<(usize, String)>,
    start_line: usize,
) {
    let mut text = String::new();
    while let Some(character) = characters.next() {
        match character {
            '"' => {
                literals.push((start_line, text));
                return;
            }
            '\\' => {
                text.push(character);
                if let Some(next) = characters.next() {
                    text.push(next);
                }
            }
            '\n' => {
                *line += 1;
                text.push(character);
            }
            _ => text.push(character),
        }
    }
}

/// Skips a raw text, which ends at the first quote and a number of hashes.
fn skip_raw_text(characters: &mut std::iter::Peekable<std::str::Chars>, line: &mut usize) {
    let mut hashes = 0usize;
    while characters.peek() == Some(&'#') {
        hashes += 1;
        characters.next();
    }
    if characters.next() != Some('"') {
        return;
    }
    for character in characters.by_ref() {
        if character == '\n' {
            *line += 1;
        }
        if character == '"' {
            break;
        }
    }
    let mut closed = 0usize;
    while closed < hashes && characters.peek() == Some(&'#') {
        characters.next();
        closed += 1;
    }
}

/// Skips a character or the beginning of a lifetime.
fn skip_character(characters: &mut std::iter::Peekable<std::str::Chars>) {
    match characters.peek() {
        Some('\\') => {
            characters.next();
            characters.next();
            if characters.peek() == Some(&'\'') {
                characters.next();
            }
        }
        Some(_) => {
            characters.next();
            if characters.peek() == Some(&'\'') {
                characters.next();
            }
        }
        None => {}
    }
}

/// Skips a comment that ends with the line.
fn skip_line_comment(characters: &mut std::iter::Peekable<std::str::Chars>, line: &mut usize) {
    for next in characters.by_ref() {
        if next == '\n' {
            *line += 1;
            break;
        }
    }
}

/// Skips a comment that ends with a star and a slash.
fn skip_block_comment(characters: &mut std::iter::Peekable<std::str::Chars>, line: &mut usize) {
    characters.next();
    let mut previous = '\0';
    for next in characters.by_ref() {
        if next == '\n' {
            *line += 1;
        }
        if previous == '*' && next == '/' {
            return;
        }
        previous = next;
    }
}

/// Whether a literal looks like a text the user reads.
///
/// The rule is deliberately narrow: a literal is a text of the user
/// interface when it holds a space and a word that begins with a capital
/// letter. The literals of the protocol hold no such word and are not texts.
fn looks_like_a_user_text(literal: &str) -> bool {
    if TECHNICAL_LITERALS.contains(&literal) {
        return false;
    }
    if !literal.contains(' ') {
        return false;
    }
    literal
        .split(|character: char| !character.is_alphabetic())
        .any(|word| word.len() >= 2 && word.starts_with(|character: char| character.is_uppercase()))
}
