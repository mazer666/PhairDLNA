# F-19 — Quality gates and developer experience

Status: Draft

Owner role: Product/Requirements

Version/date: 0.1 / 2026-11-14

Requirements: DOC-001, DOC-002, DOC-003, DOC-004, DOC-005, DOC-006, DOC-007, DOC-008, DOC-009, DOC-010, DOC-011, DOC-012,
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.
SUP-001, SUP-002, SUP-003, SUP-004, SUP-005, SUP-006, SUP-007, SUP-008, SUP-009, PLT-001, PLT-002, PLT-009, PLT-016

## 1. Purpose and user story

Quality in this project is **not** a feeling and not a badge. It is a set of commands that can be run by anybody, that fail when something is wrong,
and whose output is recorded so that a later reader can check it.

Two rules make that possible, and they outrank every convenience:

1. **A gate that cannot fail proves nothing.** Every gate has a **negative test** — a deliberately broken state that turns it red. A gate that has
   only ever been green is not evidence of anything (PROJECT-STANDARDS §1; QUALITY-GATES §1).
2. **A skip is never a pass.** If a check cannot run in the current environment, the result is written as `NOT RUN — <reason>`. An exit code that
   means "skipped" (`77`) is reported as a skip, never as success.

The same rules apply to the *written* product. This feature therefore covers two things that are usually separated: the **quality gates** that keep
the code honest, and the **developer experience** — the documentation, the navigation path and the contribution guidance that let a newcomer find the
right file and change it correctly.

**User story.** As a newcomer, I want to know exactly which file to change for a given requirement, I want the checks that protect the project to be
commands I can run myself, and I want every claim the documentation makes about speed, compatibility or safety to point at the record that proves it —
so that I can trust the project without trusting anybody's word.

## 2. Scope

**In scope** — the gate matrix (QG-01 … QG-27) with its commands, severities and result labels; the negative test that each gate must have; the
code-size, complexity and error-handling standards the gates enforce; the documentation structure, the navigation path, the link check and the archive
rule; the supply-chain controls (licence, advisory, ban, source, lock file, SBOM, checksum, `unsafe`, vendoring); the macOS signing and notarisation
plan; the traceability chain from requirement to evidence; and the portability rules that keep the core buildable on the three supported
platforms — the crate boundary (PLT-001), the build matrix (PLT-002), the native-toolchain rule (PLT-009) and the Linux packaging channel
(PLT-016).

**Explicitly out of scope** — the *content* of the feature specifications (each feature owns its own; DOC-001 states only that they exist and have the
ten mandatory sections); the performance budgets themselves (F-18, `../05-quality/PERFORMANCE-BUDGETS.md`); the interoperability matrix rows and the
device protocol (F-07, `../05-quality/`); the protocol conformance checklist (PC-01 … PC-26); the security mitigations and the threat table (F-16);
the CI *trigger* policy, which is manual-only by owner decision (`../07-project/CI-POLICY.md`); and the release policy and the publishing of artefacts
(`../08-releases/`, which does not exist yet).

## 3. Functional requirements

**DOC-001** — Every feature shall have a specification with the ten mandatory sections and stable requirement IDs. [DESIGN] The ten sections are those
of `FEATURE-TEMPLATE.md`; every specification adds the *Scope* section and the *Open questions* block. A feature without a specification is not a
feature.

**DOC-002** — Every performance, compatibility or security claim in the documentation shall link to an evidence record. [DESIGN] An evidence record is
one of: a named test with its observed output, a gate output with its label, or a row of the interoperability matrix. A number without an evidence
record is a **target**, and must say so.

**DOC-003** — `WHERE_TO_CHANGE.md` shall list every source path a newcomer may need to change and shall stay in sync with the tree. [DESIGN] The file
maps a path to the feature that owns it; a path is added only once it exists, and the file is updated in the same change as the code that creates the
path.

**DOC-004** — Every public module shall have a doc comment stating what it is, why it exists and a usage example. [DESIGN] The example is compiled:
the doctests are a gate (`QG-07`), and `cargo-nextest` does **not** run them, so `cargo test --doc` is a separate step that must not be forgotten.

**DOC-005** — The documentation shall be readable by a programming beginner: acronyms explained, terms defined, examples concrete. [DESIGN] An acronym
is expanded at first use in every document that uses it, a domain term is defined before it is used, and an abstract statement is accompanied by a
concrete example.

**DOC-006** — The documentation shall contain no duplicated normative statement; the canonical location shall be linked from everywhere else. [DESIGN]
If two documents state the same normative fact, one of them is wrong (DOCUMENTATION-POLICY §1.1). The canonical locations are: requirements in
`../01-product/TECHNICAL-REQUIREMENTS.md`, gates in `../05-quality/QUALITY-GATES.md`, the threat model in `../04-security/SECURITY-MODEL.md`, and the
dependency policy in `../04-security/DEPENDENCY-LICENSE-REVIEW.md`.

**DOC-007** — Superseded baselines shall be preserved unchanged in the archive with a manifest and a digest. [DESIGN] `../99-archive/` is immutable; a
snapshot is a new dated directory with a `MANIFEST.md` naming the archived path, the original path and the SHA-256 digest of the original. Git history
is not a substitute for an archived snapshot.

