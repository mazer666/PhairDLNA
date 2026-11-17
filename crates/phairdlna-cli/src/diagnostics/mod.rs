//! The diagnostics report of the server.
//!
//! What: the command that writes the report a user hands to a support ticket.
//! Why: a user who asks for help must not have to know what to collect, and
//!      the report must not carry the private parts of the machine by
//!      accident (`OPS-013`, `UX-011`, `SEC-094`).
//! Example: `phairdlna diagnostics` writes the report of the running server to
//!          the standard output, with the paths of the user replaced.
//!
//! **The report of a stopped server.** The command asks the administration
//! surface of the running server. A server that does not answer does not stop
//! the command: the report is written from the configuration of the machine,
//! and it says that the server is not running.

pub mod surface;

use std::path::{Path, PathBuf};

use phairdlna_core::config::Config;
use phairdlna_core::diagnostics::{render, Diagnostics, Report, RootState};
use phairdlna_core::json::Json;
use phairdlna_core::time;

use crate::Outcome;

/// What the command was asked to do.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Options {
    /// The configuration file to read.
    pub path: Option<PathBuf>,
    /// Whether the real paths of the machine may be shown.
    pub unredacted: bool,
    /// The file the report is written to, or `None` for the standard output.
    pub export: Option<PathBuf>,
    /// Whether the report is written as JSON instead of as text.
    pub json: bool,
}

/// Writes the report of the server.
///
/// The report is redacted unless the user asks for the real paths with
/// `--unredacted`: the flag is the consent of the owner of the machine
/// (`UX-011`).
pub fn run(options: &Options) -> Outcome {
    let path = crate::resolve(options.path.as_deref());
    let config = match Config::load(&path) {
        Ok(config) => config,
        Err(error) => {
            return Outcome::failure(format!(
                "the configuration \"{}\" is not usable:\n{}\n",
                path.display(),
                error.message()
            ))
        }
    };
    let (facts, report) = gather(&config);
    let text = if options.json {
        json_of(&facts, &report, !options.unredacted)
    } else {
        render(&facts, &report, !options.unredacted)
    };
    write(text, options.export.as_deref())
}

/// Asks the running server, and falls back to the machine itself.
///
/// The answer of the server is the truth about the server; the machine alone
/// knows the configuration and whether the server runs at all.
fn gather(config: &Config) -> (Diagnostics, Report) {
    match surface::ask(config) {
        Ok(answer) => {
            let mut facts = answer.diagnostics;
            fill_paths(&mut facts, config);
            let report = report_of(config, "running", &facts);
            (facts, report)
        }
        Err(reason) => {
            let facts = local_facts(config);
            let report = report_of(config, &format!("not running — {reason}"), &facts);
            (facts, report)
        }
    }
}

/// The facts of a server that does not answer.
///
/// The version and the revision are the ones of the command line itself: they
/// are the build that wrote the report, and the report says that the server
/// did not answer.
fn local_facts(config: &Config) -> Diagnostics {
    Diagnostics {
        version: phairdlna_core::PROJECT_VERSION.to_owned(),
        source_revision: phairdlna_core::SOURCE_REVISION.to_owned(),
        roots: roots_of(config),
        ..Diagnostics::default()
    }
}

/// The roots of the library, as the configuration names them.
///
/// Whether the folder is there is asked of the file system, so that the report
/// does not claim more than it knows.
fn roots_of(config: &Config) -> Vec<RootState> {
    config
        .library
        .roots
        .iter()
        .map(|root| RootState {
            name: root.name.clone(),
            path: root.path.clone(),
            mounted: Path::new(&root.path).exists(),
            objects: 0,
        })
        .collect()
}

/// Puts the real paths of the roots into the facts of the server.
///
/// The surface shows the roots with pseudonyms, because the surface is
/// reachable by every program of the machine. The command line sits on the
/// machine of the owner, and the owner may see the real paths (`UX-011`).
fn fill_paths(facts: &mut Diagnostics, config: &Config) {
    for root in &mut facts.roots {
        let Some(configured) = config
            .library
            .roots
            .iter()
            .find(|candidate| candidate.name == root.name)
        else {
            continue;
        };
        root.path = configured.path.clone();
    }
}

