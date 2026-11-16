# Quality gates — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: QA / Performance Engineer
Requirements: `DOC-001` … `DOC-012`, `SUP-001` … `SUP-008`, `PERF-010`, `PERF-011`
Test method: `TEST-STRATEGY.md` · Interop evidence: `INTEROPERABILITY-MATRIX.md`

---

## 1. The rules of evidence

These rules exist because a gate that cannot fail, or a result that is reported
looser than it was measured, destroys trust in every other number in the project.

1. **A gate that never failed has proved nothing.** Every gate in §3 must have a
   **negative test**: a deliberately broken state that turns the gate red. The
   negative test is recorded with its output next to the positive one.
2. **Only `PASS` is a pass.** The permitted result labels are:

   | Label | Meaning |
   | --- | --- |
   | `PASS` | The gate ran and succeeded. Output recorded. |
   | `FAIL` | The gate ran and failed. Output recorded. Blocks the milestone. |
   | `NOT RUN — <reason>` | The gate could not run here. **Never** a pass. |
   | `SKIPPED (exit 77) — <reason>` | The gate deliberately declined. **Never** a pass. |
   | `BLOCKED — <reason>` | Waiting on something external. **Never** a pass. |
   | `INCONCLUSIVE` | The measurement was too noisy to decide. **Never** a pass. |
   | `INFO` | Informational, does not gate. |

3. **A fuzzing run is not a pass.** It is reported as
   `NO CRASH FOUND (not a pass) — N minutes, N executions` or
   `CRASH FOUND (defect) — <artefact>`.
4. **A device test without a device is not a pass.** It is `NOT RUN — no device
   available`.
5. **A benchmark without a documented machine is not a measurement.** It is
   `INCONCLUSIVE`.

---

## 2. Toolchain

| Tool | Version | Purpose | Note |
| --- | --- | --- | --- |
| `cargo-nextest` | 0.9.x | Test runner for CI (process per test, parallel across binaries) | Does **not** run doctests — `cargo test --doc` is a separate step |
| `cargo test --doc` | — | Doctests | Must not be forgotten |
| `assert_cmd` + `predicates` | 2.x / 3.x | CLI integration tests | |
| `tempfile` | 3.x | Temporary directories for library tests | |
| `wiremock` | 0.6.x | HTTP mocking | `mockito` / `httpmock` are alternatives |
| `insta` | 1.4x | Snapshot tests for DIDL-Lite output | CI runs with `INSTA_UPDATE=no`; `cargo insta review` for humans |
| `proptest` | 1.1x | Property tests | |
| `cargo-fuzz` | 0.13.x | Fuzzing | Requires a **pinned nightly toolchain**; 6 targets with committed seed corpora |
| `criterion` | 0.8.x | Micro-benchmarks | |
| `hyperfine` | — | End-to-end CLI timing | |
| `cargo-deny` | 0.20.x | Licence, advisory, ban and source checks | `AGPL-3.0-only` + `MPL-2.0` allowed; `unused-allowed-license = warn` |
| `cargo-audit` | 0.23.x | RustSec advisories | |
| `cargo-semver-checks` | 0.51.x | Public API compatibility | Release gate only |
| `typos` | 0.10.x | Spelling in source and docs | |
| `lychee` | 0.24.x | Link checker | Must check content type: `upnp.org` answers soft-404s with HTTP 200 + HTML |
| `xmlschema` | 0.0.14 | XSD validation of DIDL-Lite | The only pure-Rust option; dev-dependency only; risk accepted and flagged |

Two things that **do not exist** and must not be assumed:

- **`cargo-lint` does not exist** as a usable tool (crates.io stub from 2020, repository
  gone). Do not plan around it.
- **There is no official UPnP device-description XSD.** Every `upnp.org/schemas/device/*`
  path answers with HTML and HTTP 200. The DIDL-Lite XSD *does* exist
  (`upnp.org/schemas/av/didl-lite.xsd`); the device description must be validated
  against hand-written fixtures instead.

---

## 3. Gate matrix

Severity: **blocker** = the milestone cannot be accepted; **warning** = must be
recorded, may be accepted with a reason.

