# PhairDLNA — binding project standards

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Lead / Architect
Applies to: every file, commit, review and release in this repository.

These standards are binding. A change to them requires an owner decision recorded in
`DECISION-LOG.md` plus, for structural changes, an ADR in `../02-architecture/adr/`.

---

## 1. Principles

Best-in-class quality, privacy-first, no ads, no telemetry, no cloud dependency,
AGPL-3.0-or-later. Rust-first and understandable to a programming beginner.
**Requirements before implementation; measured evidence before claims.**

Two rules outrank every convenience in this document:

1. **A gate that cannot fail proves nothing.** Every gate defined here needs a *negative
   test*: the old or broken state must turn the gate red. A gate that only ever passed
   once is not evidence of anything.
2. **A skip is never a pass.** If a check cannot run in the current environment, the
   result is recorded as `NOT RUN — <reason>`. An exit code that means "skipped"
   (for example `77`) is reported as a skip, never as success.

---

## 2. Code size and readability gates

The **canonical register of the gates is `../05-quality/QUALITY-GATES.md`**. The IDs
used here (`QG-nn`) are defined there; this table states what each rule means for the
code, not how it is measured.

| Gate | Target | Hard limit | How it is measured |
| --- | --- | --- | --- |
| Rust source file length | ≤ 300 physical lines | > 500 lines requires a reviewed exception | `scripts/check-file-size.sh` (QG-13) |
| Function / method length | ≤ 40 lines | > 60 lines requires refactoring or a reviewed exception | `scripts/check-file-size.sh` + reviewer |
| Cyclomatic complexity | ≤ 10 | > 15 requires refactoring | `clippy::excessive_nesting` + `clippy::too_many_lines` (QG-14) |
| Nesting depth | ≤ 3 | > 4 requires refactoring | reviewer + `rust-code-analysis` (planned) |
| Rust file count per module | ≤ 20 | — | reviewer |

Generated files, fixtures and test-data blobs are excluded from the line gates **only**
when they are explicitly classified as generated in the file header and listed in
`../05-quality/QUALITY-GATES.md`. Silent exclusions are not allowed.

### 2.1 Naming

- Meaningful English names. No unexplained abbreviations, no cryptic one-letter variables
  (loop indices and short local bindings are the only exception).
- No "generic utility" grab bags: a module named `utils`, `helpers`, `common` or `misc`
  must be split by responsibility before review.
- Public types and functions use the project's domain vocabulary (`MediaRoot`,
  `ContentDirectory`, `ObjectId`, `ProtocolInfo`, `DeviceProfile`) consistently.

### 2.2 Documentation in code

Every module carries a doc comment stating **what it is, why it exists, and one usage
example**. Every public item has rustdoc. Comments explain *why*, invariants, failure
modes and security assumptions. Comments that merely restate the code are removed.

### 2.3 Error handling

- Explicit error types with context; a single crate-level error enum per bounded domain.
- No unjustified `unwrap()`, `expect()`, `panic!`, `todo!()` or `unimplemented!()` on any
  path reachable from network input, configuration input or the filesystem.
- No silently swallowed errors. A recoverable error is logged (redacted) and counted; an
  unrecoverable one is propagated.
- Panics are reserved for programmer errors detected by `debug_assert` and for startup
  configuration errors that abort with a readable message and a non-zero exit code.

### 2.4 Structure and concurrency

- Single responsibility per module and per file.
- Explicit boundaries; dependency injection where it buys testability.
- Bounded concurrency: every queue, channel and worker pool has a documented bound.
- Resource ownership is explicit; no global mutable state; no interior mutability
  without a documented reason.

### 2.5 Configuration

- One centralized, typed, validated, versioned configuration module. No scattered
  `std::env::var` reads outside that module.
- Defaults are documented and safe. A missing or invalid value produces an actionable
  error naming the field, the file, the expected type and the accepted values.

### 2.6 Unsafe

`unsafe` is forbidden by default. Any use requires an ADR, a written safety argument
and a comment at every `unsafe` block explaining the invariant it upholds.

---

## 3. Dependencies

- No dependency without a recorded entry in `../04-security/DEPENDENCY-LICENSE-REVIEW.md`
  (name, version, licence, why it is needed, maintenance status, alternatives).
- Prefer well-maintained, widely used crates with permissive licences (MIT, Apache-2.0,
  BSD). Copyleft dependencies need an explicit compatibility assessment.
- A dependency that pulls in a native toolchain (OpenSSL, FFmpeg, libav\*) needs an
  owner decision, because it changes distribution, signing and licensing.
- Licence and advisory checks (`cargo deny`, `cargo audit`) are release gates.

---

## 4. Tests

- Every feature ships with tests covering **positive, negative, boundary, recovery and
  security-relevant cases**.
- No feature is "done" without updated documentation, updated `WHERE_TO_CHANGE.md` and a
  traceability entry in `../07-project/TRACEABILITY-MATRIX.md`.
- A test that is skipped, ignored or commented out to make a run green is forbidden.
- No stub, `todo!()` or fake integration is represented as finished work.

---

## 5. Engineering hygiene

- `cargo fmt --all -- --check`
- `cargo clippy --workspace --all-targets -- -D warnings` (lint policy reviewed, not
  silenced ad hoc; an `#[allow]` needs a comment justifying it)
- `cargo test --workspace`
- Dependency, licence and advisory checks
- Code-size and complexity checks
- Documentation link check

These commands are the *intended* gates. A gate counts as passed only when its output has
been observed and recorded; the instructions alone are not evidence.

---

## 6. Change acceptance

Every change updates, as applicable: the requirement/feature spec, the implementation,
the tests, `WHERE_TO_CHANGE.md`, the architecture or security documents, and the
evidence record.

Stop and escalate material **scope, security, licence, or release** decisions to the
owner. Do not resolve them unilaterally.

---

## 7. Release

- GitHub Actions are **manual only** until the owner approves otherwise
  (see `../07-project/CI-POLICY.md`).
- No public release and no new Git tag without explicit owner approval.
- Release criteria are listed in `../07-project/RELEASE-BLOCKERS.md` and
  `../08-releases/RELEASE-POLICY.md`.
- Superseded approved baselines are preserved unchanged in `../99-archive/`; decisions
  are never silently rewritten.

---

## 8. Exceptions

Record rationale, rule, affected paths, risk, alternatives, approver and review date in
`EXCEPTIONS.md` **before** exceeding a hard gate. An exception is not permission for
unbounded growth; every exception carries a revisit condition.
