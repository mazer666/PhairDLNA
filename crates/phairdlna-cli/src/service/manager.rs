//! What the service manager is asked to do.
//!
//! What: the commands of `launchctl` and `systemctl` that belong to the
//!       service, and the five operations of the service: install, uninstall,
//!       start, stop and status.
//! Why: the service of the server must behave the same way on every platform
//!      it supports, and the difference between the platforms must live in one
//!      place (`PLT-010`, `OPS-009`).
//! Example: `commands_for` says that on macOS the service is loaded with
//!      `launchctl load -w`, and on Linux with `systemctl --user enable`.

use std::path::PathBuf;

use super::layout::{Platform, Scope, ServiceLayout};
use super::runner::{CommandOutcome, ServiceRunner};
use super::{plist, unit};

/// The commands of the service manager that belong to the service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceCommands {
    /// The program that is run.
    pub program: String,
    /// The command that registers the service.
    pub load: Vec<String>,
    /// The command that removes the service from the manager.
    pub unload: Vec<String>,
    /// The command that starts the service.
    pub start: Vec<String>,
    /// The command that stops the service.
    pub stop: Vec<String>,
    /// The command that asks whether the service runs.
    pub status: Vec<String>,
    /// The command that asks for the number of the process, when the status
    /// command does not carry it.
    pub pid: Option<Vec<String>>,
    /// The command that makes the manager read the service file again.
    pub reload: Option<Vec<String>>,
}

/// The commands that belong to the service of the server.
///
/// The two platforms are told apart here and nowhere else: the rest of the
/// command works with the service, not with the manager of the platform.
pub fn commands_for(layout: &ServiceLayout) -> ServiceCommands {
    match (layout.platform, layout.scope) {
        (Platform::MacOs, Scope::User) => commands_of_the_agent(layout),
        (Platform::MacOs, Scope::System) => commands_of_the_daemon(layout),
        (Platform::Linux, scope) => commands_of_the_unit(layout, scope),
    }
}

/// The commands of the job of the user on macOS.
///
/// The job of the user is loaded and unloaded with the file of the job; the
/// modern commands of `launchctl` belong to the system scope.
fn commands_of_the_agent(layout: &ServiceLayout) -> ServiceCommands {
    let file = layout.unit_path.to_string_lossy().into_owned();
    ServiceCommands {
        program: "launchctl".to_owned(),
        load: vec!["load".to_owned(), "-w".to_owned(), file.clone()],
        unload: vec!["unload".to_owned(), "-w".to_owned(), file],
        start: vec!["start".to_owned(), layout.label.clone()],
        stop: vec!["stop".to_owned(), layout.label.clone()],
        status: vec!["list".to_owned(), layout.label.clone()],
        pid: None,
        reload: None,
    }
}

/// The commands of the job of the machine on macOS.
///
/// The job of the machine lives in the domain of the system, and the commands
/// of `launchctl` name that domain.
fn commands_of_the_daemon(layout: &ServiceLayout) -> ServiceCommands {
    let target = format!("system/{}", layout.label);
    ServiceCommands {
        program: "launchctl".to_owned(),
        load: vec![
            "bootstrap".to_owned(),
            "system".to_owned(),
            layout.unit_path.to_string_lossy().into_owned(),
        ],
        unload: vec!["bootout".to_owned(), target.clone()],
        start: vec!["kickstart".to_owned(), "-k".to_owned(), target.clone()],
        stop: vec!["kill".to_owned(), "SIGTERM".to_owned(), target.clone()],
        status: vec!["print".to_owned(), target.clone()],
        pid: None,
        reload: None,
    }
}

/// The commands of the unit of the server on Linux.
///
/// The unit of the user is managed by `systemctl --user`; the unit of the
/// machine is managed by the system itself.
fn commands_of_the_unit(layout: &ServiceLayout, scope: Scope) -> ServiceCommands {
    let mut prefix = Vec::new();
    if scope == Scope::User {
        prefix.push("--user".to_owned());
    }
    let unit_name = layout.service_name();
    let with_prefix = |arguments: Vec<String>| -> Vec<String> {
        let mut all = prefix.clone();
        all.extend(arguments);
        all
    };
    ServiceCommands {
        program: "systemctl".to_owned(),
        load: with_prefix(vec!["enable".to_owned(), unit_name.clone()]),
        unload: with_prefix(vec![
            "disable".to_owned(),
            "--now".to_owned(),
            unit_name.clone(),
        ]),
        start: with_prefix(vec!["start".to_owned(), unit_name.clone()]),
        stop: with_prefix(vec!["stop".to_owned(), unit_name.clone()]),
        status: with_prefix(vec!["is-active".to_owned(), unit_name.clone()]),
        pid: Some(with_prefix(vec![
            "show".to_owned(),
            "-p".to_owned(),
            "MainPID".to_owned(),
            "--value".to_owned(),
            unit_name.clone(),
        ])),
        reload: Some(with_prefix(vec!["daemon-reload".to_owned()])),
    }
}

