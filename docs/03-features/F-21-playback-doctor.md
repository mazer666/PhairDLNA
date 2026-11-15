# F-21 — Guided Playback Doctor (proposed)

Status: **Proposed — deferred beyond the v1.0 baseline**, awaiting owner approval.
Owner role: Product Owner
Version/date: 0.1 / 2026-11-14
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world
behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved.
Research sources: `docs/09-research/`.
Baseline: not part of the approved requirement baseline; it carries no canonical
requirement IDs and blocks no milestone. The `PDOC-*` identifiers are local to this
proposal and do not collide with the canonical `DOC-*` requirement identifiers.

PDOC-001 Classify failures: discovery, browse, access, codec/container, subtitles, seek, network, permissions, index and resource exhaustion.
PDOC-002 Show user-friendly symptom, likely cause, confidence and actionable next step; technical detail on demand.
PDOC-003 Never change firewall, interface, media files, system permissions or router settings without explicit user confirmation.
PDOC-004 Export a minimal, redacted, locally generated support bundle after preview/consent.
PDOC-005 Allow user to undo reversible configuration repairs and see what changed.
PDOC-006 Clearly indicate when cause is unknown; no fabricated diagnostics.

Acceptance: 10 scripted failure cases with expected classifications, remediation and safety; >=80% novice comprehension target in controlled study; negative tests for sensitive log leakage and unsafe remediation.
