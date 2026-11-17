//! The shape of the database of the index.
//!
//! What: the version of the schema, the statements that create the tables and
//!       the check that a file carries the schema this build knows.
//! Why: a file that another build wrote may carry a schema this build does not
//!       understand; the check refuses it before a query is made, because a
//!       query against a foreign schema answers nonsense (`FUN-010`).
//! Example: `create` builds the tables of a new index and writes its version.

use rusqlite::Connection;

/// The version of the schema this build writes.
///
/// A file that carries another version is moved aside and a new one is created:
/// the index is a cache, and a cache that cannot be read is worth less than a
/// new one (`FUN-010`).
pub(super) const SCHEMA_VERSION: i64 = 1;

/// The tables of the index.
///
/// The four tables are the ones the specification names; the fifth holds the
/// membership of a container, because one file may live in a folder *and* in a
/// playlist, and a playlist gives its entries a title of its own.
const TABLES: [&str; 5] = [
    "objects",
    "containers",
    "container_children",
    "roots",
    "update_id",
];

/// The statements that create the schema.
const CREATE: [&str; 5] = [
    "CREATE TABLE IF NOT EXISTS objects (
        id TEXT PRIMARY KEY,
        parent_id TEXT NOT NULL,
        kind TEXT NOT NULL,
        title TEXT NOT NULL,
        path TEXT,
        size INTEGER,
        modified INTEGER,
        duration INTEGER,
        mime TEXT,
        hash TEXT NOT NULL,
        charset TEXT,
        subtitle_id TEXT
    )",
    "CREATE TABLE IF NOT EXISTS containers (
        id TEXT PRIMARY KEY,
        update_id INTEGER NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS container_children (
        container_id TEXT NOT NULL,
        object_id TEXT NOT NULL,
        position INTEGER NOT NULL,
        title TEXT,
        PRIMARY KEY (container_id, object_id)
    )",
    "CREATE TABLE IF NOT EXISTS roots (
        id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        path TEXT NOT NULL,
        present INTEGER NOT NULL,
        entries INTEGER NOT NULL
    )",
    "CREATE TABLE IF NOT EXISTS update_id (
        key TEXT PRIMARY KEY,
        fingerprint INTEGER NOT NULL,
        generation INTEGER NOT NULL
    )",
];

/// Creates the schema of an index that has none.
///
/// The statements are the ones of the build, and the version is written with
/// them: the version is what tells a later build that the file is its own.
pub(super) fn create(connection: &Connection) -> Result<(), rusqlite::Error> {
    for statement in CREATE {
        connection.execute_batch(statement)?;
    }
    connection.pragma_update(None, "user_version", SCHEMA_VERSION)?;
    Ok(())
}

/// Whether the file carries the schema of this build.
///
/// The check is the version *and* the tables: a file that carries the right
/// version but lost a table is a file that cannot be read.
pub(super) fn matches(connection: &Connection) -> Result<bool, rusqlite::Error> {
    let version: i64 = connection.pragma_query_value(None, "user_version", |row| row.get(0))?;
    if version != SCHEMA_VERSION {
        return Ok(false);
    }
    for table in TABLES {
        let found: i64 = connection.query_row(
            "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name = ?1",
            [table],
            |row| row.get(0),
        )?;
        if found == 0 {
            return Ok(false);
        }
    }
    Ok(true)
}

/// Whether the database holds no table at all.
///
/// Such a database is a new one: it is not a file of another build, and the
/// schema is written into it.
pub(super) fn is_empty(connection: &Connection) -> Result<bool, rusqlite::Error> {
    let tables: i64 = connection.query_row(
        "SELECT count(*) FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%'",
        [],
        |row| row.get(0),
    )?;
    Ok(tables == 0)
}
