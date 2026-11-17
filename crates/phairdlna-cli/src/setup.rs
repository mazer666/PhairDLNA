//! The helper of the first run.
//!
//! What: the command that asks the three questions of the first run and
//!       writes the configuration file of the server.
//! Why: a user who has never seen the server must be able to bring it up
//!      without reading a file of settings, and the questions must be asked in
//!      the words of the user and not in the words of the protocol
//!      (`OPS-026`, `UX-010`, `UX-003`).
//! Example: `phairdlna setup` asks for the name of the server and for the
//!          folders of the media, and writes a configuration file that explains
//!          every setting it holds.
//!
//! **The file is checked before it is written.** The configuration that the
//! helper builds is read by the reader of the server, and a file that the
//! server would refuse is never written (`CFG-002`).

use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use phairdlna_core::config::Config;

use crate::service::ServiceCommand;
use crate::Outcome;

/// The name the server gets when the owner does not choose one.
const DEFAULT_DEVICE_NAME: &str = "PhairDLNA";

/// The answers of the owner of the machine.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Answers {
    /// The name the server shows in the network.
    pub device_name: String,
    /// The folders that hold the media, by name and path.
    pub roots: Vec<(String, String)>,
    /// Whether the server should start when the owner logs in.
    pub start_at_login: bool,
}

/// Asks the questions of the first run.
///
/// The questions are written to `prompts` and the answers are read from
/// `reader`, so that a test can hold the conversation without a terminal.
pub fn ask(reader: &mut impl BufRead, prompts: &mut impl Write) -> Result<Answers, String> {
    writeln!(
        prompts,
        "Welcome to PhairDLNA.\n\n\
         This helper asks three questions and writes the configuration file for you.\n"
    )
    .map_err(|error| format!("the questions cannot be written: {error}"))?;

    let name = ask_line(
        reader,
        prompts,
        "1. What should the server be called in your network?\n   \
         This is the name your television shows. For example: Living room",
    )?;
    let device_name = if name.trim().is_empty() {
        DEFAULT_DEVICE_NAME.to_owned()
    } else {
        name.trim().to_owned()
    };

    let roots = ask_roots(reader, prompts)?;

    let service = ask_line(
        reader,
        prompts,
        "3. Should the server start when you log in? (yes/no)\n   \
         A service of your user starts with your machine and is not visible to \
         the other users of the machine",
    )?;
    Ok(Answers {
        device_name,
        roots,
        start_at_login: is_yes(&service),
    })
}

/// Asks for the folders that hold the media.
///
/// The list ends with an empty line: an empty line is the only way to say
/// "that was all", because a folder may have any name.
fn ask_roots(
    reader: &mut impl BufRead,
    prompts: &mut impl Write,
) -> Result<Vec<(String, String)>, String> {
    writeln!(
        prompts,
        "2. Which folders hold your films, series and music?\n   \
         One folder per line. Press Enter on an empty line when you are done."
    )
    .map_err(|error| format!("the questions cannot be written: {error}"))?;
    let mut roots: Vec<(String, String)> = Vec::new();
    loop {
        let line = ask_line(reader, prompts, "   >")?;
        let path = line.trim();
        if path.is_empty() {
            break;
        }
        roots.push((name_of_folder(path), path.to_owned()));
    }
    Ok(roots)
}

/// Asks one question and reads the answer.
fn ask_line(
    reader: &mut impl BufRead,
    prompts: &mut impl Write,
    question: &str,
) -> Result<String, String> {
    write!(prompts, "{question}\n> ")
        .map_err(|error| format!("the questions cannot be written: {error}"))?;
    prompts
        .flush()
        .map_err(|error| format!("the questions cannot be written: {error}"))?;
    let mut answer = String::new();
    let read = reader
        .read_line(&mut answer)
        .map_err(|error| format!("the answers cannot be read: {error}"))?;
    if read == 0 {
        return Err("there was no answer; the helper needs an answer to every question".to_owned());
    }
    Ok(answer)
}

/// Whether an answer means yes.
fn is_yes(answer: &str) -> bool {
    matches!(
        answer.trim().to_lowercase().as_str(),
        "y" | "yes" | "j" | "ja" | "sure"
    )
}