/// The context of the report.
fn report_of(config: &Config, state: &str, facts: &Diagnostics) -> Report {
    let _ = facts;
    Report {
        generated_at: time::now_utc(),
        platform: platform(),
        server_state: state.to_owned(),
        configuration: configuration_of(config),
        source_offer: None,
    }
}

/// The system the command runs on.
fn platform() -> String {
    format!("{} / {}", std::env::consts::OS, std::env::consts::ARCH)
}

/// The summary of the configuration, by name.
fn configuration_of(config: &Config) -> Vec<(String, String)> {
    let mut summary = vec![
        ("Device name".to_owned(), config.server.device_name.clone()),
        ("Device id".to_owned(), identity_of()),
        ("HTTP port".to_owned(), config.server.http_port.to_string()),
        ("SSDP port".to_owned(), config.server.ssdp_port.to_string()),
        (
            "Discovery".to_owned(),
            switch(config.server.discovery_enabled),
        ),
        (
            "Administration surface".to_owned(),
            if config.security.admin_enabled {
                format!("on, port {}", config.security.admin_port)
            } else {
                "off".to_owned()
            },
        ),
        (
            "Allowed clients".to_owned(),
            if config.security.allowed_clients.is_empty() {
                "every client of the network".to_owned()
            } else {
                config.security.allowed_clients.join(", ")
            },
        ),
        (
            "Log level".to_owned(),
            config.logging.level.as_str().to_owned(),
        ),
        (
            "Redaction in the log".to_owned(),
            switch(config.logging.redact),
        ),
        (
            "Maximum streams".to_owned(),
            config.limits.max_concurrent_streams.to_string(),
        ),
    ];
    for root in &config.library.roots {
        summary.push((format!("Root \"{}\"", root.name), root.path.clone()));
    }
    summary
}

/// The identity of the installation, if it has one.
///
/// The identity is read out of its file and is never created here: a report
/// must not change the machine it reports about.
fn identity_of() -> String {
    match std::fs::read_to_string(phairdlna_core::identity::identity_path()) {
        Ok(text) => {
            let identity = text.trim();
            if identity.is_empty() {
                "not created yet".to_owned()
            } else {
                identity.to_owned()
            }
        }
        Err(_) => "not created yet".to_owned(),
    }
}

/// A switch of the configuration, in words meant for a person.
fn switch(on: bool) -> String {
    if on {
        "on".to_owned()
    } else {
        "off".to_owned()
    }
}

/// The report as JSON.
///
/// The JSON holds the same facts as the text, so that a tool can read the
/// report of a ticket (`UX-011`).
fn json_of(facts: &Diagnostics, report: &Report, redacted: bool) -> String {
    let (facts, report) = if redacted {
        phairdlna_core::diagnostics::redact(facts, report)
    } else {
        (facts.clone(), report.clone())
    };
    build_json_report(&facts, &report).to_string()
}

/// Assembles the JSON map representing the diagnostics report.
fn build_json_report(facts: &Diagnostics, report: &Report) -> Json {
    Json::map(vec![
        (
            "generated_at".to_owned(),
            Json::text(report.generated_at.clone()),
        ),
        ("version".to_owned(), Json::text(facts.version.clone())),
        (
            "source_revision".to_owned(),
            Json::text(facts.source_revision.clone()),
        ),
        ("platform".to_owned(), Json::text(report.platform.clone())),
        (
            "server_state".to_owned(),
            Json::text(report.server_state.clone()),
        ),
        (
            "uptime_seconds".to_owned(),
            Json::number(facts.uptime_seconds.to_string()),
        ),
        (
            "catalogue_objects".to_owned(),
            Json::number(facts.catalogue_objects.to_string()),
        ),
        (
            "resident_bytes".to_owned(),
            Json::maybe_number(facts.resident_bytes),
        ),
        (
            "system_update_id".to_owned(),
            Json::number(u64::from(facts.system_update_id).to_string()),
        ),
        ("roots".to_owned(), json_of_roots(&facts.roots)),
        ("transfers".to_owned(), json_of_transfers(&facts.transfers)),
        ("records".to_owned(), json_of_records(&facts.records)),
        (
            "warnings".to_owned(),
            Json::list(facts.warnings.iter().cloned().map(Json::text).collect()),
        ),
        (
            "last_scan".to_owned(),
            match &facts.last_scan {
                Some(scan) => Json::text(scan.clone()),
                None => Json::Null,
            },
        ),
        (
            "configuration".to_owned(),
            json_of_configuration(&report.configuration),
        ),
    ])
}

