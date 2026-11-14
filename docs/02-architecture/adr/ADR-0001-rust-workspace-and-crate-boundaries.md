# ADR-0001 — Rust workspace and crate boundaries

Status: **Accepted**
Date: 2026-11-14
Owner role: Architect
Requirements: `PLT-001`, `PLT-002`, `PLT-003`, `PLT-010`, `DOC-003`

## Context

PhairDLNA must ship a portable core plus three different front ends (a headless
daemon, a local administration surface and a native macOS application). The core has
to compile unchanged on macOS, Linux and Windows, and it has to be possible to test a
protocol module without starting a server.

A single crate with modules inside it would make the platform boundary a convention
rather than a rule: a single `use libc::` or `use windows_sys::` anywhere would drag
that dependency into every target. The dependency direction also has to be provable by
a script, not by a reviewer's memory.

## Decision

The project is a Cargo workspace with four crates and a strictly one-way dependency
direction:

```
phairdlna-mac  ─┐
phairdlna-admin ┼──▶ phairdlna-cli ──▶ phairdlna-core
phairdlna-cli  ─┘
```

- `phairdlna-core` is the portable domain core. It depends on **no sibling crate** and
  on **no OS-specific crate**. Everything platform-specific is reached through the
  `Platform` trait (ADR-0003).
- `phairdlna-cli` owns the process lifecycle, the CLI and the daemon wiring.
- `phairdlna-admin` is an optional local administration surface. It is a separate
  crate so that the server can be built *without* it.
- `phairdlna-mac` is an optional native macOS front end, compiled only on macOS.

The direction is checked mechanically by `scripts/check-crate-boundaries.sh`
(gate QG-07), which parses the `Cargo.toml` dependency graph and fails when a crate
depends downwards. The gate must have a negative test: an intentionally inverted
dependency has to turn the gate red.

## Consequences

- The portability requirement becomes mechanically verifiable instead of aspirational.
- Every feature has an obvious home: protocol logic in `core`, behaviour in `cli`,
  a front end in `admin` or `mac`.
- A feature that needs the macOS app cannot live in `core`; it needs a trait in
  `core` and an implementation in `mac`. This costs an abstraction, and buys a core
  that compiles on three platforms.
- The crate list will grow. Each new crate needs a reason in an ADR; the default is
  to stay inside an existing crate.

## Alternatives considered

- **One crate with a `platform` module.** Rejected: the platform boundary would be
  unenforceable, and `cfg(target_os)` branches would spread through the core.
- **A plugin architecture with dynamic loading.** Rejected: it makes the licence
  audit and the security model unbounded, and no requirement asks for it.
- **Separate repositories per crate.** Rejected: a newcomer would need five clones to
  build anything.

## Revisit condition

Revisit if a fifth front end appears that shares nothing with the CLI, or if the
`phairdlna-admin` crate grows a public API that other crates start to depend on
directly.
