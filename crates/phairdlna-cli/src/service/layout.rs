//! What the service of the server looks like.
//!
//! What: the typed description of the service — the program, the arguments,
//!       the working directory, the environment, the logs and the file that
//!       describes it — and the rule that says whether it may be used.
//! Why: a service file that points at a wrong path starts a server that does
//!      not work, and a service file that points outside the folder it belongs
//!      to is a way to write a file where it should not be (`SEC-016`).
//! Example: `ServiceLayout::build` turns the wish of the user into the layout
//!      of the service, or says why it cannot.
//!
//! **The two scopes.** A *user* service belongs to the person who installed
//! it and needs no privilege; a *system* service belongs to the machine and
//! needs one. The user scope is the default, because the server never needs to
//! be root to serve the media of its owner (`OPS-015`).

use std::path::PathBuf;

use super::paths::{environment, log_directory, unit_directory, unit_path, user_name};

/// The operating system the server runs on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// macOS: the job file is a `launchd` property list.
    MacOs,
    /// Linux: the job file is a `systemd` unit.
    Linux,
}

impl Platform {
    /// The platform this build runs on.
    ///
    /// A platform that has no service manager is refused with a clear message
    /// rather than half-installed (`PLT-010`).
    pub fn current() -> Result<Self, String> {
        match std::env::consts::OS {
            "macos" => Ok(Platform::MacOs),
            "linux" => Ok(Platform::Linux),
            other => Err(format!(
                "this platform ({other}) has no service manager the server knows; \
                 run the daemon in the foreground instead"
            )),
        }
    }

    /// The name of the platform, as it appears in the messages.
    pub fn name(self) -> &'static str {
        match self {
            Platform::MacOs => "macOS",
            Platform::Linux => "Linux",
        }
    }
}

/// Who the service belongs to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// The service belongs to the user who installed it.
    User,
    /// The service belongs to the machine and is started at boot.
    System,
}

impl Scope {
    /// The name of the scope, as it appears in the messages.
    pub fn name(self) -> &'static str {
        match self {
            Scope::User => "per user",
            Scope::System => "system wide",
        }
    }
}

/// Everything the service file needs to know.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServiceLayout {
    /// The platform the service belongs to.
    pub platform: Platform,
    /// Who the service belongs to.
    pub scope: Scope,
    /// The label of the job.
    pub label: String,
    /// The program that is started.
    pub program: PathBuf,
    /// The arguments the program is started with.
    pub arguments: Vec<String>,
    /// The folder the server works in.
    pub working_directory: PathBuf,
    /// The environment the server is given.
    pub environment: Vec<(String, String)>,
    /// The file the standard output of the server goes to.
    pub stdout_path: PathBuf,
    /// The file the standard error of the server goes to.
    pub stderr_path: PathBuf,
    /// The file that describes the service.
    pub unit_path: PathBuf,
    /// The user the server runs as.
    pub user: String,
}

/// What the caller wants the service to be.
#[derive(Debug, Clone)]
pub struct LayoutRequest {
    /// The platform the service is built for.
    pub platform: Platform,
    /// Who the service belongs to.
    pub scope: Scope,
    /// The home directory of the user.
    pub home: PathBuf,
    /// The program that is started.
    pub program: PathBuf,
    /// The configuration file the server reads.
    pub config: PathBuf,
}

impl ServiceLayout {
    /// Builds the layout of a service.
    ///
    /// The function fails when a path cannot be right: a relative program path,
    /// a configuration file that is not absolute, a home directory that does
    /// not exist. A service file that points at a wrong place is worse than no
    /// service file at all.
    pub fn build(request: &LayoutRequest) -> Result<Self, String> {
        let layout = ServiceLayout {
            platform: request.platform,
            scope: request.scope,
            label: super::paths::LABEL.to_owned(),
            program: request.program.clone(),
            arguments: vec![
                "serve".to_owned(),
                "--config".to_owned(),
                request.config.to_string_lossy().into_owned(),
            ],
            working_directory: request.home.clone(),
            environment: environment(&request.home),
            stdout_path: log_directory(request.platform, request.scope, &request.home)
                .join("server.out.log"),
            stderr_path: log_directory(request.platform, request.scope, &request.home)
                .join("server.err.log"),
            unit_path: unit_path(request.platform, request.scope, &request.home),
            user: user_name(&request.home),
        };
        layout.validate()?;
        Ok(layout)
    }

    /// The layout of the service of this machine.
    ///
    /// The program is the running binary, and the configuration is the one the
    /// server would read on its own.
    pub fn current(scope: Scope) -> Result<Self, String> {
        let program = std::env::current_exe()
            .map_err(|error| format!("the path of the running program is not known: {error}"))?;
        let request = LayoutRequest {
            platform: Platform::current()?,
            scope,
            home: phairdlna_core::config::home_directory(),
            program,
            config: phairdlna_core::config::resolve_path(None),
        };
        ServiceLayout::build(&request)
    }

    /// The name of the service, as the service manager knows it.
    pub fn service_name(&self) -> String {
        match self.platform {
            Platform::MacOs => self.label.clone(),
            Platform::Linux => super::paths::UNIT_NAME.to_owned(),
        }
    }

