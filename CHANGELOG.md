# Changelog

All notable changes to PhairDLNA will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-alpha.1] - 2026-11-17

First Alpha Release of PhairDLNA — a headless Rust DLNA / UPnP media server with a native macOS menu bar companion, loopback administration API and multi-platform architecture.

### Added

- **Core & Protocol Parity:**
  - Full UPnP MediaServer:1 and ContentDirectory / ConnectionManager SCPD implementation.
  - Deterministic and cross-platform byte-consistent XML rendering for device description, SOAP envelopes, and DIDL-Lite metadata (`PLT-010`).
  - HTTP Range streaming and DLNA header handling (`transferMode.dlna.org`, `contentFeatures.dlna.org`).
- **Resilience & Lifecycle:**
  - Network interface recovery and rebinding on sleep/wake transitions (`LIFE-001` .. `LIFE-004`).
  - Exponential bounded backoff retry strategy for SSDP discovery (`LIFE-007`).
  - In-memory ring-buffer timeline recording lifecycle transitions and interface drops with path redaction (`LIFE-011`).
  - Adaptive scheduler respecting thermal states and battery power pressure (`RES-008`).
- **Storage & Indexing:**
  - SQLite persistent catalogue index with corruption recovery and atomic commit (`M2.9`).
  - Multi-root media directories with symlink containment checks (`SEC-001` .. `SEC-004`).
  - Dynamic discovery and parsing of ID3 tags, artwork thumbnails, M3U playlists, and WebVTT/SRT subtitles.
- **Service & Administration:**
  - System-context `launchd` daemon and Linux `systemd` service integration (`PLT-013`, `OPS-009`, `LIFE-010`).
  - Configuration import/export with `--apply` support (`CFG-012`).
  - Private loopback REST administration surface with bearer token and CSRF header guards (`SEC-071`).
  - Lightweight single-page web UI for settings inspection (`UX-010`).
  - First-run onboarding flow and one-click sanitized diagnostics bundle generation (`UX-011`, `OPS-026`).
- **macOS Experience & Packaging:**
  - Native status bar menu application (`phairdlna-mac`) built with native AppKit/Cocoa bindings (`M2.6`).
  - Native Apple Silicon execution verified on `aarch64-apple-darwin` without Rosetta translation (`PLT-004`).
  - Automated universal binary packaging script producing fat Mach-O bundle (`arm64` + `x86_64`) (`PLT-014`).
- **Cross-Platform Manifests & Policies:**
  - Windows executable application manifest declaring `longPathAware` support (`PLT-015`).
  - Linux native package distribution policy preventing sandboxed media containment (`PLT-016`).

### Quality & Governance

- Zero `unsafe` code outside the isolated macOS Cocoa bridge (`crates/phairdlna-mac/src/platform/bridge.rs`).
- 11 enforced local Quality Gates (`./scripts/local-ci.sh`) covering code formatting, clippy lints, unit/integration tests, doc links, supply chain bans and license audits.
- Automated live daemon fixture and client interoperability test (`scripts/run-device-interop-test.sh`).
