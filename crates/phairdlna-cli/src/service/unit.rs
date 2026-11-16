//! The `systemd` unit file of the server.
//!
//! What: the unit that describes the server to `systemd`, and the reader that
//!       reads it back.
//! Why: the unit is what `systemctl` acts on, so it must say how the server is
//!      started, where its logs go and that it is restarted when it dies
//!      (`OPS-009`).
//! Example: the unit declares `Type=notify`, so that `systemctl start` returns
//!      only when the server can really answer.
//!
//! **`Type=notify`.** The daemon sends the readiness notification only after
//! its listeners are open, so that the service manager does not report a
//! server that is not there yet (`OPS-009`, `AC-12`).

use std::collections::BTreeMap;

use super::layout::{Scope, ServiceLayout};

/// The permissions of the unit file: everybody may read it, nobody but the
/// owner may write it.
pub const UNIT_FILE_MODE: u32 = 0o644;

/// Renders the unit file of the server.
pub fn render(layout: &ServiceLayout) -> String {
    let mut unit = String::new();
    unit.push_str("[Unit]\n");
    unit.push_str("Description=PhairDLNA media server\n");
    unit.push_str("Documentation=https://github.com/mazer666/PhairDLNA\n");
    unit.push_str("After=network-online.target\n");
    unit.push_str("Wants=network-online.target\n\n");
    unit.push_str("[Service]\n");
    unit.push_str("Type=notify\n");
    unit.push_str("NotifyAccess=main\n");
    unit.push_str(&format!("ExecStart={}\n", exec_start(layout)));
    unit.push_str(&format!(
        "WorkingDirectory={}\n",
        quote(&layout.working_directory.to_string_lossy())
    ));
    for (name, value) in &layout.environment {
        unit.push_str(&format!(
            "Environment={}\n",
            quote(&format!("{name}={value}"))
        ));
    }
    unit.push_str(&format!(
        "StandardOutput=append:{}\n",
        quote(&layout.stdout_path.to_string_lossy())
    ));
    unit.push_str(&format!(
        "StandardError=append:{}\n",
        quote(&layout.stderr_path.to_string_lossy())
    ));
    unit.push_str("Restart=on-failure\n");
    unit.push_str("RestartSec=5\n");
    unit.push_str("TimeoutStopSec=30\n");
    if layout.scope == Scope::System {
        unit.push_str(&format!("User={}\n", quote(&layout.user)));
    }
    unit.push('\n');
    unit.push_str("[Install]\n");
    match layout.scope {
        Scope::User => unit.push_str("WantedBy=default.target\n"),
        Scope::System => unit.push_str("WantedBy=multi-user.target\n"),
    }
    unit
}

/// The command line of the service, with every part quoted.
fn exec_start(layout: &ServiceLayout) -> String {
    let mut parts = vec![quote(&layout.program.to_string_lossy())];
    for argument in &layout.arguments {
        parts.push(quote(argument));
    }
    parts.join(" ")
}

/// Quotes a value for `systemd`.
///
/// A path with a space in it would otherwise be read as two words, and the
/// server would be started with the wrong arguments.
pub fn quote(value: &str) -> String {
    let mut quoted = String::with_capacity(value.len() + 2);
    quoted.push('"');
    for character in value.chars() {
        match character {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '$' => quoted.push_str("\\$"),
            other => quoted.push(other),
        }
    }
    quoted.push('"');
    quoted
}

/// A unit file: the entries of its sections.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UnitDocument {
    /// The entries, by section and by key.
    pub sections: BTreeMap<String, BTreeMap<String, String>>,
}

impl UnitDocument {
    /// The value of an entry.
    pub fn value(&self, section: &str, key: &str) -> Option<&str> {
        self.sections
            .get(section)
            .and_then(|entries| entries.get(key))
            .map(String::as_str)
    }
}

