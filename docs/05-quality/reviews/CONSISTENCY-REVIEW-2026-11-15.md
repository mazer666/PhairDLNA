# Independent consistency review — PhairDLNA (round 2)
Status: Complete — all findings addressed · Owner role: Independent reviewer · Version/date: 1.0 / 2026-11-15

Reviewer: Independent reviewer (general-purpose agent)
Date: 2026-11-15
Revision reviewed: `c6a0e81` ("docs: propose the next steps concretely, and give every open
decision a default"), branch `main`, **plus the uncommitted working-tree changes that
were present when I finished** (see F-29, F-30).

> **Revision note.** The task named `fc07b2c` as the revision to review. I started there
> and the repository advanced by four commits while I was working
> (`746b5a9` → `7b934b9` → `c6a0e81` → `019f673`). Findings below are reported against
> **`c6a0e81`**, because that is the last committed revision I could read consistently;
> where a defect existed only at `fc07b2c` and was fixed by the intervening commits, I
> say so explicitly (see F-28), and where the intervening commits *introduced* a defect
> I say so too (F-07, F-10, F-06).
>
> **The working tree is being edited concurrently.** When I finished, `git status`
> showed uncommitted changes to `scripts/generate_requirement_views.py`,
> `docs/01-product/REQUIREMENTS-INDEX.md` and `docs/07-project/TRACEABILITY-MATRIX.md`.
> They are an in-progress attempt at the F-01 fix — they allocate the 56 unallocated
> requirements — and they carry two defects of their own (F-29, F-30). I did not touch
> them; I only read them.

Scope: the whole documentation tree (`AGENTS.md`, `README.md`, `SECURITY.md`,
`CONTRIBUTING.md`, `WHERE_TO_CHANGE.md`, `docs/00-…` … `docs/09-research`, `docs/99-archive`
manifests, `scripts/`, `.github/workflows/`), the requirement baseline
(`TECHNICAL-REQUIREMENTS.md`, `PRODUCT-REQUIREMENTS.md`, the generated index and
traceability matrix), the 21 feature specifications, the architecture and the ten ADRs,
the security model, the quality gates, the performance budgets, the interoperability
matrix, the roadmap, the status report, the decision log and the open-decision list.

---

## Summary

This baseline is materially better than the one the beginner review saw, and several of
its structural decisions are genuinely good: the requirement views are generated and
cannot drift; the gate vocabulary (`PASS` / `NOT RUN` / `INCONCLUSIVE` / …) is excellent;
the research briefs are committed and every citation resolves; the archive manifests
verify byte-for-byte against their SHA-256 digests; and the security model maps all
eight mitigations and all 28 threats to real requirements. I checked all of that
mechanically and it holds.

It is **not yet ready to present to the owner**, however, for three reasons.

First, the traceability chain has a hole in the middle of it. **56 of the 341
requirements have no owning feature at all** — the generated index prints `—` in the
Feature column and silently defaults their milestone to `M0` — and the roadmap places
those same requirements in M1, M2, M3 and M4. The index's own coverage rule says a
requirement without a feature "stays `Draft` and is not part of the baseline", yet all
341 rows read `Baseline`. An implementer who trusts the milestone column will schedule
`PLT-002` (a build matrix) and `INT-016` (SSDP TTL) as M0 foundation work, and will
schedule `SEC-066` (out-of-process media parsing) as M0 rather than M4 hardening.

Second, the **canonical gate register is contradicted by the document that owns the
gates**. `F-19` §4.1 reproduces the matrix with `cargo xtask check-unsafe`,
`cargo xtask size-check` and `lychee --offline` — commands that cannot run — while
`QUALITY-GATES.md` §3.0 states in bold that its table is the single register and that
"a document that names a gate for a different check is a defect". `CI-POLICY.md` then
claims CI runs QG-03, but `ci.yml` has no QG-03 step at all.

Third, the **honesty axis has slipped in the status layer**. The decision log, the
status report and the workplan all record an "independent consistency review (round 2)"
that has not happened yet, and `STATUS.md` still carries three stale counts (17
decision-log entries against 23, 2 reviews against 1, 90 glossary terms against 78).
The last three commits introduced a fourth inconsistency: `OD-05`'s new default of
"macOS 13 and later" contradicts the glossary, `F-15`, the risk register and the
platform research, all of which assume the macOS 15+ local-network permission.

Findings by severity: **2 Blocker, 9 Major, 15 Minor, 4 Info** (F-28 records a defect
that existed at `fc07b2c` and was fixed by a commit that landed during this review;
F-29 and F-30 describe the in-progress working-tree change).

**The good news about F-01**: the uncommitted working-tree change at the time I
finished allocates all 56 requirements to features, which is the right fix. It needs
two corrections before it is committed (F-29, F-30).

---

## Findings

### F-01 — Blocker — 56 requirements have no owning feature, are dated M0, and contradict the roadmap

**Location**: `docs/01-product/REQUIREMENTS-INDEX.md` (56 rows), `scripts/generate_requirement_views.py:148-164`, `docs/07-project/ROADMAP.md:60-72`

**What is wrong**: The generator's `FEATURES` table does not cover 56 requirement IDs.
`allocate()` leaves them unmapped, and `FEATURE_MILESTONES.get("—", "M0")` then stamps
every one of them with milestone `M0`. The affected IDs are
`FUN-025`, `INT-016`…`INT-019`, `INT-044`…`INT-049`, `INT-057`…`INT-059`,
`INT-073`…`INT-079`, `INT-092`…`INT-099`, `SEC-066`…`SEC-072`, `OPS-016`, `OPS-017`,
`PLT-001`…`PLT-003`, `PLT-005`…`PLT-018` (except `PLT-004`), and `SUP-009`.

**Evidence** — the index row and the index's own coverage rule:

> | `SEC-066` | Parse media metadata and generate thumbnails in a separate | MUST | `—` | M0 | Negative security test | Baseline |

> A requirement is only *planned* when it has all of: an ID, a definition, a priority,
> an owning feature, a milestone and a verification method. A requirement missing any
> of these stays `Draft` and is not part of the baseline.
> — `docs/01-product/REQUIREMENTS-INDEX.md:456-458`

> A requirement without a feature, a milestone and a verification method is **not a
> requirement yet** — it is a wish, and it stays `Draft`.
> — `docs/00-governance/REQUIREMENTS-METHOD.md:103-105`

> | Requirements | **Complete** | 341 requirements with IDs, priorities, features, milestones and verification methods — `../01-product/REQUIREMENTS-INDEX.md` |
> — `docs/07-project/STATUS.md:16`

And the roadmap, whose header defers to the index's milestone column:

> Requirements: `../01-product/REQUIREMENTS-INDEX.md` (milestone column)
> — `docs/07-project/ROADMAP.md:4`

> | M1.3 | SSDP discovery and advertisement | `INT-001` … `INT-019` |
> | M1.10 | The security baseline | `SEC-001` … `SEC-018`, `SEC-060` … `SEC-065`, `SEC-090` … `SEC-096` |
> — `docs/07-project/ROADMAP.md:64,71`

Mechanical comparison of the roadmap's requirement ranges against the index's milestone
column: **54 of 56** unallocated requirements carry a milestone in the index that
contradicts the roadmap's placement (`INT-016`…`INT-019`: index M0 / roadmap M1;
`INT-044`, `INT-045`: M0 / M1; `INT-046`…`INT-049`: M0 / M1; `INT-073`…`INT-079`: M0 /
M1; `INT-092`…`INT-099`: M0 / M1; `SEC-066`…`SEC-072`: M0 / M4; `OPS-016`, `OPS-017`:
M0 / M2; `PLT-002`, `PLT-003`, `PLT-015`, `PLT-016`: M0 / M3; `PLT-005`, `PLT-011`: M0 /
M1; `PLT-013`: M0 / M2).

