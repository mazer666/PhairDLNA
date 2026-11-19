//! The texts of the user interface, one resource file per language.
//!
//! What: the strings the user reads, the loader that reads them and the
//!       function that fills the placeholders of a string.
//! Why: a text that lives in the code cannot be translated, and a text that
//!      is built by concatenation cannot be put into another word order
//!      (`UX-020`). The texts therefore live in `resources/<language>.strings`
//!      and are embedded in the binary, because the contents of an `.app`
//!      bundle may change and a text read from the disk may be missing.
//! Example: the resource file of `en-US` holds the line
//!          `menu.status.running = Server running — {streams} active streams`,
//!          and the code asks for that key with the number of the streams.
//!
//! **The format of a resource file.** One entry per line, `key = value`. A
//! line that starts with `#` is a comment and an empty line is ignored; a
//! line that is neither is an error, so that a resource file with a mistake
//! is refused instead of silently losing a text. A key is made of the
//! letters, the digits, the dots and the underscores — the keys of the
//! application are held in the localized resource files.
//!
//! **The placeholders are part of the contract.** A placeholder is written
//! `{name}` in the resource file. A placeholder without an argument stays
//! visible, so that a missing value is seen instead of disappearing.

use std::collections::BTreeMap;
use std::fmt;

mod format;
mod resource;

/// The languages the application ships with.
pub const SUPPORTED_LOCALES: &[&str] = &["en-US", "en-GB", "de-DE", "fr-FR", "es-ES"];

/// The texts of one language.
#[derive(Clone, Debug)]
pub struct Strings {
    /// The name of the language, for example `en-US`.
    locale: &'static str,
    /// The texts of the language, by key. The map is sorted, so that the
    /// keys come out in alphabetical order.
    texts: BTreeMap<String, String>,
}

impl Strings {
    /// Reads the texts of a language.
    ///
    /// The name of the language is matched without regard to the case and to
    /// the separator, so that `en-US`, `en_US` and `en-us` all name the same
    /// language. A language the application does not ship is refused.
    ///
    /// # Example
    ///
    /// ```
    /// use phairdlna_mac::text::Strings;
    ///
    /// let strings = Strings::load("en-GB").expect("the language is shipped");
    /// assert_eq!(strings.locale(), "en-GB");
    /// ```
    pub fn load(locale: &str) -> Result<Strings, StringsError> {
        let (name, source) = resource::resource_of(locale).ok_or_else(|| StringsError {
            reason: format!(
                "the language {locale} is not supported; the application ships with {}",
                SUPPORTED_LOCALES.join(", ")
            ),
        })?;
        let texts = resource::parse_resource(source)?;
        Ok(Strings {
            locale: name,
            texts,
        })
    }

    /// Reads the texts of the language of the environment.
    ///
    /// The language comes from `LC_ALL`, then `LC_MESSAGES`, then `LANG`; a
    /// value such as `de_DE.UTF-8` means the language `de-DE`. A language
    /// the application does not ship falls back to `en-US`.
    ///
    /// **The environment is read once, here.** The application reads the
    /// language when it starts and keeps it for the life of the process, so
    /// that a change of the environment in the middle of the run cannot make
    /// two menus show two languages.
    pub fn for_current_locale() -> Result<Strings, StringsError> {
        Strings::load(resource::current_locale())
    }

    /// Reads the texts of the first supported language in the operating
    /// system's preferred language list, then falls back to the environment.
    ///
    /// macOS applications launched from Finder do not necessarily inherit a
    /// useful `LANG` value. Passing the ordered preferences from Foundation
    /// makes the native app follow the language the user selected in macOS.
    ///
    /// # Example
    ///
    /// ```
    /// use phairdlna_mac::text::Strings;
    ///
    /// let preferences = vec!["de-AT".to_owned(), "en-US".to_owned()];
    /// let strings = Strings::for_preferred_locales(&preferences).unwrap();
    /// assert_eq!(strings.locale(), "de-DE");
    /// ```
    pub fn for_preferred_locales(preferred_locales: &[String]) -> Result<Strings, StringsError> {
        for preference in preferred_locales {
            if let Some((locale, _)) = resource::resource_of(preference) {
                return Strings::load(locale);
            }
        }
        Strings::for_current_locale()
    }

