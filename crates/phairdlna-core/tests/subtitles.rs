//! The acceptance criteria of F-11 for subtitles: what the server does with the
//! file that lies next to the film.
//!
//! What: the tests that read a real directory and check that a sidecar is
//!       paired with its film and offered to the client as an additional
//!       resource.
//! Why: a client cannot see a subtitle that merely lies next to the film. It
//!       has to be served as a resource of the object, with its own type and
//!       its own address, or the user gets a silent film (`FUN-016`, `FUN-026`,
//!       `INT-105`).
//! Example: a film with a `.vtt` file next to it carries two resources.

use std::fs;
use std::path::Path;

use phairdlna_core::catalogue::{Catalogue, ContentSource};
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::didl::{item_from_catalogue, ProtocolPolicy};
use phairdlna_core::platform::LocalFileSystem;
use tempfile::tempdir;

/// The bytes of a file the sniffer recognises as an MP4.
const MP4_HEAD: &[u8] = b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00mp42isom";

/// The bytes of a subtitle file.
const SUBTITLE: &[u8] = b"WEBVTT\n\n00:00.000 --> 00:02.000\nA line.\n";

/// The bytes of a subtitle file in the SubRip format.
const SUBRIP: &[u8] = b"1\n00:00,000 --> 00:02,000\nA line.\n";

/// Builds a catalogue over one root.
fn catalogue_over(path: &Path) -> Catalogue {
    let root = MediaRoot {
        name: "Films".to_owned(),
        path: path.to_string_lossy().into_owned(),
        exclude: Vec::new(),
        symlink_policy: None,
    };
    Catalogue::build(&[root], SymlinkPolicy::Skip, &LocalFileSystem)
}

/// Writes a file, creating the folders above it.
fn write_file(path: &Path, content: &[u8]) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("the folder is created");
    }
    fs::write(path, content).expect("the file is written");
}

/// The identifier of the file with the given title.
///
/// The root of the tests holds one folder, and that folder holds the files.
fn id_of(catalogue: &Catalogue, title: &str) -> String {
    let root = catalogue
        .container("1")
        .expect("the content directory is known")
        .children
        .first()
        .expect("the content directory has a root")
        .id
        .clone();
    catalogue
        .container(&root)
        .expect("the root is known")
        .children
        .into_iter()
        .find(|child| child.title == title)
        .expect("the file is in the catalogue")
        .id
}

/// A film with a WebVTT file next to it carries the subtitle.
#[test]
fn a_sidecar_is_paired_with_its_film() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("Nightfall.mkv"), MP4_HEAD);
    write_file(&directory.path().join("Nightfall.vtt"), SUBTITLE);

    let catalogue = catalogue_over(directory.path());
    let id = id_of(&catalogue, "Nightfall.mkv");
    let item = catalogue
        .item_of(&id)
        .expect("the film is in the catalogue");

    let subtitle = item
        .metadata
        .subtitle
        .as_ref()
        .expect("the film has a subtitle");
    assert_eq!(subtitle.mime_type, "text/vtt");
    assert_eq!(subtitle.size, Some(SUBTITLE.len() as u64));
}

/// The subtitle is offered as an additional resource of the object.
#[test]
fn a_film_carries_the_subtitle_as_a_second_resource() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("Nightfall.mkv"), MP4_HEAD);
    write_file(
        &directory.path().join("Nightfall.srt"),
        b"1\n00:00,000 --> 00:02,000\nA line.\n",
    );

    let catalogue = catalogue_over(directory.path());
    let id = id_of(&catalogue, "Nightfall.mkv");
    let item = catalogue
        .item_of(&id)
        .expect("the film is in the catalogue");
    let policy = ProtocolPolicy::plain();

    let mapped = item_from_catalogue(&item, "http://phairdlna.local:8200", &policy);

    assert_eq!(mapped.resources.len(), 2, "the film carries two resources");
    let subtitle = &mapped.resources[1];
    assert_eq!(subtitle.size, Some(SUBRIP.len() as u64));
    assert_eq!(
        subtitle.protocol_info.as_string(),
        "http-get:*:application/x-subrip:*"
    );
    assert!(subtitle
        .uri
        .starts_with("http://phairdlna.local:8200/media/"));
    assert_ne!(subtitle.uri, mapped.resources[0].uri);
}

/// A client that cannot read a subtitle still gets the film.
#[test]
fn a_policy_without_the_capability_carries_one_resource() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("Nightfall.mkv"), MP4_HEAD);
    write_file(&directory.path().join("Nightfall.vtt"), b"WEBVTT\n");

    let catalogue = catalogue_over(directory.path());
    let id = id_of(&catalogue, "Nightfall.mkv");
    let item = catalogue
        .item_of(&id)
        .expect("the film is in the catalogue");
    let policy = ProtocolPolicy::plain().with_subtitles(false);

    let mapped = item_from_catalogue(&item, "http://phairdlna.local:8200", &policy);
    let mut without_sidecar = item.clone();
    without_sidecar.metadata.subtitle = None;
    let plain = item_from_catalogue(&without_sidecar, "http://phairdlna.local:8200", &policy);

    assert_eq!(mapped.resources.len(), 1, "the film carries one resource");
    assert_eq!(plain.resources.len(), 1, "the film carries one resource");
    assert_eq!(mapped.title, plain.title, "the title is the same");
    assert_eq!(mapped.class, plain.class, "the class is the same");
    assert_eq!(mapped.parent_id, plain.parent_id, "the parent is the same");
    assert_eq!(mapped.date, plain.date, "the date is the same");
    assert_eq!(mapped.creator, plain.creator, "the creator is the same");
}

