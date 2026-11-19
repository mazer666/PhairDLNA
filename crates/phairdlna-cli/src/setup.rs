//! The command that prepares a first PhairDLNA installation.
//!
//! What: coordinate the setup questions, configuration validation and optional service
//!       registration.
//! Why: first-run behavior should be understandable and testable without mixing the
//!       conversation, file format and command-line lifecycle.
//! Example: `phairdlna-cli setup` asks for a server name and media folders, then writes
//!          a validated configuration file.

use std::path::{Path, PathBuf};

use crate::service::ServiceCommand;
use crate::Outcome;

mod config_file;
mod conversation;

pub use config_file::{check, render, write};
pub use conversation::{ask, Answers};

/// Asks the questions and writes the configuration file.
///
/// The helper is the first step of the journey of a new user: it writes the
/// file, tells the owner where it is and says what to do next (`UX-003`).
pub fn run(path: Option<&Path>, force: bool) -> Outcome {
    let path = crate::resolve(path);
    if path.exists() && !force {
        return Outcome::failure(format!(
            "the configuration \"{}\" is already there.\n\
             Change it by hand, or run the helper again with --force to replace it.\n",
            path.display()
        ));
    }
    let mut prompts = std::io::stdout();
    let stdin = std::io::stdin();
    let mut reader = stdin.lock();
    let answers = match ask(&mut reader, &mut prompts) {
        Ok(answers) => answers,
        Err(reason) => return Outcome::failure(format!("{reason}\n")),
    };
    let text = render(&answers);
    if let Err(reason) = check(&text) {
        return Outcome::failure(format!("{reason}\n"));
    }
    if let Err(reason) = write(&path, &text) {
        return Outcome::failure(format!(
            "the configuration cannot be written to \"{}\": {reason}\n",
            path.display()
        ));
    }
    let mut message = format!(
        "The configuration was written to \"{}\".\n\n",
        path.display()
    );
    message.push_str(&explanation());
    if answers.start_at_login {
        let service = install_service(&path);
        if service.exit_code == 0 {
            message.push_str("The server was registered to start when you log in.\n\n");
        } else {
            message.push_str(&service.stdout);
            message.push('\n');
        }
    }
    message.push_str(&next_steps(&answers));
    Outcome::ok(message)
}

/// The explanation of the app's local connection.
///
/// The three questions of the requirement are answered in the words of the
/// user: what is this, who can see it, how to turn it off (`OPS-026`).
pub fn explanation() -> String {
    String::from(
        "\nThe PhairDLNA app's local connection\n\
         ----------------------------------\n\
         The app uses a local service at http://127.0.0.1:8210 . It shows server\n\
         status and diagnostics and can refresh your library. Only programs on\n\
         this Mac can connect, and it asks for the key in the file\n\
         ~/.config/phairdlna/admin.token, which only you can read. You do not\n\
         have to use this directly. To switch it off, set \"admin_enabled = false\"\n\
         in the configuration file and start the server again.\n\n",
    )
}

/// What the owner of the machine can do next.
fn next_steps(answers: &Answers) -> String {
    let mut text = String::from("What to do next\n----------------\n");
    if answers.start_at_login {
        text.push_str("The server was not started yet. Start it now with:\n\n");
        text.push_str("    phairdlna serve\n\n");
        text.push_str("To let it start when you log in, run:\n\n");
        text.push_str("    phairdlna service install\n\n");
    } else {
        text.push_str("Start the server with:\n\n");
        text.push_str("    phairdlna serve\n\n");
        text.push_str("Your television and your other devices will find it within a few\n");
        text.push_str("seconds. To let the server start when you log in, run:\n\n");
        text.push_str("    phairdlna service install\n\n");
    }
    text.push_str("If something does not work, write a report for the people who help you:\n\n");
    text.push_str("    phairdlna diagnostics --export report.txt\n");
    text
}

/// The command that installs the service of the user.
pub fn install_service(path: &Path) -> Outcome {
    crate::service::run(&ServiceCommand::Install {
        force: false,
        dry_run: false,
        system: false,
        config: Some(PathBuf::from(path)),
    })
}

#[cfg(test)]
#[path = "setup/tests.rs"]
mod tests;
