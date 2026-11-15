# PhairDLNA — mandatory working rules

> **For human contributors:** you can satisfy every rule in this file yourself. The
> rules about delegation and agent roles apply to teams that work with agents; the
> rules about code, tests, evidence and documentation apply to everyone. If you are
> contributing by hand, start with
> [docs/00-governance/PROJECT-STANDARDS.md](docs/00-governance/PROJECT-STANDARDS.md)
> and
> [docs/00-governance/REQUIREMENTS-METHOD.md](docs/00-governance/REQUIREMENTS-METHOD.md),
> then come back here for the full ruleset.

Read first, in this order:

1. `docs/00-governance/PROJECT-STANDARDS.md` — the binding code rules
2. `docs/00-governance/DOCUMENTATION-POLICY.md` — the document structure
3. `docs/00-governance/REQUIREMENTS-METHOD.md` — how requirements are written and traced
4. `docs/00-governance/AGENT-ROLES.md` — the roles and the delegation rules
5. `docs/07-project/REQUIREMENTS-WORKPLAN.md` — the phase plan
6. `docs/05-quality/QUALITY-GATES.md` — what "done" means
7. `WHERE_TO_CHANGE.md` — which file owns what

These are binding project standards.

## Operating mode

**Requirements before implementation.** Elaborate and review the requirements; do
**not** begin DLNA feature implementation until the owner approves the requirements
baseline. Inventory the existing documents, map and preserve the requirements, identify
contradictions, research the standards and the clients, draft feature specifications
with acceptance criteria, and then ask only for the material decisions.

Delegate by outcome to the product, architecture, security, QA/interoperability and
beginner-documentation roles. **An independent review is required** before a milestone
is presented to the owner.

## Working rules

- One canonical identifier scheme: `PRD`, `FUN`, `INT`, `SEC`, `PERF`, `UX`, `OPS`,
  `CFG`, `PLT`, `DOC`, `SUP`, `REL`, `RES`, `LIFE`, `MB`. Never introduce a second
  numbering system, and never reuse an ID for a different meaning.
- Every requirement has: an ID, one normative sentence, a priority, an owning feature,
  a milestone and a verification method. A requirement without a verification method is
  a wish, not a requirement.
- The requirement index and the traceability matrix are **generated**. Change the
  definition, then run `python3 scripts/generate_requirement_views.py`.
- Delegate by outcome, not by line. Bound every work package by file ownership — two
  agents never write the same file.
- Report verified evidence separately from assumptions, and never claim a device test,
  a benchmark or a security result that was not observed.

## Code rules

Rust-first, modular, beginner-readable. Source files target ≤ 300 lines; > 500 is a
hard review gate. Functions target ≤ 40 lines; > 60 requires refactoring or a
documented exception. Complexity ≤ 10, nesting ≤ 3. Descriptive names, short modules,
public rustdoc, comments that explain *why*, invariants, failure modes and security
assumptions. Central, typed, validated configuration. Explicit errors; no unjustified
`unwrap`, `expect` or `panic` on a reachable path. No `unsafe` without an ADR. No
stubs represented as done. Full detail in `docs/00-governance/PROJECT-STANDARDS.md`.

## Engineering and reporting

Every feature: a traceable specification, tests including negative, boundary and
recovery cases, a security and interoperability review, updated documentation and an
updated `WHERE_TO_CHANGE.md`. Fix failures; do not wave them through. Keep CI
manual-only. No public release and no tag without owner approval. Preserve superseded
approved baselines in `docs/99-archive/`; never silently rewrite a decision.

A gate that has never failed proves nothing: every gate needs a negative test. A skip
is never a pass — record `NOT RUN — <reason>`.

## License and privacy

Public AGPL-3.0-or-later; free of charge, without advertising, telemetry, cloud
accounts or unnecessary internet access. The full licence text ships in `LICENSE`, and
the network-source offer is a product feature. Every dependency needs a recorded
licence assessment. Protect the filesystem, the local administration surface, the
multicast socket and the media parsers as trust boundaries.

## Where to change what

`WHERE_TO_CHANGE.md` is the index of the code. If a path is added, renamed or removed,
update it in the same change.
