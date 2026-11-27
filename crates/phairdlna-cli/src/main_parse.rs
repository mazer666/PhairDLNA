//! Argument parsing for `phairdlna-cli`.
//!
//! What: parses CLI arguments into strongly-typed `Command` variants.
//! Why: kept separate from `main.rs` to maintain ≤ 300 line budget.

use std::path::PathBuf;

use phairdlna_cli::service::ServiceCommand;
use phairdlna_cli::Command;

/// The options that the parser understands.
#[derive(Debug, Default)]
pub struct Options {
    pub config: Option<PathBuf>,
    pub port: Option<u16>,
    pub force: bool,
    pub dry_run: bool,
    pub system: bool,
    pub unredacted: bool,
    pub export: Option<PathBuf>,
    pub json: bool,
    pub expert: bool,
    pub apply: bool,
}

/// Parses the arguments into a command.
pub fn parse(arguments: &[String]) -> Result<Command, String> {
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
            "--expert" => {
                options.expert = true;
                index += 1;
            }
            "--apply" => {
                options.apply = true;
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
        None | Some("version") => Ok(Command::Version),
        Some("status") => {
            check_no_service_options(&options)?;
            Ok(Command::Status {
                path: options.config,
                json: options.json,
            })
        }
        Some("scan") => {
            check_no_service_options(&options)?;
            let root = rest.get(1).map(|s| (*s).to_owned());
            Ok(Command::Scan {
                path: options.config,
                root,
            })
        }
        Some("config") => {
            check_no_service_options(&options)?;
            config_command(&rest[1..], &options)
        }
        Some("ssdp") => {
            check_no_service_options(&options)?;
            ssdp_command(&rest[1..], options.config)
        }
        Some("catalogue") => {
            check_no_service_options(&options)?;
            catalogue_command(&rest[1..], options.config)
        }
        Some(other) => operational_command(other, rest, options),
    }
}

/// Parses operational and lifecycle commands.
fn operational_command(cmd: &str, rest: &[&str], options: Options) -> Result<Command, String> {
    match cmd {
        "serve" => {
            check_no_service_options(&options)?;
            Ok(Command::Serve {
                path: options.config,
                port: options.port,
            })
        }
        "service" => service_command(&rest[1..], options),
        "diagnostics" => {
            check_no_service_options(&options)?;
            Ok(Command::Diagnostics {
                path: options.config,
                unredacted: options.unredacted,
                export: options.export,
                json: options.json,
            })
        }
        "setup" => {
            check_no_diagnostics_options(&options)?;
            Ok(Command::Setup {
                path: options.config,
                force: options.force,
            })
        }
        "describe" => {
            check_no_service_options(&options)?;
            check_no_diagnostics_options(&options)?;
            Ok(Command::Describe {
                path: options.config,
            })
        }
        "interfaces" => {
            check_no_service_options(&options)?;
            Ok(Command::Interfaces {
                path: options.config,
                json: options.json,
            })
        }
        other => Err(format!("unknown command \"{other}\"")),
    }
}

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

fn ssdp_command(rest: &[&str], config_path: Option<PathBuf>) -> Result<Command, String> {
    match rest.first().copied() {
        Some("headers") => Ok(Command::SsdpHeaders { path: config_path }),
        Some("replay") => {
            let corpus = rest
                .get(1)
                .copied()
                .ok_or("ssdp replay needs a corpus path or request file")?;
            Ok(Command::SsdpReplay {
                corpus: PathBuf::from(corpus),
                path: config_path,
            })
        }
        Some(other) => Err(format!("unknown ssdp command \"{other}\"")),
        None => Err("ssdp needs a subcommand: headers, replay".to_owned()),
    }
}

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

fn config_command(rest: &[&str], options: &Options) -> Result<Command, String> {
    let config_path = options.config.clone();
    match rest.first().copied() {
        Some("path") => Ok(Command::ConfigPath),
        Some("validate") => Ok(Command::ConfigValidate { path: config_path }),
        Some("show") => Ok(Command::ConfigShow { path: config_path }),
        Some("export") => Ok(Command::ConfigExport { path: config_path }),
        Some("explain") => Ok(Command::ConfigExplain {
            expert: options.expert,
            json: options.json,
        }),
        Some("import") => {
            let file = rest.get(1).copied().ok_or("config import needs a file")?;
            let target = if options.apply {
                Some(
                    options
                        .config
                        .clone()
                        .unwrap_or_else(|| phairdlna_core::config::resolve_path(None)),
                )
            } else {
                None
            };
            Ok(Command::ConfigImport {
                path: PathBuf::from(file),
                target,
            })
        }
        Some(other) => Err(format!("unknown config command \"{other}\"")),
        None => Err(
            "config needs a subcommand: path, validate, show, export, import or explain".to_owned(),
        ),
    }
}
