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
  serve                    Start the server (not implemented yet)

Options:
  --config <path>          The configuration file to use
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
    let mut rest: Vec<&str> = Vec::new();
    let mut index = 0;
    while index < arguments.len() {
        match arguments[index].as_str() {
            "--config" => {
                let value = arguments.get(index + 1).ok_or("--config needs a path")?;
                config_path = Some(PathBuf::from(value));
                index += 2;
            }
            "-h" | "--help" => return Err("help requested".to_owned()),
            other => {
                rest.push(other);
                index += 1;
            }
        }
    }
    command_from(&rest, config_path)
}

/// Builds a command from the positional arguments.
fn command_from(rest: &[&str], config_path: Option<PathBuf>) -> Result<Command, String> {
    match rest.first().copied() {
        None => Ok(Command::Version),
        Some("version") => Ok(Command::Version),
        Some("config") => config_command(&rest[1..], config_path),
        Some("serve") => Ok(Command::Serve { path: config_path }),
        Some(other) => Err(format!("unknown command \"{other}\"")),
    }
}

/// Builds a `config` subcommand.
fn config_command(rest: &[&str], config_path: Option<PathBuf>) -> Result<Command, String> {
    match rest.first().copied() {
        Some("path") => Ok(Command::ConfigPath),
        Some("validate") => Ok(Command::ConfigValidate { path: config_path }),
        Some("show") => Ok(Command::ConfigShow { path: config_path }),
        Some(other) => Err(format!("unknown config command \"{other}\"")),
        None => Err("config needs a subcommand: path, validate or show".to_owned()),
    }
}
