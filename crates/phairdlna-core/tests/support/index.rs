//! The fixtures of the tests of the index.
//!
//! What: a catalogue over a folder, a library with films in it, the file of an
//!       index and the ways to look into a catalogue.
//! Why: the tests of the index all need the same handful of things, and a
//!       helper that is written twice is a helper that drifts apart.
//! Example: `catalogue_over` builds the catalogue of a folder, and
//!       `entry_count` counts the entries of a catalogue.

use std::fs;
use std::path::{Path, PathBuf};

use phairdlna_core::catalogue::{Catalogue, ContentItem, ContentSource};
use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
use phairdlna_core::platform::LocalFileSystem;
use phairdlna_core::upnp::Dispatcher;
use tempfile::{tempdir, TempDir};

/// The base URL the dispatcher builds the addresses with.
pub const BASE_URL: &str = "http://192.168.1.20:8200";

/// Builds a catalogue over one root.
pub fn catalogue_over(path: &Path) -> Catalogue {
    let root = MediaRoot {
        name: "Films".to_owned(),
        path: path.to_string_lossy().into_owned(),
        exclude: Vec::new(),
        symlink_policy: None,
    };
    Catalogue::build(&[root], SymlinkPolicy::Skip, &LocalFileSystem)
}

/// A library with the given films in it.
pub fn library_with(films: &[&str]) -> TempDir {
    let directory = tempdir().expect("a temporary directory");
    for film in films {
        fs::write(directory.path().join(film), b"a film").expect("the file is written");
    }
    directory
}

/// The file of an index in a temporary folder.
///
/// The folder is returned as well: it lives as long as the test does, and the
/// file of the index must not vanish under it.
pub fn index_path() -> (TempDir, PathBuf) {
    let directory = tempdir().expect("a temporary directory");
    let path = directory.path().join("catalogue.db");
    (directory, path)
}

/// The number of the entries of a catalogue: the folders and the files.
pub fn entry_count(catalogue: &Catalogue) -> usize {
    let mut total = 0;
    let mut open = vec!["1".to_owned()];
    while let Some(id) = open.pop() {
        let Some(container) = catalogue.container(&id) else {
            continue;
        };
        for child in &container.children {
            total += 1;
            open.push(child.id.clone());
        }
    }
    total
}

/// The answer of a browse of the whole tree, as a client sees it.
pub fn browse_of(catalogue: &Catalogue) -> String {
    let dispatcher = Dispatcher::new(catalogue, BASE_URL);
    let mut answer = String::new();
    for id in ["0", "1"] {
        let body = dispatcher
            .dispatch(&browse_request(id, "BrowseDirectChildren", 0, 0), None)
            .body;
        answer.push_str(&body);
    }
    answer
}

/// The identifier of the container with the given title.
///
/// The content directory holds the root of the library, and the root holds the
/// folders and the playlists.
pub fn container_titled(catalogue: &Catalogue, title: &str) -> String {
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
        .iter()
        .find(|child| child.title == title)
        .expect("the container is in the catalogue")
        .id
        .clone()
}

/// The item of the library with the given title.
///
/// The root of the tests holds one folder, and that folder holds the files.
pub fn item_titled(catalogue: &Catalogue, title: &str) -> ContentItem {
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
        .iter()
        .find(|child| child.title == title)
        .expect("the item is in the catalogue")
        .clone()
}

/// A browse request for the ContentDirectory service.
fn browse_request(object_id: &str, flag: &str, start: u32, count: u32) -> String {
    let body = format!(
        "<?xml version=\"1.0\" encoding=\"utf-8\"?>\n\
         <s:Envelope xmlns:s=\"http://schemas.xmlsoap.org/soap/envelope/\" \
         s:encodingStyle=\"http://schemas.xmlsoap.org/soap/encoding/\">\n\
         \x20 <s:Body>\n\
         \x20   <u:Browse xmlns:u=\"urn:schemas-upnp-org:service:ContentDirectory:1\">\n\
         \x20     <ObjectID>{object_id}</ObjectID>\n\
         \x20     <BrowseFlag>{flag}</BrowseFlag>\n\
         \x20     <Filter>*</Filter>\n\
         \x20     <StartingIndex>{start}</StartingIndex>\n\
         \x20     <RequestedCount>{count}</RequestedCount>\n\
         \x20     <SortCriteria></SortCriteria>\n\
         \x20   </u:Browse>\n\
         \x20 </s:Body>\n\
         </s:Envelope>\n"
    );
    body
}