**DOC-008** — Every document shall carry a status, an owner role, a version and a date. [DESIGN] The status ladder is `Draft` → `In Review` →
`Approved` → `Superseded`; the owner is a **role**, never a person's name; the date is an ISO date.

**DOC-009** — The getting-started guide shall contain commands that a reader can execute verbatim. [DESIGN] A command in the guide is one that works
in a fresh clone with the pinned toolchain; a command that needs the implementation says so, and says which milestone provides it.

**DOC-010** — The repository shall contain a single navigation path from README to the relevant feature specification and source file. [DESIGN] The
path is `README.md` → `docs/06-development/GETTING-STARTED.md` → `WHERE_TO_CHANGE.md` → feature specification → source and test paths. Every document
is reachable within two clicks of the README.

**DOC-011** — Broken internal links shall be detected by an automated gate. [DESIGN] The link gate checks **relative** links only, offline; a link
that cannot be verified offline must not be reported as passing (QUALITY-GATES §2).

**DOC-012** — Contribution guidance shall state the standards, the gates and the expectations for evidence. [DESIGN] The guidance names the binding
standards, the gate commands, and what "evidence" means: the exact command, the summary line, the toolchain versions and the revision.

**SUP-001** — The repository shall contain the complete, unmodified AGPL-3.0 licence text and the licence metadata shall match it. [DESIGN] The full
licence text lives in `LICENSE`; the short notice lives in `COPYING`; the workspace `license` field reads `AGPL-3.0-or-later`. The planned check
compares the licence text byte for byte with the canonical SPDX `AGPL-3.0-only` text and compares the package metadata with it, so that the three can
never drift apart.

**SUP-002** — Every direct dependency shall be recorded with its licence and its purpose before it is used. [DESIGN] The record is a row in
`../04-security/DEPENDENCY-LICENSE-REVIEW.md`: name, version, licence, purpose, maintenance status, alternatives considered and a **named reviewer**.
No dependency is adopted before the record exists.

**SUP-003** — The licence check shall fail the build if a dependency uses a licence that is incompatible with AGPL-3.0-or-later. [DESIGN] `cargo deny
check licenses` with an allow-list; anything outside it fails. `unused-allowed-licence = warn` keeps the list honest.

**SUP-004** — The vulnerability check shall run over the dependency graph and its result shall be recorded. [DESIGN] `cargo audit` fails on any
un-waived RustSec advisory; a waiver needs a record with an expiry date. The *result* is recorded, not merely the command.

**SUP-005** — Release artefacts shall be accompanied by a software bill of materials and a checksum. [DESIGN] The SBOM is generated from the
dependency graph (the auditable-build information embedded by `cargo auditable` is the aid the research names); the checksum is published next to the
artefact.

**SUP-006** — Any dependency that requires a native toolchain or ships a licence with distribution obligations shall be approved by the owner before
it is adopted. [DESIGN] A `build.rs`, a proc-macro or a C library counts. Approval is an owner decision recorded in the decision log, not a review
comment.

**SUP-007** — The project shall not vendor third-party source code into the repository without a record of its licence and origin. [DESIGN] The one
vendored artefact today is the DIDL-Lite XSD, which is vendored with rewritten `schemaLocation` references so the check runs offline; its origin and
licence are recorded.

**SUP-008** — The core shall carry no `unsafe` code unless an ADR approves it with a written safety argument. [DESIGN] `unsafe` is forbidden by
default; a use requires an ADR, a written safety argument and a comment at every `unsafe` block explaining the invariant it upholds (PROJECT-STANDARDS
§2.6). The check is `cargo xtask check-unsafe` in the gate matrix; the *command* is spelled out in §4.

**SUP-009** — The macOS distribution shall carry a defined code-signing and notarisation plan before the first stable release, and the plan shall be
recorded here. [DESIGN] The plan is the choice between shipping an ad-hoc-signed universal binary now and Developer ID + hardened runtime +
notarisation later (platform-integration §3). The second option requires a paid Apple Developer Program membership (99 USD/year) and a named
maintainer — a governance decision for a volunteer project, not a technical one. The plan is recorded in this specification (§4) and nowhere else.

## 4. Technical requirements

### 4.1 The gate matrix

The canonical list is `../05-quality/QUALITY-GATES.md` §3; it is reproduced here because this feature owns the *developer experience* around it.
Severity: **blocker** = the milestone cannot be accepted; **warning** = must be recorded, may be accepted with a reason.