/// The name of a folder of the library.
///
/// The name is the last part of the path, so that the configuration is short
/// and the client sees a name the owner recognises.
fn name_of_folder(path: &str) -> String {
    let trimmed = path.trim_end_matches(['/', '\\']);
    let name = trimmed
        .rsplit(['/', '\\'])
        .find(|part| !part.is_empty())
        .unwrap_or(trimmed);
    name.to_owned()
}

/// The configuration file the answers describe.
///
/// The file explains every setting it holds: a user who opens it later must
/// understand it without the documentation (`UX-010`).
pub fn render(answers: &Answers) -> String {
    let mut text = String::new();
    text.push_str("# The configuration of PhairDLNA.\n");
    text.push_str("#\n");
    text.push_str("# This file was written by \"phairdlna setup\". Every setting can be\n");
    text.push_str("# changed by hand; the server reads the file when it starts.\n");
    text.push_str("#\n");
    text.push_str("# The settings and what they mean: phairdlna config show\n\n");
    text.push_str("# The version of the format of this file. Do not change it.\n");
    text.push_str("schema_version = 1\n\n");
    text.push_str("[server]\n");
    text.push_str("# The name your television shows for this server.\n");
    text.push_str(&format!(
        "device_name = {}\n\n",
        quoted(&answers.device_name)
    ));
    text.push_str("# The port the server listens on. 8200 is the usual one.\n");
    text.push_str("http_port = 8200\n\n");
    text.push_str("# The port the announcements go out on. 1900 is the one the standard uses.\n");
    text.push_str("ssdp_port = 1900\n\n");
    text.push_str("[library]\n");
    if answers.roots.is_empty() {
        text.push_str("# No folder is shared yet. Add one like this:\n");
        text.push_str("#\n");
        text.push_str("# [[library.roots]]\n");
        text.push_str("# name = \"Movies\"\n");
        text.push_str("# path = \"/Users/you/Movies\"\n\n");
    } else {
        text.push_str("# The folders that hold your media. You may add more.\n");
        for (name, path) in &answers.roots {
            text.push_str("[[library.roots]]\n");
            text.push_str(&format!("name = {}\n", quoted(name)));
            text.push_str(&format!("path = {}\n\n", quoted(path)));
        }
    }
    text.push_str("[security]\n");
    text.push_str("# The administration surface is a small page on this machine alone, at\n");
    text.push_str("# http://127.0.0.1:8210 . It shows what the server does and lets it read the\n");
    text.push_str("# library again. Nobody else can see it, and the file\n");
    text.push_str("# ~/.config/phairdlna/admin.token is the key to it. Set this to false to\n");
    text.push_str("# switch it off.\n");
    text.push_str("admin_enabled = true\n\n");
    text.push_str("[logging]\n");
    text.push_str("# How much the server writes to its log: debug, info, warn or error.\n");
    text.push_str("level = \"info\"\n\n");
    text.push_str("# The log replaces the paths and the addresses of your machine with\n");
    text.push_str("# names like <path:1>. Keep this on unless you know why you turn it off.\n");
    text.push_str("redact = true\n");
    text
}

/// A text as a value of the configuration.
///
/// A text that carries a quote or a backslash is written with the escapes the
/// format of the file knows.
fn quoted(text: &str) -> String {
    let mut value = String::with_capacity(text.len() + 2);
    value.push('"');
    for character in text.chars() {
        match character {
            '"' => value.push_str("\\\""),
            '\\' => value.push_str("\\\\"),
            '\n' => value.push_str("\\n"),
            other => value.push(other),
        }
    }
    value.push('"');
    value
}

/// Writes the configuration file of the answers.
///
/// The file is checked before it is written: a file that the server would
/// refuse is never written (`CFG-002`).
pub fn write(path: &Path, text: &str) -> Result<(), String> {
    if let Some(folder) = path.parent() {
        if !folder.as_os_str().is_empty() && !folder.exists() {
            std::fs::create_dir_all(folder)
                .map_err(|error| format!("the folder cannot be created: {error}"))?;
        }
    }
    std::fs::write(path, text).map_err(|error| format!("the file cannot be written: {error}"))
}