/// A file that is not a subtitle of the film is not paired with it.
#[test]
fn another_file_is_not_a_subtitle() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("Nightfall.mkv"), MP4_HEAD);
    write_file(&directory.path().join("Dawn.vtt"), b"WEBVTT\n");

    let catalogue = catalogue_over(directory.path());
    let id = id_of(&catalogue, "Nightfall.mkv");
    let item = catalogue
        .item_of(&id)
        .expect("the film is in the catalogue");

    assert!(
        item.metadata.subtitle.is_none(),
        "a file with another name is not a subtitle"
    );
}

/// The subtitle is reachable by the identifier the document names.
#[test]
fn the_subtitle_is_reachable_by_its_identifier() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("Nightfall.mkv"), MP4_HEAD);
    write_file(&directory.path().join("Nightfall.vtt"), b"WEBVTT\n");

    let catalogue = catalogue_over(directory.path());
    let id = id_of(&catalogue, "Nightfall.mkv");
    let item = catalogue
        .item_of(&id)
        .expect("the film is in the catalogue");
    let subtitle = item.metadata.subtitle.as_ref().expect("a subtitle");

    let path = catalogue
        .path_of(&subtitle.id)
        .expect("the subtitle is in the catalogue");

    assert!(path.ends_with("Nightfall.vtt"));
}

/// Two sidecars of one film: the one with the higher priority wins.
#[test]
fn the_sidecar_with_the_highest_priority_wins() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("Nightfall.mkv"), MP4_HEAD);
    write_file(&directory.path().join("Nightfall.srt"), b"1\n");
    write_file(&directory.path().join("Nightfall.vtt"), b"WEBVTT\n");

    let catalogue = catalogue_over(directory.path());
    let id = id_of(&catalogue, "Nightfall.mkv");
    let item = catalogue
        .item_of(&id)
        .expect("the film is in the catalogue");

    let subtitle = item.metadata.subtitle.as_ref().expect("a subtitle");
    assert_eq!(subtitle.mime_type, "text/vtt");
}

/// A subtitle that is not UTF-8 states the character set it is written in.
///
/// The file is delivered as it is; the answer tells the client how to read it
/// (`FUN-027`).
#[test]
fn a_latin_1_subtitle_states_its_character_set() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("Nightfall.mkv"), MP4_HEAD);
    write_file(
        &directory.path().join("Nightfall.vtt"),
        b"WEBVTT\n\ncaf\xe9\n",
    );

    let catalogue = catalogue_over(directory.path());
    let id = id_of(&catalogue, "Nightfall.mkv");
    let item = catalogue
        .item_of(&id)
        .expect("the film is in the catalogue");

    let subtitle = item.metadata.subtitle.as_ref().expect("a subtitle");
    assert_eq!(subtitle.charset, "iso-8859-1");
}

/// A subtitle that is UTF-8 states UTF-8.
#[test]
fn a_utf_8_subtitle_states_utf_8() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("Nightfall.mkv"), MP4_HEAD);
    write_file(&directory.path().join("Nightfall.vtt"), SUBTITLE);

    let catalogue = catalogue_over(directory.path());
    let id = id_of(&catalogue, "Nightfall.mkv");
    let item = catalogue
        .item_of(&id)
        .expect("the film is in the catalogue");

    let subtitle = item.metadata.subtitle.as_ref().expect("a subtitle");
    assert_eq!(subtitle.charset, "utf-8");
}

/// A browse of a catalogue that holds a subtitle reads nothing from the disk.
///
/// The pairing of the subtitle with its film happens while the root is read,
/// so a browse opens no additional file and reads no additional folder
/// (`PERF-008`, AC-14).
#[test]
fn a_browse_of_a_subtitle_reads_nothing_from_the_disk() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("Nightfall.mkv"), MP4_HEAD);
    write_file(&directory.path().join("Nightfall.vtt"), SUBTITLE);

    let catalogue = catalogue_over(directory.path());
    fs::remove_dir_all(directory.path()).expect("the library is gone");

    let root = catalogue
        .container("1")
        .expect("the content directory is known")
        .children
        .first()
        .expect("the content directory has a root")
        .id
        .clone();
    let children = catalogue
        .container(&root)
        .expect("the root is known")
        .children;
    assert_eq!(children.len(), 1);
    assert_eq!(children[0].title, "Nightfall.mkv");
    let film = catalogue
        .item_of(&children[0].id)
        .expect("the film is in the catalogue");
    assert!(
        film.metadata.subtitle.is_some(),
        "the subtitle is paired with the film"
    );
}

/// A sidecar in the SMIL format is offered with its own type.
#[test]
fn a_smil_sidecar_is_offered_with_its_own_type() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("Nightfall.mkv"), MP4_HEAD);
    write_file(
        &directory.path().join("Nightfall.smi"),
        b"<smil><body/></smil>",
    );

    let catalogue = catalogue_over(directory.path());
    let id = id_of(&catalogue, "Nightfall.mkv");
    let item = catalogue
        .item_of(&id)
        .expect("the film is in the catalogue");
    let policy = ProtocolPolicy::plain();

    let mapped = item_from_catalogue(&item, "http://phairdlna.local:8200", &policy);

    assert_eq!(mapped.resources.len(), 2, "the film carries two resources");
    assert_eq!(
        mapped.resources[1].protocol_info.as_string(),
        "http-get:*:application/smil+xml:*"
    );
}
