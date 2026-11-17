//! The diagnostics of the server, and the report the user may hand to a
//! support ticket.
//!
//! What: the facts about a running server that a support ticket needs — the
//!       version, the library, the running streams, the warnings and the last
//!       records of the log — and the text that carries them.
//! Why: a user who asks for help must not have to know what to collect, and
//!      the report must not carry the private parts of the machine of the
//!      user by accident (`OPS-013`, `UX-011`, `SEC-094`). The model lives in
//!      the core so that the command line and the menu bar application show
//!      the same report and cannot disagree about it.
//! Example: `phairdlna diagnostics` writes the report of the running server
//!          to the standard output, with the paths of the user replaced by
//!          `<path:1>`.
//!
//! **The report is written in plain text.** A support ticket carries a text
//! file that every reader can open, whatever the system of the reader is
//! (`UX-011`).

pub mod text;

use crate::json::Json;
use crate::security::Redactor;

pub use text::{render, Report};

/// What the server says about itself.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Diagnostics {
    /// The version of the server.
    pub version: String,
    /// The revision of the source the server was built from.
    pub source_revision: String,
    /// How long the server has been running, in seconds.
    pub uptime_seconds: u64,
    /// How many objects the catalogue holds.
    pub catalogue_objects: usize,
    /// How many bytes the server holds in memory, if it knows.
    pub resident_bytes: Option<u64>,
    /// The number that tells a client that the catalogue has changed.
    pub system_update_id: u32,
    /// The roots of the library.
    pub roots: Vec<RootState>,
    /// The streams that are running.
    pub transfers: Vec<TransferInfo>,
    /// The last records of the log.
    pub records: Vec<LogRecord>,
    /// The warnings the server holds.
    pub warnings: Vec<String>,
    /// When the library was scanned the last time.
    pub last_scan: Option<String>,
}

/// One root of the library.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct RootState {
    /// The name of the root, as the owner named it.
    pub name: String,
    /// The path of the root.
    ///
    /// The path is the one of the machine of the owner, and a report that
    /// carries it must be redacted before it leaves the machine (`SEC-094`).
    pub path: String,
    /// Whether the root is mounted.
    pub mounted: bool,
    /// How many objects the root holds.
    pub objects: usize,
}

/// One stream that is running.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TransferInfo {
    /// The name of the client that plays the stream.
    pub client: String,
    /// The address of the client.
    pub address: String,
    /// The title of the stream.
    pub title: String,
    /// How long the stream runs, in seconds.
    pub running_seconds: u64,
}

/// One record of the log.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct LogRecord {
    /// When the record was written.
    pub time: String,
    /// The level of the record.
    pub level: String,
    /// The part of the server that wrote the record.
    pub component: String,
    /// What the record says.
    pub message: String,
    /// The fields of the record, by name.
    pub fields: Vec<(String, String)>,
}

/// The facts and the context with the private values replaced.
///
/// A report that leaves the machine is a report that must not carry the paths
/// of the user and the addresses of the clients. Every value that names such
/// a place is replaced by a pseudonym, and the same value always gets the same
/// pseudonym inside one report (`SEC-094`, `UX-011`).
///
/// The values that are not private are left alone: the number of a port and
/// the name of a device are what the reader of the report needs.
pub fn redact(diagnostics: &Diagnostics, report: &Report) -> (Diagnostics, Report) {
    let redactor = Redactor::new();
    let facts = redact_diagnostics(diagnostics, &redactor);
    let context = redact_report(report, &redactor);
    (facts, context)
}

/// Replaces private places in the server diagnostics with pseudonyms.
fn redact_diagnostics(diagnostics: &Diagnostics, redactor: &Redactor) -> Diagnostics {
    Diagnostics {
        version: diagnostics.version.clone(),
        source_revision: diagnostics.source_revision.clone(),
        uptime_seconds: diagnostics.uptime_seconds,
        catalogue_objects: diagnostics.catalogue_objects,
        resident_bytes: diagnostics.resident_bytes,
        system_update_id: diagnostics.system_update_id,
        roots: diagnostics
            .roots
            .iter()
            .map(|root| RootState {
                name: root.name.clone(),
                path: redactor.value(&root.path),
                mounted: root.mounted,
                objects: root.objects,
            })
            .collect(),
        transfers: diagnostics
            .transfers
            .iter()
            .map(|transfer| TransferInfo {
                client: transfer.client.clone(),
                address: redactor.value(&transfer.address),
                title: transfer.title.clone(),
                running_seconds: transfer.running_seconds,
            })
            .collect(),
        records: diagnostics
            .records
            .iter()
            .map(|record| LogRecord {
                time: record.time.clone(),
                level: record.level.clone(),
                component: record.component.clone(),
                message: redactor.value(&record.message),
                fields: record
                    .fields
                    .iter()
                    .map(|(name, value)| (name.clone(), redactor.value(value)))
                    .collect(),
            })
            .collect(),
        warnings: diagnostics
            .warnings
            .iter()
            .map(|warning| redactor.value(warning))
            .collect(),
        last_scan: diagnostics.last_scan.clone(),
    }
}

/// Replaces private places in the report context with pseudonyms.
fn redact_report(report: &Report, redactor: &Redactor) -> Report {
    Report {
        generated_at: report.generated_at.clone(),
        platform: report.platform.clone(),
        server_state: report.server_state.clone(),
        configuration: report
            .configuration
            .iter()
            .map(|(name, value)| (name.clone(), redactor.value(value)))
            .collect(),
        source_offer: report.source_offer.clone(),
    }
}

