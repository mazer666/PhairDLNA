//! The command line entry point of PhairDLNA.
//!
//! What: parses the arguments, runs the command and returns the exit code.
//! Why: the binary must stay a thin shell around the testable library in
//!      `lib.rs`; everything that can be tested lives there.
//! Example: `phairdlna-cli config validate --config /etc/phairdlna/config.toml`

use std::path::PathBuf;
use std::process::ExitCode;

use phairdlna_cli::{run, Command};

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

Options:
  --config <path>          The configuration file to use
  --port <number>          The port the server listens on (serve only)
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
            ExitCode::from(2)
        }
    }
}

/// Parses the arguments into a command.
///
/// The parser is deliberately small and hand-written: a command line with six
/// subcommands does not need a parsing framework, and one fewer dependency is
/// one fewer licence to review.
fn parse(arguments: &[String]) -> Result<Command, String> {
    let mut config_path: Option<PathBuf> = None;
    let mut port: Option<u16> = None;
    let mut rest: Vec<&str> = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--config" => {
                let value = arguments.get(index + 1).ok_or("--config needs a path")?;
                config_path = Some(PathBuf::from(value));
                index += 2;
            }
            "--port" => {
                let value = arguments.get(index + 1).ok_or("--port needs a number")?;
                port = Some(
                    value
                        .parse::<u16>()
                        .map_err(|_| format!("\"{value}\" is not a port number"))?,
                );
                index += 2;
            }
            "-h" | "--help" => return Err("help requested".to_owned()),
            other => {
                rest.push(other);
                index += 1;
            }
        }
    }
    command_from(&rest, config_path, port)
}

/// Builds a command from the positional arguments.
fn command_from(
    rest: &[&str],
    config_path: Option<PathBuf>,
    port: Option<u16>,
) -> Result<Command, String> {
    match rest.first().copied() {
        None => Ok(Command::Version),
        Some("version") => Ok(Command::Version),
        Some("config") => config_command(&rest[1..], config_path),
        Some("ssdp") => ssdp_command(&rest[1..], config_path),
        Some("catalogue") => catalogue_command(&rest[1..], config_path),
        Some("serve") => Ok(Command::Serve {
            path: config_path,
            port,
        }),
        Some(other) => Err(format!("unknown command \"{other}\"")),
    }
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