The feature specifications contradict the index in the same way, in their own
`Requirements:` headers: F-08 claims `PLT-003`, `PLT-005`, `PLT-006`, `PLT-007`,
`PLT-011`, `PLT-012`, `PLT-015`, `PLT-017`; F-16 claims `INT-057`…`INT-059` and
`SEC-066`…`SEC-072`; F-14 claims `OPS-016`, `OPS-017`, `SEC-070`…`SEC-072`; F-09
claims `FUN-025`. The index says none of these belong to that feature.

**Suggested fix**: Extend `FEATURES` in `scripts/generate_requirement_views.py` so every
ID is allocated (the specifications already say who owns them), and add an assertion
that fails the generator when any requirement is left unallocated — the silent
`"—"` default is what let this through. While there, make the generator refuse to emit
`Baseline` for a requirement that has no feature or no milestone.

---

### F-02 — Blocker — F-19 reproduces the gate matrix with commands that cannot run

**Location**: `docs/03-features/F-19-quality-gates-and-developer-experience.md:135-165`

**What is wrong**: F-19 §4.1 says it is reproducing the canonical register, then gives
five gates a command that does not exist and one gate a different command from the
canonical one. `Cargo.toml` has `members = ["crates/phairdlna-core", "crates/phairdlna-cli"]`
— there is no `xtask` crate, so `cargo xtask check-unsafe` fails with
`error: no such command: xtask`.

**Evidence**:

> The canonical list is `../05-quality/QUALITY-GATES.md` §3; it is reproduced here because this feature owns the *developer experience* around it.
> — `F-19:135`

> | QG-03 | No `unsafe` | `cargo xtask check-unsafe` | blocker | every milestone |
> | QG-11 | Link check | `lychee --offline docs/ README.md` | blocker | every milestone |
> | QG-13 | File size | `cargo xtask size-check` | blocker | M1+ |
> | QG-15 | Binary size | `cargo xtask binary-size` | warning | release |
> | QG-17 | DIDL-Lite conformance | `cargo xtask validate-didl` | blocker | M1+ |
> — `F-19:142,150,152,154,156`

> **This table is the single register of the gate identifiers.** Other documents refer
> to a gate as `QG-nn` and mean the row here. A document that names a gate for a
> different check is a defect — fix the document, not this table.
> — `docs/05-quality/QUALITY-GATES.md:109-111`

The canonical rows are `scripts/check-no-unsafe.sh`, `scripts/check-doc-links.sh` and
`scripts/check-file-size.sh` — all three of which exist and run. F-19's matrix also
stops at QG-27 and omits QG-28, and §2 still describes "the gate matrix (QG-01 … QG-27)"
(`F-19:36`).

This is the exact defect the beginner review raised as B-02 for
`BUILD-AND-TEST.md`. That document was fixed; **F-19 was not**.

**Suggested fix**: Delete the reproduction and link to `QUALITY-GATES.md` §3, or replace
the five commands with the canonical ones and add QG-28. A gate matrix that exists in
two places is exactly what DEC-021 was written to prevent.

---

### F-03 — Major — 17 PRD IDs are defined in PRODUCT-REQUIREMENTS.md but absent from the index, and the document claims otherwise

**Location**: `docs/01-product/PRODUCT-REQUIREMENTS.md:6-7, 52-70`; `scripts/generate_requirement_views.py:25`

**What is wrong**: `PRD-020` … `PRD-036` are the 17 user stories in §3. They use the
canonical `PRD-` requirement prefix but have a different table shape (no Priority, no
Verification column), so the generator's row regex does not match them and they are
silently dropped from both generated views. They are referenced nowhere else in the
repository. Meanwhile the document that defines them asserts that every ID below
appears in the index.

**Evidence**:

> Traceability: every ID below appears in `REQUIREMENTS-INDEX.md` with feature, milestone
> and verification method.
> — `PRODUCT-REQUIREMENTS.md:6-7`

> | PRD-020 | first-time user | to point the app at a folder and see it on my TV without reading a manual | my media works in minutes, not hours |

Machine check: `PRD-020` … `PRD-036` appear in **zero** rows of
`REQUIREMENTS-INDEX.md` and **zero** rows of `TRACEABILITY-MATRIX.md`, and in no other
document. `python3 scripts/generate_requirement_views.py --check` nevertheless reports
"up to date" — the generator's own check cannot detect its own omission.

