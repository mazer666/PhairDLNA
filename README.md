# PhairDLNA

**Privacy-first, cross-platform DLNA/UPnP AV media server built in Rust.**

Status: **requirements and architecture phase**. No functional server is implemented yet.

## Principles
- Public, free, ad-free, tracking-free, no accounts or cloud dependency.
- AGPL-3.0-or-later; full license text must be included before release.
- Rust-first core; native macOS integration first, Linux and Windows next.
- Direct Play first; transcoding optional, isolated and resource-limited.
- Secure local-network defaults; no WAN exposure by default.
- Excellent DLNA interoperability backed by real-device testing.
- Accessible configuration: simple defaults, advanced profiles and diagnostics.
- Documentation, security and reproducible verification are product features.

## Scope
UPnP/SSDP discovery, ContentDirectory and ConnectionManager, DIDL-Lite metadata, HTTP streaming with byte ranges and seek, audio/video/photo libraries, subtitles, device profiles, robust indexing and optional transcoding.

## Project documents
- [Product requirements](docs/requirements/PRODUCT.md)
- [Technical specification](docs/requirements/SPECIFICATION.md)
- [Architecture](docs/architecture/ARCHITECTURE.md)
- [Security model](docs/security/SECURITY-MODEL.md)
- [Roadmap](docs/project/ROADMAP.md)
- [Release blockers](docs/project/RELEASE-BLOCKERS.md)
- [Agent instructions](AGENTS.md)

**No performance or compatibility claim is verified yet.**
