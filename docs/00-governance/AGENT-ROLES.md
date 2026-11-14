# Agent roles and orchestration

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Lead / Orchestrator

## 1. Roles

A role is a *responsibility*, not a mandatory separate agent. Small projects may combine
roles, but every combined role must report explicitly which areas were **not**
independently reviewed.

| Role | Owns | Deliverable |
| --- | --- | --- |
| **Lead / Orchestrator** | Scope, requirement traceability, integration, acceptance, reporting to the owner | Consolidated baseline, decision list, honest report |
| **Product Owner / Requirements Engineer** | User journeys, feature specifications, priorities, measurable acceptance criteria | `01-product/`, `03-features/`, requirement IDs |
| **Rust Software Architect** | Module boundaries, dependency direction, public API, ADRs, cross-platform design | `02-architecture/`, `docs/02-architecture/adr/` |
| **DLNA / UPnP Interoperability Specialist** | Protocol conformance, client quirks, DIDL-Lite, real-device behaviour | `05-quality/INTEROPERABILITY-MATRIX.md`, interop sections in feature specs |
| **Security & Privacy Engineer** | Threat model, parser and filesystem hardening, supply chain, privacy guarantees | `04-security/` |
| **Quality Assurance / Performance Engineer** | Test strategy, negative and recovery tests, gate automation, measurement protocol, benchmarks | `05-quality/` |
| **Documentation & Beginner Experience Reviewer** | Clarity, examples, onboarding, `WHERE_TO_CHANGE`, link integrity | `06-development/`, documentation review report |
| **Independent Reviewer** | Adversarial review of a milestone against its gates | Review findings, not approval by default |

## 2. Working rules

- **Delegate outcomes, not line-by-line instructions.** A work package defines the
  question to answer, the artifacts to produce and the quality bar — not the sentences.
- **Bound every work package by file ownership.** Two agents must never write the same
  file. Shared scratch output goes to a per-agent scratch directory outside the
  repository, and is consolidated by the Lead.
- **Independent review is required** before a milestone is presented to the owner. The
  reviewer is not the author of the artifact under review.
- **No invented approvals, no simulated tests, no endless agent debates.** A reviewer
  that cannot verify something says so; that finding is the result.
- **Report evidence separately from assumptions.** Every report separates: verified and
  observed (with the command output or URL), assumed, and blocked/not runnable.
- **Ask the owner only** for decisions that materially affect product scope, architecture,
  security, licensing or release. Technical detail questions are decided by the architect
  and documented as an ADR.

## 3. Phases and where verification happens

| Phase | Agents involved | Verification |
| --- | --- | --- |
| R0 Inventory | Lead, Documentation reviewer | Structure and link audit of the repository |
| R1 Research | Interop, Architect, Security, QA | Every claim carries a source or is marked unverified |
| R2 Scenarios | Product, Interop | Journeys reviewed against the device matrix |
| R3 Specifications | Product, QA, Security | Independent review of IDs and acceptance criteria |
| R4 Design decisions | Architect | ADR with options, trade-offs and status |
| R5 Review | All roles independently | Review findings list, open decisions list |
| R6 Implementation | Implementers | Per-milestone gates, evidence recorded |

The requirements work plan that instantiates these phases for the current cycle is
`../07-project/REQUIREMENTS-WORKPLAN.md`.

## 4. Conflict handling

- Disagreement about a *technical* question is settled by the architect, in writing, as an
  ADR — and the alternative option is recorded with its trade-off.
- Disagreement about *product scope*, *licensing*, *security posture* or *release* is
  escalated to the owner with a recommendation and the consequence of each option.
- A reviewer's finding is either fixed or explicitly declined with a reason. "Not fixed"
  without a reason is not an option.
