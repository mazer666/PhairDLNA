//! The containment of a path inside the roots of the library.
//!
//! What: the lexical normalisation of a path, the check that keeps a path
//!       inside one of the roots, and the resolution of a reference that a file
//!       of the library carries — a playlist entry, for example.
//! Why: a playlist is a file the owner wrote by hand, and its entries are paths
//!       the server would follow. A server that follows one of them without a
//!       boundary would publish the whole disk (`SEC-001`, `SEC-003`, `SEC-068`,
//!       `FUN-015`). The rule lives here so that the catalogue, the media layer
//!       and the artwork finder all judge a path the same way.
//! Example:
//!
//! ```rust
//! use phairdlna_core::security::containment::{resolve_reference, ContainmentError};
//! use std::path::{Path, PathBuf};
//!
//! let roots = vec![PathBuf::from("/media/root")];
//! let base = Path::new("/media/root/playlists");
//!
//! assert!(resolve_reference("film.mkv", base, &roots).is_ok());
//! assert_eq!(
//!     resolve_reference("http://example.invalid/x.mp4", base, &roots),
//!     Err(ContainmentError::NetworkAddress)
//! );
//! ```

use std::path::{Component, Path, PathBuf};

/// Why a reference could not be used.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ContainmentError {
    /// The reference carries nothing to resolve.
    Empty,
    /// The reference names a network address.
    ///
    /// The server never makes a request for a playlist entry (`SEC-091`).
    NetworkAddress,
    /// The reference names a scheme the server does not read.
    UnsupportedScheme(String),
    /// The reference points outside every root.
    OutsideRoots,
}

/// The schemes that name a network address.
///
/// A playlist may carry such an address, and the server refuses it outright:
/// following it would make the server fetch from the network on behalf of a
/// file the client asked for (`SEC-068`, `SEC-091`).
const NETWORK_SCHEMES: [&str; 10] = [
    "http", "https", "ftp", "ftps", "rtsp", "rtmp", "smb", "mms", "sftp", "nfs",
];

/// The scheme of a `file:` address.
const FILE_SCHEME: &str = "file";

/// The scheme a reference carries, when it carries one.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Scheme {
    /// A `file:` address: a path of the local disk.
    File,
    /// A network address.
    Network,
    /// A scheme the server does not read.
    Other(String),
}

/// Resolves a reference of a file of the library against the roots.
///
/// The reference is a path, a `file:` address, or a network address. The rules
/// are the rules of `SEC-068`, in this order:
///
/// 1. a reference that carries a scheme other than `file:` is refused — a
///    network scheme such as `http:` or `smb:` is refused outright, and the
///    server makes no request for it;
/// 2. a reference that resolves outside every root is refused;
/// 3. otherwise the reference is resolved against `base` — the directory of the
///    file that carried it — and normalised.
///
/// The answer is the normalised path. The caller still has to check the file
/// that is really there: a link inside a root may point out of it, and only the
/// resolved path of the file tells the truth (`SEC-003`).
///
/// Example:
///
/// ```rust
/// use phairdlna_core::security::containment::resolve_reference;
/// use std::path::{Path, PathBuf};
///
/// let roots = vec![PathBuf::from("/media/root")];
/// let resolved = resolve_reference("../Films/film.mkv", Path::new("/media/root/lists"), &roots)
///     .expect("the entry is inside the root");
/// assert_eq!(resolved, PathBuf::from("/media/root/Films/film.mkv"));
/// ```
pub fn resolve_reference(
    reference: &str,
    base: &Path,
    roots: &[PathBuf],
) -> Result<PathBuf, ContainmentError> {
    let text = reference.trim();
    if text.is_empty() {
        return Err(ContainmentError::Empty);
    }
    let unified = text.replace('\\', "/");
    let path = match scheme_of(text) {
        Some(Scheme::File) => path_of_file_address(text),
        Some(Scheme::Network) => return Err(ContainmentError::NetworkAddress),
        Some(Scheme::Other(scheme)) => return Err(ContainmentError::UnsupportedScheme(scheme)),
        None => PathBuf::from(unified),
    };
    let joined = if path.is_absolute() {
        path
    } else {
        base.join(path)
    };
    let normalised = normalise(&joined);
    if is_inside_roots(&normalised, roots) {
        Ok(normalised)
    } else {
        Err(ContainmentError::OutsideRoots)
    }
}

