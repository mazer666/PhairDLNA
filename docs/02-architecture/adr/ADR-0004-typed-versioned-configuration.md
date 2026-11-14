# ADR-0004 — One typed, validated and versioned configuration

Status: **Accepted**
Date: 2026-11-14
Owner role: Architect
Requirements: `CFG-001` … `CFG-015`, `SEC-013`, `UX-004`

## Context

Configuration is where a media server most often becomes unsafe or unusable: a typo
silently disables a security setting, a missing field produces a cryptic error, or two
modules read the same setting differently and disagree.

Scattered `std::env::var` calls spread the schema across the code. A hand-rolled
parser accepts invalid values. A configuration format without a version cannot be
changed safely once users exist.

## Decision

- **One configuration module** (`core::config`) owns the schema. No other module
  reads an environment variable or a configuration file directly.
- The configuration is a **typed document** (TOML) deserialised into Rust structs
  with a **deny-unknown-fields** policy: an unknown key is an error, never a silent
  no-op (`CFG-013`).
- **Validation happens once, at start-up.** A typed value is not "validated" if the
  check lives at the point of use. Every field has a documented default, and an empty
  configuration file yields a working, safe server (`CFG-004`).
- **The file carries `schema_version`.** An older known version is migrated
  automatically in memory, with a log line naming the migration (`CFG-003`).
- **Errors are actionable**: the message names the field, the file, the expected type
  and the accepted values (`CFG-002`). The same formatter is used for the CLI and for
  the macOS application, so the wording is identical everywhere.
- **Secrets are not configuration values.** A token that must be secret is read from a
  file with owner-only permissions, referenced by path (`CFG-014`).
- The configuration is **versioned with the software**: a setting that changes meaning
  gets a migration, never a silent reinterpretation.

## Consequences

- A bad configuration is caught at start-up with a message a user can act on, rather
  than at the moment a client tries to stream a file.
- A typo can no longer silently weaken the configuration.
- Configuration is testable: the whole schema can be validated from a table of
  examples, and the migration path from every old version is testable.
- The cost is that adding a setting means touching the schema, its default, its
  documentation and its test — four steps instead of one line of code. That is the
  intended friction.

## Alternatives considered

- **Environment variables only.** Rejected: unreadable for a user with many settings,
  and awkward for nested structures such as per-root options.
- **JSON instead of TOML.** Rejected: TOML is more forgiving of hand editing, has
  comments (important for a file users are told to edit), and is already the de-facto
  choice for Rust configuration.
- **Runtime-reloadable configuration.** Rejected for the baseline: it multiplies the
  state combinations that must be tested. A `reload` command that re-reads and
  re-validates the file is the acceptable later step.

## Revisit condition

Revisit when the configuration needs to be changed by a running macOS application, or
when more than three profiles (default / strict / legacy) become necessary.