    /// The folder the job file of the service belongs into.
    ///
    /// The folder is the one the platform expects; a file anywhere else would
    /// not be loaded at all.
    pub fn expected_directory(&self) -> PathBuf {
        unit_directory(self.platform, self.scope, &self.working_directory)
    }

    /// Checks that the layout may be used.
    ///
    /// The check is the last line of defence before a file is written: the
    /// program must exist, the paths must be absolute, and the job file must
    /// lie in the folder the platform expects.
    pub fn validate(&self) -> Result<(), String> {
        if self.program.as_os_str().is_empty() {
            return Err("the program of the service is not known".to_owned());
        }
        if !self.program.is_absolute() {
            return Err(format!(
                "the program of the service must be an absolute path, but it is \"{}\"",
                self.program.display()
            ));
        }
        if !self.program.exists() {
            return Err(format!(
                "the program of the service, \"{}\", does not exist",
                self.program.display()
            ));
        }
        if !self.working_directory.is_absolute() {
            return Err(format!(
                "the working directory of the service must be an absolute path, but it is \"{}\"",
                self.working_directory.display()
            ));
        }
        self.check_the_unit_path()?;
        self.check_the_log_paths()?;
        Ok(())
    }

    /// Checks that the job file lies where the platform expects it.
    fn check_the_unit_path(&self) -> Result<(), String> {
        if !self.unit_path.is_absolute() {
            return Err(format!(
                "the file of the service must be an absolute path, but it is \"{}\"",
                self.unit_path.display()
            ));
        }
        let expected = self.expected_directory();
        if self.unit_path.parent() != Some(expected.as_path()) {
            return Err(format!(
                "the file of the service must lie in \"{}\", but it is \"{}\"",
                expected.display(),
                self.unit_path.display()
            ));
        }
        Ok(())
    }

    /// Checks the files the logs go to.
    fn check_the_log_paths(&self) -> Result<(), String> {
        for path in [&self.stdout_path, &self.stderr_path] {
            if !path.is_absolute() {
                return Err(format!(
                    "the file of a log must be an absolute path, but it is \"{}\"",
                    path.display()
                ));
            }
            if path == &self.unit_path {
                return Err("the log of the service must not be the file of the service".to_owned());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{LayoutRequest, Platform, Scope, ServiceLayout};
    use std::path::PathBuf;

    /// A program that exists, for the tests.
    fn a_program() -> PathBuf {
        std::env::current_exe().expect("the running program")
    }

    /// A request for a layout on macOS.
    fn a_request(home: &std::path::Path) -> LayoutRequest {
        LayoutRequest {
            platform: Platform::MacOs,
            scope: Scope::User,
            home: home.to_path_buf(),
            program: a_program(),
            config: home.join("config.toml"),
        }
    }

    /// The positive case: the server is started with the configuration it
    /// should read.
    #[test]
    fn the_server_is_started_with_its_configuration() {
        let home = PathBuf::from("/Users/tester");
        let layout = ServiceLayout::build(&a_request(&home)).expect("a layout");

        assert_eq!(layout.arguments[0], "serve");
        assert_eq!(layout.arguments[1], "--config");
        assert!(layout.arguments[2].ends_with("config.toml"), "{layout:?}");
    }

    /// The negative case: a program that does not exist stops the layout.
    #[test]
    fn a_program_that_does_not_exist_is_refused() {
        let home = PathBuf::from("/Users/tester");
        let mut request = a_request(&home);
        request.program = PathBuf::from("/nowhere/phairdlna");
        let error = ServiceLayout::build(&request).expect_err("the layout is refused");

        assert!(error.contains("does not exist"), "{error}");
    }

    /// The negative case: a relative program is refused.
    #[test]
    fn a_relative_program_is_refused() {
        let home = PathBuf::from("/Users/tester");
        let mut request = a_request(&home);
        request.program = PathBuf::from("phairdlna");
        let error = ServiceLayout::build(&request).expect_err("the layout is refused");

        assert!(error.contains("absolute"), "{error}");
    }

    /// The negative case: a job file outside the folder of the platform is
    /// refused, so that the command cannot write a file where it should not.
    #[test]
    fn a_job_file_in_the_wrong_place_is_refused() {
        let home = PathBuf::from("/Users/tester");
        let mut layout = ServiceLayout::build(&a_request(&home)).expect("a layout");
        layout.unit_path = PathBuf::from("/tmp/org.phairdlna.server.plist");

        let error = layout.validate().expect_err("the layout is refused");
        assert!(error.contains("must lie in"), "{error}");
    }

    /// The negative case: a log file that is the job file is refused.
    #[test]
    fn a_log_file_that_is_the_job_file_is_refused() {
        let home = PathBuf::from("/Users/tester");
        let mut layout = ServiceLayout::build(&a_request(&home)).expect("a layout");
        layout.stdout_path = layout.unit_path.clone();

        let error = layout.validate().expect_err("the layout is refused");
        assert!(error.contains("must not be the file"), "{error}");
    }
}
