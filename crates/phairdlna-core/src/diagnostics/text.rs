//! The plain-text report of the diagnostics.
//!
//! What: the text that a user hands to a support ticket.
//! Why: a ticket needs a text that every reader can open, and it must not
//!      carry the private parts of the machine of the user unless the owner
//!      of the machine says so (`OPS-013`, `UX-011`, `SEC-094`).
//! Example: `phairdlna diagnostics` writes the report to the standard output;
//!          the path of the library appears as `<path:1>`.
//!
//! **The redaction happens before the report is written.** The report is
//! written from the facts of the server, and every fact that names a place on
//! the disk or a client is replaced by a pseudonym. A report that is written
//! with `--unredacted` says so in its first lines, so that the reader of the
//! report knows what it holds.

use super::{Diagnostics, LogRecord, RootState};

/// What the report says besides the state of the server.
///
/// The facts of the server live in [`Diagnostics`]; the caller adds what it
/// knows about the machine and about the configuration.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    /// When the report was written.
    pub generated_at: String,
    /// The system the server runs on.
    pub platform: String,
    /// What the server does, in words meant for a person.
    pub server_state: String,
    /// The summary of the configuration, by name.
    pub configuration: Vec<(String, String)>,
    /// Where the source of the server is offered, if it is offered.
    pub source_offer: Option<String>,
}

/// Writes the report of the server.
///
/// The report is redacted unless the caller says that the owner of the
/// machine may see the real paths.
pub fn render(diagnostics: &Diagnostics, report: &Report, redacted: bool) -> String {
    let (facts, context) = if redacted {
        super::redact(diagnostics, report)
    } else {
        (diagnostics.clone(), report.clone())
    };
    let mut text = String::new();
    write_head(&mut text, redacted);
    write_facts(&mut text, &facts, &context);
    write_configuration(&mut text, &context);
    write_library(&mut text, &facts);
    write_transfers(&mut text, &facts);
    write_resources(&mut text, &facts);
    write_warnings(&mut text, &facts);
    write_records(&mut text, &facts);
    text
}

/// Writes the resource and scheduler status summary.
fn write_resources(text: &mut String, diagnostics: &Diagnostics) {
    if let Some(res) = &diagnostics.resources {
        section(text, "The resource scheduler");
        text.push_str(res);
        text.push_str("\n\n");
    }
}

/// Writes the title of the report and the warning that belongs to it.
fn write_head(text: &mut String, redacted: bool) {
    text.push_str("PhairDLNA diagnostics report\n");
    text.push_str("============================\n\n");
    if redacted {
        text.push_str(
            "This report was written for a support ticket. The places on your disk and the\n\
             addresses of your clients are replaced by <path:1> and <address:1>.\n",
        );
    } else {
        text.push_str(
            "This report carries the real paths and addresses of this machine. Share it only\n\
             with the person who helps you.\n",
        );
    }
    text.push('\n');
}

/// Writes the facts about the server and the machine.
fn write_facts(text: &mut String, diagnostics: &Diagnostics, report: &Report) {
    let mut facts = vec![
        ("Made at".to_owned(), report.generated_at.clone()),
        ("Version".to_owned(), diagnostics.version.clone()),
        ("Source".to_owned(), diagnostics.source_revision.clone()),
        ("System".to_owned(), report.platform.clone()),
        ("Server".to_owned(), report.server_state.clone()),
        ("Uptime".to_owned(), duration(diagnostics.uptime_seconds)),
    ];
    if let Some(offer) = &report.source_offer {
        facts.push(("Source offer".to_owned(), offer.clone()));
    }
    section(text, "The report");
    write_lines(text, &facts);
    text.push('\n');
}

/// Writes the summary of the configuration.
fn write_configuration(text: &mut String, report: &Report) {
    if report.configuration.is_empty() {
        return;
    }
    section(text, "The configuration");
    write_lines(text, &report.configuration);
    text.push('\n');
}

/// Writes what the library holds.
fn write_library(text: &mut String, diagnostics: &Diagnostics) {
    let mut facts = vec![
        (
            "Objects".to_owned(),
            diagnostics.catalogue_objects.to_string(),
        ),
        (
            "Update number".to_owned(),
            diagnostics.system_update_id.to_string(),
        ),
    ];
    if let Some(resident) = diagnostics.resident_bytes {
        facts.insert(1, ("In memory".to_owned(), bytes(resident)));
    }
    if let Some(scan) = &diagnostics.last_scan {
        facts.push(("Last scan".to_owned(), scan.clone()));
    }
    for root in &diagnostics.roots {
        facts.push((format!("Root \"{}\"", root.name), root_text(root)));
    }
    section(text, "The library");
    write_lines(text, &facts);
    text.push('\n');
}

/// Writes the streams that are running.
fn write_transfers(text: &mut String, diagnostics: &Diagnostics) {
    if diagnostics.transfers.is_empty() {
        return;
    }
    section(text, "The running streams");
    for transfer in &diagnostics.transfers {
        text.push_str(&format!(
            "- {} — {} — {} ({})\n",
            transfer.client,
            transfer.address,
            transfer.title,
            duration(transfer.running_seconds)
        ));
    }
    text.push('\n');
}

/// Writes the warnings the server holds.
fn write_warnings(text: &mut String, diagnostics: &Diagnostics) {
    if diagnostics.warnings.is_empty() {
        return;
    }
    section(text, "The warnings");
    for warning in &diagnostics.warnings {
        text.push_str(&format!("- {warning}\n"));
    }
    text.push('\n');
}

/// Writes the last records of the log.
fn write_records(text: &mut String, diagnostics: &Diagnostics) {
    if diagnostics.records.is_empty() {
        return;
    }
    section(text, "The last records of the log");
    for record in &diagnostics.records {
        text.push_str(&record_text(record));
    }
    text.push('\n');
}

