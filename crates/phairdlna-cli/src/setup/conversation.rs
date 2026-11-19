//! The interactive questions used by the CLI setup command.
//!
//! What: read the server name, media folders and login-start preference.
//! Why: collect the owner's intent without coupling the questions to terminal globals.
//! Example: tests can provide a byte reader and capture the prompts in a byte buffer.

use std::io::{BufRead, Write};

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