| ID | Gate | Command | Severity | Blocks |
| --- | --- | --- | --- | --- |
| QG-01 | Formatting | `cargo fmt --all -- --check` | blocker | every milestone |
| QG-02 | Lints | `cargo clippy --workspace --all-targets -- -D warnings` | blocker | every milestone |
| QG-03 | No `unsafe` | `cargo xtask check-unsafe` | blocker | every milestone |
| QG-04 | Build | `cargo build --workspace --all-targets` | blocker | every milestone |
| QG-05 | Unit tests | `cargo nextest run --workspace` | blocker | every milestone |
| QG-06 | Integration tests | `cargo nextest run --workspace --test '*'` | blocker | M1+ |
| QG-07 | Doctests | `cargo test --workspace --doc` | blocker | every milestone |
| QG-08 | Licences | `cargo deny check licenses` | blocker | M0+ |
| QG-09 | Advisories | `cargo audit` | blocker | M0+ |
| QG-10 | Bans and sources | `cargo deny check bans sources` | blocker | M0+ |
| QG-11 | Link check | `lychee --offline docs/ README.md` | blocker | every milestone |
| QG-12 | Spelling | `typos` | warning | every milestone |
| QG-13 | File size | `cargo xtask size-check` | blocker | M1+ |
| QG-14 | Complexity | `clippy::excessive_nesting` + `clippy::too_many_lines` | blocker | M1+ |
| QG-15 | Binary size | `cargo xtask binary-size` | warning | release |
| QG-16 | Snapshot review | `cargo insta test --check` | blocker | M1+ |
| QG-17 | DIDL-Lite conformance | `cargo xtask validate-didl` | blocker | M1+ |
| QG-18 | Property tests | `cargo nextest run --workspace proptest` | blocker | M1+ |
| QG-19 | Fuzz smoke | `cargo fuzz run <target> -- -runs=N` | blocker | M4 |
| QG-20 | Miri | `cargo +nightly miri test` | warning | M1+ |
| QG-21 | Benchmark baseline | `cargo bench --workspace -- --save-baseline` | blocker | M1+ |
| QG-22 | Soak test | `scripts/soak.sh --duration 7d` | blocker | release |
| QG-23 | Device matrix | `scripts/device-matrix.sh` | blocker | M1+ |
| QG-24 | Public API compatibility | `cargo semver-checks` | blocker | release |
| QG-25 | Outdated dependencies | `cargo outdated` | INFO | — |
| QG-26 | Coverage | `cargo llvm-cov` | INFO | — |
| QG-27 | Mutation testing | `cargo mutants` | INFO | — |

### 4.2 Result labels

Only `PASS` is a pass. The permitted labels are `PASS`, `FAIL`, `NOT RUN — <reason>`, `SKIPPED (exit 77) — <reason>`, `BLOCKED — <reason>`,
`INCONCLUSIVE` and `INFO`. A fuzzing run is reported as `NO CRASH FOUND (not a pass)` or `CRASH FOUND (defect)`; a device test without a device is
`NOT RUN — no device available`; a benchmark without a documented machine is `INCONCLUSIVE`.

### 4.3 The negative test each gate must have

| Gate | Broken state that must turn it red |
| --- | --- |
| QG-01 | a mis-indented line → the check reports a diff |
| QG-02 | a lint violation → clippy fails with `-D warnings` |
| QG-03 | one `unsafe` block → the check fails and names the file |
| QG-08 | a dependency with an incompatible licence → the check fails |
| QG-09 | a dependency with a known advisory → the check fails |
| QG-13 | a 600-line file → the check fails and names the file |
| QG-14 | a function with six nesting levels → clippy fails |
| QG-16 | a changed DIDL-Lite output → the snapshot check fails |
| QG-17 | DIDL-Lite with a missing required attribute → the XSD check fails |
| QG-18 | a broken escaping function → the property test finds a counterexample |
| QG-19 | a malformed input → the fuzz target aborts (this one *must* crash) |
| QG-11 | a broken relative link → the link checker fails |

### 4.4 Two tools that do not exist

They must not be planned around:

- **`cargo-lint` does not exist** as a usable tool — a crates.io stub from 2020 whose repository is gone. It is on the `cargo deny` ban list for the
  same reason.
- **There is no official UPnP device-description XSD.** Every `upnp.org/schemas/device/*` path answers with HTML and HTTP 200. The DIDL-Lite XSD
  *does* exist and is vendored; the device description is validated against hand-written fixtures instead.

### 4.5 The standards the gates enforce

| Rule | Target | Hard limit | Measured by |
| --- | --- | --- | --- |
| Rust source file length | ≤ 300 physical lines | > 500 needs a recorded exception | `scripts/check-file-size.sh` |
| Function / method length | ≤ 40 lines | > 60 needs refactoring or an exception | size script + reviewer |
| Cyclomatic complexity | ≤ 10 | > 15 needs refactoring | `clippy::cognitive_complexity` (pedantic) |
| Nesting depth | ≤ 3 | > 4 needs refactoring | reviewer + clippy lints |
| Rust files per module | ≤ 20 | — | reviewer |
| `unwrap` / `expect` / `panic!` on a network, configuration or filesystem path | none without justification | — | clippy + reviewer |
| `unsafe` | none in the core | only with an ADR and a safety argument | QG-03 |
| Configuration | one central typed, validated, versioned module | no scattered `std::env::var` | reviewer |

Generated files, fixtures and test-data blobs are excluded from the line gates **only** when they are explicitly classified as generated in the file
header and listed in the gate matrix. Silent exclusions are not allowed.

### 4.6 Toolchain, as recorded in the dependency review

