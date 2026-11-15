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
| Requirements | **Complete** | 341 requirements with IDs, priorities, features, milestones and verification methods — `../01-product/REQUIREMENTS-INDEX.md` |
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
| Implementation | **Not started** | The workspace is a scaffold by design |
| Device evidence | **None** | Every device row reads `NOT TESTED` |
| Release | **Not releasable** | `RELEASE-BLOCKERS.md` |

---

## 2. What was actually run, and what it produced

| Command | Result |
| --- | --- |
| `cargo fmt --all -- --check` | clean (it failed before this phase — the CLI entry point was reformatted) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace` | 1 passed, 0 failed, 0 ignored |
| `scripts/check-file-size.sh` | `RESULT: PASS` |
| `scripts/check-doc-links.sh` | `RESULT: PASS` (118 relative links checked) |
| `scripts/check-no-unsafe.sh` | `RESULT: PASS` (no unsafe code in 2 tracked Rust files) |
| `python3 scripts/generate_requirement_views.py --check` | up to date; 341 requirements indexed |
| `python3 scripts/generate_requirement_views.py` | 341 requirements indexed; index and traceability matrix regenerated |

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
| Requirements | 341 (MUST / SHOULD / COULD per the index totals) |
| Feature specifications | 21 (F-01 … F-19 plus two proposals, F-20 / F-21) |
| Glossary entries | 90 terms across five vocabularies |
| Independent reviews | 2 (a beginner-experience review and a consistency review) |
| Acceptance criteria | > 300 across the feature specifications |
| Protocol conformance checks | 26 (`PC-01` … `PC-26`) |
| Threats modelled | 28 (`SEC-TH-01` … `SEC-TH-28`) |
| Quality gates | 28 (`QG-01` … `QG-28`), of which 11 can run today |
| Milestone gates | 7 (`G0` … `G6`) |
| Architecture decisions | The ADRs in `../02-architecture/adr/` + 17 decision-log entries |
| Open owner decisions | 12 (`OD-01` … `OD-12`) |
| Risks | 13 (`R-01` … `R-13`) |
| Archived baselines | 2 snapshots, each with a SHA-256 manifest |
| Research briefs committed | 9 documents under `../09-research/` |

---

## 4. What happens next

1. The owner reviews the baseline and answers the open decisions in
   `OPEN-DECISIONS.md`.
2. The owner approves the baseline (or returns it with changes).
3. M0 is completed: the four crates, the boundary gate, the manual CI workflow, the
   file-size and link gates.
4. M1 begins with the configuration and platform traits, then discovery, then the
   device description, then browsing, then metadata, then streaming — with the
   conformance checklist green before any device is touched.

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
