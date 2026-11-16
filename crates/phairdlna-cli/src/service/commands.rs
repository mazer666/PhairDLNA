//! The five operations of the service.
//!
//! What: the commands that write the file of the service, that remove it, that
//!       ask the manager about it, that start it and that stop it.
//! Why: the commands are the only place where the words of the user are
//!      written; every other module of the service works with the service, not
//!      with the person who reads the answer (`F-13` § 11).
//! Example: `install` writes the file of the service and registers it with the
//!      manager of the platform.

use std::path::Path;

use super::layout::{Scope, ServiceLayout};
use super::manager::{ServiceManager, ServiceStatus};
use super::runner::ServiceRunner;
use super::Outcome;
use super::ServiceEnvironment;

/// The layout of the service, with the configuration the user named.
pub(super) fn layout_of(
    environment: &ServiceEnvironment,
    scope: Scope,
    config: Option<&Path>,
) -> Result<ServiceLayout, String> {
    if let Some(config) = config {
        if !config.is_absolute() {
            return Err(format!(
                "the configuration file must be an absolute path, but it is \"{}\"",
                config.display()
            ));
        }
    }
    environment.layout(scope, config)
}

/// Writes the service and registers it.
pub(super) fn install(
    force: bool,
    dry_run: bool,
    config: Option<&Path>,
    scope: Scope,
    environment: &ServiceEnvironment,
    runner: &dyn ServiceRunner,
) -> Outcome {
    let layout = match layout_of(environment, scope, config) {
        Ok(layout) => layout,
        Err(message) => return Outcome::failure(format!("{message}\n")),
    };
    let manager = ServiceManager::new(&layout, runner);
    let content = manager.render();
    if dry_run {
        return Outcome::ok(format!(
            "The service file would be written to \"{}\":\n\n{content}",
            layout.unit_path.display()
        ));
    }
    if let Err(message) = make_the_log_folder(&layout) {
        return Outcome::failure(format!("{message}\n"));
    }
    if let Err(message) = manager.write_unit_file(force) {
        return Outcome::failure(format!("{message}\n"));
    }
    if let Err(message) = manager.load() {
        return Outcome::failure(format!(
            "the service file was written to \"{}\", but the service could not be registered: {message}\n",
            layout.unit_path.display()
        ));
    }
    Outcome::ok(format!(
        "The service was installed at \"{}\".\nStart it with 'phairdlna service start'.\n",
        layout.unit_path.display()
    ))
}

/// Creates the folder the logs of the service go to.
///
/// A job file whose log folder does not exist is a job that starts and dies
/// at once: the manager cannot open the log file, so the server is killed
/// before it can answer anybody (`OPS-009`).
fn make_the_log_folder(layout: &ServiceLayout) -> Result<(), String> {
    let folder = layout
        .stdout_path
        .parent()
        .unwrap_or_else(|| std::path::Path::new("."));
    std::fs::create_dir_all(folder).map_err(|error| {
        format!(
            "the folder \"{}\" for the log of the service cannot be created: {error}",
            folder.display()
        )
    })
}

/// Removes the service and its file.
pub(super) fn uninstall(
    force: bool,
    scope: Scope,
    environment: &ServiceEnvironment,
    runner: &dyn ServiceRunner,
) -> Outcome {
    let layout = match layout_of(environment, scope, None) {
        Ok(layout) => layout,
        Err(message) => return Outcome::failure(format!("{message}\n")),
    };
    let manager = ServiceManager::new(&layout, runner);
    if !layout.unit_path.exists() {
        return Outcome::nothing_to_do(format!(
            "There is no service file at \"{}\"; there is nothing to remove.\n",
            layout.unit_path.display()
        ));
    }
    if let Err(message) = manager.unload() {
        if !force {
            return Outcome::failure(format!("{message}\n"));
        }
    }
    if let Err(error) = std::fs::remove_file(&layout.unit_path) {
        return Outcome::failure(format!(
            "the service file \"{}\" cannot be removed: {error}\n",
            layout.unit_path.display()
        ));
    }
    Outcome::ok(format!(
        "The service was removed from \"{}\".\n",
        layout.unit_path.display()
    ))
}

/// Says what the service manager knows about the service.
pub(super) fn status(
    scope: Scope,
    environment: &ServiceEnvironment,
    runner: &dyn ServiceRunner,
) -> Outcome {
    let layout = match layout_of(environment, scope, None) {
        Ok(layout) => layout,
        Err(message) => return Outcome::failure(format!("{message}\n")),
    };
    let manager = ServiceManager::new(&layout, runner);
    let status = match manager.status() {
        Ok(status) => status,
        Err(message) => return Outcome::failure(format!("{message}\n")),
    };
    if !status.installed {
        return Outcome::nothing_to_do(format!(
            "There is no service file at \"{}\"; the server is not installed as a service.\n",
            layout.unit_path.display()
        ));
    }
    Outcome::ok(report(&layout, &status))
}

/// The report of the status, in the words of a person.
fn report(layout: &ServiceLayout, status: &ServiceStatus) -> String {
    let mut text = format!("Service file: {}\n", layout.unit_path.display());
    text.push_str(&format!("Service name: {}\n", layout.service_name()));
    text.push_str(&format!(
        "State: {}\n",
        if status.running {
            "running"
        } else if status.loaded {
            "loaded, but not running"
        } else {
            "not loaded"
        }
    ));
    if let Some(number) = status.pid {
        text.push_str(&format!("Process: {number}\n"));
    }
    if !status.detail.is_empty() {
        text.push_str(&format!(
            "The service manager says: {}\n",
            status.detail.trim()
        ));
    }
    text
}

/// Starts the service.
pub(super) fn start(
    scope: Scope,
    environment: &ServiceEnvironment,
    runner: &dyn ServiceRunner,
) -> Outcome {
    with_service(scope, environment, runner, |manager, layout| {
        if !layout.unit_path.exists() {
            return Outcome::nothing_to_do(format!(
                "There is no service file at \"{}\"; install the service first.\n",
                layout.unit_path.display()
            ));
        }
        match manager.start() {
            Ok(()) => Outcome::ok(format!(
                "The service \"{}\" was started.\n",
                layout.service_name()
            )),
            Err(message) => Outcome::failure(format!("{message}\n")),
        }
    })
}

/// Stops the service.
pub(super) fn stop(
    scope: Scope,
    environment: &ServiceEnvironment,
    runner: &dyn ServiceRunner,
) -> Outcome {
    with_service(scope, environment, runner, |manager, layout| {
        if !layout.unit_path.exists() {
            return Outcome::nothing_to_do(format!(
                "There is no service file at \"{}\"; there is nothing to stop.\n",
                layout.unit_path.display()
            ));
        }
        match manager.stop() {
            Ok(()) => Outcome::ok(format!(
                "The service \"{}\" was stopped.\n",
                layout.service_name()
            )),
            Err(message) => Outcome::failure(format!("{message}\n")),
        }
    })
}

/// Runs an operation on the service of the machine.
fn with_service(
    scope: Scope,
    environment: &ServiceEnvironment,
    runner: &dyn ServiceRunner,
    operation: impl FnOnce(&ServiceManager<'_>, &ServiceLayout) -> Outcome,
) -> Outcome {
    let layout = match layout_of(environment, scope, None) {
        Ok(layout) => layout,
        Err(message) => return Outcome::failure(format!("{message}\n")),
    };
    let manager = ServiceManager::new(&layout, runner);
    operation(&manager, &layout)
}
