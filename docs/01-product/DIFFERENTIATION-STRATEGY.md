# Product differentiation — PhairDLNA

Status: Proposed strategy, pending owner approval.

## North star
**A media server that explains and resolves playback problems instead of making users become DLNA experts.**

## Outcome pillars
- Works reliably with the user's actual devices, not just in a standards test.
- Makes complex compatibility decisions understandable.
- Keeps source media and user privacy protected.
- Remains unobtrusive during everyday operation.
- Makes a beginner successful without hiding expert controls.

## Proposed signature experiences
### A. Compatibility Inspector (P0 research, phased implementation)
For a selected device and media item, display:
- Device identity, firmware if observed, profile version and confidence level.
- Codec/container/audio/subtitle compatibility as observed, inferred or unknown.
- Chosen path (direct, remux, transcode or unsupported) and why.
- Resource impact and non-destructive user override.
- Evidence from actual playback, without claiming client telemetry beyond what protocol reveals.

### B. Playback Doctor (P0 design, staged implementation)
Guided diagnosis of discovery, browse, stream, seek, codec, subtitles and local permissions. Provide specific remedies, explain confidence and ask consent for impactful changes. No automatic firewall bypass, WAN exposure or silent codec installation.

### C. Resilient Library (P0)
Treat temporarily unavailable NAS/drives as offline, not deleted; display state, recover without damaging original media, avoid aggressive rescans.

### D. Mac-first Setup (P0)
Select folders -> review local sharing -> discover client -> play test item; visible progress and understandable errors. Accessibility and keyboard support are release requirements.

### E. Transparent compatibility laboratory (P0)
Publish reproducible test fixtures and real-device matrix with observed outcomes, firmware, limitations and regressions; never claim universal compatibility.

## Product scorecard — proposed, not yet measured
- First-run success: >=90% of >=10 representative novices in <=3 minutes on documented test setup.
- Diagnose common failures: >=80% of novice participants choose correct next action for 10 scripted scenarios.
- Discovery: p95 <=10s in 30 trials on reference single-subnet LAN.
- Correctly classify 100% of controlled test cases as known-supported/known-unsupported/unknown without false guarantees.
- Recover gracefully from 100% of defined removable-drive, sleep/wake and index-corruption fixtures.
- Zero unauthorized media writes and zero default internet-facing admin endpoints.
- Compare cold/warm start, RSS, CPU, scan throughput and playback reliability against at least two reference servers on identical conditions.

## Tradeoffs and boundaries
DLNA client capabilities are often not reliably discoverable. Do not promise automatic certainty. Device profiles need provenance, versioning, opt-in diagnostics and user override. Transcoding can be costly and license-sensitive. Keep direct play primary; build smart fallback only after protocol correctness.

## Prioritization
P0: interoperability, robust streaming, resilient library, beginner setup, basic explanations and diagnostics.
P1: advanced device capability intelligence, explainable route optimization, extensive remediation.
P2: optional cross-protocol casting and AI/MCP only if supported by demonstrated user need; do not dilute core DLNA excellence.

## Anti-goals
No feature-count race, unverified 'fastest' claim, invasive device fingerprinting, forced accounts, dark patterns, automatic remote exposure, opaque self-healing or dependency bloat.
