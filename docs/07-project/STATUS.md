# Project status — PhairDLNA

Status: **Baseline approved (2026-11-15) · M1.11 implemented · M2.1 implemented** · Compiled: 2026-11-16
Owner role: Lead / Project Manager

---

## 1. Where the project stands

The **requirements baseline is approved** (2026-11-15), and the implementation is running:
the milestones M1.1 to M1.11 are coded, and M1.9 has been through an independent review
whose findings are addressed. The server now **serves over the network** — but **no device
test has been performed**, so no milestone exit criterion is met yet. The repository
contains a Rust workspace, a complete documentation baseline and the gates that will hold
the implementation honest.

The milestone that was closed last is **M1.11 — the live daemon**: `phairdlna-cli serve`
reads the configuration, loads the device identity, scans the library, opens the HTTP
server and the discovery socket, and answers a client until it receives `SIGINT` or
`SIGTERM`. The server serves the device description, the two service descriptions, the
SOAP control endpoints, the media with its byte ranges and the artwork route; the
discovery socket advertises the server, answers the searches of a client and withdraws its
advertisements when the daemon stops. The order of the stop is the one of `OPS-008`:
the advertisements go first, the discovery stops, the HTTP server drains its transfers,
and the process exits with the code 0.

**What is proven and how.** The daemon is tested as a client sees it, over a real socket:
`crates/phairdlna-cli/tests/serve.rs` reads the description, browses the catalogue with
a SOAP request, streams a file and asks for a byte range; `crates/phairdlna-cli/tests/discovery.rs`
speaks the real SSDP protocol and observes the advertisement, the answer of a search and
the withdrawal; `crates/phairdlna-cli/tests/serve_command.rs` starts the binary, reads the
description and ends it with a `SIGTERM`. The rules themselves stay in the core, and the
core is unchanged by M1.11 — it takes no new dependency.

**The first step of M2 is closed: M2.1 — multiple roots, external volumes and network
shares.** The catalogue keeps the roots apart: every root has its own key in the
identifiers, its own place in the tree and its own boundary, so a link from one root
into another is not followed and a root that lies inside another root is published
once. The catalogue now **remembers the volume of every root** between two reads, and
it says what changed: a drive that was unplugged takes its files with it and is
reported, a drive that comes back under the same identity keeps its identifiers, and a
drive that comes back under **another** identity is given new identifiers while the
identifier of the catalogue grows, so that a client browses again (`PLT-011`). The
daemon watches the roots — a cheap question that reads no file of the library — and
reads the catalogue again when the answer is "yes"; a file that is added *inside* a
root is noticed by the next full scan, and the schedule of the scans is the work of
`FUN-024` (M2.4). The evidence is
`crates/phairdlna-core/tests/media_roots_multiple.rs`,
`crates/phairdlna-core/tests/volume_events.rs` and
`crates/phairdlna-core/tests/volume_platform_events.rs`; the decisions are D-15 to
D-20 of `../03-features/F-08-media-roots-and-filesystem.md`.

**What is still missing.** The IPv6 discovery, the interface selection of the sending
socket (the daemon has one address), the rate limiter of the answers, the artwork (F-11
is M2), the transfer limits and the index on disk. Each of them is named in the
requirement index with the state that is true today.

The milestone before it, **M1.9 — the catalogue over one root**, is unchanged: a scanner
that reads a configured root, opaque blake3 identifiers that survive a rescan and a move
of the library, and a ContentDirectory browse that shows real folders and files. The
placeholder catalogue is retired; `phairdlna-cli catalogue scan` prints the tree with the
identifiers a client would see, and `phairdlna-cli catalogue browse <id>` prints the
document a client would receive.

**The configuration takes a list of roots**, and M2.1 is the milestone that makes the
list mean something: several roots are browsable at the same time, they keep their own
identifiers and their own boundaries, and a root that comes and goes with its drive is
handled while the server runs.

**The requirement group of M1.9 that was open is now decided.** The roadmap assigned
`FUN-008`, `FUN-009` and `FUN-010` — the SQLite index that survives a rescan, that can
be rebuilt and that recovers from corruption — to M1.9. The scope the owner approved for
M1.9 was the scanner, the opaque identifiers and the browse over one root; the index on
disk was not part of it, so the catalogue of M1 lives in memory. The owner decided on
2026-11-16 (**DEC-037**) that the three requirements **move to M2**, where the rest of the
index work lives (step M2.9 of the roadmap). The in-memory catalogue of M1 is the
accepted baseline, and the requirement index carries the new milestone.

