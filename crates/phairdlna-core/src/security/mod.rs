//! The security rules of the server.
//!
//! What: the bounds the server keeps inside (`limits`), the pseudonyms that
//!       replace the private values of a log record (`redaction`), and the
//!       names the server answers to (`host`).
//! Why: the core owns the rules and the socket code owns the sockets
//!       (`ADR-0001`). A rule that can be tested without a network is a rule
//!       that is actually tested, and the threat model of the project asks for
//!       exactly that: every threat has a negative test that fails without the
//!       mitigation (`SECURITY-MODEL` § 7).
//! Example:
//!
//! ```
//! use phairdlna_core::security::HostPolicy;
//!
//! let policy = HostPolicy::new(vec!["phairdlna.local".to_owned()]);
//! assert!(policy.accepts(Some("phairdlna.local:8200")));
//! assert!(!policy.accepts(Some("attacker.example")));
//! ```

pub mod containment;
pub mod framing;
pub mod host;
pub mod limits;
pub mod redaction;

pub use containment::{is_inside_roots, resolve_reference, ContainmentError};
pub use framing::{check, values_of, FramingError, HeaderPairs};
pub use host::HostPolicy;
pub use limits::{header_block_size, Limits};
pub use redaction::Redactor;
