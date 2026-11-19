//! Tests for token parsing, secrecy, comparison and private storage.

use super::{constant_time_eq, load_or_create, AdminToken};

/// A token is long enough and made of hexadecimal characters.
#[test]
fn a_token_is_long_and_hexadecimal() {
    let token = AdminToken::generate();
    assert_eq!(token.as_str().len(), 64);
    assert!(token.as_str().chars().all(|c| c.is_ascii_hexdigit()));
}

/// Two tokens are not the same.
#[test]
fn two_tokens_are_not_the_same() {
    assert_ne!(AdminToken::generate(), AdminToken::generate());
}

/// The negative case: a short text is not a token.
#[test]
fn a_short_text_is_not_a_token() {
    assert!(AdminToken::parse("abc").is_err());
}

/// The negative case: a text with letters that are not hexadecimal is refused.
#[test]
fn a_text_that_is_not_hexadecimal_is_refused() {
    let text = "z".repeat(64);
    assert!(AdminToken::parse(&text).is_err());
}

/// The token of the "cross-site request forgery" defence is bound to the token.
#[test]
fn the_csrf_token_belongs_to_the_token() {
    let token = AdminToken::generate();
    let other = AdminToken::generate();

    assert_eq!(token.csrf_token(), token.csrf_token());
    assert_ne!(token.csrf_token(), other.csrf_token());
}

/// The right token is accepted, a wrong one is not.
#[test]
fn the_right_token_is_accepted() {
    let token = AdminToken::generate();
    assert!(token.matches(token.as_str()));
    assert!(!token.matches("0000000000000000"));
    assert!(!token.matches(&format!("{}0", token.as_str())));
}

/// Two texts of different length are not equal.
#[test]
fn texts_of_different_length_are_not_equal() {
    assert!(!constant_time_eq(b"abc", b"abcd"));
    assert!(constant_time_eq(b"abc", b"abc"));
}

/// The token file is written, read back and kept private.
#[test]
fn the_token_file_is_written_and_read_back() {
    let folder = tempfile::tempdir().expect("a temporary directory");
    let path = folder.path().join("admin.token");

    let written = load_or_create(&path).expect("the token is written");
    let read = load_or_create(&path).expect("the token is read back");

    assert_eq!(written, read);
}

/// The negative case: a token file that others may read is refused.
#[cfg(unix)]
#[test]
fn a_token_file_that_others_may_read_is_refused() {
    use std::os::unix::fs::PermissionsExt;

    let folder = tempfile::tempdir().expect("a temporary directory");
    let path = folder.path().join("admin.token");
    load_or_create(&path).expect("the token is written");
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644))
        .expect("the permissions are set");

    assert!(load_or_create(&path).is_err(), "the file is refused");
}