/// What the service manager knows about the service.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceStatus {
    /// Whether the file of the service is on the disk.
    pub installed: bool,
    /// Whether the manager knows the service.
    pub loaded: bool,
    /// Whether the server runs.
    pub running: bool,
    /// The number of the process of the server.
    pub pid: Option<u32>,
    /// The words of the manager about the service.
    pub detail: String,
}

/// The operations of the service.
pub struct ServiceManager<'a> {
    /// The service the operations act on.
    layout: &'a ServiceLayout,
    /// The runner that runs the programs of the manager.
    runner: &'a dyn ServiceRunner,
}

impl<'a> ServiceManager<'a> {
    /// Builds a manager for a service.
    pub fn new(layout: &'a ServiceLayout, runner: &'a dyn ServiceRunner) -> Self {
        Self { layout, runner }
    }

    /// The file that describes the service.
    pub fn unit_path(&self) -> &std::path::Path {
        &self.layout.unit_path
    }

    /// The content of the file that describes the service.
    ///
    /// The content is rendered from the layout, so that the file on the disk
    /// and the file the command meant to write can never differ.
    pub fn render(&self) -> String {
        match self.layout.platform {
            Platform::MacOs => plist::render(self.layout),
            Platform::Linux => unit::render(self.layout),
        }
    }

    /// Writes the file of the service.
    ///
    /// An existing file is only replaced when the user says so: a service file
    /// that was changed by hand carries the wishes of its owner.
    pub fn write_unit_file(&self, force: bool) -> Result<(), String> {
        let path = &self.layout.unit_path;
        if path.exists() && !force {
            return Err(format!(
                "there is already a service file at \"{}\"; use --force to replace it",
                path.display()
            ));
        }
        let content = self.render();
        write_file(path, &content)
    }

    /// Registers the service with the manager of the platform.
    pub fn load(&self) -> Result<(), String> {
        let commands = commands_for(self.layout);
        if let Some(reload) = &commands.reload {
            self.run(&commands.program, reload)?;
        }
        self.run(&commands.program, &commands.load)?;
        Ok(())
    }

    /// Removes the service from the manager of the platform.
    ///
    /// A service that is not loaded is not an error: the file is removed
    /// either way, and the user is told what happened.
    pub fn unload(&self) -> Result<(), String> {
        let commands = commands_for(self.layout);
        self.try_run(&commands.program, &commands.unload)?;
        Ok(())
    }

    /// Starts the service.
    pub fn start(&self) -> Result<(), String> {
        let commands = commands_for(self.layout);
        self.run(&commands.program, &commands.start)?;
        Ok(())
    }

    /// Stops the service.
    pub fn stop(&self) -> Result<(), String> {
        let commands = commands_for(self.layout);
        self.run(&commands.program, &commands.stop)?;
        Ok(())
    }

    /// Asks the manager what it knows about the service.
    pub fn status(&self) -> Result<ServiceStatus, String> {
        let commands = commands_for(self.layout);
        let installed = self.layout.unit_path.exists();
        let outcome = self.try_run(&commands.program, &commands.status)?;
        let (loaded, running, detail) = read_the_status(self.layout.platform, &outcome);
        let pid = self.pid(&commands, running, &outcome);
        Ok(ServiceStatus {
            installed,
            loaded,
            running,
            pid,
            detail,
        })
    }

    /// The number of the process of the server.
    ///
    /// The number comes from the answer of the status command, when the
    /// manager puts it there, and from a question of its own otherwise.
    fn pid(
        &self,
        commands: &ServiceCommands,
        running: bool,
        outcome: &CommandOutcome,
    ) -> Option<u32> {
        if !running {
            return None;
        }
        if let Some(number) = number_in_the_answer(&outcome.stdout) {
            return Some(number);
        }
        let arguments = commands.pid.as_ref()?;
        let answer = self.try_run(&commands.program, arguments).ok()?;
        answer.stdout.trim().parse::<u32>().ok()
    }

    /// Runs a program and keeps the answer, whatever it is.
    ///
    /// A program that fails is not an error here: the status of a service that
    /// does not run is an answer, not a failure.
    fn try_run(&self, program: &str, arguments: &[String]) -> Result<CommandOutcome, String> {
        self.runner
            .run(program, arguments)
            .map_err(|error| error.to_string())
    }