Versions are the ones the review recorded on 2026-11-14; they are the versions the gates are expected to use.

| Tool | Version | Role |
| --- | --- | --- |
| `cargo-nextest` | 0.9.x | test runner — does **not** run doctests |
| `cargo test --doc` | — | doctests, a separate step |
| `assert_cmd` / `predicates` | 2.2.2 / 3.1.x | CLI integration tests |
| `tempfile` | 3.27.0 | temporary directories |
| `wiremock` | 0.6.5 | HTTP mocking |
| `insta` | 1.49.0 | snapshot tests; CI runs with `INSTA_UPDATE=no` |
| `proptest` | 1.12.0 | property tests |
| `cargo-fuzz` | 0.13.x | fuzzing, pinned nightly, six targets with seed corpora |
| `criterion` | 0.8.2 | micro-benchmarks |
| `cargo-deny` | 0.20.2 | licences, advisories, bans, sources |
| `cargo-audit` | 0.23.0 | RustSec advisories |
| `cargo-semver-checks` | 0.51.x | public API compatibility, release gate |
| `typos` | 0.10.x | spelling |
| `lychee` | 0.24.x | link checker — must check content type, because `upnp.org` answers soft-404s with HTTP 200 and HTML |
| `xmlschema` | 0.0.14 | XSD validation of DIDL-Lite — low adoption (≈4 800 downloads), dev-dependency only, risk flagged and accepted |

The workspace pins its minimum supported Rust version as `rust-version = "1.89"` in the workspace `Cargo.toml`; the channel used by contributors is
`stable`, pinned in `rust-toolchain.toml`.

### 4.7 The traceability chain

```
Requirement → feature specification → module / source path → milestone
            → verification method → evidence record
```

The chain is materialised in `../07-project/TRACEABILITY-MATRIX.md`, which is **generated** by `scripts/generate_requirement_views.py` from the
requirement definitions, so that the IDs, the priorities and the definitions cannot drift apart by hand-copying. A requirement without a feature, a
milestone and a verification method is not a requirement yet; it stays `Draft`.

### 4.8 The macOS signing and notarisation plan (SUP-009)

The plan is recorded here, as the requirement says, and is the only place it is stated.

1. **Now: an ad-hoc-signed universal binary.** One binary for `arm64` and `x86_64`, ad-hoc signed at build time (`codesign -s -`), because on Apple
   silicon an arm64 binary must carry a signature to execute at all. Installed with a `curl | sh` script or a Homebrew formula. Cost: nothing.
   Friction: a browser download carries the quarantine attribute and the user must use *Open Anyway*.
2. **Later: Developer ID + hardened runtime + notarisation.** Requires an Apple Developer Program membership (99 USD/year) and a named maintainer.
   Hardened Runtime is mandatory for notarisation and forbids JIT and `DYLD_*` injection — acceptable for a Rust program, which does not JIT. The
   toolchain is `xcrun notarytool submit --keychain-profile … --wait` followed by `xcrun stapler staple`; `altool` and Xcode ≤ 13 are no longer
   accepted.
3. **Constraint on both options.** The same binary must be signable without a code change: no JIT, no downloaded dylibs, no `DYLD_*` reliance. This is
   cheap now and expensive later.
4. **Open.** Whether the project buys the membership, and who holds the account, is an owner decision (§12 item 3). Until it is made, option 1 ships.

### 4.9 Portability: the crate boundary, the build matrix and the packaging channel

A gate is only worth something if it runs on the machine the project claims to support. The
rules below are the ones that keep that claim honest.

**PLT-001** — The core crate shall not depend on any OS-specific crate and shall compile on macOS, Linux and Windows from the same source.
[DESIGN] The core depends on portable crates only; every operating-system fact reaches it through a documented trait with a default
implementation (PLT-003, F-08). The ban list of the supply-chain gate (QG-10) names the crates that must never enter the core, and a crate
that wraps an operating-system API belongs to an adapter, not to the core. There is no per-platform fork of the core: the same source builds
everywhere, and a behaviour that differs per platform is a different code path behind a trait, not a different file.

