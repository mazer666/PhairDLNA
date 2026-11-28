//! The rules of the shared folders.
//!
//! What: the rules that decide whether the roots of the library and the
//!       folder of the cache of the pictures may be used.
//! Why: a root without a name cannot be told from another root by a client,
//!      and a cache folder that is empty would point at the folder of the
//!      server itself (`CFG-002`, `FUN-017`, `FUN-018`).

use super::super::error::ConfigError;
use super::super::sections::{LibrarySection, MediaRoot};

/// The shared folders.
pub(super) fn validate_library(library: &LibrarySection) -> Result<(), ConfigError> {
    for (index, root) in library.roots.iter().enumerate() {
        validate_root(index, root)?;
    }
    validate_distinct_roots(&library.roots)?;
    validate_cache_directory(library)?;
    validate_database_path(library)?;
    validate_scan_schedule(library)
}

/// A scan schedule with an interval must have a positive duration.
fn validate_scan_schedule(library: &LibrarySection) -> Result<(), ConfigError> {
    if let crate::config::ScanSchedule::Interval(secs) = library.scan_schedule {
        if secs == 0 {
            return Err(ConfigError::field(
                "library.scan_schedule",
                "field \"library.scan_schedule\" interval is 0; expected a positive number of seconds or \"manual_only\"",
            ));
        }
    }
    Ok(())
}

/// The cache of the pictures needs a folder of its own.
///
/// An empty setting is refused, because the folder of the cache would be the
/// folder of the server itself, and the cache would delete what it finds
/// there when it makes room (`FUN-018`).
fn validate_cache_directory(library: &LibrarySection) -> Result<(), ConfigError> {
    let Some(directory) = library.cache_directory.as_deref() else {
        return Ok(());
    };
    if directory.trim().is_empty() {
        return Err(ConfigError::field(
            "library.cache_directory",
            "field \"library.cache_directory\" is empty; expected a folder, or no setting at all, \
             because the data directory of the user is used when the setting is missing",
        ));
    }
    Ok(())
}

/// The index of the catalogue needs a file of its own.
///
/// An empty setting is refused: the file of the index would be the folder of
/// the server itself, and the index would be a directory where a file is
/// expected (`FUN-009`).
fn validate_database_path(library: &LibrarySection) -> Result<(), ConfigError> {
    let Some(path) = library.database_path.as_deref() else {
        return Ok(());
    };
    if path.trim().is_empty() {
        return Err(ConfigError::field(
            "library.database_path",
            "field \"library.database_path\" is empty; expected the file of the index, or no \
             setting at all, because the data directory of the user is used when the setting is \
             missing",
        ));
    }
    Ok(())
}

/// Two roots may not share a name and may not share a path.
///
/// A name is how a person and a client tell the roots apart, and a path says
/// which folder is published. Two roots with one name would be two folders
/// with one label, and two roots with one path would publish the same files
/// twice under two identifiers (`FUN-017`, `CFG-002`).
fn validate_distinct_roots(roots: &[MediaRoot]) -> Result<(), ConfigError> {
    let mut names: Vec<&str> = Vec::new();
    let mut paths: Vec<&str> = Vec::new();
    for (index, root) in roots.iter().enumerate() {
        let field = format!("library.roots[{index}]");
        let name = root.name.trim();
        let path = root.path.trim();
        if names.contains(&name) {
            return Err(ConfigError::field(
                format!("{field}.name"),
                format!(
                    "field \"{field}.name\" is \"{name}\"; expected a name that no other root uses, \
                     because a client tells the roots apart by their name"
                ),
            ));
        }
        if paths.iter().any(|known| same_folder(known, path)) {
            return Err(ConfigError::field(
                format!("{field}.path"),
                format!(
                    "field \"{field}.path\" is \"{path}\"; expected a path that no other root uses, \
                     because the same folder published twice is the same file twice"
                ),
            ));
        }
        names.push(name);
        paths.push(path);
    }
    Ok(())
}

/// Whether two paths name the same folder.
///
/// The written paths are compared first; a folder that exists is compared by
/// the folder it resolves to, because `/Media/Films` and `/Media/Films/` are
/// one folder, and so are a folder and a link that points at it.
fn same_folder(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => false,
    }
}

/// One shared folder.
fn validate_root(index: usize, root: &MediaRoot) -> Result<(), ConfigError> {
    let field = format!("library.roots[{index}]");
    if root.name.trim().is_empty() {
        return Err(ConfigError::field(
            format!("{field}.name"),
            format!(
                "field \"{field}.name\" is empty; expected a name a client can read, \
                 for example \"Films\""
            ),
        ));
    }
    if root.path.trim().is_empty() {
        return Err(ConfigError::field(
            format!("{field}.path"),
            format!(
                "field \"{field}.path\" is empty; expected an absolute path, \
                 for example \"/Users/me/Movies\""
            ),
        ));
    }
    Ok(())
}