| ID | Gate | Tool / command | Severity | Blocks | State | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| QG-01 | Formatting | `cargo fmt --all -- --check` | blocker | every milestone | **exists** |  |
| QG-02 | Lints | `cargo clippy --workspace --all-targets -- -D warnings` | blocker | every milestone | **exists** | No `#[allow]` without a justifying comment |
| QG-03 | No `unsafe` | `scripts/check-no-unsafe.sh` | blocker | every milestone | **exists** | `SUP-008`; any `unsafe` needs an ADR. **Implemented and running** |
| QG-04 | Build | `cargo build --workspace --all-targets` | blocker | every milestone | **exists** | Plus the release profile and the macOS universal binary |
| QG-05 | Unit tests | `cargo test --workspace` | blocker | every milestone | **exists** | `cargo-nextest` is the planned switch (it is not installed in the reference environment); `cargo test --workspace` is what the CI runs |
| QG-06 | Integration tests | `cargo test --workspace --test '*'` | blocker | M1+ | **exists** | **Implemented and running** — 26 integration test targets, all green. `cargo-nextest` is the planned switch |
| QG-07 | Doctests | `cargo test --workspace --doc` | blocker | every milestone | **exists** | nextest does not cover these |
| QG-08 | Licences | `cargo deny check licenses` | blocker | M0+ | **exists** | `AGPL-3.0-only` + `MPL-2.0`; `unused-allowed-license = warn` Requires `cargo-deny`. |
| QG-09 | Advisories | `cargo audit` | blocker | M0+ | **exists** | Fail on any un-waived advisory Requires `cargo-audit`. |
| QG-10 | Bans and sources | `cargo deny check bans sources` | blocker | M0+ | **exists** | Banned crates list in `deny.toml` Requires `cargo-deny`. |
| QG-11 | Link check | `scripts/check-doc-links.sh` | blocker | every milestone | **exists** | Resolves every relative link in the tracked markdown. **Implemented and running**; `lychee` is the planned replacement for external links |
| QG-12 | Spelling | `typos` | warning | every milestone | planned |  |
| QG-13 | File size | `scripts/check-file-size.sh` | blocker | M1+ | **exists** | ≤ 300 lines target, > 500 hard limit; no maintained Rust tool exists, so the project writes its own. **Implemented and running** |
| QG-14 | Complexity | `clippy::excessive_nesting` + `clippy::too_many_lines` (blocking) | blocker | M1+ | **exists** | The thresholds are in `clippy.toml`: 60 lines per function, 3 levels of nesting — the hard limits of `PROJECT-STANDARDS.md`. The 40-line target stays a review rule. `rust-code-analysis-cli` is stale (2023) and MPL-2.0 → advisory only, never gating |
| QG-15 | Binary size | *planned* — an `xtask` command | warning | release | planned | Budget per binary, documented |
| QG-16 | Snapshot review | `cargo insta test --check` | blocker | M1+ | planned | `INSTA_UPDATE=no`, `INSTA_FORCE_PASS` unset |
| QG-17 | DIDL-Lite conformance | *planned* — an `xtask` command validating against the vendored `didl-lite.xsd` via the `xmlschema` crate | blocker | M1+ | planned | Arrives with the DIDL-Lite writer |
| QG-18 | Property tests | `cargo nextest run --workspace proptest` | blocker | M1+ | planned | Escaping round-trip; path containment |
| QG-19 | Fuzz smoke | `cargo fuzz run <target> -- -runs=N` | blocker | M4 | planned | The six targets and their seed corpora arrive together; neither exists yet |
| QG-20 | Miri | `cargo +nightly miri test` | warning | M1+ | planned | Only meaningful where `unsafe` exists; mostly a "no unsafe" proof |
| QG-21 | Benchmark baseline | `cargo bench --workspace -- --save-baseline` | blocker | M1+ | planned | Compared against the recorded baseline |
| QG-22 | Soak test | `scripts/soak.sh --duration 7d` | blocker | release | planned | RSS, fd count, thread count, index growth, dropped connections |
| QG-23 | Device matrix | `scripts/device-matrix.sh` | blocker | M1+ | planned | Rows in `INTEROPERABILITY-MATRIX.md` with device, firmware, format, outcome |
| QG-24 | Public API compatibility | `cargo semver-checks` | blocker | release | planned |  |
| QG-25 | Outdated dependencies | `cargo outdated` | INFO | — | planned | Informational only |
| QG-26 | Coverage | `cargo llvm-cov` | INFO | — | planned | Informational; never a quality claim on its own |
| QG-27 | Mutation testing | `cargo mutants` | INFO | — | planned | Informational, optional |
| QG-28 | Crate boundaries | `scripts/check-crate-boundaries.sh` | blocker | M0+ | **exists** | Fails when a crate depends "downwards" (ADR-0001, amended for one edge by ADR-0012). **Implemented and running** — 4 sibling dependencies checked across 4 crate manifests |