/// Reads the diagnostics of the surface.
///
/// A document that lacks a field is read with the value of nothing rather
/// than refused: a report that names what it knows is more useful than no
/// report at all.
pub fn from_json(document: &Json) -> Diagnostics {
    Diagnostics {
        version: text_of(document, "version"),
        source_revision: text_of(document, "source_revision"),
        uptime_seconds: number_of(document, "uptime_seconds"),
        catalogue_objects: number_of(document, "catalogue_objects") as usize,
        resident_bytes: optional_number_of(document, "resident_bytes"),
        system_update_id: number_of(document, "system_update_id") as u32,
        roots: roots_of(document),
        transfers: transfers_of(document),
        records: records_of(document),
        warnings: list_of(document, "warnings"),
        last_scan: optional_text_of(document, "last_scan"),
    }
}

/// The roots of the library of a document.
fn roots_of(document: &Json) -> Vec<RootState> {
    list_of_entries(document, "roots")
        .into_iter()
        .map(|root| RootState {
            name: text_of(root, "name"),
            path: text_of(root, "path"),
            mounted: root.get("mounted").and_then(Json::as_bool).unwrap_or(false),
            objects: number_of(root, "objects") as usize,
        })
        .collect()
}

/// The running streams of a document.
fn transfers_of(document: &Json) -> Vec<TransferInfo> {
    list_of_entries(document, "transfers")
        .into_iter()
        .map(|transfer| TransferInfo {
            client: text_of(transfer, "client"),
            address: text_of(transfer, "address"),
            title: text_of(transfer, "title"),
            running_seconds: number_of(transfer, "running_seconds"),
        })
        .collect()
}

/// The records of the log of a document.
fn records_of(document: &Json) -> Vec<LogRecord> {
    list_of_entries(document, "records")
        .into_iter()
        .map(|record| LogRecord {
            time: text_of(record, "time"),
            level: text_of(record, "level"),
            component: text_of(record, "component"),
            message: text_of(record, "message"),
            fields: fields_of(record),
        })
        .collect()
}

/// The fields of one record of the log.
fn fields_of(record: &Json) -> Vec<(String, String)> {
    let Some(fields) = record.get("fields").and_then(Json::as_map) else {
        return Vec::new();
    };
    fields
        .iter()
        .map(|(name, value)| (name.clone(), value.as_str().unwrap_or("").to_owned()))
        .collect()
}

/// The entries of a list of a document.
fn list_of_entries<'a>(document: &'a Json, key: &str) -> Vec<&'a Json> {
    document
        .get(key)
        .and_then(Json::as_list)
        .map(|entries| entries.iter().collect())
        .unwrap_or_default()
}

/// The texts of a list of a document.
fn list_of(document: &Json, key: &str) -> Vec<String> {
    list_of_entries(document, key)
        .into_iter()
        .map(|entry| entry.as_str().unwrap_or("").to_owned())
        .collect()
}

/// The text of a field of a document.
fn text_of(document: &Json, key: &str) -> String {
    document
        .get(key)
        .and_then(Json::as_str)
        .unwrap_or("")
        .to_owned()
}

/// The text of a field of a document, if the field holds one.
fn optional_text_of(document: &Json, key: &str) -> Option<String> {
    document.get(key).and_then(Json::as_str).map(str::to_owned)
}

/// The number of a field of a document.
fn number_of(document: &Json, key: &str) -> u64 {
    document.get(key).and_then(Json::as_u64).unwrap_or(0)
}

/// The number of a field of a document, if the field holds one.
fn optional_number_of(document: &Json, key: &str) -> Option<u64> {
    document.get(key).and_then(Json::as_u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The positive case: the diagnostics of the surface are read.
    #[test]
    fn the_diagnostics_of_the_surface_are_read() {
        let document = Json::parse(
            r#"{"version": "0.1.0", "source_revision": "abc1234", "uptime_seconds": 42,
                "catalogue_objects": 7, "resident_bytes": null, "system_update_id": 3,
                "roots": [{"name": "Films", "path": "<path:1>", "mounted": true, "objects": 7}],
                "transfers": [{"client": "Telecast", "address": "<address:1>", "title": "Film",
                               "running_seconds": 5}],
                "records": [{"time": "10:00:00", "level": "info", "component": "catalogue",
                             "message": "scanned", "fields": {"objects": "7"}}],
                "warnings": ["a warning"], "last_scan": "10:00:00"}"#,
        )
        .expect("the document is read");

        let diagnostics = from_json(&document);

        assert_eq!(diagnostics.version, "0.1.0");
        assert_eq!(diagnostics.uptime_seconds, 42);
        assert_eq!(diagnostics.catalogue_objects, 7);
        assert_eq!(diagnostics.resident_bytes, None);
        assert_eq!(diagnostics.roots[0].name, "Films");
        assert!(diagnostics.roots[0].mounted);
        assert_eq!(diagnostics.transfers[0].running_seconds, 5);
        assert_eq!(
            diagnostics.records[0].fields,
            vec![("objects".to_owned(), "7".to_owned())]
        );
        assert_eq!(diagnostics.warnings, vec!["a warning".to_owned()]);
        assert_eq!(diagnostics.last_scan.as_deref(), Some("10:00:00"));
    }

    /// The negative case: a document that holds nothing is read as nothing.
    #[test]
    fn an_empty_document_is_read_as_nothing() {
        let document = Json::parse("{}").expect("the document is read");
        let diagnostics = from_json(&document);
        assert_eq!(diagnostics.version, "");
        assert_eq!(diagnostics.uptime_seconds, 0);
        assert!(diagnostics.roots.is_empty());
        assert!(diagnostics.records.is_empty());
        assert_eq!(diagnostics.last_scan, None);
    }
}