/// One record of the log, as a line of the report.
fn record_text(record: &LogRecord) -> String {
    let mut line = format!(
        "{}  {}  {}  {}\n",
        record.time, record.level, record.component, record.message
    );
    for (name, item) in &record.fields {
        line.push_str(&format!("    {name} = {item}\n"));
    }
    line
}

/// One root of the library, as a line of the report.
fn root_text(root: &RootState) -> String {
    let mounted = if root.mounted {
        "mounted"
    } else {
        "not mounted"
    };
    format!("{} objects, {mounted} — {}", root.objects, root.path)
}

/// A section of the report.
fn section(text: &mut String, title: &str) {
    text.push_str(title);
    text.push('\n');
    text.push_str(&"-".repeat(title.chars().count()));
    text.push('\n');
}

/// The lines of a section, in one column.
///
/// The names are as wide as the longest of them, so that the values of a
/// section begin in the same column and the report can be read at a glance.
fn write_lines(text: &mut String, facts: &[(String, String)]) {
    let width = facts
        .iter()
        .map(|(name, _)| name.chars().count())
        .max()
        .unwrap_or(0);
    for (name, value) in facts {
        text.push_str(&format!("{name:<width$}  {value}\n", width = width + 1));
    }
}

/// A duration in seconds, in words meant for a person.
fn duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    if hours > 0 {
        format!("{hours} h {minutes} min")
    } else if minutes > 0 {
        format!("{minutes} min")
    } else {
        format!("{seconds} s")
    }
}

/// A number of bytes, in words meant for a person.
fn bytes(value: u64) -> String {
    const MEBIBYTE: u64 = 1024 * 1024;
    const KIBIBYTE: u64 = 1024;
    if value >= MEBIBYTE {
        format!("{:.1} MiB", value as f64 / MEBIBYTE as f64)
    } else if value >= KIBIBYTE {
        format!("{:.1} KiB", value as f64 / KIBIBYTE as f64)
    } else {
        format!("{value} B")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The facts of a server for the tests of the report.
    fn diagnostics() -> Diagnostics {
        Diagnostics {
            version: "0.1.0".to_owned(),
            source_revision: "abc1234".to_owned(),
            uptime_seconds: 3_720,
            catalogue_objects: 128,
            resident_bytes: Some(2 * 1024 * 1024),
            system_update_id: 3,
            roots: vec![RootState {
                name: "Films".to_owned(),
                path: "/Users/owner/Movies".to_owned(),
                mounted: true,
                objects: 128,
            }],
            transfers: vec![super::super::TransferInfo {
                client: "Telecast".to_owned(),
                address: "192.168.1.20".to_owned(),
                title: "A film".to_owned(),
                running_seconds: 5,
            }],
            records: vec![LogRecord {
                time: "10:00:00".to_owned(),
                level: "info".to_owned(),
                component: "catalogue".to_owned(),
                message: "the library was scanned".to_owned(),
                fields: vec![("objects".to_owned(), "128".to_owned())],
            }],
            warnings: vec!["a warning".to_owned()],
            resources: Some("3 workers, machine calm".to_owned()),
            timeline: Some("2026-11-26T08:00:00Z ready reason=\"listening\"".to_owned()),
            last_scan: Some("10:00:00".to_owned()),
        }
    }

    /// The context of a report for the tests.
    fn report() -> Report {
        Report {
            generated_at: "2026-11-17 10:58 CET".to_owned(),
            platform: "macOS 27.2 (aarch64)".to_owned(),
            server_state: "running".to_owned(),
            configuration: vec![
                ("Device name".to_owned(), "Living room".to_owned()),
                ("Root path".to_owned(), "/Users/owner/Movies".to_owned()),
            ],
            source_offer: Some("http://192.168.1.10:8200/source".to_owned()),
        }
    }

    /// The positive case: the report names the facts of the server.
    #[test]
    fn the_report_names_the_facts_of_the_server() {
        let text = render(&diagnostics(), &report(), true);
        assert!(text.contains("PhairDLNA diagnostics report"));
        assert!(text.contains("Version"));
        assert!(text.contains("0.1.0"));
        assert!(text.contains("abc1234"));
        assert!(text.contains("The library"));
        assert!(text.contains("Objects"));
        assert!(text.contains("128"));
        assert!(text.contains("the library was scanned"));
        assert!(text.contains("a warning"));
    }

    /// The positive case: the redacted report holds no path of the user.
    #[test]
    fn the_redacted_report_holds_no_path_of_the_user() {
        let text = render(&diagnostics(), &report(), true);
        assert!(!text.contains("/Users/owner/Movies"), "{text}");
        assert!(!text.contains("192.168.1.20"), "{text}");
        assert!(text.contains("<path:1>"), "{text}");
        assert!(text.contains("<address:1>"), "{text}");
    }

    /// The positive case: the report of the owner carries the real paths.
    #[test]
    fn the_unredacted_report_carries_the_real_paths() {
        let text = render(&diagnostics(), &report(), false);
        assert!(text.contains("/Users/owner/Movies"), "{text}");
        assert!(text.contains("192.168.1.20"), "{text}");
        assert!(text.contains("real paths"), "{text}");
    }

    /// The negative case: a server without a library still has a report.
    #[test]
    fn a_server_without_a_library_still_has_a_report() {
        let text = render(&Diagnostics::default(), &report(), true);
        assert!(text.contains("PhairDLNA diagnostics report"));
        assert!(text.contains("The library"));
        assert!(!text.contains("The running streams"));
        assert!(!text.contains("The warnings"));
    }
}