### 3.0 The canonical register

**This table is the single register of the gate identifiers.** Other documents refer
to a gate as `QG-nn` and mean the row here. A document that names a gate for a
different check is a defect — fix the document, not this table.

The **State** column says whether the gate can run today. `exists` means the command
is in the repository and works; `planned` means it arrives with the code it guards.
A gate in the `planned` state is reported as `NOT RUN — <reason>`, never as a pass.

**Thirteen gates can run today**: QG-01 … QG-11, QG-13, QG-14 and QG-28. Five of them have
a script of their own and are observed running: **QG-03** (no `unsafe`), **QG-06**
(integration tests), **QG-11** (link check), **QG-13** (file size) and **QG-28** (crate
boundaries). The formatting, lint, build, complexity and test gates come with the Rust
toolchain and are marked `exists` for the same reason. The generator script
`scripts/generate_requirement_views.py` is **not** a quality gate: it is a
documentation generator that the CI runs as a check step.

### 3.1 What each gate must prove

A gate that has never been seen red is not a gate. For every gate the project records
a **negative test**:

| Gate | Negative test that must turn it red |
| --- | --- |
| QG-01 | Introduce a mis-indented line → the check reports a diff |
| QG-02 | Introduce a lint violation → clippy fails with `-D warnings` |
| QG-03 | Add one `unsafe` block → the check fails and names the file — **demonstrated on 2026-11-14** |
| QG-06 | Break an integration test → the integration target fails and the command exits non-zero — **demonstrated on 2026-11-16** |
| QG-08 | Add a dependency with an incompatible licence → the check fails |
| QG-09 | Add a dependency with a known advisory → the check fails |
| QG-13 | Add a 600-line file → the check fails and names the file — **demonstrated on 2026-11-14** |
| QG-14 | Add a function with 70 lines or four levels of nesting → clippy fails on `too_many_lines` or `excessive_nesting` — **demonstrated on 2026-11-16** |
| QG-16 | Change a DIDL-Lite output → the snapshot check fails |
| QG-17 | Emit DIDL-Lite with a missing required attribute → the XSD check fails |
| QG-18 | Break the escaping function → the property test finds a counterexample |
| QG-19 | Feed a malformed input → the fuzz target aborts (this one *must* crash) |
| QG-11 | Add a broken relative link → the link checker fails — **demonstrated on 2026-11-14** (it found six broken links in the README on its first run) |
| QG-28 | Make a core crate depend on the CLI crate → the boundary check fails and names the edge — **demonstrated on 2026-11-16**. Make the administration crate depend on the daemon crate → the check fails and names the edge — **demonstrated on 2026-11-16** (the amendment of ADR-0012 allows the edge the other way and refuses this one) |

---

## 4. Milestone gates (G0 … G6)

The gate matrix above is *horizontal* (one per property). The milestone gates below are
*vertical* (one per phase). A milestone needs both.

| Gate | Name | Exit criteria |
| --- | --- | --- |
| **G0** | Requirements | Every requirement has an ID, a definition, a priority, a feature, a milestone and a verification method. No open contradiction. Baseline approved by the owner. |
| **G1** | Architecture | Module boundaries defined and enforced (gate QG-28), ADRs written for every significant decision, threat model current, dependency and licence review complete. |
| **G2** | Implementation | QG-01 … QG-18 green. No `unsafe` without an ADR. Public API documented. |
| **G3** | Interoperability | Protocol fixtures green, range and seek verified, the device matrix has at least one observed result per supported client family, regressions captured as fixtures. |
| **G4** | Robustness | Fuzz targets clean, traversal and resource-bound tests green, crash recovery proven, soak test completed. |
| **G5** | Distribution | Artefacts built reproducibly, SBOM and licence review done, checksums published, installation, upgrade, recovery and uninstall documented. |
| **G6** | Release | No open critical or high security finding, all exceptions documented and current, **explicit owner approval**. |

---

## 5. Continuous integration

GitHub Actions are **manual only** (`workflow_dispatch`) until the owner decides
otherwise — see `../07-project/CI-POLICY.md`. A green badge therefore never exists by
accident, and no claim of "CI is green" may be made without checking the run list and
the runner list.

Before a workflow may be switched to automatic, its gates must have been run manually
at least once **and** their negative tests must have been demonstrated.