    /// The texts of the language the application falls back to.
    ///
    /// The answer is the texts of `en-US`, and the texts of no language at all
    /// when even that file cannot be read. The application must draw its menu
    /// even in that case, and an empty text is better than a crash.
    pub fn fallback() -> Strings {
        Strings::load(resource::FALLBACK_LOCALE).unwrap_or_else(|_| Strings {
            locale: resource::FALLBACK_LOCALE,
            texts: BTreeMap::new(),
        })
    }

    /// The name of the language of these texts.
    pub fn locale(&self) -> &str {
        self.locale
    }

    /// The text of a key, with its placeholders untouched.
    ///
    /// A key the language does not carry is an empty text, never a panic:
    /// the menu must draw even if a text is missing.
    pub fn text(&self, key: &str) -> &str {
        self.texts.get(key).map(String::as_str).unwrap_or("")
    }

    /// The text of a key, with the named placeholders filled in.
    ///
    /// A placeholder is written `{name}` in the resource file. A placeholder
    /// without an argument is left as it is, so that the missing value stays
    /// visible instead of disappearing silently.
    ///
    /// # Example
    ///
    /// ```
    /// use phairdlna_mac::text::Strings;
    ///
    /// let strings = Strings::load("en-US").expect("the language is shipped");
    /// let text = strings.format("menu.status.running", &[("streams", "2")]);
    /// assert!(text.contains('2'));
    /// ```
    pub fn format(&self, key: &str, arguments: &[(&str, &str)]) -> String {
        format::fill_placeholders(self.text(key), arguments)
    }

    /// The keys of the language, in alphabetical order.
    pub fn keys(&self) -> Vec<&str> {
        self.texts.keys().map(String::as_str).collect()
    }
}

/// Why the texts of a language could not be read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StringsError {
    /// What went wrong, in words meant for a person.
    pub reason: String,
}

impl fmt::Display for StringsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.reason)
    }
}

impl std::error::Error for StringsError {}

#[cfg(test)]
mod tests {
    use super::*;

    /// The languages of the constant are the languages of the embedded files.
    #[test]
    fn the_supported_locales_are_the_embedded_resources() {
        let embedded: Vec<&str> = resource::RESOURCES.iter().map(|(name, _)| *name).collect();
        assert_eq!(SUPPORTED_LOCALES, embedded.as_slice());
    }

    /// An entry is read, and the comments and the empty lines are ignored.
    #[test]
    fn an_entry_is_read() {
        let texts =
            resource::parse_resource("# a comment\n\nmenu.a = first\n").expect("the file is read");
        assert_eq!(texts.get("menu.a").map(String::as_str), Some("first"));
    }

    /// A resource file with a mistake is refused, and the reason names it.
    #[test]
    fn a_mistake_in_the_resource_file_is_refused() {
        for (source, expected) in [
            ("menu.a first\n", "="),
            ("menu a = first\n", "key"),
            ("menu.a = first\nmenu.a = second\n", "twice"),
            ("menu.a =\n", "empty"),
        ] {
            let error = resource::parse_resource(source).expect_err("the line is refused");
            assert!(error.reason.contains(expected), "{}", error.reason);
        }
    }

    /// A language that is not shipped is refused.
    #[test]
    fn a_language_that_is_not_shipped_is_refused() {
        let error = Strings::load("it-IT").expect_err("the language is not shipped");
        assert!(error.reason.contains("it-IT"), "{}", error.reason);
    }

    /// The name of a language loses its encoding and its underscores.
    #[test]
    fn a_language_name_is_normalised() {
        assert_eq!(resource::normalise_locale("de_DE.UTF-8"), "de-DE");
        assert_eq!(resource::normalise_locale("en_GB"), "en-GB");
    }

    /// A placeholder with an argument is filled in, and a placeholder without
    /// an argument stays visible.
    #[test]
    fn the_placeholders_are_filled_in() {
        let filled = format::fill_placeholders("running — {streams} streams", &[("streams", "2")]);
        assert_eq!(filled, "running — 2 streams");
        let kept = format::fill_placeholders("running — {streams} streams", &[]);
        assert_eq!(kept, "running — {streams} streams");
    }
}
