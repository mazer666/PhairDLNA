//! The `launchd` job file of the server.
//!
//! What: the property list that describes the job of the server on macOS.
//! Why: the property list is what `launchd` reads when the machine starts, so
//!      it must say exactly how the server is started, where its logs go and
//!      that it must be restarted when it dies (`OPS-009`, `PLT-013`).
//! Example: the file names the program, the arguments, the logs, and says
//!      `KeepAlive` and `RunAtLoad`.
//!
//! **The file is written with the permissions `644`.** A job file is read by
//! the system and by the user, and it must not be writable by anybody else
//! (`SEC-016`).

use super::layout::{Scope, ServiceLayout};

/// The permissions of the job file: everybody may read it, nobody but the
/// owner may write it.
pub const UNIT_FILE_MODE: u32 = 0o644;

/// The head of a property list of `launchd`.
const HEAD: &str = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n\
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">\n\
<plist version=\"1.0\">";

/// The foot of a property list of `launchd`.
const FOOT: &str = "</plist>";

/// Renders the job file of the server.
///
/// The file is rendered from the layout, so that the program, the arguments
/// and the paths are the ones the command decided on — never a copy that has
/// to be kept in step by hand.
pub fn render(layout: &ServiceLayout) -> String {
    let mut file = String::new();
    file.push_str(HEAD);
    file.push_str("\n<dict>\n");
    file.push_str(&string_entry("Label", &layout.label));
    file.push_str("  <key>ProgramArguments</key>\n  <array>\n");
    file.push_str(&array_item(&layout.program.to_string_lossy()));
    for argument in &layout.arguments {
        file.push_str(&array_item(argument));
    }
    file.push_str("  </array>\n");
    file.push_str(&string_entry(
        "WorkingDirectory",
        &layout.working_directory.to_string_lossy(),
    ));
    file.push_str("  <key>EnvironmentVariables</key>\n  <dict>\n");
    for (name, value) in &layout.environment {
        file.push_str(&string_entry(name, value));
    }
    file.push_str("  </dict>\n");
    file.push_str(&string_entry(
        "StandardOutPath",
        &layout.stdout_path.to_string_lossy(),
    ));
    file.push_str(&string_entry(
        "StandardErrorPath",
        &layout.stderr_path.to_string_lossy(),
    ));
    file.push_str("  <key>KeepAlive</key>\n  <true/>\n");
    file.push_str("  <key>RunAtLoad</key>\n  <true/>\n");
    file.push_str("  <key>ProcessType</key>\n  <string>Background</string>\n");
    if layout.scope == Scope::System {
        file.push_str(&string_entry("UserName", &layout.user));
    }
    file.push_str("</dict>\n");
    file.push_str(FOOT);
    file.push('\n');
    file
}

/// One entry of the dictionary: a key and a string.
fn string_entry(key: &str, value: &str) -> String {
    format!(
        "  <key>{}</key>\n  <string>{}</string>\n",
        escape(key),
        escape(value)
    )
}

/// One item of the array of the arguments.
fn array_item(value: &str) -> String {
    format!("    <string>{}</string>\n", escape(value))
}

/// Escapes the characters that a property list may not carry literally.
///
/// A path may carry an ampersand or a quote; a job file that is not escaped
/// cannot be read by `launchd`, and the server would simply never start.
pub fn escape(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            other => escaped.push(other),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::{escape, render, UNIT_FILE_MODE};
    use crate::service::layout::{LayoutRequest, Platform, Scope, ServiceLayout};
    use std::path::PathBuf;

    /// The program of the service, for the tests.
    fn a_program() -> PathBuf {
        std::env::current_exe().expect("the running program")
    }

    /// A layout on macOS, for the tests.
    fn a_layout() -> ServiceLayout {
        let request = LayoutRequest {
            platform: Platform::MacOs,
            scope: Scope::User,
            home: PathBuf::from("/Users/tester"),
            program: a_program(),
            config: PathBuf::from("/Users/tester/.config/phairdlna/config.toml"),
        };
        ServiceLayout::build(&request).expect("a layout")
    }

    /// The positive case: the job file names the program and the arguments.
    #[test]
    fn the_job_file_names_the_program_and_the_arguments() {
        let file = render(&a_layout());

        assert!(file.contains("<key>Label</key>"), "{file}");
        assert!(
            file.contains("<string>org.phairdlna.server</string>"),
            "{file}"
        );
        assert!(
            file.contains(&format!(
                "<string>{}</string>",
                a_program().to_string_lossy()
            )),
            "{file}"
        );
        assert!(file.contains("<string>serve</string>"), "{file}");
        assert!(file.contains("<string>--config</string>"), "{file}");
    }

    /// The positive case: the job file restarts the server and starts it at
    /// once.
    #[test]
    fn the_job_file_restarts_the_server() {
        let file = render(&a_layout());

        assert!(file.contains("<key>KeepAlive</key>"), "{file}");
        assert!(file.contains("<true/>"), "{file}");
        assert!(file.contains("<key>RunAtLoad</key>"), "{file}");
        assert!(file.contains("<key>ProcessType</key>"), "{file}");
        assert!(file.contains("<string>Background</string>"), "{file}");
    }

    /// The positive case: the job file names the logs and the working folder.
    #[test]
    fn the_job_file_names_the_logs() {
        let file = render(&a_layout());

        assert!(file.contains("<key>StandardOutPath</key>"), "{file}");
        assert!(file.contains("<key>StandardErrorPath</key>"), "{file}");
        assert!(file.contains("<key>WorkingDirectory</key>"), "{file}");
        assert!(
            file.contains("/Users/tester/Library/Logs/PhairDLNA/server.out.log"),
            "{file}"
        );
    }

    /// The negative case: a value with an ampersand is escaped.
    #[test]
    fn a_value_with_an_ampersand_is_escaped() {
        assert_eq!(escape("a & b"), "a &amp; b");
        assert_eq!(escape("<tag>"), "&lt;tag&gt;");
        assert_eq!(escape("say \"hi\""), "say &quot;hi&quot;");
    }

    /// The permissions of the job file are the ones the standards name.
    #[test]
    fn the_job_file_is_readable_by_everybody() {
        assert_eq!(UNIT_FILE_MODE, 0o644);
    }
}
