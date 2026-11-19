//! The reader of the sources of the crate.
//!
//! What: the reader that finds the texts of a Rust source and the reader
//!       that leaves the modules of the tests out.
//! Why: the test of the texts of the user interface reads the sources of the
//!      crate, and the reader of a language needs more room than a test file
//!      should hold (`F-15` section 11).
//! Example: the reader finds the literal of a text of the user and the line
//!          it stands in.

/// The expected locale keys and non-translatable source literals.
pub mod locale_contract;

use std::path::Path;

/// The string literals of a source, with the line they stand in.
///
/// The reader understands the comments, the texts, the raw texts, the
/// characters and the byte characters of the language of Rust: a text that
/// begins with a quote inside a character is not the beginning of a text.
pub fn string_literals(source: &str) -> Vec<(usize, String)> {
    let mut literals = Vec::new();
    let mut line = 1usize;
    let mut characters = source.chars().peekable();
    while let Some(character) = characters.next() {
        match character {
            '\n' => line += 1,
            '/' if characters.peek() == Some(&'/') => skip_line_comment(&mut characters, &mut line),
            '/' if characters.peek() == Some(&'*') => {
                skip_block_comment(&mut characters, &mut line)
            }
            '"' => {
                let start_line = line;
                read_text(&mut characters, &mut line, &mut literals, start_line);
            }
            '\'' => skip_character(&mut characters),
            'r' if characters.peek() == Some(&'"') => {
                skip_raw_text(&mut characters, &mut line);
            }
            _ => {}
        }
    }
    literals
}

/// Reads a text and remembers it.
pub fn read_text(
    characters: &mut std::iter::Peekable<std::str::Chars>,
    line: &mut usize,
    literals: &mut Vec<(usize, String)>,
    start_line: usize,
) {
    let mut text = String::new();
    while let Some(character) = characters.next() {
        match character {
            '"' => {
                literals.push((start_line, text));
                return;
            }
            '\\' => {
                text.push(character);
                if let Some(next) = characters.next() {
                    text.push(next);
                }
            }
            '\n' => {
                *line += 1;
                text.push(character);
            }
            _ => text.push(character),
        }
    }
}

/// Skips a raw text, which ends at the first quote and a number of hashes.
pub fn skip_raw_text(characters: &mut std::iter::Peekable<std::str::Chars>, line: &mut usize) {
    let mut hashes = 0usize;
    while characters.peek() == Some(&'#') {
        hashes += 1;
        characters.next();
    }
    if characters.next() != Some('"') {
        return;
    }
    for character in characters.by_ref() {
        if character == '\n' {
            *line += 1;
        }
        if character == '"' {
            break;
        }
    }
    let mut closed = 0usize;
    while closed < hashes && characters.peek() == Some(&'#') {
        characters.next();
        closed += 1;
    }
}

/// Skips a character or the beginning of a lifetime.
pub fn skip_character(characters: &mut std::iter::Peekable<std::str::Chars>) {
    match characters.peek() {
        Some('\\') => {
            characters.next();
            characters.next();
            if characters.peek() == Some(&'\'') {
                characters.next();
            }
        }
        Some(_) => {
            characters.next();
            if characters.peek() == Some(&'\'') {
                characters.next();
            }
        }
        None => {}
    }
}

/// Skips a comment that ends with the line.
pub fn skip_line_comment(characters: &mut std::iter::Peekable<std::str::Chars>, line: &mut usize) {
    for next in characters.by_ref() {
        if next == '\n' {
            *line += 1;
            break;
        }
    }
}

/// Skips a comment that ends with a star and a slash.
pub fn skip_block_comment(characters: &mut std::iter::Peekable<std::str::Chars>, line: &mut usize) {
    characters.next();
    let mut previous = '\0';
    for next in characters.by_ref() {
        if next == '\n' {
            *line += 1;
        }
        if previous == '*' && next == '/' {
            return;
        }
        previous = next;
    }
}

/// Whether a literal looks like a text the user reads.
///
/// The answer needs the list of the literals of the protocol: a literal of
/// the protocol is not a text of the user.
pub fn looks_like_a_user_text(literal: &str, technical: &[&str]) -> bool {
    if technical.contains(&literal) {
        return false;
    }
    if !literal.contains(' ') {
        return false;
    }
    literal
        .split(|character: char| !character.is_alphabetic())
        .any(|word| word.len() >= 2 && word.starts_with(|character: char| character.is_uppercase()))
}

/// The source without the modules of the tests.
///
/// A test may hold the sentence it checks for; the code of the application
/// may not.
pub fn without_the_tests(source: &str) -> String {
    let mut kept = String::new();
    let mut rest = source;
    while let Some(start) = rest.find("#[cfg(test)]") {
        kept.push_str(&rest[..start]);
        let tail = &rest[start..];
        if tail.find('{').is_none() {
            return kept;
        }
        let mut depth = 0i32;
        let mut opened = false;
        let mut end = tail.len();
        for (offset, character) in tail.char_indices() {
            if character == '{' {
                depth += 1;
                opened = true;
                continue;
            }
            if character != '}' {
                continue;
            }
            depth -= 1;
            if opened && depth == 0 {
                end = offset + 1;
                break;
            }
        }
        rest = &tail[end..];
    }
    kept.push_str(rest);
    kept
}

/// Collects the literals of the sources that look like a text of the user
/// interface, with the file and the line they stand in.
pub fn collect_user_texts(directory: &Path, technical: &[&str], found: &mut Vec<String>) {
    for path in sources_of(directory) {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (line, literal) in string_literals(&source) {
            if looks_like_a_user_text(&literal, technical) {
                found.push(format!("{}:{}: {literal}", path.display(), line));
            }
        }
    }
}

/// Collects the literals of the code that are not written down.
pub fn collect_unknown_literals(
    directory: &Path,
    technical: &[&str],
    allowed: &[&str],
    found: &mut Vec<String>,
) {
    for path in sources_of(directory) {
        let Ok(source) = std::fs::read_to_string(&path) else {
            continue;
        };
        for (line, literal) in string_literals(&without_the_tests(&source)) {
            if is_unknown_literal(&literal, technical, allowed) {
                found.push(format!("{}:{}: {literal}", path.display(), line));
            }
        }
    }
}

/// Whether a literal of the code is one that is not written down.
pub fn is_unknown_literal(literal: &str, technical: &[&str], allowed: &[&str]) -> bool {
    if !literal.contains(' ') {
        return false;
    }
    if technical.contains(&literal) || allowed.contains(&literal) {
        return false;
    }
    true
}

/// The Rust sources of a directory.
fn sources_of(directory: &Path) -> Vec<std::path::PathBuf> {
    let mut sources = Vec::new();
    let Ok(entries) = std::fs::read_dir(directory) else {
        return sources;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            sources.extend(sources_of(&path));
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            sources.push(path);
        }
    }
    sources
}
