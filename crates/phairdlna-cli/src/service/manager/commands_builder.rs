//! Commands builder for platform service managers.
//!
//! What: routes service commands to launchctl, systemctl, or sc.exe.
//! Why: separates command creation to keep manager.rs under line targets.

use super::windows;
use super::ServiceCommands;
use crate::service::layout::{Platform, Scope, ServiceLayout};

/// The commands that belong to the service of the server.
pub fn commands_for(layout: &ServiceLayout) -> ServiceCommands {
    match (layout.platform, layout.scope) {
        (Platform::MacOs, Scope::User) => commands_of_the_agent(layout),
        (Platform::MacOs, Scope::System) => commands_of_the_daemon(layout),
        (Platform::Linux, scope) => commands_of_the_unit(layout, scope),
        (Platform::Windows, _) => windows::commands(layout),
    }
}

/// The commands of the job of the user on macOS.
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
        status: vec!["print".to_owned(), target],
        pid: None,
        reload: None,
    }
}

/// The commands of the unit of the server on Linux.
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
            unit_name,
        ])),
        reload: Some(with_prefix(vec!["daemon-reload".to_owned()])),
    }
}
