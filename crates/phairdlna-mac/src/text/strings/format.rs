//! Placeholder replacement for translated text.
//!
//! What: fills named values in a complete translated sentence.
//! Why: translators must control the order of words and values.
//! Example: `{count} folders` with `count = 2` becomes `2 folders`.

/// Fills named placeholders, leaving unknown placeholders visible.
pub(super) fn fill_placeholders(text: &str, arguments: &[(&str, &str)]) -> String {
    let mut filled = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(open) = rest.find('{') {
        filled.push_str(&rest[..open]);
        let tail = &rest[open + 1..];
        let Some(close) = tail.find('}') else {
            filled.push_str(&rest[open..]);
            return filled;
        };
        let name = &tail[..close];
        match arguments.iter().find(|argument| argument.0 == name) {
            Some(argument) => filled.push_str(argument.1),
            None => {
                filled.push('{');
                filled.push_str(name);
                filled.push('}');
            }
        }
        rest = &tail[close + 1..];
    }
    filled.push_str(rest);
    filled
}
