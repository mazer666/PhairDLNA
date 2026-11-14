# F-COMPAT — Explainable Compatibility Inspector

Status: Proposed signature feature, not yet approved.

COMP-001 Track capability assertions by source: standards, declared device response, known profile, observed playback or user override.
COMP-002 Each assertion has confidence and provenance; unknown remains unknown.
COMP-003 Explain direct play/remux/transcode/unsupported selection with codec, container, audio, subtitle and network constraints.
COMP-004 Record versioned device profile and exact rule responsible; permit reversible per-device override.
COMP-005 Avoid silently collecting persistent device identifiers or exporting playback history; diagnostic export is user-reviewed and opt-in.
COMP-006 Never claim playback success from file extension or HTTP response alone.

Acceptance: controlled device/media fixtures with known, conflicting and missing capabilities; ensure no unsupported confidence escalation, correct explanations and safe fallback. Real-client tests required before product claims.
