//! Tests for redaction and pseudonym assignment.

use super::Redactor;
use std::net::IpAddr;
use std::path::Path;

/// A path becomes a pseudonym and keeps it.
#[test]
fn a_path_becomes_a_pseudonym() {
    let redactor = Redactor::new();
    let first = redactor.path(Path::new("/Users/me/Movies/film.mkv"));
    let second = redactor.path(Path::new("/Users/me/Movies/film.mkv"));

    assert_eq!(first, "<path:1>");
    assert_eq!(first, second, "the same path keeps the same pseudonym");
    assert!(!first.contains("Users"));
}

/// Two different paths get two different pseudonyms.
#[test]
fn two_paths_get_two_pseudonyms() {
    let redactor = Redactor::new();
    let first = redactor.path(Path::new("/Users/me/Movies/a.mkv"));
    let second = redactor.path(Path::new("/Users/me/Movies/b.mkv"));

    assert_ne!(first, second);
    assert_eq!(second, "<path:2>");
}

/// An address becomes a pseudonym.
#[test]
fn an_address_becomes_a_pseudonym() {
    let redactor = Redactor::new();
    let address: IpAddr = "192.168.1.20".parse().expect("a valid address");

    assert_eq!(redactor.address(&address), "<address:1>");
}

/// A device identifier becomes a pseudonym.
#[test]
fn a_device_identifier_becomes_a_pseudonym() {
    let redactor = Redactor::new();
    let identifier = "11111111-2222-3333-4444-555555555555";

    assert_eq!(redactor.device(identifier), "<device:1>");
}

/// A value that is nothing private is left alone.
#[test]
fn a_public_value_is_left_alone() {
    let redactor = Redactor::new();

    assert_eq!(redactor.value("8200"), "8200");
    assert_eq!(redactor.value("info"), "info");
    assert_eq!(redactor.value("film.mkv"), "film.mkv");
}

/// A path inside a field is redacted.
#[test]
fn a_path_inside_a_field_is_redacted() {
    let redactor = Redactor::new();
    let fields = redactor.fields(&[("path", "/Users/me/Movies/film.mkv")]);

    assert_eq!(fields[0].1, "<path:1>");
}

/// The address of a client is redacted in a field.
#[test]
fn an_address_inside_a_field_is_redacted() {
    let redactor = Redactor::new();
    let fields = redactor.fields(&[("client", "192.168.1.20")]);

    assert_eq!(fields[0].1, "<address:1>");
}

/// The identifier of the device is redacted in a field.
#[test]
fn a_device_identifier_inside_a_field_is_redacted() {
    let redactor = Redactor::new();
    let fields = redactor.fields(&[("udn", "11111111-2222-3333-4444-555555555555")]);

    assert_eq!(fields[0].1, "<device:1>");
}

/// Every occurrence of a value carries the same pseudonym.
#[test]
fn every_occurrence_carries_the_same_pseudonym() {
    let redactor = Redactor::new();
    let first = redactor.fields(&[("path", "/Users/me/Movies")]);
    let second = redactor.fields(&[("path", "/Users/me/Movies")]);

    assert_eq!(first[0].1, second[0].1);
}

/// An address with a port is an address.
#[test]
fn an_address_with_a_port_is_an_address() {
    let redactor = Redactor::new();

    assert_eq!(redactor.value("192.168.1.20:8200"), "<address:1>");
    assert_eq!(redactor.value("[::1]:8200"), "<address:2>");
}

/// A URL is not a path: the user must see where the server is.
#[test]
fn a_url_is_left_alone() {
    let redactor = Redactor::new();

    assert_eq!(
        redactor.value("http://127.0.0.1:8200"),
        "http://127.0.0.1:8200"
    );
    assert_eq!(
        redactor.value("http://192.168.1.20:8200/description.xml"),
        "http://192.168.1.20:8200/description.xml"
    );
}
