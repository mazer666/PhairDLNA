//! The status command of the command line (`OPS-002`, `OPS-014`).
//!
//! What: queries the running daemon via the local administration surface
//!       and prints the status report as human-readable text or JSON.
//! Why: lets users and scripts inspect whether the server is running, its uptime,
//!      configured roots, catalogue size, active streams and last scan result (`AC-02`).
//!      When stopped, exits with EXIT_NOTHING_TO_DO (`AC-03`, `OPS-014`).

use std::path::PathBuf;

use phairdlna_core::config::Config;
use phairdlna_core::json::Json;

use crate::diagnostics::surface;
use crate::Outcome;

/// Options for the status command.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StatusOptions {
    /// The configuration file to use.
    pub path: Option<PathBuf>,
    /// Whether the output is formatted as JSON.
    pub json: bool,
}

/// Executes the status command.
pub fn status(options: &StatusOptions) -> Outcome {
    let path = crate::resolve(options.path.as_deref());
    let config = match Config::load(&path) {
        Ok(config) => config,
        Err(error) => {
            return Outcome::configuration_error(format!(
                "the configuration \"{}\" is not usable:\n{}\n",
                path.display(),
                error.message()
            ));
        }
    };

    match surface::ask(&config) {
        Ok(answer) => {
            let facts = answer.diagnostics;
            if options.json {
                Outcome::ok(render_json(&facts))
            } else {
                Outcome::ok(render_text(&facts))
            }
        }
        Err(_) => Outcome::nothing_to_do("PhairDLNA daemon is not running\n"),
    }
}

/// Formats the status report as text.
fn render_text(facts: &phairdlna_core::diagnostics::Diagnostics) -> String {
    let mut out = String::new();
    out.push_str(&format!("PhairDLNA {}\n", facts.version));
    out.push_str(&format!("Uptime: {}s\n", facts.uptime_seconds));
    out.push_str(&format!("Catalogue objects: {}\n", facts.catalogue_objects));
    out.push_str(&format!("Active transfers: {}\n", facts.transfers.len()));
    let last_scan = facts.last_scan.as_deref().unwrap_or("none");
    out.push_str(&format!("Last scan: {last_scan}\n"));
    if let Some(ref iface) = facts.effective_interface {
        out.push_str(&format!("Effective interface: {iface}\n"));
    }
    if let Some(ref addr) = facts.effective_address {
        out.push_str(&format!("Effective address: {addr}\n"));
    }
    if let Some(ref listener) = facts.listener_state {
        out.push_str(&format!("Listener state: {listener}\n"));
    }
    if let Some(ref recovery) = facts.last_recovery_event {
        out.push_str(&format!("Last recovery: {recovery}\n"));
    }
    out.push_str("Configured roots:\n");
    if facts.roots.is_empty() {
        out.push_str("  (none)\n");
    } else {
        for root in &facts.roots {
            let mounted = if root.mounted { "mounted" } else { "unmounted" };
            out.push_str(&format!(
                "  - {} ({} objects, {})\n",
                root.name, root.objects, mounted
            ));
        }
    }
    out
}

/// Formats the network and recovery fields of the status as JSON (LIFE-009).
fn json_status_network_fields(
    facts: &phairdlna_core::diagnostics::Diagnostics,
) -> Vec<(String, Json)> {
    vec![
        (
            "effective_interface".to_owned(),
            facts
                .effective_interface
                .as_ref()
                .map(|s| Json::text(s.clone()))
                .unwrap_or(Json::Null),
        ),
        (
            "effective_address".to_owned(),
            facts
                .effective_address
                .as_ref()
                .map(|s| Json::text(s.clone()))
                .unwrap_or(Json::Null),
        ),
        (
            "listener_state".to_owned(),
            facts
                .listener_state
                .as_ref()
                .map(|s| Json::text(s.clone()))
                .unwrap_or(Json::Null),
        ),
        (
            "last_recovery_event".to_owned(),
            facts
                .last_recovery_event
                .as_ref()
                .map(|s| Json::text(s.clone()))
                .unwrap_or(Json::Null),
        ),
    ]
}

/// Formats the status report as JSON.
fn render_json(facts: &phairdlna_core::diagnostics::Diagnostics) -> String {
    let roots_json = facts
        .roots
        .iter()
        .map(|r| {
            Json::map(vec![
                ("name".to_owned(), Json::text(&r.name)),
                ("mounted".to_owned(), Json::Bool(r.mounted)),
                ("objects".to_owned(), Json::number(r.objects.to_string())),
            ])
        })
        .collect();
    let mut map = vec![
        ("version".to_owned(), Json::text(&facts.version)),
        (
            "uptime_seconds".to_owned(),
            Json::number(facts.uptime_seconds.to_string()),
        ),
        (
            "catalogue_objects".to_owned(),
            Json::number(facts.catalogue_objects.to_string()),
        ),
        (
            "active_transfers".to_owned(),
            Json::number(facts.transfers.len().to_string()),
        ),
        (
            "last_scan".to_owned(),
            facts
                .last_scan
                .as_ref()
                .map(|s| Json::text(s.clone()))
                .unwrap_or(Json::Null),
        ),
        ("roots".to_owned(), Json::list(roots_json)),
    ];
    map.extend(json_status_network_fields(facts));
    format!("{}\n", Json::map(map))
}
