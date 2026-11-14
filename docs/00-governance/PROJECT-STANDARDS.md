# PhairDLNA — binding project standards (v0.1 draft)

## Principles
Best-in-class quality, privacy-first, no ads, no telemetry, no cloud dependency, AGPL-3.0-or-later. Rust-first and understandable to a programming beginner. Requirements before implementation; measured evidence before claims.

## Code size and readability gates
- Rust source file target <=300 physical lines, hard review gate >500 lines. Generated files and fixtures excluded with explicit classification.
- Function/method target <=40 lines; >60 requires refactoring or a documented, reviewed exception.
- Cyclomatic complexity target <=10; nesting depth <=3. Tooling must define and enforce measurement before claiming these gates automated.
- Meaningful English names; no unexplained abbreviations, cryptic one-letter variables or giant generic utility modules.
- Module-level purpose, public API rustdoc, explanations of *why*, invariants, failure modes and security assumptions. Avoid comments that simply restate code.
- Explicit error types/context; no unjustified unwrap/expect, panic or silent error swallowing in production paths.
- Single responsibility, explicit boundaries, dependency injection where useful, bounded concurrency and resource ownership.
- Centralized, typed, validated and versioned configuration; defaults documented.
- rustfmt, Clippy with reviewed lint policy, cargo test and security/dependency checks; no fabricated gate results.
- Tests cover positive, negative, boundary, recovery and relevant security cases. No feature is done without updated docs.
- No feature flags that conceal incomplete production implementations, no stubs claimed as done.

## Exceptions
Record rationale, owner, alternatives and review date in docs/00-governance/EXCEPTIONS.md before exceeding a hard gate. An exception is not permission for unbounded growth.

## Changes and acceptance
Each change updates requirements/feature spec, implementation, tests, WHERE_TO_CHANGE, architecture/security as relevant, and evidence. Stop and escalate material scope/security/license decisions. No release or tag without owner approval. GitHub Actions manual-only.
