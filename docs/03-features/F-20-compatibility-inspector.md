# F-20 — Explainable Compatibility Inspector (proposed)

Status: **Proposed — deferred beyond the v1.0 baseline**, awaiting owner approval.
Owner role: Product Owner
Version/date: 0.1 / 2026-11-14
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world
behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved.
Research sources: `docs/09-research/`.
Baseline: not part of the approved requirement baseline; it carries no canonical
requirement IDs and blocks no milestone. The `COMP-*` identifiers are local to this
proposal.

COMP-001 Track capability assertions by source: standards, declared device response, known profile, observed playback or user override.
COMP-002 Each assertion has confidence and provenance; unknown remains unknown.
COMP-003 Explain direct play/remux/transcode/unsupported selection with codec, container, audio, subtitle and network constraints.
COMP-004 Record versioned device profile and exact rule responsible; permit reversible per-device override.
COMP-005 Avoid silently collecting persistent device identifiers or exporting playback history; diagnostic export is user-reviewed and opt-in.
COMP-006 Never claim playback success from file extension or HTTP response alone.

Acceptance: controlled device/media fixtures with known, conflicting and missing capabilities; ensure no unsupported confidence escalation, correct explanations and safe fallback. Real-client tests required before product claims.
