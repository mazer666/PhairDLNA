//! The command line of PhairDLNA.
//!
//! What: the commands a user can run, implemented as pure functions over their
//!       arguments so that they can be tested without starting a server.
//! Why: the headless server is the product (`PRD-006`), and a command that
//!      cannot be tested is a command that will be wrong.
//! Example: `run(ConfigCommand::Validate { .. })` prints what is wrong with a
//!      configuration file, or says that it is valid.

use std::path::PathBuf;

use phairdlna_core::config::{resolve_path, Config};

/// The commands the command line offers.
#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    /// Print the version and the licence.
    Version,
    /// Print the path the configuration would be read from.
    ConfigPath,
    /// Check a configuration file and report the first problem.
    ConfigValidate {
        /// The file to check. `None` means the resolved default location.
        path: Option<PathBuf>,
    },
    /// Print the effective configuration.
    ConfigShow {
        /// The file to read. `None` means the resolved default location.
        path: Option<PathBuf>,
    },
    /// Start the server. Not implemented yet.
    Serve {
        /// The configuration file to use.
        path: Option<PathBuf>,
    },
}

/// The result of a command: what to print, and the exit code.
#[derive(Debug, Clone, PartialEq)]
pub struct Outcome {
    /// The text to print on standard output.
    pub stdout: String,
    /// The exit code the process should return.
    pub exit_code: i32,
}

impl Outcome {
    /// A successful outcome with the given text.
    pub fn ok(stdout: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
            exit_code: 0,
        }
    }

    /// A failed outcome with the given text.
    pub fn failure(stdout: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
            exit_code: 1,
        }
    }
}

/// Runs a command and returns what it should print.
///
/// The function never panics and never exits the process: the caller decides
/// what to do with the outcome. That is what makes it testable.
pub fn run(command: &Command) -> Outcome {
    match command {
        Command::Version => version(),
        Command::ConfigPath => config_path(),
        Command::ConfigValidate { path } => config_validate(path.as_deref()),
        Command::ConfigShow { path } => config_show(path.as_deref()),
        Command::Serve { path } => serve(path.as_deref()),
    }
}

/// Prints the version and the licence.
fn version() -> Outcome {
    Outcome::ok(format!(
        "{} {} — AGPL-3.0-or-later\n{}\n",
        phairdlna_core::PROJECT_NAME,
        phairdlna_core::PROJECT_VERSION,
        "The DLNA server is not implemented yet. See docs/07-project/ROADMAP.md."
    ))
}

/// Prints where the configuration would be read from.
fn config_path() -> Outcome {
    Outcome::ok(format!("{}\n", resolve_path(None).display()))
}

/// The path a command works on: the given one, or the resolved default.
fn resolve(path: Option<&std::path::Path>) -> PathBuf {
    match path {
        Some(path) => path.to_path_buf(),
        None => resolve_path(None),
    }
}

/// Checks a configuration file.
fn config_validate(path: Option<&std::path::Path>) -> Outcome {
    let path = resolve(path);
    match Config::load(&path) {
        Ok(config) => Outcome::ok(format!(
            "the configuration \"{}\" is valid (schema version {})\n",
            path.display(),
            config.schema_version
        )),
        Err(error) => Outcome::failure(format!(
            "the configuration \"{}\" is not usable:\n{}\n",
            path.display(),
            error.message()
        )),
    }
}

/// Prints the effective configuration.
fn config_show(path: Option<&std::path::Path>) -> Outcome {
    let path = resolve(path);
    match Config::load(&path) {
        Ok(config) => match config.render() {
            Ok(text) => Outcome::ok(text),
            Err(error) => Outcome::failure(format!("cannot render the configuration: {error}\n")),
        },
        Err(error) => Outcome::failure(format!(
            "the configuration \"{}\" is not usable:\n{}\n",
            path.display(),
            error.message()
        )),
    }
}

/// Starts the server.
fn serve(path: Option<&std::path::Path>) -> Outcome {
    let path = resolve(path);
    Outcome::failure(format!(
        "the server is not implemented yet; the configuration \"{}\" was not loaded\n",
        path.display()
    ))
}
