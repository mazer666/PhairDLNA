//! What the administration surface tells about the server.
//!
//! What: the snapshots that the answers of the surface become.
//! Why: the surface answers with JSON, and the application thinks in
//!      snapshots; the translation lives here and not in the menu and not in
//!      the drawing (`MB-001`).
//! Example: the answer `"active_streams": 2` becomes a snapshot whose
//!          `active_streams` is two.

use crate::ipc::json::Json;

/// One root of the library, as the surface tells it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RootSnapshot {
    /// The name of the root.
    pub name: String,
    /// The path of the root, with the private parts already replaced.
    pub path: String,
    /// Whether the volume of the root is there.
    pub mounted: bool,
    /// How many objects the root holds.
    pub objects: u64,
}

/// The state of the server, as the surface tells it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StatusSnapshot {
    /// How long the server has been running, in seconds.
    pub uptime_seconds: u64,
    /// How many streams are running.
    pub active_streams: u64,
    /// How many objects the catalogue holds.
    pub catalogue_objects: u64,
    /// The roots of the library.
    pub roots: Vec<RootSnapshot>,
}

/// One stream that is running.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TransferSnapshot {
    /// The name of the client that plays the stream.
    pub client: String,
    /// The address of the client, with the private parts already replaced.
    pub address: String,
    /// What is being played.
    pub title: String,
}

/// What the server has to say about itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DiagnosticsSnapshot {
    /// The warnings of the server.
    pub warnings: Vec<String>,
    /// The streams that are running.
    pub transfers: Vec<TransferSnapshot>,
    /// What the last scan of the library found.
    pub last_scan: Option<String>,
}

/// What a scan of the library found.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RescanReport {
    /// Whether the library was scanned again.
    pub rescanned: bool,
    /// How many objects the catalogue holds after the scan.
    pub objects: u64,
    /// How long the scan took, in seconds.
    pub duration_seconds: u64,
    /// What happened, in one sentence.
    pub message: String,
}

/// The number of a key of a document, or zero when the key is not there.
pub(crate) fn number_of(document: &Json, key: &str) -> u64 {
    document.get(key).and_then(Json::as_u64).unwrap_or(0)
}

/// The texts of a key of a document that holds a list.
pub(crate) fn texts_of(document: &Json, key: &str) -> Vec<String> {
    document
        .get(key)
        .and_then(Json::as_list)
        .map(|values| {
            values
                .iter()
                .filter_map(Json::as_str)
                .map(str::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

/// The roots of a document.
pub(crate) fn roots_of(document: &Json) -> Vec<RootSnapshot> {
    document
        .get("roots")
        .and_then(Json::as_list)
        .map(|values| values.iter().filter_map(root_of).collect())
        .unwrap_or_default()
}

/// One root of a document.
pub(crate) fn root_of(value: &Json) -> Option<RootSnapshot> {
    Some(RootSnapshot {
        name: value.get("name")?.as_str()?.to_owned(),
        path: value
            .get("path")
            .and_then(Json::as_str)
            .unwrap_or_default()
            .to_owned(),
        mounted: value
            .get("mounted")
            .and_then(Json::as_bool)
            .unwrap_or(false),
        objects: number_of(value, "objects"),
    })
}

/// The streams of a document.
pub(crate) fn transfers_of(document: &Json) -> Vec<TransferSnapshot> {
    document
        .get("transfers")
        .and_then(Json::as_list)
        .map(|values| {
            values
                .iter()
                .filter_map(|value| {
                    Some(TransferSnapshot {
                        client: value.get("client")?.as_str()?.to_owned(),
                        address: value
                            .get("address")
                            .and_then(Json::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                        title: value
                            .get("title")
                            .and_then(Json::as_str)
                            .unwrap_or_default()
                            .to_owned(),
                    })
                })
                .collect()
        })
        .unwrap_or_default()
}