/// Checks that the server can read the configuration.
///
/// The check is the reader of the server itself: the helper must not write a
/// file that the server would refuse.
pub fn check(text: &str) -> Result<(), String> {
    let path = std::env::temp_dir().join(format!("phairdlna-check-{}.toml", unique()));
    std::fs::write(&path, text).map_err(|error| format!("the check cannot be written: {error}"))?;
    let answer = Config::load(&path)
        .map(|_| ())
        .map_err(|error| format!("the configuration cannot be used:\n{}", error.message()));
    let _ = std::fs::remove_file(&path);
    answer
}

/// A number that makes the name of the file of the check unique.
fn unique() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|since| since.as_nanos())
        .unwrap_or_default()
}

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

/// The explanation of the administration surface.
///
/// The three questions of the requirement are answered in the words of the
/// user: what is this, who can see it, how to turn it off (`OPS-026`).
pub fn explanation() -> String {
    String::from(
        "\nThe administration page\n\
         ----------------------\n\
         The server keeps a small page for itself on this machine alone, at\n\
         http://127.0.0.1:8210 . It shows what the server does and lets it read\n\
         your folders again. Nobody on your network can see it: it listens only\n\
         on this machine, and it asks for the key in the file\n\
         ~/.config/phairdlna/admin.token, which only you can read.\n\
         You do not have to use it. To switch it off, set \"admin_enabled = false\"\n\
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
mod tests {
    use super::*;

    /// The answers of an owner who named the server and one folder.
    fn answers() -> Answers {
        Answers {
            device_name: "Living room".to_owned(),
            roots: vec![("Movies".to_owned(), "/Users/owner/Movies".to_owned())],
            start_at_login: true,
        }
    }

    /// The positive case: the questions are asked and the answers are read.
    #[test]
    fn the_questions_are_asked_and_the_answers_are_read() {
        let input = "Living room\n/Users/owner/Movies\n\nyes\n";
        let mut reader = std::io::BufReader::new(input.as_bytes());
        let mut prompts: Vec<u8> = Vec::new();

        let answers = ask(&mut reader, &mut prompts).expect("the answers are read");

        assert_eq!(answers.device_name, "Living room");
        assert_eq!(
            answers.roots,
            vec![("Movies".to_owned(), "/Users/owner/Movies".to_owned())]
        );
        assert!(answers.start_at_login);
        let text = String::from_utf8(prompts).expect("the prompts are text");
        assert!(text.contains("What should the server be called"), "{text}");
    }

    /// The positive case: the configuration the helper writes is a valid one.
    #[test]
    fn the_configuration_the_helper_writes_is_valid() {
        let text = render(&answers());
        assert!(text.contains("device_name = \"Living room\""));
        assert!(text.contains("name = \"Movies\""));
        assert!(check(&text).is_ok(), "{text}");
    }

    /// The negative case: an answer that stops in the middle is refused.
    #[test]
    fn an_answer_that_stops_in_the_middle_is_refused() {
        let input = "Living room\n";
        let mut reader = std::io::BufReader::new(input.as_bytes());
        let mut prompts: Vec<u8> = Vec::new();
        assert!(ask(&mut reader, &mut prompts).is_err());
    }

    /// The negative case: a name with a quote does not break the file.
    #[test]
    fn a_name_with_a_quote_does_not_break_the_file() {
        let answers = Answers {
            device_name: "The \"Living\" room".to_owned(),
            roots: Vec::new(),
            start_at_login: false,
        };
        let text = render(&answers);
        if let Err(err) = check(&text) {
            panic!("check failed: {err}\nText:\n{text}");
        }
    }

    /// The negative case: the explanation answers the three questions.
    #[test]
    fn the_explanation_answers_the_three_questions() {
        let text = explanation();
        assert!(text.contains("this machine alone"), "{text}");
        assert!(text.contains("Nobody on your network"), "{text}");
        assert!(text.contains("admin_enabled = false"), "{text}");
    }
}
