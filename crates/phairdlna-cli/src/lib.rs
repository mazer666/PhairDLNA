//! The command line of PhairDLNA.
//!
//! What: the commands a user can run, implemented as pure functions over their
//!       arguments so that they can be tested without starting a server.
//! Why: the headless server is the product (`PRD-006`), and a command that
//!      cannot be tested is a command that will be wrong.
//! Example: `run(ConfigCommand::Validate { .. })` prints what is wrong with a
//!      configuration file, or says that it is valid.

mod catalogue;
pub mod describe;
pub mod diagnostics;
pub mod explain;
pub mod interfaces;
pub mod scan;
pub mod serve;
pub mod service;
pub mod setup;
pub mod ssdp_cmd;
pub mod status;

use std::path::PathBuf;

use phairdlna_core::config::{resolve_path, Config};

/// The address the documents are built with.
///
/// The address is a placeholder: the server replaces it with the address of the
/// interface a request arrived on. The commands print it so that the shape of
/// the document can be looked at (`INT-065`).
pub const BASE_URL: &str = "http://192.168.0.10:8200";

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
    /// Export the effective configuration.
    ConfigExport {
        /// The file to read. `None` means the resolved default location.
        path: Option<PathBuf>,
    },
    /// Read a configuration file and print the effective configuration it
    /// would become, or apply it to a target path.
    ConfigImport {
        /// The file to read.
        path: PathBuf,
        /// The target path to write the imported configuration to.
        target: Option<PathBuf>,
    },
    /// Print an explanation of every configuration setting and its tier.
    ConfigExplain {
        /// Whether expert-level settings are included.
        expert: bool,
        /// Whether the output is formatted as JSON.
        json: bool,
    },
    /// Print the SSDP headers the server would send.
    SsdpHeaders {
        /// The configuration file to read.
        path: Option<PathBuf>,
    },
    /// Replay stored M-SEARCH requests and inspect the responses (`INT-103`).
    SsdpReplay {
        /// The file or directory containing stored M-SEARCH requests.
        corpus: PathBuf,
        /// The configuration file to read.
        path: Option<PathBuf>,
    },
    /// Print the tree of the library with the identifiers a client would see.
    CatalogueScan {
        /// The configuration file to read.
        path: Option<PathBuf>,
    },
    /// Print the document a client would receive for a container.
    CatalogueBrowse {
        /// The identifier of the container to browse.
        object_id: String,
        /// The configuration file to read.
        path: Option<PathBuf>,
    },
    /// Start the server in the foreground.
    Serve {
        /// The configuration file to use.
        path: Option<PathBuf>,
        /// The port to listen on, or `None` for the configured one.
        port: Option<u16>,
    },
    /// Manage the server as a service of the operating system.
    Service {
        /// What the user asked the service to do.
        command: service::ServiceCommand,
    },
    /// Write the report a user attaches to a support ticket.
    Diagnostics {
        /// The configuration file to read.
        path: Option<PathBuf>,
        /// Whether the real paths of the machine may be shown.
        unredacted: bool,
        /// The file the report is written to, or `None` for the standard output.
        export: Option<PathBuf>,
        /// Whether the report is written as JSON instead of as text.
        json: bool,
    },
    /// Ask the questions of the first run and write the configuration.
    Setup {
        /// The configuration file to write.
        path: Option<PathBuf>,
        /// Whether a configuration that is already there may be replaced.
        force: bool,
    },
    /// Print the device description and the SSDP headers it would advertise (`OPS-005`, `INT-015`).
    Describe {
        /// The configuration file to read.
        path: Option<PathBuf>,
    },
    /// List the known network interfaces and their usage (`OPS-004`).
    Interfaces {
        /// The configuration file to read.
        path: Option<PathBuf>,
        /// Whether the output is formatted as JSON.
        json: bool,
    },
    /// Print the server status (`OPS-002`, `OPS-014`).
    Status {
        /// The configuration file to read.
        path: Option<PathBuf>,
        /// Whether the output is formatted as JSON.
        json: bool,
    },
    /// Trigger an on-demand scan of the library (`OPS-003`).
    Scan {
        /// The configuration file to read.
        path: Option<PathBuf>,
        /// Optional specific root name or path to scan.
        root: Option<String>,
    },
}

/// The exit code of a command that failed (`OPS-014`).
pub const EXIT_FAILURE: i32 = 1;

/// The exit code of a command whose arguments or configuration were wrong
/// (`OPS-014`).
pub const EXIT_CONFIGURATION: i32 = 2;

