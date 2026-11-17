//! The command line entry point of PhairDLNA.
//!
//! What: parses the arguments, runs the command and returns the exit code.
//! Why: the binary must stay a thin shell around the testable library in
//!      `lib.rs`; everything that can be tested lives there.
//! Example: `phairdlna-cli config validate --config /etc/phairdlna/config.toml`

use std::path::PathBuf;
use std::process::ExitCode;

use phairdlna_cli::service::ServiceCommand;
use phairdlna_cli::{run, Command, EXIT_CONFIGURATION};

/// The usage text.
const USAGE: &str = "\
Usage: phairdlna-cli <command> [options]

Commands:
  version                  Print the version and the licence
  config path              Print the path the configuration would be read from
  config validate          Check a configuration file and report the first problem
  config show              Print the effective configuration
  config export            Export the effective configuration
  config import <file>     Read a configuration file and print what it would become
  ssdp headers             Print the SSDP headers the server would send
  catalogue scan           Print the tree of the library with the client identifiers
  catalogue browse <id>    Print the document a client receives for a container
  serve                    Start the server in the foreground
  service install          Write the service file and register the service
  service uninstall        Remove the service and its file
  service status           Say what the service manager knows about the service
  service start            Start the service
  service stop             Stop the service
  diagnostics              Write the report for a support ticket
  setup                    Ask the questions of the first run and write the configuration

Options:
  --config <path>          The configuration file to use
  --port <number>          The port the server listens on (serve only)
  --force                  Replace a file that is already there (service and setup)
  --dry-run                Say what would be done, and do nothing (service install only)
  --system                 Act on the service of the machine, not of the user
  --unredacted             Show the real paths of the machine (diagnostics only)
  --export <file>          Write the report to a file (diagnostics only)
  --json                   Write the report as JSON (diagnostics only)
  -h, --help               Print this text
";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    match parse(&arguments) {
        Ok(command) => {
            let outcome = run(&command);
            print!("{}", outcome.stdout);
            ExitCode::from(outcome.exit_code.clamp(0, 255) as u8)
        }
        Err(message) => {
            eprint!("{message}\n{USAGE}");
            ExitCode::from(EXIT_CONFIGURATION as u8)
        }
    }
}

/// The options that the parser understands.
#[derive(Debug, Default)]
struct Options {
    /// The configuration file the command works on.
    config: Option<PathBuf>,
    /// The port the server listens on.
    port: Option<u16>,
    /// Whether a file that is already there may be replaced.
    force: bool,
    /// Whether the command only says what it would do.
    dry_run: bool,
    /// Whether the command acts on the service of the machine.
    system: bool,
    /// Whether the real paths of the machine may be shown.
    unredacted: bool,
    /// The file the report is written to.
    export: Option<PathBuf>,
    /// Whether the report is written as JSON.
    json: bool,
}

/// Parses the arguments into a command.
///
/// The parser is deliberately small and hand-written: a command line with six
/// subcommands does not need a parsing framework, and one fewer dependency is
/// one fewer licence to review.
fn parse(arguments: &[String]) -> Result<Command, String> {
    let mut options = Options::default();
    let mut rest: Vec<&str> = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--config" => {
                let value = arguments.get(index + 1).ok_or("--config needs a path")?;
                options.config = Some(PathBuf::from(value));
                index += 2;
            }
            "--port" => {
                let value = arguments.get(index + 1).ok_or("--port needs a number")?;
                options.port = Some(
                    value
                        .parse::<u16>()
                        .map_err(|_| format!("\"{value}\" is not a port number"))?,
                );
                index += 2;
            }
            "--force" => {
                options.force = true;
                index += 1;
            }
            "--dry-run" => {
                options.dry_run = true;
                index += 1;
            }
            "--system" => {
                options.system = true;
                index += 1;
            }
            "--unredacted" => {
                options.unredacted = true;
                index += 1;
            }
            "--export" => {
                let value = arguments.get(index + 1).ok_or("--export needs a file")?;
                options.export = Some(PathBuf::from(value));
                index += 2;
            }
            "--json" => {
                options.json = true;
                index += 1;
            }
            "-h" | "--help" => return Err("help requested".to_owned()),
            other => {
                rest.push(other);
                index += 1;
            }
        }
    }
    command_from(&rest, options)
}

