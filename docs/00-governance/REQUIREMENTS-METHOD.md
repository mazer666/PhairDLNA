# Requirements method

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Product Owner / Requirements

This document defines how requirements are written, numbered, prioritised, traced and
changed in PhairDLNA. It is the rulebook behind `../01-product/REQUIREMENTS-INDEX.md`
and `../03-features/`.

---

## 1. Requirement identifiers

Every requirement has exactly one stable ID. The prefix names the domain:

| Prefix | Domain | Example |
| --- | --- | --- |
| `PRD` | Product principles and non-negotiables | `PRD-004` |
| `FUN` | Functional behaviour of the product | `FUN-031` |
| `INT` | Interoperability: DLNA/UPnP protocols and clients | `INT-034` |
| `SEC` | Security and privacy | `SEC-021` |
| `PERF` | Performance and resource budgets | `PERF-012` |
| `UX` | Usability, accessibility, onboarding | `UX-006` |
| `OPS` | Operations: CLI, service, administration | `OPS-014` |
| `CFG` | Configuration surface | `CFG-007` |
| `PLT` | Platform and portability | `PLT-009` |
| `DOC` | Documentation and developer experience | `DOC-005` |
| `SUP` | Supply chain, licensing, provenance | `SUP-004` |
| `REL` | Reliability under failure | `REL-001` |
| `RES` | Adaptive resource management | `RES-001` |
| `LIFE` | Service lifecycle and network recovery | `LIFE-001` |
| `MB` | The macOS menu bar application | `MB-001` |

**Rules**

1. An ID is never reused for a different requirement, and never recycled after deletion.
2. A requirement that changes keeps its ID; the change is recorded in the revision
   history of the index and in the feature specification.
3. A requirement that is withdrawn becomes `Withdrawn` with a reason and a pointer to
   its successor — it is never silently deleted.
4. Numbers are assigned per prefix, in blocks of ten, so that related requirements stay
   adjacent and insertion does not renumber the tree.

---

## 2. Requirement text

A requirement is one normative sentence in the form:

> The system **shall** …

with the actor, the behaviour and the observable result. Avoid "should", "may" and
"nice to have" inside a requirement — the priority field carries that information.

Requirements describe **observable behaviour**, not implementation. "The browse response
for a container with 500 children contains at most 50 items" is a requirement;
"the implementation uses an iterator" is not.

---

## 3. Priority

| Priority | Meaning | Effect on the roadmap |
| --- | --- | --- |
| **MUST** | Required for the product to be usable and correct in the stated scope | Blocks the milestone that claims the feature |
| **SHOULD** | Strongly desirable; a release may be delayed by it | Does not block, but is tracked |
| **COULD** | Optional enhancement | Explicitly outside the current baseline |

A milestone cannot be accepted while a `MUST` requirement of that milestone is open.

---

## 4. Status

`Draft` → `In Review` → `Approved` → `Scheduled` → `Implemented` → `Verified` →
`Superseded` / `Withdrawn`

The generated index uses the value **`Baseline`** for every requirement of the current
baseline. `Baseline` means *Approved, minus the owner's sign-off on the baseline as a
whole*: the requirement is defined, internally consistent and traceable, but the
owner has not yet approved the set.

"Verified" requires recorded evidence: a named test with observed output, a recorded
benchmark, or a device-test record. A requirement that is implemented but not yet
verified is `Implemented`, never `Verified`.

---

## 5. Traceability chain

```
Requirement (PRD/FUN/INT/SEC/PERF/UX/OPS/CFG/PLT/DOC/SUP-nnn)
   → Feature specification (docs/03-features/F-nn-*.md)
   → Module / source path (WHERE_TO_CHANGE.md)
   → Milestone (M0…M5)
   → Verification method (automated test / gate / manual protocol / device test)
   → Evidence record (test name, gate output, device matrix row)
```

The chain is **populated as each milestone lands**: today the *evidence* column is
empty by design, because nothing is implemented. An empty evidence cell means *not
implemented*, never *passed*.

The chain is materialised in `../07-project/TRACEABILITY-MATRIX.md`. A requirement without
a feature, a milestone and a verification method is **not a requirement yet** — it is a
wish, and it stays `Draft`.

---

## 6. Feature specifications

One specification per feature, in `../03-features/`, named `F-nn-slug.md`, following
`FEATURE-TEMPLATE.md`. Every specification contains all ten mandatory sections:

1. Purpose and user story
2. Functional requirements
3. Technical requirements
4. Performance requirements
5. Usability and accessibility
6. Security and privacy requirements
7. Interoperability requirements
8. Error and recovery behaviour
9. Measurable acceptance criteria
10. Test strategy and dependencies

A specification additionally states:

- **Scope**: what is explicitly *not* part of the feature.
- **Priority split**: which requirements are MUST / SHOULD / COULD.
- **Dependencies**: other features, platform capabilities and external tools.
- **Acceptance criteria** in Given/When/Then form, each with a verification method and a
  named test or manual protocol.

---

## 7. Optional features and later extensions

A feature that is not part of the baseline is marked **Optional** or **Deferred** in its
specification and in the index, and it must not be presented as part of the delivered
product. Deferred features still get IDs, so that the backlog is traceable, but their
acceptance criteria are marked as not yet binding.

---

## 8. Change control

- A new requirement needs: ID, text, priority, status, source, feature, milestone,
  verification method.
- A requirement that contradicts another is resolved before both enter the index; the
  contradiction is reported in the decision log or the ADR set.
- Requirement changes made after the owner approves the baseline are recorded in the
  revision history of the index and, when they affect scope, in
  `../07-project/OPEN-DECISIONS.md`.
