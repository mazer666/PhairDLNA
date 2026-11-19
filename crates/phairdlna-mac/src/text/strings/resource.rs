//! The embedded translations and the rules for choosing one.
//!
//! What: parses the text resources and maps system language names to files.
//! Why: translation selection and file validation are separate from lookup.
//! Example: `de_AT.UTF-8` resolves to the shipped `de-DE` resource.

use std::collections::BTreeMap;

use super::StringsError;

/// The language used when no supported system language is available.
pub(super) const FALLBACK_LOCALE: &str = "en-US";

/// Translation files embedded in the application binary.
pub(super) const RESOURCES: &[(&str, &str)] = &[
    ("en-US", include_str!("../../../resources/en-US.strings")),
    ("en-GB", include_str!("../../../resources/en-GB.strings")),
    ("de-DE", include_str!("../../../resources/de-DE.strings")),
    ("fr-FR", include_str!("../../../resources/fr-FR.strings")),
    ("es-ES", include_str!("../../../resources/es-ES.strings")),
];

/// Finds a shipped resource, including language-only fallback.
pub(super) fn resource_of(locale: &str) -> Option<(&'static str, &'static str)> {
    let wanted = normalise_locale(locale);
    if let Some(found) = RESOURCES
        .iter()
        .find(|(name, _)| name.eq_ignore_ascii_case(&wanted))
        .copied()
    {
        return Some(found);
    }
    let prefix = wanted.split('-').next().unwrap_or(&wanted);
    match prefix.to_ascii_lowercase().as_str() {
        "de" => resource_of("de-DE"),
        "fr" => resource_of("fr-FR"),
        "es" => resource_of("es-ES"),
        "en" => resource_of("en-US"),
        _ => None,
    }
}

/// Removes an encoding suffix and standardises the locale separator.
pub(super) fn normalise_locale(raw: &str) -> String {
    let without_encoding = raw.split(['.', '@']).next().unwrap_or(raw);
    without_encoding.trim().replace('_', "-")
}

/// Reads the supported language from standard process locale variables.
pub(super) fn current_locale() -> &'static str {
    for name in ["LC_ALL", "LC_MESSAGES", "LANG"] {
        let Ok(value) = std::env::var(name) else {
            continue;
        };
        if value.trim().is_empty() {
            continue;
        }
        if let Some((locale, _)) = resource_of(&value) {
            return locale;
        }
        break;
    }
    FALLBACK_LOCALE
}

/// Parses a translation file, refusing malformed or duplicate entries.
pub(super) fn parse_resource(source: &str) -> Result<BTreeMap<String, String>, StringsError> {
    let mut texts = BTreeMap::new();
    for (number, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let (key, value) = parse_entry(trimmed).map_err(|reason| StringsError {
            reason: format!("line {} of the resource file: {reason}", number + 1),
        })?;
        if texts.insert(key.clone(), value).is_some() {
            return Err(StringsError {
                reason: format!(
                    "line {} of the resource file: the key {key} is written twice",
                    number + 1
                ),
            });
        }
    }
    Ok(texts)
}

/// Reads the key and text from one resource-file line.
fn parse_entry(line: &str) -> Result<(String, String), &'static str> {
    let Some((key, value)) = line.split_once('=') else {
        return Err("the line has no `=` between the key and the value");
    };
    let key = key.trim();
    let value = value.trim();
    if key.is_empty() {
        return Err("the line has no key in front of the `=`");
    }
    if !key
        .chars()
        .all(|character| character.is_ascii_alphanumeric() || character == '.' || character == '_')
    {
        return Err(
            "the key holds a character that is not a letter, a digit, a dot or an underscore",
        );
    }
    if value.is_empty() {
        return Err("the value of the key is empty");
    }
    Ok((key.to_owned(), value.to_owned()))
}
