# Requirements elaboration work plan

Status: **Phases R0–R5 complete; R6 implementation running (2026-11-19)** · Owner role: Lead / Product Owner
Version/date: 1.1 / 2026-11-19
Purpose: Record the requirements-elaboration phases and point to current implementation status; not a substitute for the live roadmap or status report.
References: `../00-governance/PROJECT-STANDARDS.md`, `../00-governance/REQUIREMENTS-METHOD.md`, `ROADMAP.md`, `STATUS.md`, `../99-archive/2026-11-14-preconsolidation-baseline/MANIFEST.md`

This is the phase plan for the requirements cycle that produced the v1.0 baseline.
Each phase lists what it produced and where the result lives.

---

## R0 — Inventory

*Inspect the repository, map the current requirements, check inconsistencies and
blockers. Do not delete or overwrite owner decisions.*

**Done.** The repository carried four parallel document trees next to the numbered
ones the documentation policy prescribes. The consolidation is recorded in
`docs/00-governance/DECISION-LOG.md` (DEC-013) and the pre-consolidation state is
preserved byte-identical in `docs/99-archive/2026-11-14-preconsolidation-baseline/`
with a SHA-256 manifest. The owner decisions DEC-001 … DEC-004 were carried forward
unchanged and confirmed.

## R1 — Research

*Compare the standards, the implementation options, the competing servers, the
operating-system constraints and the device quirks. Cite the sources and distinguish
the standards from the observed behaviour.*

**Done.** Five research briefs were produced and are committed under
`docs/09-research/`: the DLNA/UPnP standards, the platform integration facts, the
security landscape, the quality and performance tooling, and the competitive
landscape, plus four per-client-family briefs. Every citation of the form
`standards §8.3 C12` resolves into those files; all 324 section citations were
checked mechanically.

Three findings changed the requirements: the ContentDirectory action list was
corrected against the primary specification, the "CallStranger" weakness was
reclassified from an XML problem to a GENA callback SSRF, and the AGPL network-use
clause was turned into a product requirement.

## R2 — Scenarios

*macOS-first personas, the setup, operation and troubleshooting flows, and the
device, network and media matrix.*

**Done.** Seven journeys in `docs/01-product/USER-JOURNEYS.md`, including the failure
paths that must not break the flow; five personas in the product vision; and the
priority client families in `docs/05-quality/INTEROPERABILITY-MATRIX.md` § 4.1.

## R3 — Specifications

*Feature by feature: functional, technical, performance, usability, security and
interoperability requirements with stable IDs.*

**Done.** 346 requirements with stable IDs, defined once in
`docs/01-product/TECHNICAL-REQUIREMENTS.md` and
`docs/01-product/PRODUCT-REQUIREMENTS.md`, and elaborated by 22 feature
specifications with 298 acceptance criteria. The index and the traceability matrix
are generated, so they cannot drift from the definitions.

## R4 — Design decisions

*Modular architecture, the native UI option, the SSDP and interface policy, the
indexing, the transcoding boundary, the external dependencies and the licence
compatibility.*

**Done.** Ten ADRs in `docs/02-architecture/adr/`, 38 decision-log entries, a
dependency and licence review with the crates.io data retrieved on 2026-11-14, and
the threat model with its eight mitigations.

## R5 — Review

*An independent product, architecture, security, QA and beginner-readability review;
an unresolved decision list and a proposed baseline for owner approval.*

**Done, twice.** The beginner-experience review found 6 blocker, 16 major, 11 minor
and 4 info findings; all of them are addressed, and the report is kept as evidence at
`docs/05-quality/reviews/BEGINNER-REVIEW-2026-11-14.md`. A second independent
consistency review found 2 blocker, 6 major, 15 minor and 4 info findings — among them
56 requirements that had lost their owning feature in the generated index. All of them
are addressed, and its report is kept at
`../05-quality/reviews/CONSISTENCY-REVIEW-2026-11-15.md`.

## R6 — Implementation

*Only after the owner approves the requirements baseline, begin M0/M1 in small
verified increments.*

**Running.** The owner approved the baseline on 2026-11-15 (DEC-031). M0 and M1.1–M1.11
are implemented. M2 is in progress: M2.1–M2.3 and M2.9 are recorded complete; M2.4–M2.8
are partial, with M2.6 (the native macOS application) specifically gated by its remaining
first-run, service, settings-activation and real-Mac acceptance work. The current evidence,
open decisions and exact state of each milestone live in `STATUS.md` and `ROADMAP.md`.

---

## Rules that applied throughout

- Deliver concise decision-oriented reports: done, evidence, risks, blockers, next
  actions.
- **Never claim a real device test without a real device.** The interoperability
  matrix is empty on purpose, and every row reads `NOT TESTED`.
- **Never claim a measurement that was not taken.** Every performance number in the
  documentation is a target with a method, never a result.
- A gate that has never failed proves nothing; every gate has a demonstrated negative
  test or is marked as not yet run.

## Revision history

| Version/date | Change |
| --- | --- |
| 1.1 / 2026-11-19 | Updates R6 to the current M2 implementation state, removes a stale duplicate of R3, and adds the required document metadata. |
