# Project status — PhairDLNA

Status: **Baseline awaiting owner approval** · Compiled: 2026-11-14
Owner role: Lead / Project Manager

---

## 1. Where the project stands

The **requirements phase is complete** and presented for approval. No DLNA feature is
implemented. The repository contains a Rust workspace scaffold, a complete
documentation baseline and the gates that will hold the implementation honest.

| Area | State | Evidence |
| --- | --- | --- |
| Requirements | **Complete** | 346 requirements with IDs, priorities, features, milestones and verification methods — `../01-product/REQUIREMENTS-INDEX.md` |
| Traceability | **Complete** | Generated matrix requirement → feature → milestone → module → verification → evidence — `TRACEABILITY-MATRIX.md` |
| Architecture | **Complete** | Module boundaries, data flows, runtime model — `../02-architecture/ARCHITECTURE.md` |
| Decisions | **Complete** | The ADRs in `../02-architecture/adr/` plus 17 entries in the decision log — `../00-governance/DECISION-LOG.md` |
| Security model | **Complete** | Eight mitigations, 28 threats, each with its verification — `../04-security/SECURITY-MODEL.md` |
| Privacy model | **Complete** | Data inventory, network posture, source offer — `../04-security/PRIVACY.md` |
| Dependency review | **Complete** | Licence data for every candidate crate, retrieved 2026-11-14 — `../04-security/DEPENDENCY-LICENSE-REVIEW.md` |
| Quality gates | **Defined** | 28 gates with their commands, the label vocabulary and 12 recorded negative tests; 11 gates can run today — `../05-quality/QUALITY-GATES.md` |
| Test strategy | **Complete** | Test layers, property tests, negative catalogue, fuzz targets — `../05-quality/TEST-STRATEGY.md` |
| Interoperability | **Framework complete, evidence empty** | 26 conformance checks defined; **no device test has been performed** — `../05-quality/INTEROPERABILITY-MATRIX.md` |
| Performance | **Budgets defined, not measured** | Budgets and the measurement protocol — `../05-quality/PERFORMANCE-BUDGETS.md` |
| Roadmap | **Complete** | Six milestones with exit criteria — `ROADMAP.md` |
| Risk register | **Complete** | 13 risks with mitigations and owners — `RISK-REGISTER.md` |
| Open decisions | **12 awaiting the owner** | `OPEN-DECISIONS.md` |
| Glossary | **Complete** | The protocol, product, platform, security and project vocabulary — `../01-product/GLOSSARY.md` |
| Research inputs | **Complete** | The briefs the specifications cite — `../09-research/` |
| Implementation | **M0 started** | The four crates exist, the configuration module is implemented with 14 tests, the command line has `version`, `config path`, `config validate` and `config show` |
| Device evidence | **None** | Every device row reads `NOT TESTED` |
| Release | **Not releasable** | `RELEASE-BLOCKERS.md` |

---

