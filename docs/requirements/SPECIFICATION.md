# Technical and acceptance requirements — draft

## Protocols
- SSDP discovery/advertisement with appropriate interface binding and TTL; graceful byebye on shutdown.
- UPnP AV MediaServer: ContentDirectory, ConnectionManager, device/service descriptions and eventing where required.
- DIDL-Lite metadata with correct escaping, media classes, protocolInfo and stable object identifiers.
- HTTP/1.1 GET/HEAD, byte ranges (206/416), content-length/type, conditional handling where applicable, robust seek.
- Device-profile fallback for client quirks without compromising standards-compliant behavior.
- Document actual supported DLNA profiles and limitations; do not claim certification.

## Library
- Multiple roots, symlink policy, removable/mounted media, incremental rescans.
- SQLite index with migrations, corruption recovery, deterministic IDs, bounded memory.
- Videos, music, images, playlists and subtitles; extraction in resource-bounded workers.
- Avoid path traversal and unauthorized access, including encoded paths and symlink races.

## Playback
- Prefer direct play; never transcode without a configured policy.
- Optional transcoding with explicit resource budgets, cancellation and process isolation.
- Validate licensing and redistribution requirements of external codecs/tools.
- Support common subtitle sidecars where clients permit; document variations.

## Administration
- Headless daemon plus CLI; native macOS UI later.
- Local admin endpoint bound to loopback by default, with authentication and CSRF protections where applicable.
- Validated versioned configuration, clear errors, diagnostics with redaction.

## Quality gates
- Unit, integration, protocol-conformance, fuzz and negative-security tests.
- Real-client matrix including device model, firmware, media format and observed outcome.
- Benchmark targets (to validate): <50 MB idle RSS for headless process, <2 s warm startup, 100k library objects, 10 simultaneous direct-play streams under specified test conditions.
- Crash recovery, no media modification, seven-day soak testing before stable.
- CI manually triggered only until explicitly revised.

## Definition of done
Feature code, tests, threat-model updates, user docs, compatibility notes, benchmarks where relevant and observed evidence. No stubs, fake integrations or unsupported green claims.
