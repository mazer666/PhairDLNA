# ADR-0006 — The catalogue is a SQLite index that can be thrown away

Status: **Accepted**
Date: 2026-11-14
Owner role: Architect
Requirements: `FUN-009` … `FUN-012`, `FUN-018`, `SEC-095`, `PERF-003`, `PERF-009`

## Context

A media server needs to answer "what is in this folder?" quickly, with metadata that
is expensive to recompute (duration, dimensions, artwork). Scanning a filesystem on
every browse is unacceptable, and so is recomputing metadata on every start.

At the same time the index must never become a second source of truth that can
disagree with the user's files. A corrupt or stale index that the user cannot repair
is worse than a slow server.

## Decision

- The catalogue is stored in **one SQLite database** owned by the application
  (`FUN-009`).
- **The index is a cache.** The filesystem is the truth. Deleting the database must
  be a supported recovery action that loses nothing but time (`FUN-010`).
- **Object identifiers are computed, not stored.** They are a deterministic function
  of the root id and the path relative to the root (ADR-0007), so they survive a
  rebuild unchanged.
- **Scanning is incremental.** The scanner compares size and modification time
  against the index and re-reads only what changed (`FUN-006`, `PERF-003`).
- **Writes are transactional and batched.** A scan applies its changes in
  transactions; a crash mid-scan leaves the index consistent, not half-updated.
- **Corruption is detected and recovered.** Opening the database verifies its
  integrity; on failure the server rebuilds from scratch and reports that it did so
  (`FUN-010`). The user is never left with a server that only works after manual
  surgery.
- The database file is created with **user-only permissions** (`SEC-095`) in a
  user-owned directory, never in a shared location.
- The SQLite binding is used in its **bundled** configuration with
  `loadable_extension` disabled, so the server does not depend on the system's SQLite
  version and cannot load arbitrary extensions.

## Consequences

- Start-up is fast and independent of library size; a scan of a million files costs
  nothing until something changes.
- A corrupt index is an inconvenience, not a data-loss event — the user's media is
  never inside the database.
- The server depends on SQLite (via a Rust binding). This is a deliberate, recorded
  dependency with a licence assessment (`SUP-002`).
- The scanner and the indexer are separated, so the scanning logic can be tested
  against an in-memory filesystem and the index logic against an in-memory database.

## Alternatives considered

- **No database: scan on demand with an in-memory cache.** Rejected: it makes
  start-up time proportional to library size and re-breaks on every restart.
- **A document store or a full-text search engine.** Rejected: heavier, harder to
  keep transactional, and the access pattern is "children of one folder", which
  SQLite answers directly with an index.
- **Storing the index inside the media root.** Rejected: it pollutes the user's
  folders, risks being picked up by other tools, and breaks on read-only volumes.

## Revisit condition

Revisit if the library model needs many-to-many relationships (for example, the same
file in several playlists with per-playlist metadata) that make a relational schema
awkward, or if a full-text search over metadata becomes a requirement.