/// The scheme a reference carries, when it carries one.
///
/// A single letter before the colon is a drive letter and not a scheme, so that
/// a Windows path is read as a path (`PLT-005`).
fn scheme_of(text: &str) -> Option<Scheme> {
    let (scheme, rest) = text.split_once(':')?;
    if rest.starts_with("//")
        && !scheme
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
    {
        return None;
    }
    let mut characters = scheme.chars();
    let first = characters.next()?;
    if !first.is_ascii_alphabetic() {
        return None;
    }
    if !characters.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.') {
        return None;
    }
    if scheme.len() == 1 {
        return None;
    }
    let scheme = scheme.to_ascii_lowercase();
    if scheme == FILE_SCHEME {
        return Some(Scheme::File);
    }
    if NETWORK_SCHEMES.contains(&scheme.as_str()) {
        return Some(Scheme::Network);
    }
    Some(Scheme::Other(scheme))
}

/// The path of a `file:` address.
///
/// The authority of the address is dropped: the server reads local paths only,
/// and a path that names a host is not one of them. The rest is decoded, so
/// that an address with an escaped space names the file it means.
fn path_of_file_address(text: &str) -> PathBuf {
    let rest = text
        .split_once(':')
        .map(|(_, rest)| rest)
        .unwrap_or_default();
    let without_authority = match rest.strip_prefix("//") {
        Some(after) => match after.find('/') {
            Some(slash) => &after[slash..],
            None => "",
        },
        None => rest,
    };
    match percent_decode(without_authority) {
        Some(decoded) => PathBuf::from(decoded),
        None => PathBuf::from(without_authority),
    }
}

/// Whether a path is inside one of the roots.
///
/// The comparison is lexical: the path is normalised first, so that a
/// candidate that walks out of a root with `..` does not pass. The caller
/// repeats the check with the resolved path of the file, after the symbolic
/// links have been followed (`INT-098`, `SEC-003`).
pub fn is_inside_roots(candidate: &Path, roots: &[PathBuf]) -> bool {
    roots.iter().any(|root| is_inside(candidate, root))
}

/// Whether a path is strictly inside a root.
pub fn is_inside(candidate: &Path, root: &Path) -> bool {
    let candidate = normalise(candidate);
    let root = normalise(root);
    candidate != root && candidate.starts_with(&root)
}

/// Normalises a path without touching the disk.
///
/// The `.` and the `..` are resolved lexically, as the file system would do for
/// a path that has no symbolic link; a path that walks above its start keeps
/// the `..`, so that it cannot be made to look inside a root.
pub fn normalise(path: &Path) -> PathBuf {
    let mut parts: Vec<String> = Vec::new();
    let mut absolute = false;
    for component in path.components() {
        match component {
            Component::RootDir => {
                absolute = true;
                parts.clear();
            }
            Component::Prefix(_) => {}
            Component::CurDir => {}
            Component::ParentDir => match parts.last() {
                Some(last) if last != ".." => {
                    parts.pop();
                }
                _ => {
                    if !absolute {
                        parts.push("..".to_owned());
                    }
                }
            },
            Component::Normal(part) => parts.push(part.to_string_lossy().into_owned()),
        }
    }
    let mut result = PathBuf::new();
    if absolute {
        result.push("/");
    }
    for part in parts {
        result.push(part);
    }
    result
}

/// Decodes a percent-encoded path.
///
/// A text that carries an encoding the server cannot read is refused rather
/// than guessed: a half-decoded path could name a different file than the one
/// that was written (`SEC-007`).
pub fn percent_decode(value: &str) -> Option<String> {
    let bytes = value.as_bytes();
    let mut decoded = Vec::with_capacity(bytes.len());
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'%' => {
                let hex = value.get(index + 1..index + 3)?;
                decoded.push(u8::from_str_radix(hex, 16).ok()?);
                index += 3;
            }
            byte => {
                decoded.push(byte);
                index += 1;
            }
        }
    }
    String::from_utf8(decoded).ok()
}

#[cfg(test)]
mod tests {
    use super::{is_inside_roots, normalise, percent_decode, resolve_reference, ContainmentError};
    use std::path::{Path, PathBuf};

    /// The roots of the tests.
    fn roots() -> Vec<PathBuf> {
        vec![PathBuf::from("/media/root")]
    }

