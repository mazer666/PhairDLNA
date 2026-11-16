//! The acceptance criteria of F-11 for playlists: what the server does with a
//! file the owner wrote by hand.
//!
//! What: the tests that read a real directory and check that a playlist is a
//!       container whose children are the tracks it names, in the order it
//!       names them.
//! Why: a playlist is a file the owner wrote by hand, and every entry in it
//!       is a path the server would follow. A server that follows one of them
//!       without a boundary would publish the whole disk, and a server that
//!       sorts the entries would play the wrong film (`FUN-015`, `SEC-068`).
//! Example: a playlist that names two films produces a container with those
//!       two films, in the order the playlist writes them.

use std::fs;
use std::path::{Path, PathBuf};

use phairdlna_core::catalogue::{Catalogue, ContentSource, MediaKind};
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::didl::folder_from_catalogue;
use phairdlna_core::platform::LocalFileSystem;
use tempfile::tempdir;

/// The bytes of a file the sniffer recognises as an MP4.
const MP4_HEAD: &[u8] = b"\x00\x00\x00\x18ftypmp42\x00\x00\x00\x00mp42isom";

/// Builds a catalogue over one root.
fn catalogue_over(path: &Path) -> Catalogue {
    let root = MediaRoot {
        name: "Library".to_owned(),
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

/// The folder of the root of the catalogue.
///
/// The content directory holds the root of the library; the root holds the
/// files of the library.
fn root_folder(catalogue: &Catalogue) -> phairdlna_core::catalogue::ContentItem {
    catalogue
        .container("1")
        .expect("the content directory is known")
        .children
        .first()
        .cloned()
        .expect("the content directory has a root")
}

/// The child of a container with the given title.
fn child_of(
    catalogue: &Catalogue,
    container_id: &str,
    title: &str,
) -> phairdlna_core::catalogue::ContentItem {
    catalogue
        .container(container_id)
        .expect("the container is known")
        .children
        .into_iter()
        .find(|child| child.title == title)
        .expect("the child is in the catalogue")
}

/// A playlist is a container whose children are the tracks it names.
#[test]
fn a_playlist_is_a_container_with_its_tracks() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("King Kong.mkv"), MP4_HEAD);
    write_file(&directory.path().join("Nightfall.mkv"), MP4_HEAD);
    write_file(
        &directory.path().join("Best of.m3u"),
        b"King Kong.mkv\nNightfall.mkv\n",
    );

    let catalogue = catalogue_over(directory.path());
    let root = root_folder(&catalogue);
    let playlist = child_of(&catalogue, &root.id, "Best of.m3u");

    assert_eq!(playlist.kind, MediaKind::Playlist);
    let container = folder_from_catalogue(&playlist);
    assert_eq!(
        container.class.as_str(),
        "object.container.playlistContainer"
    );
    let children = catalogue
        .container(&playlist.id)
        .expect("the playlist is a container")
        .children
        .iter()
        .map(|child| child.title.clone())
        .collect::<Vec<_>>();
    assert_eq!(children, vec!["King Kong.mkv", "Nightfall.mkv"]);
}

/// The children of a playlist keep the order of the playlist.
#[test]
fn the_children_keep_the_order_of_the_playlist() {
    let directory = tempdir().expect("a temporary directory");
    for name in ["a.mkv", "b.mkv", "c.mkv"] {
        write_file(&directory.path().join(name), MP4_HEAD);
    }
    write_file(
        &directory.path().join("order.m3u"),
        b"c.mkv\na.mkv\nb.mkv\n",
    );

    let catalogue = catalogue_over(directory.path());
    let root = root_folder(&catalogue);
    let playlist = child_of(&catalogue, &root.id, "order.m3u");

    let children = catalogue
        .container(&playlist.id)
        .expect("the playlist is a container")
        .children
        .iter()
        .map(|child| child.title.clone())
        .collect::<Vec<_>>();
    assert_eq!(children, vec!["c.mkv", "a.mkv", "b.mkv"]);
}

/// The three formats of a playlist are read.
#[test]
fn the_three_formats_are_read() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("King Kong.mkv"), MP4_HEAD);
    write_file(&directory.path().join("Nightfall.mkv"), MP4_HEAD);
    write_file(&directory.path().join("one.m3u"), b"King Kong.mkv\n");
    write_file(&directory.path().join("two.m3u8"), b"Nightfall.mkv\n");
    write_file(
        &directory.path().join("three.pls"),
        b"[playlist]\nFile1=King Kong.mkv\nTitle1=A film\nNumberOfEntries=1\n",
    );

    let catalogue = catalogue_over(directory.path());
    let root = root_folder(&catalogue);

    for name in ["one.m3u", "two.m3u8", "three.pls"] {
        let playlist = child_of(&catalogue, &root.id, name);
        assert_eq!(playlist.kind, MediaKind::Playlist, "{name} is a playlist");
        assert_eq!(
            catalogue
                .container(&playlist.id)
                .expect("the playlist is a container")
                .children
                .len(),
            1,
            "{name} names one track"
        );
    }
}