| Area | State | Evidence |
| --- | --- | --- |
| Requirements | **Complete** | 346 requirements with IDs, priorities, features, milestones and verification methods — `../01-product/REQUIREMENTS-INDEX.md` |
| Traceability | **Complete** | Generated matrix requirement → feature → milestone → module → verification → evidence — `TRACEABILITY-MATRIX.md` |
| Architecture | **Complete** | Module boundaries, data flows, runtime model — `../02-architecture/ARCHITECTURE.md` |
| Decisions | **Complete** | The ADRs in `../02-architecture/adr/` plus 41 entries in the decision log — `../00-governance/DECISION-LOG.md` |
| Security model | **Complete** | Eight mitigations, 28 threats, each with its verification — `../04-security/SECURITY-MODEL.md` |
| Privacy model | **Complete** | Data inventory, network posture, source offer — `../04-security/PRIVACY.md` |
| Dependency review | **Complete** | Licence data for every candidate crate, retrieved 2026-11-14 — `../04-security/DEPENDENCY-LICENSE-REVIEW.md` |
| Quality gates | **Defined** | 28 gates with their commands, the label vocabulary and 15 recorded negative tests; 13 gates can run today — `../05-quality/QUALITY-GATES.md` |
| Test strategy | **Complete** | Test layers, property tests, negative catalogue, fuzz targets — `../05-quality/TEST-STRATEGY.md` |
| Interoperability | **Framework complete, evidence empty** | 26 conformance checks defined; **no device test has been performed** — `../05-quality/INTEROPERABILITY-MATRIX.md` |
| Performance | **Budgets defined, not measured** | Budgets and the measurement protocol — `../05-quality/PERFORMANCE-BUDGETS.md` |
| Roadmap | **Complete** | Six milestones with exit criteria — `ROADMAP.md` |
| Risk register | **Complete** | 13 risks with mitigations and owners — `RISK-REGISTER.md` |
| Open decisions | **12 awaiting the owner** — OD-05 (minimum macOS version) was decided on 2026-11-15 | `OPEN-DECISIONS.md` |
| Glossary | **Complete** | The protocol, product, platform, security and project vocabulary — `../01-product/GLOSSARY.md` |
| Research inputs | **Complete** | The briefs the specifications cite — `../09-research/` |
| Implementation | **M0 done, M1.1–M1.11 implemented, M2.1 implemented** | The four crates exist with the boundary enforced; the configuration, the eight platform traits, the discovery protocol, the device description and the service descriptions, the SOAP dispatcher with the ContentDirectory and ConnectionManager services, the DIDL-Lite generation, the HTTP media delivery, the catalogue over one root, the catalogue over several roots with the memory of their volumes, and the live daemon are implemented; **no device test has been performed**, so no milestone exit criterion is met yet |
| Device evidence | **None** | Every device row reads `NOT TESTED` |
| Release | **Not releasable** | `RELEASE-BLOCKERS.md` |

---

## 2. What was actually run, and what it produced

