# Documentation governance

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Lead / Documentation reviewer

## 1. Canonical structure

```
docs/
  00-governance/   standards, ADR policy, exceptions, agent roles, decision log, requirements method
  01-product/      vision, product requirements, technical requirements, user journeys, glossary, requirement index
  02-architecture/ component boundaries, data flows, protocols, persistence, adr/
  03-features/     one specification per feature (+ the feature template)
  04-security/     threat model, privacy model, dependency and licence review
  05-quality/      test strategy, interoperability matrix, performance budgets, quality gates
  06-development/  setup, build and test, troubleshooting, WHERE_TO_CHANGE
  07-project/      requirements work plan, roadmap, traceability, risks, status, CI policy, blockers
  08-releases/     release policy, distribution, provenance, changelog
  09-research/     the research briefs the requirements cite (evidence, not normative)
  99-archive/      immutable snapshots of superseded baselines
```

Directory numbers are fixed. A document moves only through a recorded migration with a
mapping table and updated links (see § 4).

### 1.1 What belongs where

| Question | Directory |
| --- | --- |
| What must the product do, and for whom? | `01-product/` |
| How is it built, in which modules, with which data flows? | `02-architecture/` |
| What exactly does one feature do, with acceptance criteria? | `03-features/` |
| How is it kept safe and private? | `04-security/` |
| How do we know it works? | `05-quality/` |
| How do I build and run it? | `06-development/` |
| When will it be done, and what is left? | `07-project/` |
| How is it shipped and trusted? | `08-releases/` |
| What is the evidence behind a requirement? | `09-research/` |
| What did we decide earlier, and why? | `99-archive/` |

**A single source of truth.** If two documents state the same normative fact, one of them
is wrong. Link instead of copying. Requirements live in `01-product/REQUIREMENTS-INDEX.md`
and are *expanded*, never re-stated, in feature specifications.

---

## 2. Document rules

Every document carries a header with:

- **Status** — `Draft` → `In Review` → `Approved` → `Superseded`
- **Owner role** — the role accountable for the content (never a person's name)
- **Version / date** — semantic version for documents, ISO date for snapshots
- **Purpose** — one sentence; what this document is for and what it is *not* for
- **References** — links to the governing documents and to the requirement IDs it covers
- **Revision history** — for documents that change over time

Additional binding rules:

- Requirements have **stable IDs** and objectively verifiable acceptance criteria. An ID
  is never reused for a different meaning; a changed requirement keeps its ID and records
  what changed.
- **No silent deletion of approved requirements.** Supersede with traceability and
  preserve the snapshot in `99-archive/`.
- Git history is *not* a substitute for an explicitly archived baseline.
- No duplicate or conflicting normative documents. Where a legacy document exists, one of
  the two must state that it is superseded and link to the other.
- Every document must be readable by a programming beginner who has never seen the
  project: expand acronyms, define domain terms on first use, and give a concrete
  example where an abstract statement would be ambiguous.

---

## 3. Beginner entry path

```
README.md  →  docs/06-development/GETTING-STARTED.md  →  WHERE_TO_CHANGE.md  →  feature spec  →  source / test paths
```

This path must stay intact. Every document is reachable from it within two clicks, and
every feature specification names the source paths it owns.

---

## 4. Migration rule

Migrating an existing document requires:

1. A mapping table (old path → new path) recorded in the pull request and in the
   migration note of the new document.
2. All inbound links updated in the same change.
3. The pre-migration content preserved byte-identical in `99-archive/<date>-<slug>/`
   together with a `MANIFEST.md` listing the archived path, the original path and the
   SHA-256 digest of the original.
4. A `git mv` so that history follows the file.

The consolidated baseline of 2026-11-14 was migrated this way; see
`../99-archive/2026-11-14-preconsolidation-baseline/MANIFEST.md`.

**The `09-research` section** holds the research briefs that the feature
specifications cite (`standards §8.3`, `security-landscape §5.1`, …). They are
committed so that every `[SPEC]`, `[OBSERVED]` and `[DESIGN]` label can be checked
against its origin. They are **evidence, not norm**: where a research document and a
requirement disagree, the requirement wins.

---

## 5. Archiving

`99-archive/` is **immutable**. Archived documents are never edited, moved or deleted.
A new snapshot is a new dated directory. The archive README explains what each snapshot
contains and why it was superseded.

---

## 6. Review

A documentation change is reviewed for:

- **Structure** — correct directory, correct header, no orphan document.
- **Links** — every relative link resolves (checked by the link-check gate).
- **Consistency** — no contradiction with a requirement, ADR or decision log entry.
- **Beginner readability** — no unexplained jargon, no missing context.
- **Honesty** — no performance, compatibility or security claim without evidence.