/// An entry that points out of the root is refused, and the rest survives.
#[test]
fn an_entry_outside_the_root_is_refused() {
    let directory = tempdir().expect("a temporary directory");
    let outside = tempdir().expect("another temporary directory");
    write_file(&outside.path().join("secret.mkv"), MP4_HEAD);
    write_file(&directory.path().join("King Kong.mkv"), MP4_HEAD);
    write_file(
        &directory.path().join("mix.m3u"),
        format!(
            "King Kong.mkv\n{}\n",
            outside.path().join("secret.mkv").display()
        )
        .as_bytes(),
    );

    let catalogue = catalogue_over(directory.path());
    let root = root_folder(&catalogue);
    let playlist = child_of(&catalogue, &root.id, "mix.m3u");

    let children = catalogue
        .container(&playlist.id)
        .expect("the playlist is a container")
        .children
        .iter()
        .map(|child| child.title.clone())
        .collect::<Vec<_>>();
    assert_eq!(children, vec!["King Kong.mkv"], "only the film is offered");
    assert!(
        catalogue
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.contains("outside the shared folder")),
        "the refusal is reported: {:?}",
        catalogue.diagnostics()
    );
}

/// A network address in a playlist is refused.
#[test]
fn a_network_address_is_refused() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("King Kong.mkv"), MP4_HEAD);
    write_file(
        &directory.path().join("mix.m3u"),
        b"King Kong.mkv\nhttp://example.invalid/film.mp4\n",
    );

    let catalogue = catalogue_over(directory.path());
    let root = root_folder(&catalogue);
    let playlist = child_of(&catalogue, &root.id, "mix.m3u");

    let children = catalogue
        .container(&playlist.id)
        .expect("the playlist is a container")
        .children
        .iter()
        .map(|child| child.title.clone())
        .collect::<Vec<_>>();
    assert_eq!(children, vec!["King Kong.mkv"], "only the film is offered");
    assert!(
        catalogue
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.contains("network address")),
        "the refusal is reported: {:?}",
        catalogue.diagnostics()
    );
}

/// An entry that names a file that is not there stays in the playlist.
#[test]
fn a_missing_entry_stays_in_the_playlist() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("King Kong.mkv"), MP4_HEAD);
    write_file(
        &directory.path().join("mix.m3u"),
        b"King Kong.mkv\nGhost.mkv\n",
    );

    let catalogue = catalogue_over(directory.path());
    let root = root_folder(&catalogue);
    let playlist = child_of(&catalogue, &root.id, "mix.m3u");

    let children = catalogue
        .container(&playlist.id)
        .expect("the playlist is a container")
        .children;
    assert_eq!(children.len(), 2, "the missing entry is still listed");
    let missing = &children[1];
    assert_eq!(missing.title, "Ghost.mkv");
    assert!(
        missing.metadata.mime_type.is_none(),
        "the missing entry carries no resource"
    );
    assert!(
        catalogue
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.contains("Ghost.mkv")),
        "the diagnostics name the entry: {:?}",
        catalogue.diagnostics()
    );
}

/// A playlist that cannot be read does not take the root down.
#[test]
fn a_broken_playlist_does_not_take_the_root_down() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("King Kong.mkv"), MP4_HEAD);
    write_file(
        &directory.path().join("broken.m3u"),
        b"[playlist]\nFile1=x\n",
    );
    write_file(&directory.path().join("empty.pls"), b"nothing here\n");

    let catalogue = catalogue_over(directory.path());
    let root = root_folder(&catalogue);

    let children = catalogue
        .container(&root.id)
        .expect("the root is known")
        .children
        .iter()
        .map(|child| child.title.clone())
        .collect::<Vec<_>>();
    assert!(
        children.contains(&"King Kong.mkv".to_owned()),
        "the film is still indexed: {children:?}"
    );
    assert!(
        children.contains(&"broken.m3u".to_owned()),
        "the broken playlist is still listed: {children:?}"
    );
    assert!(
        !catalogue.diagnostics().is_empty(),
        "the broken playlist is reported"
    );
}

/// A file that is named twice in one playlist is one object, named twice.
#[test]
fn a_file_named_twice_is_one_object() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("King Kong.mkv"), MP4_HEAD);
    write_file(
        &directory.path().join("mix.m3u"),
        b"King Kong.mkv\nKing Kong.mkv\n",
    );

    let catalogue = catalogue_over(directory.path());
    let root = root_folder(&catalogue);
    let playlist = child_of(&catalogue, &root.id, "mix.m3u");

    let children = catalogue
        .container(&playlist.id)
        .expect("the playlist is a container")
        .children;
    assert_eq!(children.len(), 2, "the film is named twice");
    assert_eq!(children[0].id, children[1].id, "and it is one object");
}

