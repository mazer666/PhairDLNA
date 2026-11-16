//! The command line of PhairDLNA.
//!
//! What: the commands a user can run, implemented as pure functions over their
//!       arguments so that they can be tested without starting a server.
//! Why: the headless server is the product (`PRD-006`), and a command that
//!      cannot be tested is a command that will be wrong.
//! Example: `run(ConfigCommand::Validate { .. })` prints what is wrong with a
//!      configuration file, or says that it is valid.

mod catalogue;
pub mod serve;

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
    /// would become.
    ConfigImport {
        /// The file to read.
        path: PathBuf,
    },
    /// Print the SSDP headers the server would send.
    SsdpHeaders {
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
        Command::ConfigExport { path } => config_export(path.as_deref()),
        Command::ConfigImport { path } => config_import(path),
        Command::SsdpHeaders { path } => ssdp_headers(path.as_deref()),
        Command::CatalogueScan { path } => catalogue::scan(&resolve(path.as_deref())),
        Command::CatalogueBrowse { object_id, path } => {
            catalogue::browse(&resolve(path.as_deref()), object_id)
        }
        Command::Serve { path, port } => serve(path.as_deref(), *port),
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

/// Exports the effective configuration.
///
/// The export is the configuration as the server understands it: every setting
/// that was left out is written down with its default, and the schema version
/// is the one of the file it was read from (`CFG-012`).
fn config_export(path: Option<&std::path::Path>) -> Outcome {
    config_show(path)
}

/// Reads a configuration file and prints the configuration it would become.
///
/// The command writes nothing: it tells the user what the file would look like
/// after the import, and the user decides what to do with it (`CFG-012`).
fn config_import(path: &std::path::Path) -> Outcome {
    match Config::load(path) {
        Ok(config) => match config.render() {
            Ok(text) => Outcome::ok(format!(
                "the configuration \"{}\" would become (schema version {}):\n{text}",
                path.display(),
                config.schema_version
            )),
            Err(error) => Outcome::failure(format!("cannot render the configuration: {error}\n")),
        },
        Err(error) => Outcome::failure(format!(
            "the configuration \"{}\" is not usable:\n{}\n",
            path.display(),
            error.message()
        )),
    }
}

/// Prints the SSDP bytes the server would send.
///
/// This is the diagnostic switch of `INT-015` and `OPS-005`: an interoperability
/// problem can be looked at without capturing a packet.
fn ssdp_headers(path: Option<&std::path::Path>) -> Outcome {
    let path = resolve(path);
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
    let identity = match load_identity(&config) {
        Ok(identity) => identity,
        Err(error) => {
            return Outcome::failure(format!("cannot read the device identity: {error}\n"))
        }
    };
    Outcome::ok(ssdp_report(&identity, &config))
}

/// Loads the persistent device identity.
fn load_identity(config: &Config) -> std::io::Result<phairdlna_core::identity::InstalledIdentity> {
    let path = phairdlna_core::identity::identity_path();
    phairdlna_core::identity::load_or_create(&path, &config.server.device_name)
}

/// The text of the SSDP report.
fn ssdp_report(identity: &phairdlna_core::identity::InstalledIdentity, config: &Config) -> String {
    use phairdlna_core::ssdp::{
        advertised_targets, build_advertisement, build_response, location_url, ServerIdentity,
    };
    let device = identity.as_device_identity();
    let server = ServerIdentity {
        friendly_name: config.server.device_name.clone(),
        product: phairdlna_core::identity::PRODUCT_NAME.to_owned(),
        version: phairdlna_core::PROJECT_VERSION.to_owned(),
    };
    let location = location_url("192.168.0.10", config.server.http_port);
    let mut report = String::new();
    report.push_str(&format!(
        "Device identity: {}\nDevice name:      {}\n\n",
        device.uuid(),
        identity.friendly_name
    ));
    report.push_str("=== advertisement (one per advertised target) ===\n\n");
    for target in advertised_targets(&device) {
        report.push_str(&build_advertisement(&target, &server, &location, true));
    }
    report.push_str("=== example answer to 'M-SEARCH * HTTP/1.1' with ST: ssdp:all ===\n\n");
    let target = advertised_targets(&device)
        .into_iter()
        .next()
        .unwrap_or_else(|| unreachable!("the root device is always advertised"));
    report.push_str(&build_response(&target, &server, &location));
    report.push_str("=== notes ===\n\n");
    report.push_str(
        "The LOCATION above uses a placeholder address. The server replaces it with the\n\
         address of the interface the request arrived on, never a wildcard or a loopback\n\
         address. Run 'ssdp headers' on the machine that serves the library.\n",
    );
    report
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