/// Reads a unit file.
///
/// A file that cannot be read is refused: a unit that was only half read is a
/// unit the command cannot compare with the one it wrote.
pub fn parse(text: &str) -> Result<UnitDocument, String> {
    let mut document = UnitDocument::default();
    let mut section: Option<String> = None;
    for (number, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }
        if let Some(name) = section_name(line) {
            section = Some(name.clone());
            document.sections.entry(name).or_default();
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            return Err(format!(
                "the line {} of the unit file is not an entry",
                number + 1
            ));
        };
        let key = key.trim();
        if key.is_empty() {
            return Err(format!(
                "the line {} of the unit file has no name",
                number + 1
            ));
        }
        let Some(section) = section.clone() else {
            return Err(format!(
                "the line {} of the unit file is not in a section",
                number + 1
            ));
        };
        document
            .sections
            .entry(section)
            .or_default()
            .insert(key.to_owned(), value.trim().to_owned());
    }
    if document.sections.is_empty() {
        return Err("the unit file has no section".to_owned());
    }
    Ok(document)
}

/// The name of a section, if the line is one.
fn section_name(line: &str) -> Option<String> {
    let name = line.strip_prefix('[')?.strip_suffix(']')?;
    Some(name.trim().to_owned())
}

#[cfg(test)]
mod tests {
    use super::{parse, quote, render, UnitDocument};
    use crate::service::layout::{LayoutRequest, Platform, Scope, ServiceLayout};
    use std::path::PathBuf;

    /// The program of the service, for the tests.
    fn a_program() -> PathBuf {
        std::env::current_exe().expect("the running program")
    }

    /// A layout on Linux, for the tests.
    fn a_layout() -> ServiceLayout {
        let request = LayoutRequest {
            platform: Platform::Linux,
            scope: Scope::User,
            home: PathBuf::from("/home/tester"),
            program: a_program(),
            config: PathBuf::from("/home/tester/.config/phairdlna/config.toml"),
        };
        ServiceLayout::build(&request).expect("a layout")
    }

    /// The positive case: the unit declares the readiness notification.
    #[test]
    fn the_unit_declares_the_readiness_notification() {
        let unit = render(&a_layout());

        assert!(unit.contains("Type=notify"), "{unit}");
        assert!(unit.contains("NotifyAccess=main"), "{unit}");
    }

    /// The positive case: the unit starts the server with its configuration.
    #[test]
    fn the_unit_starts_the_server_with_its_configuration() {
        let unit = render(&a_layout());
        let document = parse(&unit).expect("the unit is read");

        let command = document.value("Service", "ExecStart").expect("the command");
        assert!(
            command.contains(&*a_program().to_string_lossy()),
            "{command}"
        );
        assert!(command.contains("serve"), "{command}");
        assert!(command.contains("--config"), "{command}");
    }

    /// The positive case: the unit names the logs and restarts the server.
    #[test]
    fn the_unit_names_the_logs_and_restarts_the_server() {
        let unit = render(&a_layout());
        let document = parse(&unit).expect("the unit is read");

        assert!(document
            .value("Service", "StandardOutput")
            .expect("the output")
            .contains("server.out.log"));
        assert_eq!(document.value("Service", "Restart"), Some("on-failure"));
        assert_eq!(
            document.value("Install", "WantedBy"),
            Some("default.target")
        );
    }

    /// The negative case: a value with a space is quoted.
    #[test]
    fn a_value_with_a_space_is_quoted() {
        assert_eq!(quote("/Users/a b/phairdlna"), "\"/Users/a b/phairdlna\"");
        assert_eq!(quote("a\"b"), "\"a\\\"b\"");
        assert_eq!(quote("$HOME"), "\"\\$HOME\"");
    }

    /// The negative case: a line that is not an entry is refused.
    #[test]
    fn a_line_that_is_not_an_entry_is_refused() {
        let error = parse("[Service]\nthis is not an entry\n").expect_err("the unit is refused");

        assert!(error.contains("not an entry"), "{error}");
    }

    /// The negative case: a file without a section is refused.
    #[test]
    fn a_file_without_a_section_is_refused() {
        let error = parse("Type=notify\n").expect_err("the unit is refused");

        assert!(error.contains("not in a section"), "{error}");
    }

    /// The positive case: a comment is not an entry.
    #[test]
    fn a_comment_is_not_an_entry() {
        let document: UnitDocument =
            parse("# a comment\n[Service]\nType=notify\n").expect("the unit is read");

        assert_eq!(document.value("Service", "Type"), Some("notify"));
    }
}