**PLT-002** — The project shall provide a build matrix covering macOS (arm64, x86_64), Linux (x86_64) and Windows (x86_64). [DESIGN] The
matrix is the set of targets the build gate runs on: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu` and
`x86_64-pc-windows-msvc`. Every one of them builds the workspace and runs the tests; a target that was not run is reported as
`NOT RUN — <reason>` and never as a pass (§4.2). The matrix is recorded in the CI policy and in the build documentation; a workflow that was
never triggered is not a green workflow (`../07-project/CI-POLICY.md` §5).

**PLT-009** — The build shall not require a C toolchain beyond what the platform already provides, and any native dependency shall be
documented. [DESIGN] A dependency that needs a C compiler, a `build.rs` that compiles C, or a system library beyond the platform's own is
approved by the owner before it is adopted (SUP-006) and recorded in the dependency review with its purpose, its licence and its build
requirement. The build matrix therefore runs on a machine that has the platform toolchain and the pinned Rust toolchain, and nothing else.

**PLT-016** — The Linux distribution shall provide native packages and shall not use a filesystem-sandboxed package format as its primary
distribution channel, because such a sandbox blocks access to arbitrary media folders. [DESIGN] The primary channel is the native package of
the distribution — `deb` and `rpm` — with the `systemd` unit, the `polkit` policy and the `udev` rules installed alongside it
(platform-integration §8.5). A filesystem-sandboxed format is not the primary channel, because its access to the filesystem is a permission
list: the default sandbox exposes the application's own directories and the XDG directories, and reaching `/media` or a network share needs
an explicit and brittle set of grants that a server for arbitrary user folders cannot ask for (platform-integration §8.5).

| Target | Platform | Built and tested by | Status |
| --- | --- | --- | --- |
| `aarch64-apple-darwin` | macOS, Apple Silicon | build matrix gate | planned |
| `x86_64-apple-darwin` | macOS, Intel | build matrix gate | planned |
| `x86_64-unknown-linux-gnu` | Linux | build matrix gate | planned |
| `x86_64-pc-windows-msvc` | Windows | build matrix gate | planned |

## 5. Performance requirements

The gates are not a product feature with a latency budget, but three performance rules apply to them:

- [DESIGN] Target: the fast gate set (format, lint, build, unit tests) completes within a working session on the reference hardware. Method:
  `hyperfine` over ten runs.
- [DESIGN] Target: the documentation link check and the requirement-view generator complete in seconds, so that they are cheap enough to run before
  every commit. Method: `hyperfine` over ten runs.
- [DESIGN] Target: the benchmark baseline is comparable across runs, so that a change is visible above the run-to-run variance. Method: `criterion`
  with the recorded baseline, median and p95 (PERF-011).

No figure here is a measured value.

## 6. Usability and accessibility

- A gate that a newcomer cannot run is a gate that will not be run. Every gate states its exact command, and the commands are the ones in the
  getting-started guide (DOC-009). The result labels are plain words with one meaning each; there is no "green except for the flaky one". The
  documentation is readable by a programming beginner (DOC-005): acronyms expanded at first use, terms defined, examples concrete. The contribution
  guidance states what a contribution needs — a requirement, a specification, tests including a
- **negative** test, evidence, documentation, and an explicit statement of whether the change touches a trust boundary. The navigation path from the
  README to the file to change must stay intact and must work for a reader who has never seen the project (DOC-010).
- [DOC-005] *Gate*, *blocker*, *lint*, *doctest*, *fixture*, *snapshot*, *property test*, *fuzzing*, *SBOM* and *notarisation* are explained at first
  use.

## 7. Security and privacy

- The gates are a **supply-chain control**: a licence check, an advisory check, a ban list, a source allow-list and a committed lock file reviewed
  like source (SUP-003, SUP-004, SUP-007, M-08). The ban list is real and includes `ffmpeg-next`, `ffmpeg-sys`, `cocoa`, `dispatch`, `rumperd`,
  `libupnp-sys` and `cargo-lint`, each with a recorded reason. The advisory check fails on any un-waived RustSec advisory, and a waiver needs an
  expiry date, so a waiver cannot become permanent by accident. `Cargo.lock` is committed and diff-reviewed like source; the diff is part of the
  review, and the dependency with a `build.rs` or a proc-macro is treated as code that has been read. The `unsafe` check keeps the core free of unsafe
  code unless an ADR says otherwise with a written safety argument (SUP-008). Miri is a warning-level gate that is only meaningful where `unsafe`
  exists. The gates themselves must not leak: a failing gate prints the command, the exit code and the relevant output, and nothing else. A gate that
  prints a home directory path or a token is a bug in the gate. The toolchain adds no telemetry: the gates run locally and report to the console.

## 8. Interoperability

- The gates do not change what the server does on the wire; they change whether it is allowed to be released. Interoperability evidence lives in the
  interoperability matrix (F-07), and the device matrix gate (`QG-23`) is what turns that table from a plan into evidence. A conformance fixture that
  was captured from a real client becomes a regression test; the SSDP corpus has to be captured by the project itself, because the only public capture
  contains no `M-SEARCH` requests at all. The protocol conformance checklist (`PC-01` … `PC-26`) maps one test per obligation to a requirement, and
  every one of those requirements has a row in the traceability matrix. The device test protocol is manual and structured: device model, firmware,
  client application and version, media format, four operations, outcome per operation, and the server log for the session.

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| A gate cannot run (tool missing, no network, no device) | `NOT RUN — <reason>` or `SKIPPED (exit 77) — <reason>`; never a pass | QUALITY-GATES §1 |
| A gate fails | `FAIL` with the output recorded; the milestone is blocked | QUALITY-GATES §1 |
| A benchmark is too noisy to decide | `INCONCLUSIVE`; never a pass | QUALITY-GATES §1 |
| A fuzzing run finds nothing | `NO CRASH FOUND (not a pass) — N minutes, N executions` | QUALITY-GATES §1 |
| A hard size or complexity limit must be exceeded | an entry in `../00-governance/EXCEPTIONS.md` with rationale, risk, approver and review date, recorded **before** the breach | PROJECT-STANDARDS §8 |
| A link check cannot reach a target | the link is reported as broken; an unverifiable link is not reported as passing | DOC-011 |
| The requirement index and the definitions drift apart | the generator's `--check` mode fails; the generated files are never edited by hand | DOC-006 |
| A dependency is added without a review record | the licence check and the review rule block it; the dependency is not adopted | SUP-002, SUP-003 |

## 10. Acceptance criteria and traceability

All criteria below are *planned*; no gate output has been recorded yet.

**AC-01** — *Given* the feature catalog, *when* every feature is checked, *then* each has a specification file containing the ten mandatory sections,
the Scope section and stable requirement IDs that appear in the requirement catalogue. Requirements: DOC-001 · Verification: documentation gate ·
Planned location: `scripts/check-feature-specs.py`

**AC-02** — *Given* the documentation tree, *when* a search for a normative statement finds the same rule in two documents, *then* one of them links
to the canonical location instead of restating it. Requirements: DOC-006 · Verification: reviewer check · Planned location:
`docs/00-governance/DOCUMENTATION-POLICY.md`

**AC-03** — *Given* the navigation path, *when* a newcomer starts at `README.md` and follows `docs/06-development/GETTING-STARTED.md` →
`WHERE_TO_CHANGE.md` → a feature specification → a source path, *then* every step resolves to a file that exists. Requirements: DOC-010, DOC-003 ·
Verification: documentation gate · Planned location: `scripts/check-doc-links.sh`

**AC-04** — *Given* `WHERE_TO_CHANGE.md`, *when* it is compared with the tracked source paths, *then* every path it lists exists and every module that
exists is listed with the feature that owns it. Requirements: DOC-003 · Verification: documentation gate · Planned location:
`scripts/check-feature-specs.py`

**AC-05** — *Given* a relative link to a file that does not exist, *when* the link gate runs, *then* it fails and names the broken link; and *given* a
document whose header is missing a status, an owner role, a version or a date, *then* the header check fails. Requirements: DOC-011, DOC-008 ·
Verification: documentation gate · Planned location: `scripts/check-feature-specs.py`

**AC-06** — *Given* the getting-started guide, *when* each command in it is executed in a fresh clone with the pinned toolchain, *then* each runs
verbatim and prints what the guide says it prints. Requirements: DOC-009 · Verification: manual protocol · Planned location:
`docs/06-development/GETTING-STARTED.md`

**AC-07** — *Given* a public module, *when* its documentation is read, *then* it states what the module is, why it exists and carries a usage example,
and *given* the doctest step, *then* the example is compiled and run. Requirements: DOC-004 · Verification: documentation gate · Planned location:
`crates/phairdlna-core/src/lib.rs` and `cargo test --workspace --doc`

**AC-08** — *Given* a document in `docs/99-archive/`, *when* the archive is inspected, *then* the snapshot directory carries a `MANIFEST.md` naming
the archived path, the original path and the SHA-256 digest of the original, and the archived files are byte-identical to what was archived.
Requirements: DOC-007 · Verification: documentation gate · Planned location: `scripts/check-feature-specs.py`

**AC-09** — *Given* the documentation, *when* a performance, compatibility or security claim is read, *then* it either names an evidence record — a
named test with output, a gate output, or a matrix row — or it is explicitly marked as a *target*. Requirements: DOC-002 · Verification: reviewer
check · Planned location: `docs/00-governance/PROJECT-STANDARDS.md`

**AC-10** — *Given* the contribution guidance, *when* a new contributor reads it, *then* it states the binding standards, the gate commands and what
evidence means, including that a requirement, a specification change and a negative test are needed for a behaviour change. Requirements: DOC-012 ·
Verification: reviewer check · Planned location: `CONTRIBUTING.md`

**AC-11** — *Given* the licence files and the package metadata, *when* the licence check runs, *then* `LICENSE` matches the canonical AGPL-3.0 text,
`COPYING` carries the short notice and the workspace `license` field reads `AGPL-3.0-or-later`. Requirements: SUP-001 · Verification: licence and
advisory gate · Planned location: `scripts/quality-gates.sh`

**AC-12** — *Given* a dependency with a licence outside the allow-list, *when* the licence check runs, *then* it fails; and *given* a dependency with
a known RustSec advisory, *when* the advisory check runs, *then* it fails. Requirements: SUP-003, SUP-004 · Verification: licence and advisory gate ·
Planned location: `scripts/quality-gates.sh`

**AC-13** — *Given* every direct dependency, *when* the dependency review is inspected, *then* each has a row with name, version, licence, purpose,
maintenance status, alternatives and a named reviewer, and each row matches the `Cargo.lock` diff. Requirements: SUP-002 · Verification: licence and
advisory gate · Planned location: `docs/04-security/DEPENDENCY-LICENSE-REVIEW.md`

**AC-14** — *Given* a release artefact, *when* it is published, *then* a software bill of materials and a checksum accompany it. Requirements: SUP-005
· Verification: licence and advisory gate · Planned location: `scripts/quality-gates.sh`

**AC-15** — *Given* one `unsafe` block added to the core, *when* the unsafe check runs, *then* it fails and names the file; and *given* an ADR that
approves a specific `unsafe` block with a safety argument, *then* the check accepts exactly that block. Requirements: SUP-008 · Verification: licence
and advisory gate · Planned location: `scripts/check-unsafe.sh`

**AC-16** — *Given* the release artefact on macOS, *when* `spctl --assess --type execute` and `codesign --verify --deep --strict` are run, *then* both
pass, and the signing and notarisation plan of §4.8 is recorded in this specification. Requirements: SUP-009 · Verification: licence and advisory gate
· Planned location: `scripts/quality-gates.sh`

**AC-17** — *Given* the gate matrix, *when* each gate's negative test is applied, *then* the gate turns red and the broken state is named.
Requirements: DOC-001, DOC-011 · Verification: documentation gate · Planned location: `scripts/quality-gates.sh`

**AC-18** — *Given* the requirement definitions, *when* the generator runs in `--check` mode, *then* the generated index and traceability matrix match
the definitions and no hand-edited drift exists. Requirements: DOC-006, DOC-010 · Verification: documentation gate · Planned location:
`scripts/generate_requirement_views.py`

**AC-19** — *Given* a dependency that needs a C toolchain or ships distribution obligations, *when* it is proposed, *then* it is not adopted before an
owner decision is recorded in the decision log; and *given* vendored third-party source, *then* a record of its licence and origin exists.
Requirements: SUP-006, SUP-007 · Verification: reviewer check · Planned location: `docs/00-governance/DECISION-LOG.md`

**AC-20** — *Given* the dependency graph of the core crate, *when* the ban and licence checks run, *then* no OS-specific crate is a dependency
of the core, and the same source builds on macOS, Linux and Windows with no per-platform fork of the core. Requirements: PLT-001, PLT-003 ·
Verification: automated dependency check plus the build matrix · Planned location: `crates/phairdlna-core/Cargo.toml` and
`.github/workflows/ci-matrix.yml`

**AC-21** — *Given* the build matrix, *when* it is run, *then* it builds and tests macOS on arm64, macOS on x86_64, Linux on x86_64 and
Windows on x86_64, and a target that was not run is reported as `NOT RUN — <reason>` rather than as a pass. Requirements: PLT-002 ·
Verification: build matrix run · Planned location: `.github/workflows/ci-matrix.yml` and `docs/07-project/CI-POLICY.md`

**AC-22** — *Given* a machine with only the platform's own toolchain and the pinned Rust toolchain, *when* the workspace is built, *then* no
C toolchain beyond the platform's own is required, and every dependency that needs a native toolchain is recorded in the dependency review
with its purpose and carries an owner approval. Requirements: PLT-009, SUP-006 · Verification: reviewer check plus the build matrix run ·
Planned location: `docs/04-security/DEPENDENCY-LICENSE-REVIEW.md` and `crates/phairdlna-core/Cargo.toml`

**AC-23** — *Given* the Linux distribution, *when* its packaging is inspected, *then* native packages are provided as the primary channel,
no filesystem-sandboxed package format is offered as the primary channel, and the packaging documentation states the reason. Requirements:
PLT-016 · Verification: reviewer check · Planned location: `docs/06-development/BUILD-AND-TEST.md`

## 11. Test strategy and dependencies

**Planned test locations** (the first three exist in the repository today; the rest are planned):

| Purpose | Path |
| --- | --- |
| Relative-link check (QG-11) | `scripts/check-doc-links.sh` |
| File and function size (QG-13) | `scripts/check-file-size.sh` |
| Requirement index and traceability generator | `scripts/generate_requirement_views.py` |
| Feature-specification structure, headers, archive manifest | `scripts/check-feature-specs.py` (planned) |
| `unsafe` scan (QG-03) | `scripts/check-unsafe.sh` (planned) |
| Aggregate gate run | `scripts/quality-gates.sh` (planned) |
| Licence, advisory, ban, source and artefact checks | `scripts/quality-gates.sh` (planned) |
| Spelling (QG-12) | `scripts/quality-gates.sh` (planned) |
| Gate documentation and negative-test records | `docs/05-quality/QUALITY-GATES.md` |
| Core crate boundary and the OS-specific dependency ban | `crates/phairdlna-core/Cargo.toml`, `.github/workflows/ci-matrix.yml` |
| Build matrix over the four required targets | `.github/workflows/ci-matrix.yml` |
| Native-toolchain review | `docs/04-security/DEPENDENCY-LICENSE-REVIEW.md` |
| Linux packaging channel | `docs/06-development/BUILD-AND-TEST.md` |

**Method.** A gate is a script with three exit codes — `0` pass, `1` fail, `77` skipped — that prints the command, the result and the relevant output.
The *negative* test of a gate is part of the gate's own definition and must be demonstrated before the gate can be trusted; the demonstration is
recorded next to the positive run. A gate that has never been seen red is not a gate.

**Dependencies.** `../05-quality/QUALITY-GATES.md` (the canonical matrix and the label vocabulary), `../05-quality/TEST-STRATEGY.md` (the test
layers), `../00-governance/` (the standards, the documentation policy, the requirements method, the exceptions file and the decision log),
`../04-security/DEPENDENCY-LICENSE-REVIEW.md` (the dependency records and the FFmpeg exclusion), `../01-product/REQUIREMENTS-INDEX.md` and
`../07-project/TRACEABILITY-MATRIX.md` (the generated views), and F-16 (the security mitigations the gates protect).

## 12. Open questions and decisions

1. **[OPEN] The `cargo xtask` crate does not exist.** The gate matrix names `cargo xtask check-unsafe`, `cargo xtask size-check`, `cargo xtask
   binary-size` and `cargo xtask validate-didl`, but the workspace contains only `phairdlna-core` and `phairdlna-cli`, and the real gates that exist
   today are shell and Python scripts in `scripts/`. *Proposed reading, used here:* the matrix names the *intent* of each gate; the command is the one
   the repository actually runs, and the matrix is updated when the `xtask` crate is created. *Closes when:* the owner decides whether an `xtask`
   crate is created or the matrix is rewritten to name the scripts.
2. **[OPEN] Gate-ID drift between the governance documents.** `PROJECT-STANDARDS.md` §2 cites `scripts/check-file-size.sh` as "QG-08" and the
   complexity check as "QG-09", while `QUALITY-GATES.md` has file size as **QG-13** and complexity as **QG-14**, with QG-08 = licences and QG-09 =
   advisories. The script's own header comment says QG-13. `ARCHITECTURE.md` §2 calls the crate-boundary check "QG-07", while QG-07 is the doctest
   gate and the boundary check has no gate ID at all. *Proposed decision, not in the baseline:* the gate matrix is canonical; the two governance
   documents are corrected. *Closes when:* the owner confirms and the corrections are made.
3. **[OPEN] The macOS signing decision.** Option A (ad-hoc-signed universal binary, no cost, some Gatekeeper friction) versus Option B (Developer ID +
   hardened runtime + notarisation, 99 USD/year and a named maintainer) is an owner decision with a governance dimension: the account is a person's
   legal identity in an AGPL volunteer project (platform-integration §3.2, §3.3). *Closes when:* the owner decides before the first stable release.
4. **[OPEN] Where the notarisation plan is recorded.** SUP-009 says the plan "shall be recorded here", which this specification reads as §4.8. The
   canonical structure reserves `docs/08-releases/` for distribution and provenance, but that directory does not exist yet. *Proposed decision, not in
   the baseline:* the plan lives in this specification and the release documents link to it, rather than being duplicated (DOC-006). *Closes when:*
   the owner confirms.
5. **[OPEN] The minimum supported Rust version versus the macOS menu bar crates.** The workspace pins `rust-version = "1.89"`, but `muda` and
   `tray-icon` — the evaluated menu bar crates — declare an MSRV of **1.90**, while `objc2-app-kit` needs 1.71 and the stable `notify` 8.x line needs
   1.88. *Proposed decision, not in the baseline:* either raise the MSRV to 1.90 or call `objc2-app-kit` directly for the status item and avoid
   `muda`. *Closes when:* the owner fixes the MSRV.
6. **[OPEN] The link gate's command.** The matrix names `lychee --offline docs/ README.md`, but the repository already contains
   `scripts/check-doc-links.sh`, which checks relative links with Python and needs no external tool. The matrix's note that `upnp.org` answers
   soft-404s with HTTP 200 is the reason the check must look at content type, not only at the status code. *Proposed reading, used here:* both exist;
   the script is the offline gate and `lychee` is the optional external cross-check. *Closes when:* the owner picks one.
7. **[OPEN] Whether the requirement index and the traceability matrix may be generated files.** They are generated today by
   `scripts/generate_requirement_views.py`, and the file says "Do not edit by hand". `DOCUMENTATION-POLICY.md` §2 does not mention generated
   documents, and its rule that "no duplicate normative statement" may be read as forbidding a generated copy of the requirement table. *Proposed
   reading, used here:* a generated view is not a duplicate normative statement, because the definition lives only in the requirement catalogue and
   the generator refuses to drift from it. *Closes when:* the policy states the rule for generated documents.
8. **[OPEN] The build matrix does not yet name the Intel macOS target.** PLT-002 requires macOS on arm64 **and** macOS on x86_64, but
   `ci-matrix.yml` names three operating systems — `macos-latest`, `ubuntu-latest`, `windows-latest` — and no Intel macOS target, so half of
   the required matrix is not covered by the workflow as it stands. *Proposed reading:* the matrix gains an explicit `x86_64-apple-darwin`
   entry, and an entry the runner set cannot provide is reported as `NOT RUN — <reason>` rather than being quietly left out. *Closes when:* the
   owner decides how the Intel target is run.
9. **[OPEN] The CI workflow file names.** `CI-POLICY.md` §2 names `ci.yml` and `ci-matrix.yml` as "skeletons with the commands listed but the
   implementation not yet present", and names `scripts/check-requirement-views.sh` — a file that does not exist; the script in the repository is
   `scripts/generate_requirement_views.py`. *Closes when:* the policy is corrected to match the tree.