/// The exit code of a command that had nothing to do (`OPS-014`).
///
/// A script that asks a stopped server how it is can tell "not running" from
/// "misconfigured" without reading the words of the answer.
pub const EXIT_NOTHING_TO_DO: i32 = 3;

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
            exit_code: EXIT_FAILURE,
        }
    }

    /// An outcome that says the arguments or the configuration were wrong.
    pub fn configuration_error(stdout: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
            exit_code: EXIT_CONFIGURATION,
        }
    }

    /// An outcome that says there was nothing to do.
    pub fn nothing_to_do(stdout: impl Into<String>) -> Self {
        Self {
            stdout: stdout.into(),
            exit_code: EXIT_NOTHING_TO_DO,
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
        Command::ConfigExport { path } => config_export(path.as_deref()),
        Command::ConfigImport { path, target } => config_import(path, target.as_deref()),
        Command::ConfigExplain { expert, json } => explain::explain(*expert, *json),
        Command::SsdpHeaders { path } => ssdp_cmd::headers(path.as_deref()),
        Command::SsdpReplay { corpus, path } => ssdp_cmd::replay(corpus.as_path(), path.as_deref()),
        Command::CatalogueScan { path } => catalogue::scan(&resolve(path.as_deref())),
        Command::CatalogueBrowse { object_id, path } => {
            catalogue::browse(&resolve(path.as_deref()), object_id)
        }
        Command::Serve { path, port } => serve(path.as_deref(), *port),
        Command::Service { command } => service::run(command),
        Command::Diagnostics {
            path,
            unredacted,
            export,
            json,
        } => diagnostics::run(&diagnostics::Options {
            path: path.clone(),
            unredacted: *unredacted,
            export: export.clone(),
            json: *json,
        }),
        Command::Setup { path, force } => setup::run(path.as_deref(), *force),
        Command::Describe { path } => describe::describe(path.as_deref()),
        Command::Interfaces { path, json } => interfaces::interfaces(path.as_deref(), *json),
        Command::Status { path, json } => status::status(&status::StatusOptions {
            path: path.clone(),
            json: *json,
        }),
        Command::Scan { path, root } => scan::scan(&scan::ScanOptions {
            path: path.clone(),
            root: root.clone(),
        }),
    }
}

/// Prints the version and the licence.
fn version() -> Outcome {
    Outcome::ok(format!(
        "{} {} — AGPL-3.0-or-later\n{}\n",
        phairdlna_core::PROJECT_NAME,
        phairdlna_core::PROJECT_VERSION,
        "The server speaks DLNA on the local network. See docs/07-project/ROADMAP.md."
    ))
}

/// Prints where the configuration would be read from.
fn config_path() -> Outcome {
    Outcome::ok(format!("{}\n", resolve_path(None).display()))
}

/// The path a command works on: the given one, or the resolved default.
pub(crate) fn resolve(path: Option<&std::path::Path>) -> PathBuf {
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

/// Exports the effective configuration.
///
/// The export is the configuration as the server understands it: every setting
/// that was left out is written down with its default, and the schema version
/// is the one of the file it was read from (`CFG-012`).
fn config_export(path: Option<&std::path::Path>) -> Outcome {
    config_show(path)
}

/// Reads a configuration file and prints the configuration it would become,
/// or writes it atomically to the target path when one is given.
///
/// When `target` is `None`, the command writes nothing: it tells the user what
/// the file would look like after the import (`CFG-012`). When `target` is
/// `Some(dest)`, the imported configuration is saved to `dest` preserving the
/// schema version (`CFG-012`).
fn config_import(path: &std::path::Path, target: Option<&std::path::Path>) -> Outcome {
    let config = match Config::load(path) {
        Ok(c) => c,
        Err(error) => {
            return Outcome::failure(format!(
                "the configuration \"{}\" is not usable:\n{}\n",
                path.display(),
                error.message()
            ))
        }
    };
    if let Some(target_path) = target {
        match config.save(target_path) {
            Ok(()) => Outcome::ok(format!(
                "the configuration from \"{}\" was written to \"{}\" (schema version {})\n",
                path.display(),
                target_path.display(),
                config.schema_version
            )),
            Err(error) => Outcome::failure(format!(
                "cannot write configuration to \"{}\": {}\n",
                target_path.display(),
                error.message()
            )),
        }
    } else {
        match config.render() {
            Ok(text) => Outcome::ok(format!(
                "the configuration \"{}\" would become (schema version {}):\n{text}",
                path.display(),
                config.schema_version
            )),
            Err(error) => Outcome::failure(format!("cannot render the configuration: {error}\n")),
        }
    }
}

/// Loads the persistent device identity.
pub(crate) fn load_identity(
    config: &Config,
) -> std::io::Result<phairdlna_core::identity::InstalledIdentity> {
    let path = phairdlna_core::identity::identity_path();
    phairdlna_core::identity::load_or_create(&path, &config.server.device_name)
}

/// Starts the server.
///
/// The daemon runs in the foreground until it receives `SIGINT` or `SIGTERM`;
/// its log records go to standard error, and standard output carries nothing
/// but the result of the command (`OPS-001`).
fn serve(path: Option<&std::path::Path>, port: Option<u16>) -> Outcome {
    let path = resolve(path);
    let mut options = serve::ServeOptions::new(path);
    options.http_port = port;
    match serve::run(options) {
        Ok(()) => Outcome::ok(""),
        Err(message) => Outcome::failure(format!("{message}\n")),
    }
}
