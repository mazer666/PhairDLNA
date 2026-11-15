# Competitor feature evaluation and adoption policy

Status: **Approved (v1.0 baseline, 2026-11-15)** · Owner role: Lead / Product Owner
Applies to: every feature that a competitor has, and every feature idea in general.
Records: `../01-product/COMPETITOR-FEATURE-ATLAS.md` (the research),
`../05-quality/FEATURE-EVALUATION-TEMPLATE.md` (the record template),
`../01-product/COMPETITIVE-PARITY-ELMEDIA.md` (the worked example).

## Objective
Not a feature-count contest: deliver measurable user value beyond competing products while retaining best-in-class performance, reliability, security, accessibility and privacy.

## Every competitor feature requires an evaluation record
- Stable ID, source product/version/edition and primary-source URL.
- What the feature does; which user problem it solves; user frequency and impact.
- What competing implementations do well/poorly, with evidence vs inference separated.
- PhairDLNA's independent, better user outcome and why it is meaningful.
- Architecture fit (DLNA server vs local player vs caster vs mirroring), portability, operational cost.
- Security/privacy analysis: permissions, network exposure, external requests, data retention, licensing.
- Performance/resource budgets and interaction with active streams.
- Simple-mode behavior (minimal UI), Advanced controls, documented defaults, opt-in/opt-out, reversibility.
- Concrete functional, usability, accessibility, interoperability, reliability and performance acceptance tests.
- Priority, milestone, decision (CORE/OPTIONAL/LATER/REJECT), reviewer and owner decision when material.

## Decision rules
CORE: directly necessary for excellent LAN DLNA serving and maintainable within release gates.
OPTIONAL: meaningful value but requires explicit enablement, permission or extra resources.
LATER: valuable but outside initial focus; retain approved backlog.
REJECT: low user value, duplicated functionality, unacceptable risk, poor maintenance economics or product drift.

Do not automatically implement every competitor feature. The instruction to implement features that meet best-in-class requirements means **only after** user-value assessment, design review, security/performance acceptance and milestone approval. An unapproved idea is not an implementation mandate.

## Configuration and UX contract
- Simple by default, Advanced by choice; one engine and validated versioned settings schema.
- No hidden forced integrations, telemetry, external lookups, accounts, updates, scanning or transcoding.
- All meaningful user-visible behaviors configurable where safe; not every internal tuning knob needs UI exposure.
- Per-feature toggles with clear explanation of consequences, defaults and resource costs.
- Features expanding network attack surface or transmitting data off-device default OFF.
- Turning a feature OFF stops its work, closes unnecessary listeners, releases resources and preserves user-owned data.
- Safety-critical warnings visible in both modes; Advanced cannot disable mandatory validation.
- Configuration import/export and rollback must be secure and non-destructive.

## Quality bar
A feature is accepted only if its specified use case is tested, docs are current, security review passed, resource overhead measured and negative/recovery tests passed. Features must not degrade existing direct-play reliability or introduce unbounded CPU, RAM, disk, sockets or background tasks.

## No copying
Reimplement independently. Any third-party source, algorithm, codec, asset, UI design or device profile requires license/provenance review. Do not import GPL-incompatible or proprietary code by accident.

## Agent workflow
Inventory -> evaluate -> recommend -> review -> approve baseline -> implement milestone -> benchmark -> accept/reject. Report evidence, uncertainty and unresolved decisions. Keep GitHub Actions manual-only, no releases/tags without owner approval.
