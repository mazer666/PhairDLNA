# Competitor feature atlas — November 2026

Status: **Research input — evidence, not scope** · Owner role: Product Owner · Version/date: 1.0 / 2026-11-15
Governed by: `../00-governance/FEATURE-EVALUATION-AND-ADOPTION-POLICY.md`
Record template: `../05-quality/FEATURE-EVALUATION-TEMPLATE.md`
Worked example: `COMPETITIVE-PARITY-ELMEDIA.md`

Status of the content: research baseline, not approved implementation scope. Sources are vendor/project descriptions unless stated otherwise. Feature availability varies by version, edition, OS and device. No hands-on tests performed. This atlas covers **server, player, caster and mirroring** products separately: a capability in a player does not imply the same capability belongs in a server.

## Elmedia Player (Electronic Team) — macOS local player and caster
Source: https://www.elmedia-video-player.com/features/ ; https://www.elmedia-video-player.com/ ; https://help.electronic.us/support/solutions/articles/44002278728
- Local video/audio playback; broad container/codec support (vendor claims); playlist editing and streaming.
- Cast local files to DLNA Smart TVs, AirPlay/Apple TV, Chromecast/Google TV and Roku.
- Playback controls: play/pause/seek/stop and compatible receiver volume.
- Direct playback or on-the-fly transcoding; 4K where client/network/hardware support it.
- Embedded, external and burned-in subtitles; online OpenSubtitles search; language selection and subtitle timing/style.
- Audio track/output choice, EQ, audio delay; video color controls, speed, PiP, rotation, preview thumbnails and bookmarks.
- AirPlay receiver capability in some editions.
- **Edition nuance:** site free version has five-minute streaming limitation; App Store editions differ. Verify version/edition before benchmarking.
PhairDLNA candidate: Core — device-aware delivery, playlists, subtitles, remote playback control where protocol allows. Optional — external subtitle search (strict opt-in and privacy), casting protocols. Later/reject as standalone features — PiP, local video filters, full local player and AirPlay receiver unless clear user demand.

## FireStream (Cyaneous) — native macOS UPnP/DLNA server
Source: https://apps.apple.com/at/app/firestream/id1005325119?mt=12
- Mac-first DLNA media serving for video, photos and music.
- Integration with macOS Music, TV, Podcasts, Books and Photos libraries plus user-selected folders.
- Preserves recognizable organization, metadata and thumbnails.
- On-the-fly transcoding for incompatible clients.
PhairDLNA candidate: Core — effortless folder/library selection, faithful metadata, artwork, Mac permissions and reliable serving. Optional — individual Apple library adapters, each with scoped consent and OS-API feasibility review. No hidden access to personal libraries. Validate library integration semantics on current macOS before commitments.

## JustStream (Electronic Team) — macOS casting and screen mirroring
Sources: https://apps.apple.com/de/app/juststream/id1474804779?mt=12 ; https://help.electronic.us/support/solutions/articles/44001810724-juststream-preferences
- Cast media to TV; mirror Mac screen to supported receivers.
- Broad video/audio formats with automatic conversion if necessary.
- Output resolution choices including Auto/Match Screen/1080p/720p; quality and bitrate controls for transcoding.
- Subtitle detection, embedded subtitle extraction, encoding fallback, burn-in, appearance and ASS style overrides.
- Device workaround to send STOP before PLAY.
- Video transcoding can require substantial CPU (vendor documentation).
PhairDLNA candidate: Core — per-device compatibility fixes, subtitle paths, resource-aware transcoding policy, explainable quality settings. Optional/Later — screen mirroring as separately consented module with explicit capture permissions and CPU/network budget; never make it a default server dependency.

## Universal Media Server — DLNA media server
Sources: https://www.universalmediaserver.com/ ; https://github.com/UniversalMediaServer/UniversalMediaServer
- DLNA/UPnP media sharing, renderer compatibility, on-the-fly media conversion, media browsing, metadata and subtitles; verify edition/version details.
- Extensive mature device-specific compatibility workarounds and format handling.
PhairDLNA candidate: Core — compatibility profile system, graceful transcoding fallback, client regression matrix. Differentiator — explain why a profile/rule is used, expose safe overrides, improve novice diagnostics.

## Serviio — DLNA server
Sources: https://serviio.org/ ; https://serviio.org/features ; https://serviio.org/component/content/article/10-uncategorised/60-free-vs-pro
- DLNA renderer profiles, audio/video/image library, metadata, transcoding, subtitles, device-specific delivery.
- Additional remote/web or library capabilities depend on edition; verify exact current edition table.
PhairDLNA candidate: Core — per-device profiles, subtitle and metadata fidelity. Optional — remote capabilities only with separate threat model; never forced WAN exposure or feature paywall.

