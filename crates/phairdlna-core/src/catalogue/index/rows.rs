//! The rows of the index.
//!
//! What: the shape of a row of every table and the queries that read them.
//! Why: the reader of the index must not depend on the order of the columns and
//!       must not build a query out of a value; every value is bound as a
//!       parameter (`SEC-005`).
//! Example: `read_objects` reads every object of the index.

use std::collections::HashMap;

use super::super::MediaKind;
use super::super::RootSummary;

/// One object of the tree, as the index holds it.
pub(super) struct ObjectRow {
    pub(super) id: String,
    pub(super) parent_id: String,
    pub(super) kind: MediaKind,
    pub(super) title: String,
    pub(super) path: Option<String>,
    pub(super) size: Option<u64>,
    pub(super) modified: Option<u64>,
    pub(super) duration: Option<u64>,
    pub(super) mime: Option<String>,
    pub(super) charset: Option<String>,
    pub(super) subtitle_id: Option<String>,
}

/// One child of a container, as the index holds it.
pub(super) struct ChildRow {
    pub(super) object_id: String,
    pub(super) title: Option<String>,
}

/// Reads every object of the index.
pub(super) fn read_objects(
    transaction: &rusqlite::Transaction<'_>,
) -> Result<Vec<ObjectRow>, rusqlite::Error> {
    let mut statement = transaction.prepare(
        "SELECT id, parent_id, kind, title, path, size, modified, duration, mime, charset, \
         subtitle_id FROM objects",
    )?;
    let rows = statement.query_map([], |row| {
        Ok(ObjectRow {
            id: row.get(0)?,
            parent_id: row.get(1)?,
            kind: MediaKind::from_storage_key(&row.get::<_, String>(2)?),
            title: row.get(3)?,
            path: row.get(4)?,
            size: row.get::<_, Option<i64>>(5)?.map(|size| size as u64),
            modified: row.get::<_, Option<i64>>(6)?.map(|seconds| seconds as u64),
            duration: row.get::<_, Option<i64>>(7)?.map(|seconds| seconds as u64),
            mime: row.get(8)?,
            charset: row.get(9)?,
            subtitle_id: row.get(10)?,
        })
    })?;
    rows.collect()
}

/// Reads the identifiers of the containers.
pub(super) fn read_containers(
    transaction: &rusqlite::Transaction<'_>,
) -> Result<HashMap<String, u32>, rusqlite::Error> {
    let mut statement = transaction.prepare("SELECT id, update_id FROM containers")?;
    let rows = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
    let mut containers = HashMap::new();
    for row in rows {
        let (id, update_id) = row?;
        containers.insert(id, update_id);
    }
    Ok(containers)
}

/// Reads the membership of the containers.
pub(super) fn read_children(
    transaction: &rusqlite::Transaction<'_>,
) -> Result<HashMap<String, Vec<ChildRow>>, rusqlite::Error> {
    let mut statement = transaction.prepare(
        "SELECT container_id, object_id, title FROM container_children \
         ORDER BY container_id, position",
    )?;
    let rows = statement.query_map([], |row| {
        Ok((
            row.get::<_, String>(0)?,
            ChildRow {
                object_id: row.get(1)?,
                title: row.get(2)?,
            },
        ))
    })?;
    let mut children: HashMap<String, Vec<ChildRow>> = HashMap::new();
    for row in rows {
        let (container_id, child) = row?;
        children.entry(container_id).or_default().push(child);
    }
    Ok(children)
}

/// Reads the roots of the library.
pub(super) fn read_roots(
    transaction: &rusqlite::Transaction<'_>,
) -> Result<Vec<RootSummary>, rusqlite::Error> {
    let mut statement =
        transaction.prepare("SELECT id, name, path, present, entries FROM roots")?;
    let rows = statement.query_map([], |row| {
        Ok(RootSummary {
            id: row.get(0)?,
            name: row.get(1)?,
            path: row.get::<_, String>(2)?.into(),
            present: row.get::<_, i64>(3)? != 0,
            entries: row.get::<_, i64>(4)? as usize,
        })
    })?;
    rows.collect()
}

/// Reads the fingerprint and the identifier of the catalogue.
pub(super) fn read_update_id(
    transaction: &rusqlite::Transaction<'_>,
) -> Result<(u64, u32), rusqlite::Error> {
    let mut statement = transaction
        .prepare("SELECT fingerprint, generation FROM update_id WHERE key = 'system'")?;
    let mut rows = statement.query([])?;
    let Some(row) = rows.next()? else {
        return Ok((0, 0));
    };
    Ok((row.get::<_, i64>(0)? as u64, row.get(1)?))
}
