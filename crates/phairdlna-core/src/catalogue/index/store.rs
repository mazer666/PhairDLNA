//! The writing of a catalogue into the index.
//!
//! What: the rows of the catalogue and the transaction that writes them.
//! Why: the write must be atomic. A reader sees the catalogue of before or the
//!       catalogue of now, never a mixture of the two, and a daemon that dies
//!       while it writes leaves an index that can still be read (`FUN-008`).
//! Example: `store` writes the whole catalogue in one transaction.

use std::path::Path;

use rusqlite::{Connection, Transaction};

use super::super::{Catalogue, ContentItem, MediaKind};
use super::IndexError;

/// The statements that empty the index before the new catalogue is written.
const DELETE: [&str; 5] = [
    "DELETE FROM objects",
    "DELETE FROM containers",
    "DELETE FROM container_children",
    "DELETE FROM roots",
    "DELETE FROM update_id",
];

/// The statement that writes one object.
const INSERT_OBJECT: &str = "INSERT INTO objects
    (id, parent_id, kind, title, path, size, modified, duration, mime, hash, charset, subtitle_id)
    VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)";

/// The statement that writes one container.
const INSERT_CONTAINER: &str = "INSERT INTO containers (id, update_id) VALUES (?1, ?2)";

/// The statement that writes one membership.
const INSERT_CHILD: &str = "INSERT INTO container_children
    (container_id, object_id, position, title) VALUES (?1, ?2, ?3, ?4)";

/// The statement that writes one root.
const INSERT_ROOT: &str = "INSERT INTO roots
    (id, name, path, present, entries) VALUES (?1, ?2, ?3, ?4, ?5)";

/// The statement that writes the identifier of the catalogue.
const INSERT_UPDATE_ID: &str =
    "INSERT INTO update_id (key, fingerprint, generation) VALUES (?1, ?2, ?3)";

/// The key under which the identifier of the catalogue is written.
const SYSTEM_KEY: &str = "system";

/// Writes the catalogue into the index.
pub(super) fn store(
    writer: &mut Connection,
    catalogue: &mut Catalogue,
    path: &Path,
) -> Result<(), IndexError> {
    let persisted = read_persisted(writer).map_err(|error| unusable(path, error))?;
    let generation = generation_for(catalogue, persisted);
    catalogue.system_update_id = generation;
    let transaction = writer
        .transaction()
        .map_err(|error| unusable(path, error))?;
    empty(&transaction).map_err(|error| unusable(path, error))?;
    write_objects(&transaction, catalogue).map_err(|error| unusable(path, error))?;
    write_containers(&transaction, catalogue).map_err(|error| unusable(path, error))?;
    write_membership(&transaction, catalogue).map_err(|error| unusable(path, error))?;
    write_roots(&transaction, catalogue).map_err(|error| unusable(path, error))?;
    write_update_id(&transaction, catalogue, generation).map_err(|error| unusable(path, error))?;
    transaction.commit().map_err(|error| unusable(path, error))
}

/// Empties the index, so that nothing of the old catalogue is left.
fn empty(transaction: &Transaction<'_>) -> Result<(), rusqlite::Error> {
    for statement in DELETE {
        transaction.execute(statement, [])?;
    }
    Ok(())
}

/// Writes every object of the catalogue: the containers and the items.
fn write_objects(
    transaction: &Transaction<'_>,
    catalogue: &Catalogue,
) -> Result<(), rusqlite::Error> {
    for node in catalogue.containers.values() {
        write_object(
            transaction,
            node.id.as_str(),
            node.parent_id.as_str(),
            node.kind,
            node.title.as_str(),
        )?;
    }
    for item in catalogue.items.values() {
        let path = catalogue.files.get(&item.id);
        write_item(
            transaction,
            item,
            path.map(|path| path.to_string_lossy().into_owned()),
        )?;
    }
    Ok(())
}

/// Writes one container as an object of the tree.
fn write_object(
    transaction: &Transaction<'_>,
    id: &str,
    parent_id: &str,
    kind: MediaKind,
    title: &str,
) -> Result<(), rusqlite::Error> {
    transaction.execute(
        INSERT_OBJECT,
        rusqlite::params![
            id,
            parent_id,
            kind.storage_key(),
            title,
            Option::<String>::None,
            Option::<i64>::None,
            Option::<i64>::None,
            Option::<i64>::None,
            Option::<String>::None,
            id,
            Option::<String>::None,
            Option::<String>::None,
        ],
    )?;
    Ok(())
}