These IDs also break `AGENTS.md` ("Every requirement has: an ID, one normative
sentence, a priority, an owning feature, a milestone and a verification method") and
`REQUIREMENTS-METHOD.md` §8.

**Suggested fix**: Decide what they are. If they are requirements, give them a priority,
a verification method and an owning feature and extend the generator's regex to read
them. If they are user stories, renumber them out of the `PRD-` namespace (e.g.
`US-*`, with `US-*` added to the method's prefix table) and correct the traceability
sentence. Either way, make the generator fail loudly when an ID in a definition file
does not reach the index.

---

### F-04 — Major — CI-POLICY claims CI runs QG-03; ci.yml has no QG-03 step

**Location**: `docs/07-project/CI-POLICY.md:36-38, 62-66`; `.github/workflows/ci.yml`

**What is wrong**: Two places in CI-POLICY assert that `scripts/check-no-unsafe.sh`
(QG-03) is one of the gates a CI run executes. It is not in the workflow.

**Evidence**:

> The scripts the workflows call are `scripts/check-file-size.sh` (QG-13),
> `scripts/check-doc-links.sh` (QG-11), `scripts/check-no-unsafe.sh` (QG-03) and
> `scripts/generate_requirement_views.py`.
> — `CI-POLICY.md:36-38`

> A `ci.yml` run covers:
> `QG-01` formatting · `QG-02` lints · `QG-03` no unsafe · `QG-04` build · …
> — `CI-POLICY.md:64-66`

Actual `ci.yml` steps: `QG-01 formatting`, `QG-02 lints`, `QG-04 build`, `QG-05 unit and
integration tests`, `QG-07 doctests`, `QG-08 licences`, `QG-09 advisories`,
`QG-10 bans and sources`, `QG-11 documentation links`, `QG-13 file size`,
`Requirement views are current`.

```
$ grep -c 'check-no-unsafe' .github/workflows/ci.yml
0
$ grep -c 'QG-03' .github/workflows/ci.yml
0
```

QG-03 is a **blocker** gate for "every milestone" in the canonical register, with state
`exists` and the note "**Implemented and running**" — so the register, the policy and
the workflow disagree about whether a gate runs.

**Suggested fix**: Add the step (`- name: QG-03 no unsafe` / `run: ./scripts/check-no-unsafe.sh`)
or remove QG-03 from CI-POLICY §2 and §4. A policy that lists a gate the pipeline does
not run is a "skip mistaken for a pass" in the place the project most wants accuracy.

---

### F-05 — Major — FEATURE-CATALOG's product-commitment table is not supported by any feature specification

**Location**: `docs/01-product/FEATURE-CATALOG.md:58-81`

**What is wrong**: The "Product commitment coverage" table maps each of `PRD-001` …
`PRD-015` to the features that deliver it. For 14 of the 15 rows, at least one named
feature's specification never mentions that requirement ID, and **10 of the 15 PRD IDs
appear in no feature specification at all**.

**Evidence**:

> | Free of charge, no advertising, no telemetry, no account | `PRD-002` | F-16, F-19 | Outbound-connection audit, dependency review |
> | Documentation is a product deliverable | `PRD-011` | F-19 | Documentation gate (QG-11, QG-13) |
> — `FEATURE-CATALOG.md:68,77`

Machine check (for each catalogue row, is the PRD ID present in the named feature spec?):

```
PRD-001: catalogue says delivered by ['F-19']            -> not mentioned in ['F-19']
PRD-002: catalogue says delivered by ['F-16', 'F-19']    -> not mentioned in ['F-16', 'F-19']
PRD-003: catalogue says delivered by ['F-16', 'F-13']    -> not mentioned in ['F-16', 'F-13']
PRD-004: catalogue says delivered by ['F-01', 'F-16']    -> not mentioned in ['F-01', 'F-16']
PRD-005: catalogue says delivered by ['F-08', 'F-19']    -> not mentioned in ['F-08', 'F-19']
PRD-006: catalogue says delivered by ['F-13']            -> not mentioned in none      <- the only supported row
PRD-007: catalogue says delivered by ['F-08', 'F-16']    -> not mentioned in ['F-08', 'F-16']
PRD-008: catalogue says delivered by ['F-08','F-15','F-19'] -> not mentioned in ['F-08','F-15','F-19']
PRD-009: catalogue says delivered by ['F-12', 'F-13']    -> not mentioned in ['F-13']
PRD-010: catalogue says delivered by ['F-13', 'F-15']    -> not mentioned in ['F-13', 'F-15']
PRD-011: catalogue says delivered by ['F-19']            -> not mentioned in ['F-19']
PRD-012: catalogue says delivered by ['F-19']            -> not mentioned in ['F-19']
PRD-013: catalogue says delivered by ['F-16']            -> not mentioned in ['F-16']
PRD-014: catalogue says delivered by ['F-14', 'F-15']    -> not mentioned in ['F-14', 'F-15']
PRD-015: catalogue says delivered by ['F-06', 'F-17']    -> not mentioned in ['F-06', 'F-17']
```

The PRD IDs that appear in **no** feature specification: `PRD-002`, `PRD-003`, `PRD-005`,
`PRD-008`, `PRD-010`, `PRD-011`, `PRD-012`, `PRD-013`, `PRD-014`, `PRD-015`.

The requirements method requires the chain Requirement → Feature specification → … to
exist (`REQUIREMENTS-METHOD.md:90-96`), and the index marks every `PRD-*` as
`cross-cutting` precisely because it is delivered by several features — but no
specification says so.

**Suggested fix**: Add the `PRD-*` IDs to the `Requirements:` header and to the relevant
acceptance criteria of each delivering feature (F-16 must say how it delivers
`PRD-002`, `PRD-003`, `PRD-013`; F-19 how it delivers `PRD-001`, `PRD-005`, `PRD-011`,
`PRD-012`). A commitment table that no specification backs is a promise with no owner.

---

### F-06 — Major — STATUS.md carries three stale counts and contradicts itself in one table

**Location**: `docs/07-project/STATUS.md:19, 69, 70, 72, 76`

**What is wrong**: The status report is the document the README sends the owner to. Three
of its numbers no longer match the tree, and one table contains two rows with the same
name and different values.

**Evidence**:

> | Decisions | **Complete** | The ADRs in `../02-architecture/adr/` plus 17 entries in the decision log — `../00-governance/DECISION-LOG.md` |
> — `STATUS.md:19`

> | Glossary entries | 90 terms across five vocabularies |
> | Independent reviews | 2 (a beginner-experience review and a consistency review) |
> — `STATUS.md:69-70`

> | Acceptance criteria | 298 across the 21 specifications |
> …
> | Acceptance criteria | > 300 across the feature specifications |
> — `STATUS.md:69,72`

Two rows with the same label and different values, three lines apart, in the same table.
Neither is right: the tree contains **315** distinct acceptance criteria across the 19
baseline specifications (F-20 and F-21 define none), so "298 across the 21
specifications" is wrong on both the count and the scope, and "> 300" is true only by
coincidence. The first of the two rows was *added* by commit `019f673`
("docs: correct the requirement priority counts in the status report"), whose message
asserts that "the baseline holds … 298 acceptance criteria across the 21 specifications"
as a correction. It is a correction in the wrong direction: the row it replaced
("341 (MUST / SHOULD / COULD per the index totals)") was correct, and the number it
adds is the wrong one.

> | Architecture decisions | The ADRs in `../02-architecture/adr/` + 17 decision-log entries |
> — `STATUS.md:76`

Measured:
- `DECISION-LOG.md` contains **23** entries (`DEC-001` … `DEC-023`), not 17. The
  beginner review's M-16 corrected the ADR half of this sentence to a pointer but left
  the decision-log half; the three new decisions added on 2026-11-15 (DEC-021 … DEC-023)
  and the six added on 2026-11-14 (DEC-017 … DEC-020) were never counted.
- `GLOSSARY.md` contains **78** terms across five vocabularies (28 protocol, 14 product,
  14 platform, 11 security, 11 project), not 90.
- `docs/05-quality/reviews/` contains **1** review document
  (`BEGINNER-REVIEW-2026-11-14.md`), not 2.

**Suggested fix**: Update the three numbers, or replace them with pointers ("the entries
in `DECISION-LOG.md`", "the terms in `GLOSSARY.md`") the way the ADR count was fixed.
A status report whose numbers drift is the document the project uses to argue that it
does not drift.

---

### F-07 — Major — The decision log, the status report and the workplan record a review that has not happened

**Location**: `docs/00-governance/DECISION-LOG.md:57-62`; `docs/07-project/STATUS.md:70`;
`docs/07-project/REQUIREMENTS-WORKPLAN.md:73-79`

**What is wrong**: The three commits that landed during this review added a row that
asserts an independent consistency review (round 2) has been performed, dated
2026-11-15, with a pointer to a directory that does not contain it. At the revision
reviewed, the directory contains only the beginner review.

**Evidence**:

> ## Reviews and evidence
>
> | Review | Date | Result |
> | --- | --- | --- |
> | Independent beginner-experience review | 2026-11-14 | 6 blocker, 16 major, 11 minor, 4 info findings — **all addressed**; the report is kept at `../05-quality/reviews/BEGINNER-REVIEW-2026-11-14.md` |
> | Independent consistency review (round 2) | 2026-11-15 | See `../05-quality/reviews/` |
> — `DECISION-LOG.md:57-62`

> **Done, twice.** … A second independent consistency review follows up, with its report in the same directory.
> — `REQUIREMENTS-WORKPLAN.md:73-79`

```
$ ls docs/05-quality/reviews/
BEGINNER-REVIEW-2026-11-14.md
```

This is precisely the failure mode `QUALITY-GATES.md` §1 rule 2 exists to prevent, and
it was written while this review was in progress — the project asserted the *result*
before the *evidence*. It is also the one defect in this report that this report
itself resolves: as soon as this file is committed to `docs/05-quality/reviews/`, the
pointer becomes true. Until then it is a claim without evidence.

**Suggested fix**: Commit this report to `docs/05-quality/reviews/` and point the
decision-log row and the status row at the exact filename, with the finding counts
filled in — not at the directory. Do not write a review row before the report exists.

---

### F-08 — Major — The architecture module tree omits two modules that other documents require

**Location**: `docs/02-architecture/ARCHITECTURE.md:32-53`; `WHERE_TO_CHANGE.md:53-68`;
`docs/07-project/TRACEABILITY-MATRIX.md:284-292` and the `LIFE-*` rows

**What is wrong**: Three documents describe the module layout and no two agree.
`ARCHITECTURE.md` §2 lists `config/ upnp/ ssdp/ didl/ catalogue/ media/ http/ security/
platform/ logging/ error.rs`. `WHERE_TO_CHANGE.md`'s planned table adds `scheduler/` but
not `lifecycle/`. The generated traceability matrix maps `RES-001`…`RES-009` to
`crates/phairdlna-core/src/scheduler` and `LIFE-001`…`LIFE-011` to
`crates/phairdlna-core/src/lifecycle`.

**Evidence**:

> │   │       ├── platform/       platform trait + per-OS implementations
> │   │       ├── logging/        structured logging + redaction
> │   │       └── error.rs        the crate error type
> — `ARCHITECTURE.md:47-49`

```
$ grep -c 'scheduler' docs/02-architecture/ARCHITECTURE.md
0
$ grep -c 'lifecycle' WHERE_TO_CHANGE.md
0
$ grep -oE 'crates/phairdlna-core/src/[a-z]+' docs/07-project/TRACEABILITY-MATRIX.md | sort -u
crates/phairdlna-core/src/catalogue
crates/phairdlna-core/src/config
crates/phairdlna-core/src/didl
crates/phairdlna-core/src/http
crates/phairdlna-core/src/lifecycle
crates/phairdlna-core/src/media
crates/phairdlna-core/src/platform
crates/phairdlna-core/src/scheduler
crates/phairdlna-core/src/security
crates/phairdlna-core/src/upnp
```

`F-18` states that the scheduler "must exist before `RES-001` … `RES-009` can be
implemented; it is a core module, not an optimisation layer added later" — and the
normative architecture document does not list it. `DOC-003` requires
`WHERE_TO_CHANGE.md` to "list every source path a newcomer may need to change"; the
`src/lifecycle/` path is in neither navigation document.

**Suggested fix**: Generate the module list once and reference it from all three
documents, or at minimum add `scheduler/` and `lifecycle/` to `ARCHITECTURE.md` §2 and
`lifecycle/` to `WHERE_TO_CHANGE.md`.

---

### F-09 — Major — RESILIENCE-ACCEPTANCE-PLAN.md declares canonical requirement IDs to be local

**Location**: `docs/05-quality/RESILIENCE-ACCEPTANCE-PLAN.md:3-4`

**What is wrong**: The banner tells the reader that `RES-*`, `LIFE-*` and `MB-*` are not
canonical requirement IDs. They are: all 29 of them are defined in
`TECHNICAL-REQUIREMENTS.md` (§17, §19, §15), appear in the generated index and the
traceability matrix, and are owned by F-18 and F-15. The document's own local identifiers
are `RES-A01`, `LIFE-A01`, `MB-A01` — a different set that the banner does not mention.

**Evidence**:

> **Non-normative plan.** This is a proposed acceptance plan. The identifiers below
> (`RES-*`, `LIFE-*`, `MB-*`) are **local to this document** and are not canonical
> requirement IDs.
> — `RESILIENCE-ACCEPTANCE-PLAN.md:3-4`

> | `RES-001` | Arbitrate streaming, discovery, metadata reading … through one scheduler … | MUST | `F-18` | M1 | Resource-pressure test | Baseline |
> — `REQUIREMENTS-INDEX.md`

A reader who trusts the banner will conclude that `RES-001` does not exist as a
requirement and will not trace the plan to F-18. The document also carries no owner role
and no date (see F-18 below).

**Suggested fix**: Rewrite the banner to say that the *acceptance-case* identifiers
(`RES-A01`, `LIFE-A01`, `MB-A01`) are local, and that the `RES-*`, `LIFE-*` and `MB-*`
**requirements** are canonical and live in `TECHNICAL-REQUIREMENTS.md` §17, §19 and §15.

---

### F-10 — Major — The new OD-05 default (macOS 13+) contradicts the glossary, F-15, the risk register and the research

**Location**: `docs/07-project/OPEN-DECISIONS.md:26, 121-136`; `docs/01-product/GLOSSARY.md:70`;
`docs/03-features/F-15-macos-native-experience.md:25`; `docs/07-project/RISK-REGISTER.md:45`;
`docs/09-research/platform-integration.md` §11.1 D3

**What is wrong**: Commit `c6a0e81` changed OD-05's recommendation and default from
"macOS 15+" to "macOS 13 and later", and added it to the new "recommendations at a
glance" table. Three other documents still assume the macOS 15+ local-network privacy
permission exists, and two of them use it as the *justification* for a design decision.

**Evidence**:

> | OD-05 | Minimum macOS version | macOS 13 and later | macOS 13 |
> — `OPEN-DECISIONS.md:26`

> **Recommendation:** **macOS 13 and later**, with the local-network permission handled explicitly and the daemon model used throughout. This keeps one model instead of two.
> — `OPEN-DECISIONS.md:131-132`

> | **Local network privacy** | The macOS 15+ permission that governs UDP multicast traffic. A daemon is exempt, an agent is not — which is why the server runs as a daemon |
> — `GLOSSARY.md:70`

> 1. **macOS 15 introduced Local Network privacy** (platform-integration §1.2, Apple TN3179). …
> — `F-15:25`

> | Detection | The first macOS device test on macOS 15 or later |
> — `RISK-REGISTER.md:45` (R-03)

> | **D3** | Minimum macOS version | **macOS 15+** | Local Network privacy exists only on 15+ (VERIFIED). Supporting 13/14 means *not* knowing whether a prompt appeared → worse diagnostics. …
> — `docs/09-research/platform-integration.md` §11.1

`PLT-013` is the requirement that makes the server a launchd **daemon**, and its stated
reason is the local-network permission's daemon exemption — a macOS 15+ fact. Under the
new default that requirement's rationale does not hold on macOS 13/14.

**Suggested fix**: Either keep macOS 15+ as the default (which is what the research
recommends and what the glossary, F-15 and R-03 assume), or change the default **and**
update `GLOSSARY.md`, `F-15`, `RISK-REGISTER.md` R-03, `PLT-013`'s rationale and
`platform-integration.md` D3 in the same change. If macOS 13/14 is genuinely supported,
the local-network story needs its own requirement, because on 13/14 there is no prompt
to handle at all.

---

### F-11 — Minor — F-19's open-question register is stale and now contradicts the documents it describes

**Location**: `docs/03-features/F-19-quality-gates-and-developer-experience.md:487-494, 508-510`

**What is wrong**: §12 is the project's live register of unresolved questions. Items 1,
2 and 6 describe drift that the intervening commits already fixed, and they are written
in the present tense as if the drift still exists.

**Evidence**:

> 2. **[OPEN] Gate-ID drift between the governance documents.** `PROJECT-STANDARDS.md` §2 cites `scripts/check-file-size.sh` as "QG-08" and the complexity check as "QG-09", while `QUALITY-GATES.md` has file size as **QG-13** … `ARCHITECTURE.md` §2 calls the crate-boundary check "QG-07", while QG-07 is the doctest gate and the boundary check has no gate ID at all.
> — `F-19:492-494`

Today: `PROJECT-STANDARDS.md:36` says `scripts/check-file-size.sh` **(QG-13)**, `:38`
says **QG-14** for complexity, and `ARCHITECTURE.md:69` says "gate **QG-28**". The
drift is gone; the register still reports it, and still defers a decision that has
effectively been taken.

> 6. **[OPEN] The link gate's command.** The matrix names `lychee --offline docs/ README.md`, but the repository already contains `scripts/check-doc-links.sh` …
> — `F-19:508`

The matrix now names `scripts/check-doc-links.sh`.

**Suggested fix**: Close items 1, 2 and 6 with the commit that closed them (or delete
them), and add a "last verified against the tree" date to each remaining `[OPEN]` item —
which is what the beginner review's M-13 recommended and which was not done.

---

### F-12 — Minor — ROADMAP.md still points at the wrong gate ID for the crate boundary check

**Location**: `docs/07-project/ROADMAP.md:40`

**What is wrong**:

> | Rust workspace with the four crates, boundary check | `PLT-001`, QG-07-style gate |

QG-07 is **Doctests**. The crate-boundary gate is **QG-28** (`QUALITY-GATES.md:105`,
`ARCHITECTURE.md:69`, `ADR-0001:40`). "QG-07-style gate" means nothing in this codebase.
The beginner review's finding m-08 was fixed in `QUALITY-GATES.md` §4 (G1 now says
"gate QG-28") but not here.

**Suggested fix**: Replace with `QG-28`.

---

### F-13 — Minor — PERFORMANCE-BUDGETS.md omits two budgets and its header claims a range it does not cover

**Location**: `docs/05-quality/PERFORMANCE-BUDGETS.md:4, 45-78`

**What is wrong**: The header claims `PERF-001` … `PERF-014`. The budget tables contain
`PERF-001`, `002`, `003`, `004`, `005`, `006`, `007`, `008`, `010`, `011`, `012`, `013`,
`014`. **`PERF-009` and `PERF-015` have no budget row at all.**

**Evidence**:

> Requirements: `PERF-001` … `PERF-014`
> — `PERFORMANCE-BUDGETS.md:4`

> - The performance budgets in `PERF-001` … `PERF-015` and the measurement protocol in `../05-quality/PERFORMANCE-BUDGETS.md`.
> — `docs/03-features/F-18-performance-and-reliability.md:37`

F-18 §3 restates `PERF-015` ("A client shall discover the server within 10 seconds at
the 95th percentile over 30 trials") and F-18 §8 cites it as "a client-visible
property", but the document that owns the budgets has no target for it. `PERF-009`'s
missing budget is at least acknowledged as an `[OPEN]` item in `F-09:418`; `PERF-015`'s
absence is flagged nowhere.

**Suggested fix**: Add budget rows for `PERF-009` and `PERF-015`, or state in the header
which IDs the document covers and add an `[OPEN]` entry for each requirement it does not
cover.

---

### F-14 — Minor — Two competing "reference hardware" definitions, and the one in the budget document is an open owner decision

**Location**: `docs/05-quality/PERFORMANCE-BUDGETS.md:27-42`;
`docs/09-research/qa-performance.md` §5.3; `docs/07-project/OPEN-DECISIONS.md` OD-11

**What is wrong**:

> Because results only mean something relative to a machine, the project defines one
> reference profile. Any measurement on other hardware must state its profile.
> — `PERFORMANCE-BUDGETS.md:29-30`

| Property | Reference value |
| --- | --- |
| Platform | macOS on Apple Silicon (primary), macOS on Intel, Linux x86_64, Windows x86_64 |
| Memory | 16 GB |
| Storage | NVMe SSD for the index; the media library on a spinning disk or a NAS is a *separate* profile … |
| Library size | 100 000 objects unless stated otherwise |

The research brief states a second, different profile:

> **Hardware profile** — every performance report embeds this block verbatim:
> ```
> hardware:
>   model: MacBook Pro 14" (2023)
>   chip: Apple M2 Pro (10-core CPU, 16-core GPU)
>   os: macOS 15.6 (build 24G90)
>   network: loopback (127.0.0.1) unless the row says otherwise
> ```

And both `OPEN-DECISIONS.md` OD-11 ("which machine defines the release targets?") and
`F-18-Q-03` say the reference machine is still an open owner decision, while the budget
document presents a profile as settled.

**Suggested fix**: Say in `PERFORMANCE-BUDGETS.md` §2 that the *shape* of the profile is
defined and the *machine* is OD-11, and point at `qa-performance.md` §5.3 as the report
template rather than as a second profile.

---

### F-15 — Minor — BUILD-AND-TEST.md lists QG-12 and QG-14 in two contradictory tables

**Location**: `docs/06-development/BUILD-AND-TEST.md:65-86` and `:101-121`

**What is wrong**: §3 is headed "The gates you can run today" and lists `QG-12 spelling |
typos` and `QG-14 complexity | cargo clippy …` with no "planned" marker; §3.1 "The gates
that arrive with the implementation" lists QG-12 and QG-14 again. In the canonical
register both are state `planned`. `typos` is not installed by any documented step.

**Evidence**:

> ## 3. The gates you can run today
> … | QG-12 spelling | `typos` |
> — `BUILD-AND-TEST.md:65,80`

> ## 3.1 The gates that arrive with the implementation
> | QG-12 | Spelling | `typos` |
> — `BUILD-AND-TEST.md:101,106`

Other planned gates in §3 (QG-06, QG-16, QG-17, QG-18, QG-21) do carry a
"*(planned — …)*" marker; QG-12 and QG-14 do not.

**Suggested fix**: Add the "*(planned)*" marker to QG-12 and QG-14 in §3, or drop them
from §3 and rely on §3.1.

---

### F-16 — Minor — The gate register's own summary does not add up, and one repository script has no gate at all

**Location**: `docs/05-quality/QUALITY-GATES.md:107-119`; `WHERE_TO_CHANGE.md:70-80`

**What is wrong**:

> Four gates have a script of their own and were observed running on 2026-11-14:
> **QG-03** (no `unsafe`), **QG-11** (link check) and **QG-13** (file size), plus the
> formatting, lint, build and test gates that come with the Rust toolchain.
> — `QUALITY-GATES.md:117-119`

Three script gates are named, not four. The fourth script in the repository —
`scripts/generate_requirement_views.py` — has **no gate ID at all** in the register,
although it runs in CI (`ci.yml`, "Requirement views are current"), is the subject of
`AGENTS.md` ("Change the definition, then run `python3 scripts/generate_requirement_views.py`"),
and is what keeps the index and the traceability matrix from drifting. `WHERE_TO_CHANGE.md:77`
gives it a gate cell reading "CI step 'Requirement views are current'" instead of a
`QG-nn` ID.

**Suggested fix**: Either allocate a gate ID for the generator (and add the row), or
rewrite the sentence to name the three script gates and say that the generated-view
check is a CI step outside the `QG-nn` series — deliberately, and say why.

---

### F-17 — Minor — docs/README.md says every requirement is defined in one file; two files define them

**Location**: `docs/README.md:35-36`; `docs/01-product/TECHNICAL-REQUIREMENTS.md:9`

**What is wrong**:

> 1. **[01-product/TECHNICAL-REQUIREMENTS.md](../../01-product/TECHNICAL-REQUIREMENTS.md)** — the
>    normative catalogue. Every requirement is defined here exactly once.
> — `docs/README.md:35-36`

> Product-level commitments (`PRD-*`) are defined in `PRODUCT-REQUIREMENTS.md`.
> — `TECHNICAL-REQUIREMENTS.md:9`

`PRD-001` … `PRD-015` (and the 17 user stories) are defined in `PRODUCT-REQUIREMENTS.md`,
not in `TECHNICAL-REQUIREMENTS.md`. `docs/README.md` is the map of the tree, so a reader
who trusts it will look for the product commitments in the wrong file.

**Suggested fix**: "Every requirement is defined exactly once — in
`TECHNICAL-REQUIREMENTS.md` (technical domains) or `PRODUCT-REQUIREMENTS.md` (product
commitments)", matching the generated index's own header.

---

### F-18 — Minor — The `qa-performance` research brief carries its own rival QG-nn matrix

**Location**: `docs/09-research/qa-performance.md` §7 "Proposed gate matrix" (lines ~660-720)

**What is wrong**: The brief assigns the canonical `QG-nn` identifiers to a different
set of checks: `QG-02` is `cargo clippy --workspace --all-targets --all-features -- -D
warnings` (the register has no `--all-features`), `QG-11` is
`lychee --no-progress --max-retries 2 '**/*.md'` (the register has
`scripts/check-doc-links.sh`), `QG-13` is `tokei --output json + xtask size-check`.

**Evidence**:

> **This table is the single register of the gate identifiers.** … A document that names a gate for a different check is a defect — fix the document, not this table.
> — `QUALITY-GATES.md:109-111`

Mitigating: the brief is explicitly "evidence, not norm" (DEC-022, `docs/09-research/README.md` §2),
and it is the matrix the project's own matrix "grew out of"
(`docs/09-research/README.md:21`). But the identifiers collide with the canonical
series, and the brief contains an illustrative `QG-05` record with `"status": "PASS"`
and `"executed": 812` that a skimming reader could mistake for an observed result.

**Suggested fix**: Renumber the brief's matrix to a distinct prefix (`RQ-nn`) or add a
one-line note at its head saying that its `QG-nn` column refers to the brief's own
draft numbering, not the canonical register. While there, mark the JSON block as an
example.

---

### F-19 — Minor — REQUIREMENTS-WORKPLAN.md counts are wrong

**Location**: `docs/07-project/REQUIREMENTS-WORKPLAN.md:32, 54-56, 73-79`

**What is wrong**: Two counts in the freshly rewritten workplan do not match the tree.

> `standards §8.3 C12` resolves into those files; all 324 section citations were checked mechanically.
> — `REQUIREMENTS-WORKPLAN.md:32`

Measured on the current tree: **343** citation groups (`<brief> §N`, …) and **430** `§`
section references to the research briefs, across **88** distinct sections; at the
previous commit the figure was already 340. Under no counting method does 324 match.

> **Done.** 341 requirements with stable IDs, defined once in `docs/01-product/TECHNICAL-REQUIREMENTS.md` and `docs/01-product/PRODUCT-REQUIREMENTS.md`, and elaborated by 21 feature specifications with 298 acceptance criteria.
> — `REQUIREMENTS-WORKPLAN.md:54-56`

There are **315** distinct acceptance criteria across the 19 baseline specifications
(`F-20` and `F-21` define none). `STATUS.md:72`'s "> 300" is true by coincidence;
`STATUS.md:69`'s "298 across the 21 specifications" is wrong on both the count and the
scope.

The workplan also says "Ten ADRs … 23 decision-log entries" (`R4`) — that one is
correct, and is the number `STATUS.md` should be using.

**Suggested fix**: Recompute both numbers, or state the counting method next to the
number so the next reader can reproduce it.

---

### F-20 — Minor — Twenty documents in `docs/` lack the status, owner role and date that DOC-008 makes a MUST

**Location**: `docs/01-product/TECHNICAL-REQUIREMENTS.md:460` (DOC-008)

**What is wrong**: DOC-008 requires that "Every document shall carry a status, an owner
role, a version and a date", and `DOCUMENTATION-POLICY.md` §2 repeats the rule. Twenty
documents in `docs/` (excluding the immutable archive) do not:

```
docs/00-governance/EXCEPTIONS.md                              status=0 owner=0 date=0
docs/01-product/COMPETITIVE-LANDSCAPE.md                      status=1 owner=0 date=1
docs/01-product/DIFFERENTIATION-STRATEGY.md                   status=1 owner=0 date=0
docs/01-product/SIMPLE-ADVANCED-EXPERIENCE.md                 status=1 owner=0 date=0
docs/01-product/USABILITY-REQUIREMENTS.md                     status=1 owner=0 date=0
docs/02-architecture/adr/ADR_TEMPLATE.md                      status=1 owner=1 date=0
docs/03-features/FEATURE-TEMPLATE.md                          status=1 owner=1 date=0
docs/05-quality/COMPETITIVE-BENCHMARK-PLAN.md                 status=1 owner=0 date=0
docs/05-quality/RESILIENCE-ACCEPTANCE-PLAN.md                 status=1 owner=0 date=0
docs/07-project/RELEASE-BLOCKERS.md                           status=0 owner=0 date=0
docs/09-research/clients-a-samsung-lg-philips-panasonic.md    status=0 owner=0 date=0
docs/09-research/clients-b-sony-xbox-playstation.md           status=0 owner=0 date=0
docs/09-research/clients-d-foobar2000.md                      status=0 owner=0 date=12
docs/09-research/clients-d-windows-media-center.md            status=0 owner=0 date=1
docs/09-research/ecosystem.md                                 status=0 owner=0 date=0
docs/09-research/platform-integration.md                      status=1 owner=0 date=19
docs/09-research/qa-performance.md                            status=1 owner=0 date=20
docs/09-research/security-landscape.md                        status=1 owner=0 date=5
docs/09-research/standards.md                                 status=1 owner=0 date=4
docs/README.md                                                status=0 owner=0 date=0
```

The root-level `README.md`, `AGENTS.md`, `CONTRIBUTING.md`, `SECURITY.md` and
`WHERE_TO_CHANGE.md` likewise carry no status, owner role or date.

**Suggested fix**: Add the three fields to the documents that are normative or
owner-facing (`EXCEPTIONS.md`, `RELEASE-BLOCKERS.md`, `docs/README.md`,
`RESILIENCE-ACCEPTANCE-PLAN.md`, the four non-normative strategy documents, and the two
templates). For the research briefs, one line in `docs/09-research/README.md` saying
that briefs are immutable evidence and therefore carry no document header would close
the requirement honestly.

---

### F-21 — Minor — F-20 and F-21 are feature specifications without the label legend, the source line, the ten sections or an index row

**Location**: `docs/03-features/F-20-compatibility-inspector.md:1-6`;
`docs/03-features/F-21-playback-doctor.md:1-6`; `docs/03-features/FEATURE-TEMPLATE.md`

**What is wrong**: All 19 baseline specifications carry a `Claim tags:` legend and a
`Research sources:` line; F-20 and F-21 carry neither. They also carry none of the ten
mandatory sections that `DOC-001` requires of "every feature", and they appear in no
row of the requirement index, although `REQUIREMENTS-METHOD.md` §7 says a deferred
feature "is marked as Optional or Deferred in its specification **and in the index**".

**Evidence**:

> Status: **Proposed — deferred beyond the v1.0 baseline**, awaiting owner approval.
> Owner role: Product Owner
> Version/date: 0.1 / 2026-11-14
> Baseline: not part of the approved requirement baseline; it carries no canonical requirement IDs and blocks no milestone.
> — `F-20:3-6`

Their local identifier schemes (`COMP-*`, `PDOC-*`) are properly declared non-canonical,
which satisfies DEC-023 — that part is right.

**Suggested fix**: Either add the legend/source line and the ten sections (a proposal
can have them) or state in the catalogue that F-20/F-21 are sketches, not specifications,
and add a `Deferred` row for each to the index so §7 is satisfied.

---

### F-22 — Minor — "Committed seed corpora" is stated in the present tense in the gate register and the test strategy

**Location**: `docs/05-quality/QUALITY-GATES.md:50, 96`; `docs/05-quality/TEST-STRATEGY.md:19`

**What is wrong**:

> | `cargo-fuzz` | 0.13.x | Fuzzing | Requires a **pinned nightly toolchain**; 6 targets with committed seed corpora |
> — `QUALITY-GATES.md:50`

> | QG-19 | Fuzz smoke | `cargo fuzz run <target> -- -runs=N` | blocker | M4 | planned | Each target has a committed seed corpus |
> — `QUALITY-GATES.md:96`

> | **Fuzz** | Parser robustness | `cargo-fuzz`, nightly toolchain, committed seed corpora | Continuous |
> — `TEST-STRATEGY.md:19`

There is no `fuzz/` directory and no corpus in the repository. `BUILD-AND-TEST.md` §5
was corrected by the earlier fix ("Six targets are **planned** … Their seed corpora
arrive with them — there is no `fuzz/` directory in the repository yet"), but the
register and the test strategy were not.

**Suggested fix**: Use the planned tense in both places.

---

### F-23 — Minor — The new "recommendations at a glance" table has unreadable cells and one typo

**Location**: `docs/07-project/OPEN-DECISIONS.md:22-33, 165`

**What is wrong**: The "If you do not decide" column reads "That" for five rows
(`OD-06`, `OD-07`, `OD-09`, `OD-10`, `OD-11`), "Accepted" for `OD-12` and "Yes, minimal"
for `OD-08` — a column that carries no information where it matters most. The same file
also contains a typo introduced by the same commit:

> **Default, if no decision is make:** the ten priority families already listed in the interoperability matrix, with at least three independently implemented clients before the first stable release.
> — `OPEN-DECISIONS.md:165`

**Suggested fix**: Replace "That" with the actual default value in each cell, and fix
"is make" → "is made".

---

### F-24 — Minor — One archive manifest row is malformed

**Location**: `docs/99-archive/2026-11-14-agent-concept-drafts/MANIFEST.md`

**What is wrong**: The `PERFORMANCE-RELIABILITY-SLO.md` row and the
`ADAPTIVE-RESOURCE-MANAGEMENT.md` row are joined by `||` on a single physical line, so
the Markdown table renders with the wrong number of cells. The digests themselves are
correct (see "Checks that passed").

**Suggested fix**: Split the line.

---

### F-25 — Info — Two requirements are owned by a specification that never mentions them

**Location**: `docs/01-product/REQUIREMENTS-INDEX.md` (`LIFE-010`, `PERF-014`)

**What is wrong**: The index assigns `LIFE-010` to `F-13` and `PERF-014` to `F-18`.
Neither specification mentions the ID. `LIFE-010` is in fact elaborated in
`F-15` (`F-15:152` and `F-15:318`), and `PERF-014` ("the server shall survive a scan of
a root that contains a broken or slow network share without blocking browse requests")
appears in no specification at all and has no acceptance criterion.

**Suggested fix**: Re-allocate `LIFE-010` to F-15, and give `PERF-014` an owner, an
acceptance criterion and a budget row (see F-13).

---

### F-26 — Info — PERF-004 is claimed by three specifications; the index assigns it to one

**Location**: `docs/03-features/F-09-catalogue-index.md:6`, `F-10-metadata-and-artwork.md:6`,
`F-18-performance-and-reliability.md:6`

**What is wrong**: `F-09`'s header claims `PERF-003, PERF-004, …`; `F-10`'s claims
`PERF-004`; `F-18`'s claims "PERF-001 … PERF-015". The index assigns `PERF-004` to
`F-09` only. The generator's `FEATURES` table is the tie-breaker, so the index is right
and the two specification headers are not.

**Suggested fix**: Narrow F-18's header range to the IDs it owns
(`PERF-001`, `PERF-002`, `PERF-006`, `PERF-007`, `PERF-010`, `PERF-011`, `PERF-013` … `PERF-015`),
or narrow it to a range that matches the generator.

---

### F-27 — Info — The research README describes a label vocabulary the briefs do not use

**Location**: `docs/09-research/README.md:28-31`

**What is wrong**:

> They are **research**: what the sources say, with the URL that was actually fetched, and a label of `VERIFIED`, `OBSERVED`, `ASSUMPTION` or `OPEN` on every claim.

In practice `standards.md` uses `NORMATIVE` (72 occurrences) and `VERIFIED` once;
`ecosystem.md` uses `OBSERVED`/`ASSUMPTION` and never `VERIFIED`;
`platform-integration.md`, `security-landscape.md` and `qa-performance.md` use
`VERIFIED` and never `OBSERVED`. The reader is told there is one label set; there are
three.

**Suggested fix**: Say that each brief declares its own tags in its own "how to read this
document" section, and list the tags per brief.

---

### F-28 — Info — Fixed during this review (defect that existed at `fc07b2c` only)

**Location**: `docs/01-product/GLOSSARY.md:124`

At `fc07b2c` the glossary's last row linked to a non-existent directory:

> | Where did a rule come from? | [../09-research/](../../09-research/) |

The link target was relative to `docs/01-product/`, so it resolved to
`docs/01-product/09-research/`. The link gate was therefore **red** on the tree I was
asked to review, and CI (`ci.yml`, "QG-11 documentation links") would have failed:

```
$ bash scripts/check-doc-links.sh
FAIL - broken relative links:
  docs/01-product/GLOSSARY.md: 09-research/

RESULT: FAIL (1 broken of 123 checked)
exit=1
```

while `STATUS.md:46` recorded `RESULT: PASS (118 relative links checked)` and
`QUALITY-GATES.md:88` marked QG-11 "**Implemented and running**". Commit `746b5a9`
fixed the link. The recorded evidence in `STATUS.md` was not updated with it (the gate
now checks 123 links, not 118) — see F-06.

---

### F-29 — Major — The in-progress generator fix reuses feature ID `F-20` and still dates the `PLT-*` requirements to M0

*(F-29 and F-30 are appended after the Info findings because they describe the
uncommitted working-tree change, not the committed revision `c6a0e81`.)*

**Location**: `scripts/generate_requirement_views.py` (uncommitted working-tree change),
`docs/01-product/REQUIREMENTS-INDEX.md` (uncommitted), `docs/01-product/FEATURE-CATALOG.md:35`

**What is wrong**: The uncommitted change in the working tree fixes the allocation of F-01
(it removes all 56 `—` rows, which is exactly right) but introduces a new identifier
collision and leaves the milestone defect in place.

**Evidence** — the new allocation entry:

> +    ("F-20", "platform-portability", ["PLT-001..PLT-003", "PLT-005..PLT-018"]),

`F-20` is already taken:

> | Proposed | F-20 Compatibility Inspector (proposed, deferred) | `F-20-compatibility-inspector.md` | — | COULD |
> — `docs/01-product/FEATURE-CATALOG.md:35`

and `docs/03-features/F-20-compatibility-inspector.md` exists. The generator's
`FEATURE_MILESTONES` table has no `F-20` key, so `FEATURE_MILESTONES.get("F-20", "M0")`
stamps every one of the 17 `PLT-*` requirements with **M0**:

> | `PLT-001` | Not depend on any OS-specific crate and shall compile on macOS | MUST | `F-20` | M0 | Build matrix | Baseline |
> | `PLT-016` | Provide native packages and shall not use a filesystem-sandboxed package format as its primary… | MUST | `F-20` | M0 | Build matrix | Baseline |

The roadmap puts `PLT-002`, `PLT-003`, `PLT-015`, `PLT-016` in **M3**, `PLT-005` and
`PLT-011` in **M1**, `PLT-013` in **M2** and `PLT-014` in **M2** — so the milestone
contradiction from F-01 survives the fix, and the index now names a feature that
`FEATURE-CATALOG.md` describes as a *deferred proposal* with no milestone at all. It
also breaks the rule in `AGENTS.md`: "Never … reuse an ID for a different meaning."

**Suggested fix**: Use a free feature ID (F-22) or, better, allocate the `PLT-*`
requirements to the features that already claim them in their `Requirements:` headers
(F-08 claims `PLT-003`, `PLT-005`, `PLT-006`, `PLT-007`, `PLT-011`, `PLT-012`, `PLT-015`,
`PLT-017`; F-15 claims `PLT-004`; F-13 and F-19 also reference them). Then add the
missing `F-nn` keys to `FEATURE_MILESTONES` so the milestone column follows the
roadmap.

---

### F-30 — Minor — The same change adds a duplicate `PRD` key to the generator's `module_area` dictionary

**Location**: `scripts/generate_requirement_views.py` (uncommitted working-tree change)

**What is wrong**: The dictionary literal already contains `"PRD": "—"` in its first
entry; the change appends a second `"PRD"` key at the end of the same literal:

> +        "PRD": "cross-cutting (see docs/01-product/FEATURE-CATALOG.md)",

Python keeps the last value and silently discards the first, so the script still
works — but a duplicate key in a dict literal is exactly the kind of quiet defect a
generator exists to prevent, and a reader of the script will not notice which one
survives.

**Suggested fix**: Replace the existing `"PRD": "—"` entry rather than appending a
second one.

---

## Checks that passed

All commands were run in the repository root, against the committed revision `c6a0e81`
and the working tree as it stood when I finished (the tree was moving under me; the
link-check count of 123 is the count at that moment).

**The four repository scripts exist, are executable, and behave as documented.**

```
$ bash scripts/check-file-size.sh
RESULT: PASS
exit=0

$ bash scripts/check-doc-links.sh
RESULT: PASS (123 relative links checked)
exit=0

$ bash scripts/check-no-unsafe.sh
RESULT: PASS (no unsafe code in 2 tracked Rust source file(s))
exit=0

$ python3 scripts/generate_requirement_views.py --check
up to date: .../docs/01-product/REQUIREMENTS-INDEX.md
up to date: .../docs/07-project/TRACEABILITY-MATRIX.md
341 requirements indexed
exit=0
```

The size gate correctly scans only Rust sources (`crates/**/*.rs`), the unsafe gate
strips comments before matching, and the link gate resolves relative links only and
skips `docs/99-archive/` — all as their header comments state.

**Requirement integrity.** Every ID in `REQUIREMENTS-INDEX.md` is defined exactly once
across `TECHNICAL-REQUIREMENTS.md` and `PRODUCT-REQUIREMENTS.md`; no ID appears in both
files; no duplicate definitions; the index has 341 rows, matching the script's "341
requirements indexed"; every row has a priority, a milestone and a verification method.
The only failures are the *feature* column (F-01) and the 17 PRD user stories that
never reach the index at all (F-03).

**Research citations resolve.** All 343 citation groups to the research briefs
(`standards §…`, `platform-integration §…`, `security-landscape §…`, `qa-performance §…`)
resolve to a section heading that exists in the named file, and the labelled
sub-references (`§8.1 D6`, `§8.3 C12`, `§11.1 D14`) resolve to the labelled row inside
the cited section. A twelve-citation sample was checked for *support*, not just
existence, and all twelve hold:

| Citation | Claim | Verdict |
| --- | --- | --- |
| `standards §2.4, §8.1 D6` (F-01) | the seven required `M-SEARCH` response headers | supported — D6 lists exactly `CACHE-CONTROL, DATE, EXT, LOCATION, SERVER, ST, USN` |
| `standards §4.3` (F-11) | `object.container.playlistContainer` exists | supported — the CD:4 class table includes it |
| `standards §6.5` (F-05) | the `sec:` namespace and the two names that do not exist | supported — §6.5 is titled exactly that |
| `standards §8.3 C12` (DEC-022) | the ContentDirectory obligation list | supported — C12 is the "do not implement these actions" row |
| `platform-integration §5.1` (F-08) | APFS preserves the on-disk form but is normalisation-insensitive | supported |
| `platform-integration §11.1 D14` (F-08) | the consent model is a platform fact behind a `Consent` trait | supported — D14 is the consent-model row |
| `security-landscape §4.4` (F-08) | canonicalise-then-contain, never textual `..` collapsing | supported |
| `security-landscape §5.6` (F-16) | the GENA callback SSRF / "CallStranger" class | supported |
| `security-landscape §6.2` (F-14) | constant-time token comparison, `Host` validation | supported |
| `qa-performance §5.2` (F-04) | `criterion` for micro-benchmarks, median + p95 | supported |
| `qa-performance §5.3` (F-04) | the reference hardware block and the measurement protocol | supported — §5.3 does contain a hardware profile |
| `qa-performance §7.3` (F-19) | no maintained Rust tool enforces a maximum file size | supported |

**The archive is intact.** All 29 rows of the two archive manifests were re-hashed with
SHA-256: **29 / 29 digests match the archived files**. The 19 files of the
pre-consolidation snapshot are also byte-identical to their originals at commit
`7bce620`, as the manifest claims. The archive is therefore genuinely immutable, and
DOC-007's "manifest and a digest" is satisfied.

**Security coverage is complete.** All eight mitigations `M-01` … `M-08` have at least
one requirement behind them and one planned negative test; all 28 threat rows
`SEC-TH-01` … `SEC-TH-28` name a mitigation *and* the requirement(s) that implement it
(`F-16` §4 and §10). The threat count in `SECURITY-MODEL.md` (28) and in `STATUS.md` (28)
agree.

**The counts that do agree.** Protocol conformance checks `PC-01` … `PC-26` = 26 in both
`INTEROPERABILITY-MATRIX.md` and `STATUS.md`; risks `R-01` … `R-13` = 13; open
decisions `OD-01` … `OD-12` = 12; milestone gates `G0` … `G6` = 7; quality gates
`QG-01` … `QG-28` = 28, of which 11 carry state `exists`; the glossary has five
vocabularies as claimed.

**Every baseline feature specification carries the label legend and the source line**
(all 19 of F-01 … F-19), and every baseline specification has its `Requirements:`
header — but, as F-01 and F-26 record, several of those headers disagree with the
index.

**The gate register is internally consistent in shape.** Every row has exactly one
`State` (`exists` or `planned`), no gate ID is used for two different checks *within*
`QUALITY-GATES.md`, and `DEC-021` is honoured by `PROJECT-STANDARDS.md` (QG-13, QG-14),
`ARCHITECTURE.md` (QG-28) and `ADR-0001` (QG-28). The deviations are all in other
documents: F-19 §4.1 (F-02), `ROADMAP.md:40` (F-12), `qa-performance.md` §7 (F-18), and
the `ci.yml`/`CI-POLICY.md` mismatch (F-04).

**`FEATURE-CATALOG.md` points at files that all exist** — all 21 specification
filenames in its table resolve — and the milestone and priority columns agree with the
generator's `FEATURE_MILESTONES` table.

**Honesty in the places that matter most.** `SECURITY.md` now presents its hardening
list as "**requirements that the implementation must satisfy**" with a
"specified, not implemented" state per row. `PRIVACY.md`'s storage table and
`INTEROPERABILITY-MATRIX.md`'s device table are honest: the device table has one row
reading `NOT TESTED` and says "*The table is empty on purpose: no device test has been
performed yet*". The research briefs state "Nothing here was tested locally" and
"Nothing here was verified on hardware". `RESILIENCE-ACCEPTANCE-PLAN.md` says "**All
results NOT RUN.**" and "No design intent counts as PASS". `PERFORMANCE-BUDGETS.md`
says "**Nothing in this document is a measurement.**" and every budget row's
"Measured on" column is `—`. No document claims a device test, a benchmark result, a
certification or a security assurance that was not observed. The two exceptions found
are in the *status* layer (F-06, F-07), not in the technical documents.

---

## What I could not verify

- **No Rust toolchain.** `cargo` is not on `PATH` in this sandbox, so I could not run
  `cargo fmt`, `cargo clippy`, `cargo test`, `cargo nextest`, `cargo deny`, `cargo audit`,
  `cargo insta`, `cargo fuzz`, `cargo bench` or `cargo xtask`. The four repository
  scripts do not need it and I ran them all. Consequently the `STATUS.md` §2 records
  (`cargo fmt` clean, `cargo clippy` exit 0, `cargo test` 1 passed / 0 failed) are
  unverified by me — the beginner review confirmed them at `17fef39`, but the workspace
  has changed since.
- **Nothing was measured and nothing was device-tested.** Every performance number, every
  `PC-nn` conformance check, every `SEC-TH-nn` negative test and every device-matrix row
  in this repository is a plan. I verified that the *documents* say so; I could not
  verify any of the underlying behaviour, because there is none to verify.
- **CI was never triggered.** Both workflows are `workflow_dispatch`-only by policy, and
  no run exists that I can see. My statement that "a `ci.yml` run would fail on QG-11"
  at `fc07b2c` is derived from running the same script locally, not from a CI run.
- **The research briefs' factual claims about UPnP, Apple, Windows and Linux are not
  independently verified.** I verified that the citations *resolve* and that the cited
  sections *support* the claims made of them, which is a consistency question. I did not
  re-fetch the upstream specifications, and I am not in a position to confirm that
  `standards.md`'s quotes of UDA 1.1 are faithful.
- **The archive immutability check covers only the 19 files of the pre-consolidation
  snapshot.** The ten files of the concept-drafts snapshot did not exist at commit
  `7bce620`, so their digests could only be checked against the manifest itself (they
  match); their content could not be compared to an original.
- **I did not review the `docs/99-archive/` documents for internal consistency.** They
  are immutable historical snapshots; I read them only to verify the manifests.
- **The tree moved during the review**, twice, and was being edited in the working tree
  when I finished. Any finding that is fixed by a commit landing after `c6a0e81` is
  stale on arrival — F-07 in particular resolves itself the moment this report is
  committed to `docs/05-quality/reviews/`, and F-01 is largely addressed by the
  uncommitted generator change (with F-29 and F-30 outstanding). I have deliberately not
  re-based my findings on that in-flight change, because it is not a reviewable revision.