    /// Runs a program and turns its answer into a message when it fails.
    fn run(&self, program: &str, arguments: &[String]) -> Result<CommandOutcome, String> {
        let outcome = self
            .runner
            .run(program, arguments)
            .map_err(|error| error.to_string())?;
        if outcome.is_success() {
            return Ok(outcome);
        }
        let reason = if outcome.stderr.trim().is_empty() {
            outcome.stdout.trim().to_owned()
        } else {
            outcome.stderr.trim().to_owned()
        };
        Err(format!(
            "\"{}\" failed: {}",
            command_line(program, arguments),
            if reason.is_empty() {
                "the program said nothing".to_owned()
            } else {
                reason
            }
        ))
    }
}

/// Reads the answer of the manager about the service.
///
/// The three answers differ per platform, and the words of the manager are
/// kept: the user should see what the system said.
fn read_the_status(platform: Platform, outcome: &CommandOutcome) -> (bool, bool, String) {
    match platform {
        Platform::MacOs => read_the_status_of_launchd(outcome),
        Platform::Linux => read_the_status_of_systemd(outcome),
    }
}

/// Reads the answer of `launchctl`.
///
/// The answer of `launchctl list` is a table: the first column is the number
/// of the process, and a dash means that the job is loaded and does not run.
fn read_the_status_of_launchd(outcome: &CommandOutcome) -> (bool, bool, String) {
    if !outcome.is_success() {
        return (false, false, the_words_of_the_manager(outcome));
    }
    let text = outcome.stdout.trim();
    if text.is_empty() {
        return (true, false, "the job is loaded and does not run".to_owned());
    }
    let row = text.lines().nth(1).unwrap_or(text);
    let number = row.split_whitespace().next().unwrap_or("");
    let running = number.parse::<u32>().is_ok();
    (true, running, text.lines().next().unwrap_or("").to_owned())
}

/// Reads the answer of `systemctl`.
///
/// The answer of `systemctl is-active` is one word: `active` when the server
/// runs, and something else when it does not.
fn read_the_status_of_systemd(outcome: &CommandOutcome) -> (bool, bool, String) {
    let text = outcome.stdout.trim();
    if !outcome.is_success() && text.is_empty() {
        return (false, false, the_words_of_the_manager(outcome));
    }
    let running = text == "active" || text == "activating";
    (true, running, text.to_owned())
}

/// The words of the manager about a service that is not there.
fn the_words_of_the_manager(outcome: &CommandOutcome) -> String {
    let reason = outcome.stderr.trim();
    if reason.is_empty() {
        "the service is not loaded".to_owned()
    } else {
        reason.to_owned()
    }
}

/// The number of a process in the answer of the manager.
fn number_in_the_answer(text: &str) -> Option<u32> {
    for line in text.lines() {
        let first = line.split_whitespace().next()?;
        if let Ok(number) = first.parse::<u32>() {
            return Some(number);
        }
    }
    None
}

/// The command line of a program, as a person would type it.
fn command_line(program: &str, arguments: &[String]) -> String {
    let mut line = program.to_owned();
    for argument in arguments {
        line.push(' ');
        line.push_str(argument);
    }
    line
}

/// Writes a file with the permissions of the standards.
///
/// The file is written beside the file it replaces and moved into place, so
/// that a service file is never half written (`OPS-009`).
fn write_file(path: &std::path::Path, content: &str) -> Result<(), String> {
    let parent = path.parent().unwrap_or_else(|| std::path::Path::new("."));
    std::fs::create_dir_all(parent).map_err(|error| {
        format!(
            "the folder \"{}\" cannot be created: {error}",
            parent.display()
        )
    })?;
    let temporary: PathBuf = path.with_extension("tmp");
    write_with_mode(&temporary, content).map_err(|error| {
        format!(
            "the file \"{}\" cannot be written: {error}",
            temporary.display()
        )
    })?;
    std::fs::rename(&temporary, path).map_err(|error| {
        format!(
            "the file \"{}\" cannot be put into place: {error}",
            path.display()
        )
    })?;
    Ok(())
}

/// Writes a file that everybody may read and nobody but the owner may write.
#[cfg(unix)]
fn write_with_mode(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .mode(0o644)
        .open(path)?;
    file.write_all(content.as_bytes())?;
    file.flush()
}

/// Writes a file, on a platform without file modes.
#[cfg(not(unix))]
fn write_with_mode(path: &std::path::Path, content: &str) -> std::io::Result<()> {
    std::fs::write(path, content)
}

#[cfg(test)]
#[cfg(test)]
#[path = "manager/tests.rs"]
mod tests;
