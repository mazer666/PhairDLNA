# Simple by default, advanced by choice
Status: Proposed — pending owner approval · Owner role: Product Owner · Version/date: 1.0 / 2026-11-15

> **Non-normative.** This document is research and strategy. It defines **no**
> requirement IDs and creates no obligation. The binding requirements are in
> [`../01-product/TECHNICAL-REQUIREMENTS.md`](../01-product/TECHNICAL-REQUIREMENTS.md)
> and the cross-cutting view in
> [`../01-product/REQUIREMENTS-INDEX.md`](../01-product/REQUIREMENTS-INDEX.md).
> Where this document names an identifier that looks like a requirement ID, it is a
> **local proposal identifier**, not a canonical one.


Status: Product direction; acceptance thresholds proposed pending baseline approval.

## Principle
One Rust server engine, one typed/versioned configuration and one security model. Simple and Advanced are **presentation modes**, not product tiers, licenses, service variants or different runtime engines. No feature paywall, advertising, telemetry or cloud requirement.

## Simple mode
- Default on first launch; uncluttered main screen: server state, media roots, library scan progress, discovery/client status and a clear help path.
- Setup wizard: choose folders -> confirm LAN sharing -> start -> verify discoverability.
- Safe, sensible defaults; no protocol acronyms, ports or codec knobs unless troubleshooting requires them.
- Start/stop, pause/resume scanning, add/remove folder, view playback problems and switch mode.
- Explain errors in plain language, with actionable remedies and expandable technical details.

## Advanced mode
- Expose network interfaces, SSDP/UPnP, HTTP ports, device profiles, streaming/transcoding policies, indexer scheduling, CPU/RAM limits, logs, diagnostics, access and security settings.
- Searchable settings with explanatory text, defaults, impact and safe reset.
- Show effective values, inherited/overridden values and warnings for dangerous choices.
- Advanced options remain accessible without modifying defaults simply by opening the view.

## Cross-mode invariants
- Switching views must never reset settings, restart the server, change network exposure, create background tasks or affect active streams.
- All settings use one schema, validation layer and transactional update path.
- Simple mode does not conceal a security hazard created in Advanced mode; surface essential warnings in both.
- Advanced users can export/import versioned configuration with validation and redaction.
- Mode preference is reversible, persists across restarts and is independent of server configuration.

## Proposed acceptance
UX-MODE-001: 100 consecutive mode switches preserve an identical effective server configuration and do not interrupt an active stream.
UX-MODE-002: all safety-critical warnings remain visible in both modes.
UX-MODE-003: >=90% of >=10 novice participants complete initial setup in <=3 minutes under documented conditions.
UX-MODE-004: keyboard and VoiceOver navigation complete all Simple tasks; Advanced controls have clear labels and help.
UX-MODE-005: user can find any Advanced setting by search or logical grouping; evaluate with scripted tasks.
UX-MODE-006: changing presentation mode does not alter measurable server throughput beyond normal run-to-run variance; compare controlled repeated trials.

No results have been measured yet.