## 2. What was actually run, and what it produced

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | clean (it failed before this phase — the CLI entry point was reformatted) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace` | 26 passed, 0 failed, 0 ignored (2 doctests included) |
| `scripts/check-file-size.sh` | `RESULT: PASS` |
| `scripts/check-crate-boundaries.sh` | `RESULT: PASS` (3 sibling dependencies across 4 crate manifests) |
| `cargo deny check licenses` | `licenses ok` (exit 0) |
| `cargo deny check bans sources` | `bans ok, sources ok` (exit 0) |
| `cargo audit` | exit 0 — 1509 advisories loaded, 35 dependencies scanned, no finding |
| `scripts/check-doc-links.sh` | `RESULT: PASS` (118 relative links checked) |
| `scripts/check-no-unsafe.sh` | `RESULT: PASS` (no unsafe code in 2 tracked Rust files) |
| `python3 scripts/generate_requirement_views.py --check` | up to date; 346 requirements indexed |
| `python3 scripts/generate_requirement_views.py` | 346 requirements indexed; index and traceability matrix regenerated |

Toolchain used: rustc 1.100.0, cargo 1.100.0, clippy 0.1.100, rustfmt 1.11.0-stable.

Everything else in this repository is **documented and unmeasured**. In particular:

- no performance number has been recorded,
- no device test has been performed,
- no fuzzing run has been performed,
- no soak test has been performed,
- no claim of DLNA certification exists and none will be made.

---

## 3. The baseline in numbers

| Number | Value |
| --- | --- |
| Requirements | 346 — 284 MUST, 57 SHOULD, 5 COULD |
| Feature specifications | 22 (F-01 … F-19, F-22, plus two proposals, F-20 / F-21) |
| Acceptance criteria | 330 across the 20 baseline specifications (F-20 and F-21 are proposals and define none) |
| Glossary entries | 77 terms across five vocabularies |
| Independent reviews | 2 — a beginner-experience review and a consistency review, both in `../05-quality/reviews/` |
| Protocol conformance checks | 26 (`PC-01` … `PC-26`) |
| Threats modelled | 28 (`SEC-TH-01` … `SEC-TH-28`) |
| Quality gates | 28 (`QG-01` … `QG-28`), of which 11 can run today |
| Milestone gates | 7 (`G0` … `G6`) |
| Architecture decisions | The ADRs in `../02-architecture/adr/` + 23 decision-log entries |
| Open owner decisions | 12 (`OD-01` … `OD-12`) |
| Risks | 13 (`R-01` … `R-13`) |
| Archived baselines | 2 snapshots, each with a SHA-256 manifest |
| Research briefs committed | 9 documents under `../09-research/` |

---

## 4. What we should do next — the proposal

### 4.1 The first thing: M0, and it is small

M0 has four pieces of work left, and all four are mechanical:

| Step | What | Done when |
| --- | --- | --- |
| M0.1 | The four crates exist: `phairdlna-core`, `phairdlna-cli`, `phairdlna-admin`, `phairdlna-mac` | `cargo build --workspace` succeeds on macOS, Linux and Windows |
| M0.2 | `scripts/check-crate-boundaries.sh` (gate QG-28) | an intentionally inverted dependency turns it red |
| M0.3 | `deny.toml` proven on the real dependency set | `cargo deny check licenses bans sources` is green, and `cargo audit` is green |
| M0.4 | The manual CI workflow has been run once on every platform | three run URLs exist in the run list, and every step passed |

**Why this order:** the boundary check is the gate that protects everything the
project builds afterwards. It is cheap to write now and expensive to retrofit.

### 4.2 Then M1, in nine small steps

Each step ends in something observable, and each is small enough to review in one
sitting. The order is not negotiable: each step needs the one before it.

| # | Step | The observable that says it is done |
| --- | --- | --- |
| 1 | Configuration and the platform traits | `phairdlna-cli config validate` accepts a generated configuration and rejects a broken one with a named field |
| 2 | SSDP discovery | `phairdlna-cli ssdp headers` prints the exact bytes that will be sent, and a recorded `M-SEARCH` from a real client is answered |
| 3 | The device description and the SCPD | `curl http://localhost:PORT/description.xml` returns a document that a real client accepts |
| 4 | The SOAP dispatcher and ContentDirectory | A recorded `Browse` returns a DIDL-Lite document that validates against the vendored XSD (gate QG-17) |
| 5 | ConnectionManager | `GetProtocolInfo` names only MIME types the server can actually serve |
| 6 | DIDL-Lite generation | The escaping property test passes, and the XSD validation passes |
| 7 | HTTP media delivery | A byte range returns 206 with a correct `Content-Range`; an unsatisfiable range returns 416 |
| 8 | The catalogue over one root | A browse shows one folder of real files with stable ids |
| 9 | The security baseline | Every threat row in F-16 has its negative test, and they pass |

**Step 9 is not optional and not last.** The threat rows are cheap to test while the
code is small, and expensive to retrofit.

### 4.3 The first real device, and when

**No device test happens before steps 1 to 8 are green.** The conformance checklist
(`PC-01` … `PC-26`) is automated and runs first; a device is the *second* test, not
the first. The first device round is scheduled inside M1, and its result — including
failures — goes into the interoperability matrix the same day.

**Which device first:** a Samsung or LG television, because they are the most
frequently reported source of DLNA incompatibilities. If only one is available, use
VLC on a laptop, because it logs what it receives.

### 4.4 What we deliberately do not do first

- No transcoding (M5), no artwork (M2), no menu bar app (M2), no second root (M2),
  no administration surface (M2), no Linux and Windows packaging (M3).
- No plugin system, ever, without an ADR.
- No automatic CI. Manual only, until the gates have been run three times green and
  their negative tests demonstrated.

### 4.5 What the owner has to answer, and what happens if they do not

Twelve open decisions are listed in `OPEN-DECISIONS.md`, each with a recommendation
**and** the default that applies if there is no answer. Only **OD-05** (the minimum
macOS version) blocks work, because the service model and the first-run flow depend
on it. Everything else can be answered while M1 is already running.

### 4.6 The honest risk of this plan

The plan is a plan, not a forecast. The largest uncertainty is still the device
matrix: the first device round may reveal that a requirement is wrong. That is the
point of the matrix, and it is cheaper to learn now than after M2.

---

## 5. Honest limitations of this baseline

- **The interoperability matrix is empty.** The mechanism is defined; the evidence is
  not there yet. The first device round will change several requirements, and that is
  the intended purpose of the matrix.
- **The performance budgets are targets.** They were chosen to be plausible and
  measurable; they have not been checked against a real machine.
- **The macOS firewall and local-network behaviour are documented by Apple and not yet
  verified on a device.** They are the two largest macOS-specific risks (R-02, R-03).
- **The competitive landscape rests on public documentation, not on comparative
  testing.** No competitor was benchmarked.
- **Some protocol details could not be verified against a primary specification** and
  are recorded as open questions in the feature specifications — in particular the
  `writeStatus` attribute, the exact `dc:date` format constraints, and the DLNA
  guideline table that a media server must satisfy.
