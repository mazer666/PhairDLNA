//! The life of the server as a service of the operating system.
//!
//! What: the commands `install`, `uninstall`, `start`, `stop` and `status`,
//!       and the files they write and read.
//! Why: a server that only runs in a terminal is a server the user has to
//!      watch. The service is the way the server runs without a person, and it
//!      must be installable, removable and readable by the person who owns it
//!      (`OPS-009`, `LIFE-001`).
//! Example: `phairdlna service install` writes the job file of the server and
//!      registers it with the service manager of the machine.
//!
//! **What is written, and where.** On macOS the job file is a `launchd`
//! property list in `~/Library/LaunchAgents` (or in `/Library/LaunchDaemons`
//! for the system scope); on Linux it is a `systemd` unit in
//! `~/.config/systemd/user` (or in `/etc/systemd/system` for the system
//! scope). The file is written with the permissions `644` and is never
//! replaced without the word of the user (`--force`).
//!
//! **What is never done.** The commands of this module never start the server
//! in the foreground, never open a socket and never make a request to the
//! network: every command is local (`SEC-091`).

mod commands;
pub mod layout;
pub mod manager;
pub mod notify;
pub mod paths;
pub mod plist;
pub mod plist_parse;
pub mod runner;
pub mod unit;

use std::path::{Path, PathBuf};

use crate::Outcome;

use commands::{install, start, status, stop, uninstall};
pub use layout::{Platform, Scope, ServiceLayout};
use runner::ServiceRunner;

/// What the user asked the service to do.
#[derive(Debug, Clone, PartialEq)]
pub enum ServiceCommand {
    /// Write the file of the service and register it.
    Install {
        /// Whether an existing file may be replaced.
        force: bool,
        /// Whether the command only says what it would do.
        dry_run: bool,
        /// Whether the service belongs to the machine and not to the user.
        system: bool,
        /// The configuration file the server reads.
        config: Option<PathBuf>,
    },
    /// Remove the service from the manager and delete its file.
    Uninstall {
        /// Whether the removal goes on even when the service does not run.
        force: bool,
        /// Whether the service belongs to the machine and not to the user.
        system: bool,
    },
    /// Say what the service manager knows about the service.
    Status {
        /// Whether the service belongs to the machine and not to the user.
        system: bool,
    },
    /// Start the service.
    Start {
        /// Whether the service belongs to the machine and not to the user.
        system: bool,
    },
    /// Stop the service.
    Stop {
        /// Whether the service belongs to the machine and not to the user.
        system: bool,
    },
}

/// The machine the service is built for.
///
/// The environment is a value and not a reading of the process, so that a test
/// can build the service of a machine that does not exist (`F-13` § 11).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceEnvironment {
    /// The platform the service is built for.
    pub platform: Platform,
    /// The home directory of the user.
    pub home: PathBuf,
    /// The program that is started.
    pub program: PathBuf,
    /// The configuration file the server reads.
    pub config: PathBuf,
}

impl ServiceEnvironment {
    /// The environment of the machine this build runs on.
    pub fn current() -> Result<Self, String> {
        let program = std::env::current_exe()
            .map_err(|error| format!("the path of the running program is not known: {error}"))?;
        Ok(ServiceEnvironment {
            platform: Platform::current()?,
            home: phairdlna_core::config::home_directory(),
            program,
            config: phairdlna_core::config::resolve_path(None),
        })
    }

    /// The layout of the service of this environment.
    pub fn layout(&self, scope: Scope, config: Option<&Path>) -> Result<ServiceLayout, String> {
        let request = layout::LayoutRequest {
            platform: self.platform,
            scope,
            home: self.home.clone(),
            program: self.program.clone(),
            config: match config {
                Some(config) => config.to_path_buf(),
                None => self.config.clone(),
            },
        };
        ServiceLayout::build(&request)
    }
}

/// Runs a service command and returns what it should print.
pub fn run(command: &ServiceCommand) -> Outcome {
    run_with(command, &runner::SystemRunner)
}

/// Runs a service command with a runner of its own.
///
/// The function exists so that the tests can ask what the command *would* do
/// without touching the machine of the user (`F-13` § 11).
pub fn run_with(command: &ServiceCommand, runner: &dyn ServiceRunner) -> Outcome {
    let environment = match ServiceEnvironment::current() {
        Ok(environment) => environment,
        Err(message) => return Outcome::failure(format!("{message}\n")),
    };
    run_in(command, runner, &environment)
}

/// Runs a service command on a given machine.
pub fn run_in(
    command: &ServiceCommand,
    runner: &dyn ServiceRunner,
    environment: &ServiceEnvironment,
) -> Outcome {
    let scope = scope_of(command);
    match command {
        ServiceCommand::Install {
            force,
            dry_run,
            config,
            ..
        } => install(
            *force,
            *dry_run,
            config.as_deref(),
            scope,
            environment,
            runner,
        ),
        ServiceCommand::Uninstall { force, .. } => uninstall(*force, scope, environment, runner),
        ServiceCommand::Status { .. } => status(scope, environment, runner),
        ServiceCommand::Start { .. } => start(scope, environment, runner),
        ServiceCommand::Stop { .. } => stop(scope, environment, runner),
    }
}

/// The scope the command asks for.
fn scope_of(command: &ServiceCommand) -> Scope {
    let system = match command {
        ServiceCommand::Install { system, .. }
        | ServiceCommand::Uninstall { system, .. }
        | ServiceCommand::Status { system }
        | ServiceCommand::Start { system }
        | ServiceCommand::Stop { system } => *system,
    };
    if system {
        Scope::System
    } else {
        Scope::User
    }
}

#[cfg(test)]
mod tests;
