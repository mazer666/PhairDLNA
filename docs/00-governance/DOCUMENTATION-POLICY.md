# Documentation governance

## Canonical structure
- 00-governance: standards, ADRs, exceptions, agents, document rules
- 01-product: vision, product requirements, personas and user journeys
- 02-architecture: component boundaries, data flows, protocols, persistence
- 03-features: one specification per feature
- 04-security: threat model, privacy, dependency/license review
- 05-quality: test strategy, interoperability matrix, performance evidence, quality gates
- 06-development: setup, troubleshooting, WHERE_TO_CHANGE
- 07-project: roadmap, milestone plans, decisions, status, blockers
- 08-releases: release policy, distribution, provenance and changelog
- 99-archive: immutable snapshots of superseded baselines

## Document rules
Every document has title, status (draft/reviewed/approved/superseded), owner or responsible role, version/date, purpose, references and revision history where appropriate. Single source of truth: link instead of copying. Requirements have stable IDs and verifiable acceptance criteria. No silent deletion of approved requirements; supersede with traceability and preserve snapshots in archive. Git history is not a substitute for an explicitly archived baseline when one is required.

Keep a beginner's entry path: README -> docs/06-development/GETTING-STARTED.md -> WHERE_TO_CHANGE.md -> feature spec -> source/test paths. Avoid duplicate or conflicting normative documents. Migrate existing docs only with a mapping and updated links.