/// Builds a command from the positional arguments.
fn command_from(rest: &[&str], options: Options) -> Result<Command, String> {
    match rest.first().copied() {
        None => Ok(Command::Version),
        Some("version") => Ok(Command::Version),
        Some("config") => {
            check_no_service_options(&options)?;
            config_command(&rest[1..], options.config)
        }
        Some("ssdp") => {
            check_no_service_options(&options)?;
            ssdp_command(&rest[1..], options.config)
        }
        Some("catalogue") => {
            check_no_service_options(&options)?;
            catalogue_command(&rest[1..], options.config)
        }
        Some("serve") => {
            check_no_service_options(&options)?;
            Ok(Command::Serve {
                path: options.config,
                port: options.port,
            })
        }
        Some("service") => service_command(&rest[1..], options),
        Some("diagnostics") => {
            check_no_service_options(&options)?;
            Ok(Command::Diagnostics {
                path: options.config,
                unredacted: options.unredacted,
                export: options.export,
                json: options.json,
            })
        }
        Some("setup") => {
            check_no_diagnostics_options(&options)?;
            Ok(Command::Setup {
                path: options.config,
                force: options.force,
            })
        }
        Some(other) => Err(format!("unknown command \"{other}\"")),
    }
}

/// Refuses the options of the service in a command that is not one.
///
/// The options belong to the service commands; a command that ignores them
/// would take them silently, and the user would believe that something
/// happened that did not happen.
fn check_no_service_options(options: &Options) -> Result<(), String> {
    if options.force {
        return Err("--force belongs to the service commands".to_owned());
    }
    if options.dry_run {
        return Err("--dry-run belongs to the service commands".to_owned());
    }
    if options.system {
        return Err("--system belongs to the service commands".to_owned());
    }
    Ok(())
}

/// Refuses the options of the diagnostics in a command that is not one.
///
/// The options belong to the diagnostics command; a command that ignores them
/// would take them silently, and the user would believe that the report was
/// written somewhere it was not.
fn check_no_diagnostics_options(options: &Options) -> Result<(), String> {
    if options.unredacted {
        return Err("--unredacted belongs to the diagnostics command".to_owned());
    }
    if options.export.is_some() {
        return Err("--export belongs to the diagnostics command".to_owned());
    }
    if options.json {
        return Err("--json belongs to the diagnostics command".to_owned());
    }
    Ok(())
}

/// Builds a `service` subcommand.
///
/// The five commands of the service are the ones of `F-13`: the service file
/// is written and registered, removed, asked about, started and stopped.
fn service_command(rest: &[&str], options: Options) -> Result<Command, String> {
    let command = match rest.first().copied() {
        Some("install") => ServiceCommand::Install {
            force: options.force,
            dry_run: options.dry_run,
            system: options.system,
            config: options.config,
        },
        Some("uninstall") => ServiceCommand::Uninstall {
            force: options.force,
            system: options.system,
        },
        Some("status") => ServiceCommand::Status {
            system: options.system,
        },
        Some("start") => ServiceCommand::Start {
            system: options.system,
        },
        Some("stop") => ServiceCommand::Stop {
            system: options.system,
        },
        Some(other) => return Err(format!("unknown service command \"{other}\"")),
        None => {
            return Err(
                "service needs a subcommand: install, uninstall, status, start or stop".to_owned(),
            )
        }
    };
    Ok(Command::Service { command })
}

/// Builds an `ssdp` subcommand.
fn ssdp_command(rest: &[&str], config_path: Option<PathBuf>) -> Result<Command, String> {
    match rest.first().copied() {
        Some("headers") => Ok(Command::SsdpHeaders { path: config_path }),
        Some(other) => Err(format!("unknown ssdp command \"{other}\"")),
        None => Err("ssdp needs a subcommand: headers".to_owned()),
    }
}

/// Builds a `catalogue` subcommand.
fn catalogue_command(rest: &[&str], config_path: Option<PathBuf>) -> Result<Command, String> {
    match rest.first().copied() {
        Some("scan") => Ok(Command::CatalogueScan { path: config_path }),
        Some("browse") => {
            let object_id = rest
                .get(1)
                .copied()
                .ok_or("catalogue browse needs an object id")?;
            Ok(Command::CatalogueBrowse {
                object_id: object_id.to_owned(),
                path: config_path,
            })
        }
        Some(other) => Err(format!("unknown catalogue command \"{other}\"")),
        None => Err("catalogue needs a subcommand: scan or browse".to_owned()),
    }
}

/// Builds a `config` subcommand.
fn config_command(rest: &[&str], config_path: Option<PathBuf>) -> Result<Command, String> {
    match rest.first().copied() {
        Some("path") => Ok(Command::ConfigPath),
        Some("validate") => Ok(Command::ConfigValidate { path: config_path }),
        Some("show") => Ok(Command::ConfigShow { path: config_path }),
        Some("export") => Ok(Command::ConfigExport { path: config_path }),
        Some("import") => {
            let file = rest.get(1).copied().ok_or("config import needs a file")?;
            Ok(Command::ConfigImport {
                path: PathBuf::from(file),
            })
        }
        Some(other) => Err(format!("unknown config command \"{other}\"")),
        None => Err("config needs a subcommand: path, validate, show, export or import".to_owned()),
    }
}