/// The document of a playlist names the container class of the schema.
#[test]
fn the_document_of_a_playlist_names_its_class() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("King Kong.mkv"), MP4_HEAD);
    write_file(&directory.path().join("mix.m3u"), b"King Kong.mkv\n");

    let catalogue = catalogue_over(directory.path());
    let root = root_folder(&catalogue);
    let playlist = child_of(&catalogue, &root.id, "mix.m3u");
    let document = phairdlna_core::didl::render(
        &phairdlna_core::didl::DidlDocument {
            containers: vec![folder_from_catalogue(&playlist)],
            items: Vec::new(),
        },
        None,
    );

    assert!(document.contains("object.container.playlistContainer"));
}

/// The server reads a playlist and a subtitle, and writes nothing.
#[test]
fn the_server_writes_nothing_to_the_library() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("King Kong.mkv"), MP4_HEAD);
    write_file(&directory.path().join("King Kong.vtt"), b"WEBVTT\n");
    write_file(&directory.path().join("mix.m3u"), b"King Kong.mkv\n");
    let before: Vec<(PathBuf, u64)> = ["King Kong.mkv", "King Kong.vtt", "mix.m3u"]
        .iter()
        .map(|name| {
            let path = directory.path().join(name);
            let size = fs::metadata(&path).expect("the file is there").len();
            (path, size)
        })
        .collect();

    let catalogue = catalogue_over(directory.path());

    for (path, size) in &before {
        let after = fs::metadata(path).expect("the file is still there");
        assert_eq!(after.len(), *size, "the file was not written to");
    }
    assert_eq!(
        fs::read_dir(directory.path())
            .expect("the root is readable")
            .count(),
        3,
        "no file was added to the root"
    );
    assert_eq!(catalogue.diagnostics().len(), 0);
}

/// A playlist with more entries than the bound is cut off, and the cut is
/// reported.
#[test]
fn a_playlist_beyond_the_bound_is_cut_off_and_reported() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("King Kong.mkv"), MP4_HEAD);
    let mut playlist = String::new();
    for _ in 0..10_001 {
        playlist.push_str("King Kong.mkv\n");
    }
    write_file(&directory.path().join("long.m3u"), playlist.as_bytes());

    let catalogue = catalogue_over(directory.path());

    assert!(
        catalogue
            .diagnostics()
            .iter()
            .any(|diagnostic| diagnostic.contains("not read entirely")),
        "the truncation is reported: {:?}",
        catalogue.diagnostics()
    );
}

/// An entry that names an absolute path inside the root is accepted.
#[test]
fn an_absolute_entry_inside_the_root_is_accepted() {
    let directory = tempdir().expect("a temporary directory");
    write_file(&directory.path().join("King Kong.mkv"), MP4_HEAD);
    let absolute = directory.path().join("King Kong.mkv");
    write_file(
        &directory.path().join("mix.m3u"),
        format!("{}\n", absolute.display()).as_bytes(),
    );

    let catalogue = catalogue_over(directory.path());
    let root = root_folder(&catalogue);
    let playlist = child_of(&catalogue, &root.id, "mix.m3u");

    let children = catalogue
        .container(&playlist.id)
        .expect("the playlist is a container")
        .children
        .iter()
        .map(|child| child.title.clone())
        .collect::<Vec<_>>();
    assert_eq!(children, vec!["King Kong.mkv"], "the film is offered");
}

/// A link inside the root that points out of it is refused.
#[test]
fn a_link_out_of_the_root_is_refused() {
    let directory = tempdir().expect("a temporary directory");
    let outside = tempdir().expect("another temporary directory");
    write_file(&outside.path().join("secret.mkv"), MP4_HEAD);
    write_file(&directory.path().join("King Kong.mkv"), MP4_HEAD);
    let link = directory.path().join("Link.mkv");
    std::os::unix::fs::symlink(outside.path().join("secret.mkv"), &link)
        .expect("the link is written");
    write_file(
        &directory.path().join("mix.m3u"),
        b"King Kong.mkv\nLink.mkv\n",
    );

    let catalogue = catalogue_over(directory.path());
    let root = root_folder(&catalogue);
    let playlist = child_of(&catalogue, &root.id, "mix.m3u");

    let children = catalogue
        .container(&playlist.id)
        .expect("the playlist is a container")
        .children
        .iter()
        .map(|child| child.title.clone())
        .collect::<Vec<_>>();
    assert_eq!(children, vec!["King Kong.mkv"], "only the film is offered");
}