/// Formats the configuration entries as JSON list.
fn json_of_configuration(configuration: &[(String, String)]) -> Json {
    Json::list(
        configuration
            .iter()
            .map(|(name, value)| {
                Json::map(vec![
                    ("name".to_owned(), Json::text(name.clone())),
                    ("value".to_owned(), Json::text(value.clone())),
                ])
            })
            .collect(),
    )
}

/// The roots of the library, as JSON.
fn json_of_roots(roots: &[RootState]) -> Json {
    Json::list(
        roots
            .iter()
            .map(|root| {
                Json::map(vec![
                    ("name".to_owned(), Json::text(root.name.clone())),
                    ("path".to_owned(), Json::text(root.path.clone())),
                    ("mounted".to_owned(), Json::Bool(root.mounted)),
                    ("objects".to_owned(), Json::number(root.objects.to_string())),
                ])
            })
            .collect(),
    )
}

/// The streams that are running, as JSON.
fn json_of_transfers(transfers: &[phairdlna_core::diagnostics::TransferInfo]) -> Json {
    Json::list(
        transfers
            .iter()
            .map(|transfer| {
                Json::map(vec![
                    ("client".to_owned(), Json::text(transfer.client.clone())),
                    ("address".to_owned(), Json::text(transfer.address.clone())),
                    ("title".to_owned(), Json::text(transfer.title.clone())),
                    (
                        "running_seconds".to_owned(),
                        Json::number(transfer.running_seconds.to_string()),
                    ),
                ])
            })
            .collect(),
    )
}

/// The last records of the log, as JSON.
fn json_of_records(records: &[phairdlna_core::diagnostics::LogRecord]) -> Json {
    Json::list(
        records
            .iter()
            .map(|record| {
                Json::map(vec![
                    ("time".to_owned(), Json::text(record.time.clone())),
                    ("level".to_owned(), Json::text(record.level.clone())),
                    ("component".to_owned(), Json::text(record.component.clone())),
                    ("message".to_owned(), Json::text(record.message.clone())),
                    (
                        "fields".to_owned(),
                        Json::map(
                            record
                                .fields
                                .iter()
                                .map(|(name, value)| (name.clone(), Json::text(value.clone())))
                                .collect::<Vec<(String, Json)>>(),
                        ),
                    ),
                ])
            })
            .collect(),
    )
}

/// Writes the report to the standard output, or to a file.
///
/// A file that is written holds the facts of the machine of the user, and it
/// is written so that only the owner may read it (`SEC-094`).
fn write(text: String, export: Option<&Path>) -> Outcome {
    match export {
        None => Outcome::ok(text),
        Some(path) => match write_private(path, &text) {
            Ok(()) => Outcome::ok(format!(
                "the report was written to \"{}\" (only you may read it)\n",
                path.display()
            )),
            Err(reason) => Outcome::failure(format!(
                "the report cannot be written to \"{}\": {reason}\n",
                path.display()
            )),
        },
    }
}

/// Writes a file that only the owner may read.
fn write_private(path: &Path, text: &str) -> Result<(), String> {
    use std::io::Write;
    let mut file = std::fs::File::create(path).map_err(|error| error.to_string())?;
    file.write_all(text.as_bytes())
        .map_err(|error| error.to_string())?;
    restrict_to_owner(path)
}

/// Restricts a file to its owner.
fn restrict_to_owner(path: &Path) -> Result<(), String> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let permissions = std::fs::Permissions::from_mode(0o600);
        std::fs::set_permissions(path, permissions).map_err(|error| error.to_string())
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        Ok(())
    }
}
