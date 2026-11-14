# PhairDLNA — mandatory agent instructions

Read first: docs/00-governance/PROJECT-STANDARDS.md, DOCUMENTATION-POLICY.md, AGENT-ROLES.md, docs/07-project/REQUIREMENTS-WORKPLAN.md, docs/05-quality/QUALITY-GATES.md and WHERE_TO_CHANGE.md. These are binding project standards.

## Operating mode
First elaborate and review requirements. Do **not** begin DLNA feature implementation until the owner approves the requirements baseline. Inventory existing docs, map and preserve requirements, identify contradictions, research standards/clients, draft feature specs and acceptance tests, then request only material decisions. Delegate by outcomes to product, architecture, security, QA/interoperability and beginner-documentation roles. An independent review is required.

## Code rules
Rust-first, modular, easy for beginners. Source files target <=300 lines; >500 is a hard review gate. Functions target <=40 lines; >60 requires refactoring or approved exception. Complexity target <=10, nesting <=3. Descriptive names, short modules, public rustdoc, explain why/invariants, central typed configuration, explicit errors. See PROJECT-STANDARDS for details and exceptions. No stubs represented as done.

## Engineering and reporting
Every feature: traceable spec, tests including negative/boundary cases, security/interop review, docs and WHERE_TO_CHANGE update. Report verified evidence separately from assumptions and blockers. Fix failures, do not wave them through. Keep CI manual-only. No public releases/tags without owner approval. Preserve superseded approved baselines in docs/99-archive; do not silently rewrite decisions.

## License and privacy
Public AGPL-3.0-or-later, no ads, telemetry, cloud account or unnecessary internet access. Full license text and dependency audit required before release. Protect filesystem, local admin, multicast and media parser trust boundaries.
