//! The media of the library, as the delivery layer of the core wants it.
//!
//! What: the bridge between the catalogue of the core and the delivery layer
//!       that answers a media request.
//! Why: the core knows the catalogue and the rules of a media answer; the
//!       daemon knows the file system. Neither of them may decide alone what
//!       a client may read (`SEC-001`).
//! Example: `MediaLibrary::path_of` answers the path of an object, but only
//!       when the path is inside one of the roots.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use phairdlna_core::catalogue::{Catalogue, ContentItem, ContentSource, MediaKind};
use phairdlna_core::http::source::{ContentKind, MediaObject, MediaSource};

/// The media of a catalogue, and the roots they may not leave.
#[derive(Clone)]
pub struct MediaLibrary {
    catalogue: Arc<Catalogue>,
    roots: Vec<PathBuf>,
}

impl MediaLibrary {
    /// The media of a catalogue and the roots it may not leave.
    ///
    /// The roots are resolved before they are kept: a root the user writes as
    /// `/var/folders/…` is the same folder as `/private/var/folders/…` on
    /// macOS, and a comparison of the two spellings would refuse every file
    /// (`SEC-001`). The resolution happens once, when the daemon starts.
    pub fn new(catalogue: Arc<Catalogue>, roots: Vec<PathBuf>) -> Self {
        let roots = roots
            .iter()
            .map(|root| std::fs::canonicalize(root).unwrap_or_else(|_| root.clone()))
            .collect();
        Self { catalogue, roots }
    }

    /// The path of an object, when it is inside one of the roots.
    ///
    /// The path is resolved again before it is returned: a file that was
    /// replaced by a link to somewhere else after the scan must not be read
    /// (`SEC-001`, `INT-098`).
    pub fn path_of(&self, id: &str) -> Option<PathBuf> {
        let path = self.catalogue.path_of(id)?;
        let resolved = std::fs::canonicalize(path).ok()?;
        if self.roots.iter().any(|root| is_inside(&resolved, root)) {
            Some(resolved)
        } else {
            None
        }
    }

    /// The roots the media may not leave.
    pub fn roots(&self) -> &[PathBuf] {
        &self.roots
    }
}

impl MediaSource for MediaLibrary {
    /// The metadata of an object of the catalogue.
    fn metadata(&self, id: &str) -> Option<MediaObject> {
        let item = self.catalogue.as_ref().item_of(id)?;
        Some(media_object(&item))
    }
}

/// Builds the object of a file of the catalogue.
///
/// The object carries what the delivery layer needs: the length, the type and
/// the validator of the file. The validator is the date of the modification,
/// which is what an `If-Range` carries (`INT-093`).
fn media_object(item: &ContentItem) -> MediaObject {
    let mut object = MediaObject::new(
        item.id.clone(),
        item.metadata.size.unwrap_or(0),
        item.metadata.mime_type.clone().unwrap_or_default(),
        content_kind(item.kind),
    );
    object.validator = item.metadata.date.clone();
    object
}

/// The kind of the delivery layer that fits the kind of the catalogue.
fn content_kind(kind: MediaKind) -> ContentKind {
    match kind {
        MediaKind::MusicTrack => ContentKind::Audio,
        MediaKind::Movie | MediaKind::VideoBroadcast => ContentKind::Video,
        MediaKind::Photo => ContentKind::Image,
        MediaKind::Folder | MediaKind::Other => ContentKind::Other,
    }
}

/// Whether a path is inside a root.
///
/// The comparison is the one of the core (`http::path::is_inside`): the parts
/// of the path are compared one by one, so that a folder that only shares a
/// prefix with the root is not inside it.
fn is_inside(candidate: &Path, root: &Path) -> bool {
    candidate != root && candidate.starts_with(root)
}

#[cfg(test)]
mod tests {
    use super::{is_inside, MediaLibrary};
    use phairdlna_core::catalogue::{Catalogue, ContentSource};
    use phairdlna_core::config::{MediaRoot, SymlinkPolicy};
    use phairdlna_core::http::source::MediaSource;
    use phairdlna_core::platform::LocalFileSystem;
    use std::path::PathBuf;
    use std::sync::Arc;

    /// The metadata of a file of the library is answered.
    #[test]
    fn the_metadata_of_a_file_is_answered() {
        let directory = tempfile::tempdir().expect("a temporary directory");
        std::fs::write(directory.path().join("film.mkv"), b"a film").expect("the film is written");
        let catalogue = catalogue_of(directory.path());
        let library = MediaLibrary::new(Arc::new(catalogue), vec![directory.path().to_path_buf()]);
        let id = identifier_of(&library, "film.mkv");

        let object = library.metadata(&id).expect("the film is in the catalogue");

        assert_eq!(object.mime_type, "video/x-matroska");
        assert_eq!(object.length, 6);
    }

    /// A file outside the root is not opened.
    #[test]
    fn a_file_outside_the_root_is_not_opened() {
        let directory = tempfile::tempdir().expect("a temporary directory");
        let outside = tempfile::tempdir().expect("another temporary directory");
        std::fs::write(outside.path().join("secret.mkv"), b"a secret")
            .expect("the file is written");
        let catalogue = catalogue_of(outside.path());
        let library = MediaLibrary::new(Arc::new(catalogue), vec![directory.path().to_path_buf()]);
        let id = identifier_of(&library, "secret.mkv");

        assert!(
            library.path_of(&id).is_none(),
            "a file outside the root is not answered"
        );
    }

    /// A folder that shares a prefix with the root is not inside it.
    #[test]
    fn a_shared_prefix_is_not_inside() {
        assert!(
            !is_inside(
                &PathBuf::from("/media/library-evil"),
                &PathBuf::from("/media/library")
            ),
            "a sibling with a shared prefix is not inside the root"
        );
        assert!(
            is_inside(
                &PathBuf::from("/media/library/film.mkv"),
                &PathBuf::from("/media/library")
            ),
            "a file of the root is inside it"
        );
    }

    /// Builds the catalogue of a folder.
    fn catalogue_of(directory: &std::path::Path) -> Catalogue {
        let root = MediaRoot {
            name: "Films".to_owned(),
            path: directory.to_string_lossy().into_owned(),
            exclude: Vec::new(),
            symlink_policy: None,
        };
        Catalogue::build(&[root], SymlinkPolicy::Skip, &LocalFileSystem)
    }

    /// The identifier of a file of the catalogue.
    ///
    /// The catalogue has two levels: the root, and the folder the user
    /// shared.
    fn identifier_of(library: &MediaLibrary, title: &str) -> String {
        let container = library
            .catalogue
            .container("1")
            .expect("the root is in the catalogue");
        for child in &container.children {
            if child.title == title {
                return child.id.clone();
            }
            if let Some(id) = find_in_folder(library, &child.id, title) {
                return id;
            }
        }
        panic!("the file is not in the catalogue");
    }

    fn find_in_folder(library: &MediaLibrary, folder_id: &str, title: &str) -> Option<String> {
        let folder = library.catalogue.container(folder_id)?;
        for item in &folder.children {
            if item.title == title {
                return Some(item.id.clone());
            }
        }
        None
    }
}
