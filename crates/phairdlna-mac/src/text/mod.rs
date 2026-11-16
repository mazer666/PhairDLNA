//! The texts of the user interface.
//!
//! What: the module that holds the strings the user reads, one resource file
//!       per language, and the function that fills the placeholders of a
//!       string.
//! Why: a string that lives in the code is a string that cannot be
//!       translated, and a string that is built by concatenation is a string
//!       that no translator can put into another word order (`UX-020`).
//! Example: the resource file of `en-US` holds the line
//!          `menu.status.running = Server running — {streams} active streams`,
//!          and the code asks for that key with the number of the streams.
//!
//! **The placeholders are part of the contract.** A string is never built by
//! joining pieces in the code: the resource file holds the whole sentence
//! with named placeholders, and the code fills them in. A translator can
//! therefore move a placeholder anywhere in the sentence.

pub mod strings;

pub use strings::{Strings, StringsError, SUPPORTED_LOCALES};
