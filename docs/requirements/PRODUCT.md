# Product requirements — PhairDLNA (draft baseline)

## Product goal
Best-in-class, privacy-first, configurable and highly compatible DLNA media server with macOS-first native experience and portable Rust core.

## Non-negotiable
- Public repository; AGPL-3.0-or-later; free of charge, advertising, telemetry and vendor accounts.
- Offline-first and LAN-first. No internet dependency for normal operation.
- Native macOS Apple Silicon and Intel first; Linux and Windows in later milestones.
- Separate headless server, CLI, and optional local UI; server must run without GUI.
- Clear setup, sensible defaults, advanced configurable device/network profiles.
- Media files read-only by default; never alter originals without explicit consent.
- Documentation understandable for newcomers, with WHERE_TO_CHANGE guidance.

## User stories
1. Select one or more local or mounted folders and make their media visible to DLNA clients.
2. Discover the server from TVs, receivers and DLNA apps on the same local network.
3. Browse by folders and media categories, with accurate metadata and artwork.
4. Play compatible files directly and seek reliably.
5. Handle missing drives, malformed media and interrupted streams gracefully.
6. Inspect client discovery, playback compatibility and failures locally.
7. Configure permitted interfaces, clients, resource limits and optional transcoding.
8. Export and restore configuration safely.

## Out of scope initially
Internet-facing streaming, cloud accounts, DRM bypass, proprietary media catalogs, remote administration by default, automatic metadata scraping.

## Success criteria
Real device interoperability matrix; reproducible performance benchmarks; long-running reliability tests; security review and passing automated gates. Proposed performance targets are not established measurements.
