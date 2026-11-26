//! JSON representations of the server's status.
//!
//! What: serializes status and shared-folder facts for the authenticated API.
//! Why: normal app views need the advertised server identity from the same
//!      source as the rest of the daemon state.
//! Example: `/api/status` includes a device name and an optional LAN address.

use phairdlna_core::security::Redactor;

use crate::backend::{RootState, Status};
use crate::json::Json;

/// The state of the server, as JSON.
pub(super) fn json_of_status(status: &Status, redactor: &Redactor) -> Json {
    Json::map(vec![
        (
            "lifecycle_state".to_owned(),
            Json::text(status.lifecycle_state.as_str().to_owned()),
        ),
        (
            "lifecycle_reason".to_owned(),
            Json::text(status.lifecycle_reason.clone()),
        ),
        (
            "device_name".to_owned(),
            Json::text(status.device_name.clone()),
        ),
        (
            "device_address".to_owned(),
            status
                .device_address
                .as_ref()
                .map(|address| Json::text(address.clone()))
                .unwrap_or(Json::Null),
        ),
        (
            "discovery_available".to_owned(),
            Json::flag(status.discovery_available),
        ),
        ("version".to_owned(), Json::text(status.version.clone())),
        (
            "source_revision".to_owned(),
            Json::text(status.source_revision.clone()),
        ),
        (
            "uptime_seconds".to_owned(),
            Json::uint(status.uptime_seconds),
        ),
        (
            "active_streams".to_owned(),
            Json::uint(status.active_streams as u64),
        ),
        (
            "catalogue_objects".to_owned(),
            Json::uint(status.catalogue_objects as u64),
        ),
        (
            "resident_bytes".to_owned(),
            Json::maybe_uint(status.resident_bytes),
        ),
        ("cache_bytes".to_owned(), Json::uint(status.cache_bytes)),
        (
            "system_update_id".to_owned(),
            Json::uint(u64::from(status.system_update_id)),
        ),
        ("roots".to_owned(), json_of_roots(&status.roots, redactor)),
    ])
}

/// The roots of the library, as JSON.
pub(super) fn json_of_roots(roots: &[RootState], redactor: &Redactor) -> Json {
    Json::list(
        roots
            .iter()
            .map(|root| json_of_root(root, redactor))
            .collect(),
    )
}

/// One root of the library, as JSON.
fn json_of_root(root: &RootState, redactor: &Redactor) -> Json {
    Json::map(vec![
        ("name".to_owned(), Json::text(root.name.clone())),
        ("path".to_owned(), Json::text(redactor.value(&root.path))),
        ("mounted".to_owned(), Json::flag(root.mounted)),
        ("objects".to_owned(), Json::uint(root.objects as u64)),
    ])
}