    /// An entry that names a file next to the playlist is resolved.
    #[test]
    fn a_relative_entry_is_resolved_against_the_playlist() {
        let resolved = resolve_reference("film.mkv", Path::new("/media/root/lists"), &roots())
            .expect("the entry is inside the root");
        assert_eq!(resolved, PathBuf::from("/media/root/lists/film.mkv"));
    }

    /// An entry that walks up stays inside the root when the root is there.
    #[test]
    fn an_entry_that_walks_up_stays_inside_the_root() {
        let resolved = resolve_reference(
            "../Films/film.mkv",
            Path::new("/media/root/lists"),
            &roots(),
        )
        .expect("the entry is inside the root");
        assert_eq!(resolved, PathBuf::from("/media/root/Films/film.mkv"));
    }

    /// An entry that walks out of the root is refused.
    #[test]
    fn an_entry_that_walks_out_of_the_root_is_refused() {
        assert_eq!(
            resolve_reference("../../etc/passwd", Path::new("/media/root/lists"), &roots()),
            Err(ContainmentError::OutsideRoots)
        );
    }

    /// An absolute entry outside every root is refused.
    #[test]
    fn an_absolute_entry_outside_the_root_is_refused() {
        assert_eq!(
            resolve_reference("/etc/passwd", Path::new("/media/root/lists"), &roots()),
            Err(ContainmentError::OutsideRoots)
        );
    }

    /// A network address is refused, and no request is made for it.
    #[test]
    fn a_network_address_is_refused() {
        for address in [
            "http://example.invalid/x.mp4",
            "https://example.invalid/x.mp4",
            "rtsp://example.invalid/x.mp4",
            "smb://server/share/x.mp4",
        ] {
            assert_eq!(
                resolve_reference(address, Path::new("/media/root"), &roots()),
                Err(ContainmentError::NetworkAddress),
                "{address} must be refused"
            );
        }
    }

    /// A scheme the server does not read is refused.
    #[test]
    fn an_unknown_scheme_is_refused() {
        assert_eq!(
            resolve_reference(
                "gopher://example.invalid/x",
                Path::new("/media/root"),
                &roots()
            ),
            Err(ContainmentError::UnsupportedScheme("gopher".to_owned()))
        );
    }

    /// A `file:` address names a path of the disk.
    #[test]
    fn a_file_address_names_a_path() {
        let resolved = resolve_reference(
            "file:///media/root/film.mkv",
            Path::new("/media/root/lists"),
            &roots(),
        )
        .expect("the address is inside the root");
        assert_eq!(resolved, PathBuf::from("/media/root/film.mkv"));
    }

    /// A `file:` address with an escaped space names the file it means.
    #[test]
    fn a_file_address_is_decoded() {
        let resolved = resolve_reference(
            "file:///media/root/My%20Films/film.mkv",
            Path::new("/media/root/lists"),
            &roots(),
        )
        .expect("the address is inside the root");
        assert_eq!(resolved, PathBuf::from("/media/root/My Films/film.mkv"));
    }

    /// A `file:` address outside every root is refused.
    #[test]
    fn a_file_address_outside_the_root_is_refused() {
        assert_eq!(
            resolve_reference("file:///etc/passwd", Path::new("/media/root"), &roots()),
            Err(ContainmentError::OutsideRoots)
        );
    }

    /// An empty reference names nothing.
    #[test]
    fn an_empty_reference_names_nothing() {
        assert_eq!(
            resolve_reference("   ", Path::new("/media/root"), &roots()),
            Err(ContainmentError::Empty)
        );
    }

    /// A path that only shares a prefix with the root is not inside it.
    #[test]
    fn a_shared_prefix_is_not_inside() {
        assert!(!is_inside_roots(
            &PathBuf::from("/media/root2/film.mkv"),
            &roots()
        ));
    }

    /// The normalisation resolves the `.` and the `..` of a path.
    #[test]
    fn the_normalisation_resolves_the_dots() {
        assert_eq!(
            normalise(Path::new("/media/root/./lists/../film.mkv")),
            PathBuf::from("/media/root/film.mkv")
        );
        assert_eq!(
            normalise(Path::new("/media/root/../secret")),
            PathBuf::from("/media/secret")
        );
    }

    /// A text with a broken encoding is refused rather than guessed.
    #[test]
    fn a_broken_encoding_is_refused() {
        assert_eq!(percent_decode("%zz"), None);
        assert_eq!(percent_decode("%2"), None);
        assert_eq!(percent_decode("a%20b").as_deref(), Some("a b"));
    }
}
