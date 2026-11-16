//! The media URL and the containment of what it names.
//!
//! What: the identifier of a media object as it appears in a URL, the check
//!       that keeps a resolved path inside the configured roots, and the
//!       percent-decoding of a request target.
//! Why: the media URL is the trust boundary. A request may never make the
//!       server read outside a root, follow a link out of them, or expose a
//!       path in an answer (INT-098, SEC-001, SEC-003, SEC-013). The check
//!       happens twice: once on the identifier, which must be an opaque token,
//!       and once on the resolved path, which must be inside a root.
//! Example: `MediaId::from_target` reads `/media/a3f1` into the identifier
//!       `a3f1` and refuses `/media/../etc/passwd`.

use std::path::{Path, PathBuf};

/// The path prefix of the media URLs.
///
/// The URL of a media object is built by the DIDL-Lite module, and the HTTP
/// layer serves exactly that URL (INT-090).
pub const MEDIA_PATH: &str = "/media";

/// The largest number of characters an identifier may have.
const MAX_ID_LENGTH: usize = 256;

/// The identifier of a media object, as it appears in a URL.
///
/// The identifier is opaque: it carries no path and no separator, so a
/// request that tries to walk out of the catalogue never gets the chance
/// (INT-062, INT-098).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MediaId(String);

impl MediaId {
    /// The identifier as a string.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What a request target names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// The target names a media object.
    Media(MediaId),
    /// The target is not a media URL.
    NotMedia,
    /// The target is malformed and cannot be read.
    Malformed,
}

/// Reads the identifier of a media object from a request target.
///
/// The target is the path of the request, with or without a query string. A
/// target that is not a media URL names nothing; a target whose encoding is
/// broken is malformed; a target whose identifier is not an opaque token
/// names nothing, so that a traversal attempt is answered like an unknown
/// object and never reveals the layout of the disk (SEC-013).
pub fn from_target(target: &str) -> Target {
    let path = target.split(['?', '#']).next().unwrap_or_default();
    let Some(rest) = path.strip_prefix(MEDIA_PATH) else {
        return Target::NotMedia;
    };
    let Some(rest) = rest.strip_prefix('/') else {
        return Target::NotMedia;
    };
    if rest.is_empty() {
        return Target::NotMedia;
    }
    let Some(decoded) = percent_decode(rest) else {
        return Target::Malformed;
    };
    if !is_opaque(&decoded) {
        return Target::NotMedia;
    }
    Target::Media(MediaId(decoded))
}

/// Whether an identifier is an opaque token.
///
/// The identifier is produced by the catalogue, so it needs no separator and
/// no character that a URL would have to escape.
fn is_opaque(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_ID_LENGTH
        && value != "."
        && value != ".."
        && value
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// Decodes a percent-encoded path segment.
///
/// A segment that carries an encoding the server cannot read is malformed;
/// the answer is `400`, because the request line itself is broken (INT-094).
///
/// The decoder lives in `security::containment`, where the same decoding is
/// needed for the references that a playlist carries (`SEC-007`).
fn percent_decode(value: &str) -> Option<String> {
    crate::security::containment::percent_decode(value)
}

/// Whether a path is inside one of the roots.
///
/// The comparison is lexical: the path is normalised first, so that a
/// candidate that walks out of a root with `..` does not pass. The caller
/// repeats the check with the canonical path of the file, after the symbolic
/// links have been resolved (INT-098, SEC-003).
///
/// The rule itself lives in `security::containment`, so that the catalogue, the
/// artwork finder and the media layer judge a path the same way (`SEC-001`).
pub fn is_inside_roots(candidate: &Path, roots: &[PathBuf]) -> bool {
    crate::security::containment::is_inside_roots(candidate, roots)
}

#[cfg(test)]
mod tests {
    use super::{from_target, is_inside_roots, MediaId, Target};
    use std::path::PathBuf;

    /// A media URL names its object.
    #[test]
    fn a_media_url_names_its_object() {
        assert_eq!(
            from_target("/media/a3f1"),
            Target::Media(MediaId("a3f1".to_owned()))
        );
    }

    /// A query string does not change the object.
    #[test]
    fn a_query_string_is_ignored() {
        assert_eq!(
            from_target("/media/a3f1?x=1"),
            Target::Media(MediaId("a3f1".to_owned()))
        );
    }

    /// A target that is not a media URL names nothing.
    #[test]
    fn another_target_names_nothing() {
        assert_eq!(from_target("/"), Target::NotMedia);
        assert_eq!(from_target("/art/a3f1"), Target::NotMedia);
        assert_eq!(from_target("/media/"), Target::NotMedia);
        assert_eq!(from_target("/media/a3f1/extra"), Target::NotMedia);
    }

    /// A traversal attempt names nothing, so it is answered like an unknown
    /// object and never reveals the layout of the disk.
    #[test]
    fn a_traversal_attempt_names_nothing() {
        assert_eq!(from_target("/media/../etc/passwd"), Target::NotMedia);
        assert_eq!(from_target("/media/.."), Target::NotMedia);
        assert_eq!(from_target("/media/a%2Fb"), Target::NotMedia);
        assert_eq!(from_target("/media/a%5Cb"), Target::NotMedia);
        assert_eq!(from_target("/media/a%00b"), Target::NotMedia);
    }

    /// A broken encoding is a broken request line.
    #[test]
    fn a_broken_encoding_is_malformed() {
        assert_eq!(from_target("/media/a%zz"), Target::Malformed);
        assert_eq!(from_target("/media/a%2"), Target::Malformed);
    }

    /// A path inside a root is inside it.
    #[test]
    fn a_path_inside_a_root_is_inside() {
        let roots = vec![PathBuf::from("/media/root")];
        assert!(is_inside_roots(
            &PathBuf::from("/media/root/film.mp4"),
            &roots
        ));
    }

    /// A path that walks out of a root with `..` is not inside it.
    #[test]
    fn a_path_that_walks_out_is_not_inside() {
        let roots = vec![PathBuf::from("/media/root")];
        assert!(!is_inside_roots(
            &PathBuf::from("/media/root/../secret"),
            &roots
        ));
        assert!(!is_inside_roots(
            &PathBuf::from("/media/other/film.mp4"),
            &roots
        ));
        assert!(!is_inside_roots(&PathBuf::from("/etc/passwd"), &roots));
    }

    /// A root itself is not inside the root.
    #[test]
    fn the_root_itself_is_not_inside() {
        let roots = vec![PathBuf::from("/media/root")];
        assert!(!is_inside_roots(&PathBuf::from("/media/root"), &roots));
    }

    /// A path that only shares a prefix is not inside the root.
    #[test]
    fn a_shared_prefix_is_not_inside() {
        let roots = vec![PathBuf::from("/media/root")];
        assert!(!is_inside_roots(
            &PathBuf::from("/media/root2/film.mp4"),
            &roots
        ));
    }

    /// A relative path is compared as it is.
    #[test]
    fn a_relative_path_is_compared_as_it_is() {
        let roots = vec![PathBuf::from("media/root")];
        assert!(is_inside_roots(
            &PathBuf::from("media/root/film.mp4"),
            &roots
        ));
        assert!(!is_inside_roots(
            &PathBuf::from("media/root/../secret"),
            &roots
        ));
    }
}