| Command | Result |
| --- | --- |
| `./scripts/local-ci.sh` | **all eleven gates green** (2026-11-16, the run of M2.1): format, clippy, tests, no unsafe, file size, crate boundaries, doc links, licences, bans and sources, advisories, requirement views |
| `cargo fmt --all -- --check` | clean (it failed before this phase — the CLI entry point was reformatted) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace` | **616 passed, 0 failed, 0 ignored** (2026-11-16, the run of M2.1; 469 before it) |
| `scripts/check-file-size.sh` | `RESULT: PASS` |
| `scripts/check-crate-boundaries.sh` | `RESULT: PASS` (3 sibling dependencies across 4 crate manifests) |
| `cargo deny check licenses` | `licenses ok` (exit 0) |
| `cargo deny check bans sources` | `bans ok, sources ok` (exit 0) |
| `cargo audit` | exit 0 — 1509 advisories loaded, 85 dependencies scanned, no finding |
| `scripts/check-doc-links.sh` | `RESULT: PASS` (relative links checked) |
| `scripts/check-no-unsafe.sh` | `RESULT: PASS` (no unsafe code in the tracked Rust files) |
| `python3 scripts/generate_requirement_views.py --check` | up to date; 346 requirements indexed |
| GitHub Actions, manual runs | **success** — `43698100636`, `43698591233`, `43699136828`, all 13 gate steps green each time: https://github.com/mazer666/PhairDLNA/actions/runs/43699136828 |
| `python3 scripts/generate_requirement_views.py` | 346 requirements indexed; index and traceability matrix regenerated |

Toolchain used for the M2.1 run (and for M1.11 before it): rustc 1.99.1, cargo 1.99.1,
clippy 0.1.99, rustfmt 1.8.0-stable (Homebrew). `rust-toolchain.toml` pins the channel `stable`, so the toolchain
is within the support window (`rust-version` is 1.89).

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
| Independent reviews | 3 — a beginner-experience review, a consistency review and the review of M1.9, all in `../05-quality/reviews/` |
| Protocol conformance checks | 26 (`PC-01` … `PC-26`) |
| Threats modelled | 28 (`SEC-TH-01` … `SEC-TH-28`) |
| Quality gates | 28 (`QG-01` … `QG-28`), of which 13 can run today |
| Milestone gates | 7 (`G0` … `G6`) |
| Architecture decisions | The ADRs in `../02-architecture/adr/` + 41 decision-log entries |
| Open owner decisions | 13 (`OD-01` … `OD-13`), of which OD-05 is decided |
| Risks | 13 (`R-01` … `R-13`) |
| Archived baselines | 2 snapshots, each with a SHA-256 manifest |
| Research briefs committed | 9 documents under `../09-research/` |
| Competitors mapped | 11 products with per-feature proposals — `../01-product/COMPETITOR-FEATURE-ATLAS.md` |
| Competitive criteria | 11 testable criteria against the macOS field — `../01-product/COMPETITIVE-PARITY-ELMEDIA.md` |

---

## 4. What we should do next — the proposal

### 4.1 M0 is closed, with two items still open

M0 was the foundation: the approved baseline, the architecture, the four crates with the
boundary enforced, the licence and advisory gates on the real dependency set, and the
manual CI workflow. All of it is done. Two items of it are not:

| Item | What | Done when |
| --- | --- | --- |
| M0.5 | The manual CI workflow has been run on macOS and on Windows, not only on Linux | three run URLs exist in the run list, one per platform, and every step passed |
| M0.6 | The Intel macOS target of the build matrix builds | the matrix shows a green x86_64-apple-darwin build |

### 4.2 M1, in ten small steps

Each step ends in something observable, and each is small enough to review in one
sitting. The order is not negotiable: each step needs the one before it.

| # | Step | The observable that says it is done | State |
| --- | --- | --- | --- |
| 1 | Configuration and the platform traits | `phairdlna-cli config validate` accepts a generated configuration and rejects a broken one with a named field | done (M1.1, M1.2) |
| 2 | SSDP discovery | `phairdlna-cli ssdp headers` prints the exact bytes that will be sent, and a recorded `M-SEARCH` from a real client is answered | done (M1.3) |
| 3 | The device description and the SCPD | `curl http://localhost:PORT/description.xml` returns a document that a real client accepts | done (M1.4) |
| 4 | The SOAP dispatcher and ContentDirectory | A recorded `Browse` returns a DIDL-Lite document that validates against the vendored XSD (gate QG-17) | done (M1.5, M1.6); the XSD validation of QG-17 is still **planned** |
| 5 | ConnectionManager | `GetProtocolInfo` names only MIME types the server can actually serve | done (M1.6) |
| 6 | DIDL-Lite generation | The escaping property test passes, and the XSD validation passes | done (M1.7) |
| 7 | HTTP media delivery | A byte range returns 206 with a correct `Content-Range`; an unsatisfiable range returns 416 | done (M1.8) |
| 8 | The catalogue over one root | A browse shows one folder of real files with stable ids | **done (M1.9)** |
| 9 | The live daemon | `phairdlna-cli serve` answers a client over HTTP and over SSDP, and stops gracefully on a signal | **done (M1.11)** |
| 10 | The security baseline | Every threat row in F-16 has its negative test, and they pass | next |

**Step 10 is not optional and not last.** The threat rows are cheap to test while the
code is small, and expensive to retrofit. It is also the step that M1.11 was implemented
ahead of; it is closed next, against the code that is already written.

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
**and** the default that applies if there is no answer. **OD-05** (the minimum macOS
version) was decided on 2026-11-15: macOS 15 (Sequoia) and later, with the launchd daemon
model throughout. Everything else can be answered while M1 is already running.

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
