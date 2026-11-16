//! The memory the process itself occupies.
//!
//! What: the number of bytes the running process holds in memory, when the
//!       platform can be asked.
//! Why: the status output tells the user how much the server uses. A question
//!      about the operating system must not be asked outside the boundary of
//!      the platform traits, or the core stops being portable (`ADR-0003`).
//! Example: on Linux the answer comes from `/proc/self/statm`; on a platform
//!      that does not tell, the answer is `None` and the status output says
//!      that the figure is not available.
//!
//! **The honest answer.** A platform that cannot be read returns `None` and
//! not a guess. A status output that carries a made-up number is worse than a
//! status output that says the number is not known.

use std::path::Path;

/// What the process itself uses.
pub trait ProcessMemory {
    /// The bytes the process occupies, or `None` when the platform does not say.
    fn resident_bytes(&self) -> Option<u64>;
}

/// The memory of the process, read from the platform.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LocalProcessMemory;

impl ProcessMemory for LocalProcessMemory {
    /// Reads the resident size of the process.
    ///
    /// The file `/proc/self/statm` holds the sizes of the pages of the
    /// process; the second number is the resident one. A platform without that
    /// file answers `None`.
    fn resident_bytes(&self) -> Option<u64> {
        resident_bytes_of(Path::new("/proc/self/statm"))
    }
}

/// Reads the resident size of a process out of a `statm` file.
///
/// The function takes the path of the file, so that the rule can be tested
/// without a process of its own.
fn resident_bytes_of(path: &Path) -> Option<u64> {
    let text = std::fs::read_to_string(path).ok()?;
    let pages = text.split_whitespace().nth(1)?;
    let pages: u64 = pages.parse().ok()?;
    Some(pages * page_size())
}

/// The size of a page of memory, in bytes.
///
/// A page of 4096 bytes is the size on every platform the project supports
/// that has a `/proc` at all; the answer is only used where the file exists.
fn page_size() -> u64 {
    4096
}

#[cfg(test)]
mod tests {
    use super::{resident_bytes_of, LocalProcessMemory, ProcessMemory};
    use std::path::Path;

    /// The negative case: a file that does not exist has no answer.
    #[test]
    fn a_file_that_is_not_there_has_no_answer() {
        assert_eq!(resident_bytes_of(Path::new("/proc/self/statm.not")), None);
    }

    /// The positive case: the second number of the file is the resident size.
    #[test]
    fn the_second_number_is_the_resident_size() {
        let folder = tempfile::tempdir().expect("a temporary directory");
        let path = folder.path().join("statm");
        std::fs::write(&path, "1000 250 0 0 0 0 0\n").expect("the file is written");

        assert_eq!(resident_bytes_of(&path), Some(250 * 4096));
    }

    /// The platform that cannot be read says so.
    #[test]
    fn a_platform_without_the_file_says_so() {
        // The answer is `None` on every platform but Linux; the test asserts
        // the contract, not the platform.
        let answer = LocalProcessMemory.resident_bytes();
        if cfg!(target_os = "linux") {
            assert!(answer.is_some(), "Linux tells the resident size");
        } else {
            assert_eq!(answer, None, "this platform does not tell");
        }
    }
}
