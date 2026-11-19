//! Rendering, validating and writing the setup configuration.
//!
//! What: convert setup answers to a documented settings file and validate it with the
//!       server's configuration reader before writing.
//! Why: setup must never create a file the daemon rejects.
//! Example: `render` produces a candidate and `check` validates that candidate.

use std::path::Path;

use super::Answers;
use phairdlna_core::config::Config;

/// The configuration file the answers describe.
/// The file explains every setting it holds: a user who opens it later must
/// understand it without the documentation (`UX-010`).
pub fn render(answers: &Answers) -> String {
    let mut text = String::new();
    text.push_str("# The configuration of PhairDLNA.\n");
    text.push_str("#\n");
    text.push_str("# This file was written by \"phairdlna-cli setup\". Every setting can be\n");
    text.push_str("# changed by hand; the server reads the file when it starts.\n");
    text.push_str("#\n");
    text.push_str("# The settings and what they mean: phairdlna config show\n\n");
    text.push_str("# The version of the format of this file. Do not change it.\n");
    text.push_str("schema_version = 1\n\n");
    text.push_str("[server]\n");
    text.push_str("# The name your television shows for this server.\n");
    text.push_str(&format!(
        "device_name = {}\n\n",
        quoted(&answers.device_name)
    ));
    text.push_str("# The port the server listens on. 8200 is the usual one.\n");
    text.push_str("http_port = 8200\n\n");
    text.push_str("# The port the announcements go out on. 1900 is the one the standard uses.\n");
    text.push_str("ssdp_port = 1900\n\n");
    text.push_str("[library]\n");
    if answers.roots.is_empty() {
        text.push_str("# No folder is shared yet. Add one like this:\n");
        text.push_str("#\n");
        text.push_str("# [[library.roots]]\n");
        text.push_str("# name = \"Movies\"\n");
        text.push_str("# path = \"/Users/you/Movies\"\n\n");
    } else {
        text.push_str("# The folders that hold your media. You may add more.\n");
        for (name, path) in &answers.roots {
            text.push_str("[[library.roots]]\n");
            text.push_str(&format!("name = {}\n", quoted(name)));
            text.push_str(&format!("path = {}\n\n", quoted(path)));
        }
    }
    text.push_str("[security]\n");
    text.push_str("# The PhairDLNA app uses the local connection at http://127.0.0.1:8210 .\n");
    text.push_str("# It shows server status and diagnostics and can refresh your library.\n");
    text.push_str("# Only programs on this Mac can connect. The file\n");
    text.push_str("# ~/.config/phairdlna/admin.token is the key it asks for. Set this to false\n");
    text.push_str("# to switch off the local connection.\n");
    text.push_str("admin_enabled = true\n\n");
    text.push_str("[logging]\n");
    text.push_str("# How much the server writes to its log: debug, info, warn or error.\n");
    text.push_str("level = \"info\"\n\n");
    text.push_str("# The log replaces the paths and the addresses of your machine with\n");
    text.push_str("# names like <path:1>. Keep this on unless you know why you turn it off.\n");
    text.push_str("redact = true\n");
    text
}

/// A text as a value of the configuration.
///
/// A text that carries a quote or a backslash is written with the escapes the
/// format of the file knows.
fn quoted(text: &str) -> String {
    let mut value = String::with_capacity(text.len() + 2);
    value.push('"');
    for character in text.chars() {
        match character {
            '"' => value.push_str("\\\""),
            '\\' => value.push_str("\\\\"),
            '\n' => value.push_str("\\n"),
            other => value.push(other),
        }
    }
    value.push('"');
    value
}

/// Writes the configuration file of the answers.
///
/// The file is checked before it is written: a file that the server would
/// refuse is never written (`CFG-002`).
pub fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(folder) = path.parent() {
        if !folder.as_os_str().is_empty() && !folder.exists() {
            std::fs::create_dir_all(folder)
                .map_err(|error| format!("the folder cannot be created: {error}"))?;
        }
    }
    std::fs::write(path, text).map_err(|error| format!("the file cannot be written: {error}"))
}

/// Checks that the server can read the configuration.
///
/// The check is the reader of the server itself: the helper must not write a
/// file that the server would refuse.
pub fn check(text: &str) -> Result<(), String> {
    let path = std::env::temp_dir().join(format!("phairdlna-check-{}.toml", unique()));
    std::fs::write(&path, text).map_err(|error| format!("the check cannot be written: {error}"))?;
    let answer = Config::load(&path)
        .map(|_| ())
        .map_err(|error| format!("the configuration cannot be used:\n{}", error.message()));
    let _ = std::fs::remove_file(&path);
    answer
}

/// A number that makes the name of the file of the check unique.
fn unique() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_nanos())
        .unwrap_or_default()
}