/// Writes one item of the catalogue as an object of the tree.
fn write_item(
    transaction: &Transaction<'_>,
    item: &ContentItem,
    path: Option<String>,
) -> Result<(), rusqlite::Error> {
    let metadata = &item.metadata;
    transaction.execute(
        INSERT_OBJECT,
        rusqlite::params![
            item.id,
            item.parent_id,
            item.kind.storage_key(),
            item.title,
            path,
            metadata.size.map(|size| size as i64),
            metadata.modified.map(|seconds| seconds as i64),
            metadata.duration.map(|duration| duration.as_secs() as i64),
            metadata.mime_type,
            item.id,
            metadata.charset,
            metadata
                .subtitle
                .as_ref()
                .map(|subtitle| subtitle.id.clone()),
        ],
    )?;
    Ok(())
}

/// Writes the containers with the identifier that changes with their content.
fn write_containers(
    transaction: &Transaction<'_>,
    catalogue: &Catalogue,
) -> Result<(), rusqlite::Error> {
    for node in catalogue.containers.values() {
        transaction.execute(INSERT_CONTAINER, rusqlite::params![node.id, node.update_id])?;
    }
    Ok(())
}

/// Writes the membership of the containers, in the order the client sees it.
fn write_membership(
    transaction: &Transaction<'_>,
    catalogue: &Catalogue,
) -> Result<(), rusqlite::Error> {
    for node in catalogue.containers.values() {
        for (position, child) in node.children.iter().enumerate() {
            let title = match own_title(catalogue, child) {
                Some(title) if title == child.title => Option::<String>::None,
                Some(title) => Some(title.to_owned()),
                None => Some(child.title.clone()),
            };
            transaction.execute(
                INSERT_CHILD,
                rusqlite::params![node.id, child.id, position as i64, title],
            )?;
        }
    }
    Ok(())
}

/// The title the object carries for itself.
///
/// The answer is `None` for an object the catalogue does not know, such as an
/// entry of a playlist that names a file which is not there: its title is
/// written into the membership, or it would be lost.
fn own_title<'a>(catalogue: &'a Catalogue, child: &ContentItem) -> Option<&'a str> {
    if let Some(node) = catalogue.containers.get(&child.id) {
        return Some(node.title.as_str());
    }
    catalogue
        .items
        .get(&child.id)
        .map(|item| item.title.as_str())
}

/// Writes the roots of the library.
fn write_roots(
    transaction: &Transaction<'_>,
    catalogue: &Catalogue,
) -> Result<(), rusqlite::Error> {
    for root in &catalogue.roots {
        transaction.execute(
            INSERT_ROOT,
            rusqlite::params![
                root.id,
                root.name,
                root.path.to_string_lossy(),
                i64::from(root.present),
                root.entries as i64,
            ],
        )?;
    }
    Ok(())
}

/// Writes the identifier of the catalogue and the fingerprint it belongs to.
fn write_update_id(
    transaction: &Transaction<'_>,
    catalogue: &Catalogue,
    generation: u32,
) -> Result<(), rusqlite::Error> {
    transaction.execute(
        INSERT_UPDATE_ID,
        rusqlite::params![SYSTEM_KEY, catalogue.fingerprint() as i64, generation],
    )?;
    Ok(())
}

/// The fingerprint and the identifier that the last run wrote.
fn read_persisted(writer: &Connection) -> Result<Option<(u64, u32)>, rusqlite::Error> {
    let mut statement =
        writer.prepare("SELECT fingerprint, generation FROM update_id WHERE key = ?1")?;
    let mut rows = statement.query([SYSTEM_KEY])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    Ok(Some((row.get::<_, i64>(0)? as u64, row.get(1)?)))
}

/// The identifier that goes with the catalogue.
///
/// A catalogue that did not change since the last run keeps the identifier it
/// had, so that a client that browsed before the restart does not browse again
/// for nothing. A catalogue that changed gets an identifier that is higher than
/// every identifier that was handed out before (`INT-037`).
fn generation_for(catalogue: &Catalogue, persisted: Option<(u64, u32)>) -> u32 {
    match persisted {
        Some((fingerprint, generation)) if fingerprint == catalogue.fingerprint() => generation,
        Some((_, generation)) => catalogue.system_update_id.max(generation.saturating_add(1)),
        None => catalogue.system_update_id,
    }
}

/// The error of a file that cannot be written.
fn unusable(path: &Path, error: rusqlite::Error) -> IndexError {
    IndexError::Unusable {
        path: path.to_path_buf(),
        reason: error.to_string(),
    }
}