## Gerbera — open-source UPnP media server
Sources: https://github.com/gerbera/gerbera ; https://docs.gerbera.io/
- Web UI with filesystem/database trees, editing and browsing.
- Metadata extraction, thumbnails, flexible virtual layouts, automatic rescans.
- Transcoding via scripts/plugins; latest release notes also discuss experimental FFmpeg API transcoding.
- External URL items, optional scrobbling, configurable device behavior.
PhairDLNA candidate: Core — resilient scanning, virtual views, metadata and device compatibility. Optional — advanced custom layouts and external URLs. Reject as default — online scrobbling and uncontrolled external fetches.

## Jellyfin — full media ecosystem
Sources: https://jellyfin.org/ ; https://jellyfin.org/docs/general/networking/dlna/
- Multi-user media libraries, metadata, clients, remote playback and transcoding.
- DLNA supplied through official plugin in recent versions, rather than always bundled.
PhairDLNA candidate: Core — high-quality catalog and media handling. Later — richer media organization only if justified. Reject by default — user accounts, remote-service complexity and heavyweight ecosystem features not required for LAN DLNA.

## Plex — media platform
Sources: https://www.plex.tv/ ; https://support.plex.tv/articles/200350536-dlna/
- Rich media discovery, libraries, metadata, playback clients, optional DLNA, device-specific playback and transcoding.
- Account/subscription/network feature boundaries must be checked per current edition.
PhairDLNA candidate: Core — polished library, useful artwork and understandable playback state. Differentiate through account-free, local-first, no upsell, low footprint and transparent compatibility. Do not emulate online services by default.

## ReadyMedia / MiniDLNA — minimal server
Source: https://sourceforge.net/projects/minidlna/
- Lightweight UPnP-AV/DLNA server for media sharing with simple configuration and modest scope.
PhairDLNA candidate: Core — small footprint, predictable behavior and minimal maintenance. Differentiate with Mac-native setup, safe diagnostics and broader verified client support.

## Rygel — GNOME UPnP framework/server
Source: https://gnome.pages.gitlab.gnome.org/rygel/
- UPnP media serving/rendering, plugin-based sources and media conversion.
PhairDLNA candidate: Core — protocol correctness and extensible source adapters. Avoid unnecessary framework complexity.

## VuIO — Rust DLNA/casting server (direct technical competitor)
Sources: https://github.com/vuiodev/vuio ; https://github.com/vuiodev/vuio/blob/main/docs/api.md
- Rust/Tokio/Axum/SQLite, cross-platform builds and packaging, headless binary.
- DLNA/UPnP plus Chromecast and AirPlay casting, web dashboards and REST APIs.
- Fast SQLite indexing, filesystem watching, audio tags, playlists, HTTP ranges.
- HLS/browser player, subtitles SRT->WebVTT, optional metadata providers, radio broadcasting.
- Optional admin auth/CIDR restrictions, logs/metrics/health endpoints and MCP integration.
- Resource figures in README are **vendor claims**, not independent results.
PhairDLNA candidate: Core — strong headless operation, library/index/stream performance, safe settings and diagnostics. Optional — casting, local API, metadata providers, HLS and MCP (all independently permissioned and disabled by default when they expand attack surface). Differentiate through native Mac-first UX, explicit compatibility evidence, safe defaults, recovery and measured superiority.

## Spritz (dfallman) — Rust lightweight DLNA server
Source: https://github.com/dfallman/spritz
- Lightweight CLI-oriented, minimal-setup DLNA serving; sidecar subtitle support and network troubleshooting per project description.
- Check exact current capabilities and test before treating missing features as fact.
PhairDLNA candidate: Core — frictionless start, simple CLI, understandable network help. Differentiate with native UI, durable catalog, measured reliability and profile intelligence.

## Cross-product feature families to evaluate
1. DLNA/UPnP protocol conformance and renderer quirks.
2. Direct play, remux, transcoding, HDR/audio/subtitle fallback.
3. AirPlay, Google Cast, Roku, screen mirroring (separate protocols and risk surfaces).
4. Media sources: folders, NAS, removable drives, Apple Music/TV/Photos/Podcasts/Books.
5. Metadata, artwork, virtual organization, playlists, search, optional external providers.
6. User experience: native menu bar, onboarding, Simple/Advanced, accessibility, diagnostics.
7. Reliability: sleep/wake, IP/interface changes, index corruption, unavailable shares, recovery.
8. Resource efficiency, background scheduling, long-running stability.
9. Security/privacy: filesystem scopes, parser isolation, LAN access, admin auth, internet opt-in.
10. Platform support, packaging, maintenance, license/provenance, beginner docs.

## Research protocol
For each product, record exact version/date, official feature source, edition/platform limits, screenshots or tested observations where available, uncertainty and last verified date. Missing marketing mention is **not** proof of absent functionality. Independently benchmark on identical fixtures before any superiority claim. Do not copy code, UX assets or proprietary profiles without legal/provenance review.
