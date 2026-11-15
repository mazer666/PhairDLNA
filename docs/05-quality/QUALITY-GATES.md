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

| ID | Gate | Tool / command | Severity | Blocks | Notes |
| --- | --- | --- | --- | --- | --- |
| QG-01 | Formatting | `cargo fmt --all -- --check` | blocker | every milestone | |
| QG-02 | Lints | `cargo clippy --workspace --all-targets -- -D warnings` | blocker | every milestone | No `#[allow]` without a justifying comment |
| QG-03 | No `unsafe` | `scripts/check-no-unsafe.sh` | blocker | every milestone | `SUP-008`; any `unsafe` needs an ADR. **Implemented and running** |
| QG-04 | Build | `cargo build --workspace --all-targets` | blocker | every milestone | Plus the release profile and the macOS universal binary |
| QG-05 | Unit tests | `cargo nextest run --workspace` | blocker | every milestone | |
| QG-06 | Integration tests | `cargo nextest run --workspace --test '*'` | blocker | M1+ | |
| QG-07 | Doctests | `cargo test --workspace --doc` | blocker | every milestone | nextest does not cover these |
| QG-08 | Licences | `cargo deny check licenses` | blocker | M0+ | `AGPL-3.0-only` + `MPL-2.0`; `unused-allowed-license = warn` |
| QG-09 | Advisories | `cargo audit` | blocker | M0+ | Fail on any un-waived advisory |
| QG-10 | Bans and sources | `cargo deny check bans sources` | blocker | M0+ | Banned crates list in `deny.toml` |
| QG-11 | Link check | `scripts/check-doc-links.sh` | blocker | every milestone | Resolves every relative link in the tracked markdown. **Implemented and running**; `lychee` is the planned replacement for external links |
| QG-12 | Spelling | `typos` | warning | every milestone | |
| QG-13 | File size | `scripts/check-file-size.sh` | blocker | M1+ | ≤ 300 lines target, > 500 hard limit; no maintained Rust tool exists, so the project writes its own. **Implemented and running** |
| QG-14 | Complexity | `clippy::excessive_nesting` + `clippy::too_many_lines` (blocking) | blocker | M1+ | `rust-code-analysis-cli` is stale (2023) and MPL-2.0 → advisory only, never gating |
| QG-15 | Binary size | *planned* — an `xtask` command | warning | release | Budget per binary, documented |
| QG-16 | Snapshot review | `cargo insta test --check` | blocker | M1+ | `INSTA_UPDATE=no`, `INSTA_FORCE_PASS` unset |
| QG-17 | DIDL-Lite conformance | *planned* — an `xtask` command validating against the vendored `didl-lite.xsd` via the `xmlschema` crate | blocker | M1+ | Arrives with the DIDL-Lite writer |
| QG-18 | Property tests | `cargo nextest run --workspace proptest` | blocker | M1+ | Escaping round-trip; path containment |
| QG-19 | Fuzz smoke | `cargo fuzz run <target> -- -runs=N` | blocker | M4 | Each target has a committed seed corpus |
| QG-20 | Miri | `cargo +nightly miri test` | warning | M1+ | Only meaningful where `unsafe` exists; mostly a "no unsafe" proof |
| QG-21 | Benchmark baseline | `cargo bench --workspace -- --save-baseline` | blocker | M1+ | Compared against the recorded baseline |
| QG-22 | Soak test | `scripts/soak.sh --duration 7d` | blocker | release | RSS, fd count, thread count, index growth, dropped connections |
| QG-23 | Device matrix | `scripts/device-matrix.sh` | blocker | M1+ | Rows in `INTEROPERABILITY-MATRIX.md` with device, firmware, format, outcome |
| QG-24 | Public API compatibility | `cargo semver-checks` | blocker | release | |
| QG-25 | Outdated dependencies | `cargo outdated` | INFO | — | Informational only |
| QG-26 | Coverage | `cargo llvm-cov` | INFO | — | Informational; never a quality claim on its own |
| QG-27 | Mutation testing | `cargo mutants` | INFO | — | Informational, optional |
| QG-28 | Crate boundaries | *planned* — `scripts/check-crate-boundaries.sh` | blocker | M0+ | Fails when a crate depends "downwards". Arrives with the crates it guards (ADR-0001) |

### 3.0 The canonical register

**This table is the single register of the gate identifiers.** Other documents refer
to a gate as `QG-nn` and mean the row here. A document that names a gate for a
different check is a defect — fix the document, not this table.

Four gates exist and run today: **QG-01** (formatting), **QG-03** (no `unsafe`),
**QG-11** (link check) and **QG-13** (file size). The rest arrive with the code they
guard, and a gate that does not exist is reported as `NOT RUN — <reason>`, never as a
pass.

### 3.1 What each gate must prove

A gate that has never been seen red is not a gate. For every gate the project records
a **negative test**:

| Gate | Negative test that must turn it red |
| --- | --- |
| QG-01 | Introduce a mis-indented line → the check reports a diff |
| QG-02 | Introduce a lint violation → clippy fails with `-D warnings` |
| QG-03 | Add one `unsafe` block → the check fails and names the file — **demonstrated on 2026-11-14** |
| QG-08 | Add a dependency with an incompatible licence → the check fails |
| QG-09 | Add a dependency with a known advisory → the check fails |
| QG-13 | Add a 600-line file → the check fails and names the file — **demonstrated on 2026-11-14** |
| QG-14 | Add a function with six nesting levels → clippy fails |
| QG-16 | Change a DIDL-Lite output → the snapshot check fails |
| QG-17 | Emit DIDL-Lite with a missing required attribute → the XSD check fails |
| QG-18 | Break the escaping function → the property test finds a counterexample |
| QG-19 | Feed a malformed input → the fuzz target aborts (this one *must* crash) |
| QG-11 | Add a broken relative link → the link checker fails — **demonstrated on 2026-11-14** (it found six broken links in the README on its first run) |

---

## 4. Milestone gates (G0–G6)

The gate matrix above is *horizontal* (one per property). The milestone gates below are
*vertical* (one per phase). A milestone needs both.

| Gate | Name | Exit criteria |
| --- | --- | --- |
| **G0** | Requirements | Every requirement has an ID, a definition, a priority, a feature, a milestone and a verification method. No open contradiction. Baseline approved by the owner. |
| **G1** | Architecture | Module boundaries defined and enforced (QG-07-style boundary check), ADRs written for every significant decision, threat model current, dependency and licence review complete. |
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
