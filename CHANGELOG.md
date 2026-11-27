# Changelog

All notable changes to PhairDLNA will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0-alpha.6] - 2026-11-27

Sixth Alpha Release of PhairDLNA — dedicated Protocol Conformance Suite PC-01 to PC-26, UPnP/DLNA specification alignment, and automated quality gate enforcement.

### Added

- **Automated Protocol Conformance Suite (`PC-01` .. `PC-26`):**
  - Added dedicated integration test suite [`crates/phairdlna-core/tests/protocol_conformance.rs`](crates/phairdlna-core/tests/protocol_conformance.rs) implementing mandatory UPnP Device Architecture and AV checks per `INTEROPERABILITY-MATRIX.md`.
  - Automated verification for SSDP discovery matching & USN formats (`PC-01`, `PC-04`), missing MX header rejection (`PC-02`), device description schema conformance (`PC-06`), ContentDirectory and ConnectionManager SCPD validity (`PC-07`).
  - Automated verification for SOAP Browse response parameters (`PC-08`, `PC-09`), NoSuchObjectID 701 error handling (`PC-10`), standard capability boundaries (`PC-11`, `PC-12`), GetProtocolInfo MIME alignment (`PC-13`, `PC-22`), DIDL-Lite double-escaping (`PC-15`), and graceful fault handling without panics (`PC-24`, `PC-25`).
- **Roadmap & Navigation Integrity:**
  - Registered `protocol_conformance.rs` in `WHERE_TO_CHANGE.md`.
  - All 11 local quality gates (`./scripts/local-ci.sh`) fully passed.

---

## [0.1.0-alpha.5] - 2026-11-27

Fifth Alpha Release of PhairDLNA — incremental rescan verification, battery/energy platform telemetry for adaptive resource scheduling, and local-ci hardening.

### Added

- **Incremental Rescan Verification (`PERF-003`, `AC-08`):**
  - Added dedicated integration test (`catalogue_incremental.rs`) verifying that incremental scans across 100 media files avoid re-reading file bodies when metadata (size, mtime) is unchanged.
  - Proved via zero-read-permission checks (`chmod 000`) that scanning uncorrupted files relies entirely on cached directory entries without file body I/O overhead.
- **Adaptive Platform Telemetry (`RES-008`, `M2.8`):**
  - Integrated energy and battery constraints check into background scheduling arbitration (`is_energy_constrained`).
  - Read battery discharge states via sysfs on Linux (`/sys/class/power_supply/BAT0/`) and system signals without subprocess overhead (`SEC-TH-11`).
  - Background scanning automatically yields during low battery states (< 20% capacity) while active streaming and media exposure remain unhindered.
- **Service Integration and Architecture Hardening (`M2.7`, `PLT-013`):**
  - Consolidated `ServiceControl` and `SystemRunner` boundaries across platforms.
  - Verified documentation links, strict file budgets, and updated requirement traceability.

---

## [0.1.0-alpha.4] - 2026-11-27

Fourth Alpha Release of PhairDLNA — native filesystem change monitoring (FSEvents/inotify), enhanced folder toolbar ergonomics with Finder integration, hardened discovery/interop suite, and cross-platform CI fixes.

### Added

- **Native Filesystem Change Monitoring (`FUN-025`):**
  - Continuous media folder watching via OS-native backends (FSEvents on macOS, inotify on Linux via `notify 8.2.2`).
  - Event burst coalescing and 500ms debounce to prevent redundant scan floods during large file copies.
  - Automatic background catalog refresh when media files are added, modified, or removed, without manual rescan.
  - Integration tests verifying rapid visibility of added media files within seconds.
- **Enhanced macOS Folder Management & Ergonomics (`UX-020`):**
  - "Im Finder zeigen" ("Reveal in Finder") action with SF Symbol `folder` allowing instant navigation to shared media folders.
  - Responsive 2x2 folder toolbar layout separating addition, removal, reveal, and rescan operations.
  - Deduplicated rescan dispatch ensuring single trigger invocation.
- **Hardened Interoperability Verification (`INT-001`..`INT-003`, `PC-01`..`PC-08`):**
  - End-to-end device interop script (`scripts/run-device-interop-test.sh`) updated with genuine UDP multicast SSDP M-SEARCH probing.
  - Hierarchical ContentDirectory Service (CDS) browse verifying media resource URL extraction and byte-range delivery.
- **Cross-Platform & CI Hardening:**
  - Linux service dry-run test aligned with `phairdlna.service` systemd unit naming.
  - Non-macOS target clippy warning cleanups for auxiliary configuration helpers.

---

## [0.1.0-alpha.3] - 2026-11-26

Third Alpha Release of PhairDLNA — responsive 2-column landscape grid, card-based dashboard containers, interactive folder confirmation dialog, library rescan action, and Milestone 4 hardening completion.

### Added

- **macOS Responsive 2-Column Grid (`UX-020`):**
  - Dynamic 2-column layout when window width ≥ 750 pt (landscape/widescreen), grouping Status & Folders on the left and Settings & TV Connection on the right.
  - Native card containers (`NSBoxCustom`) with rounded corners and subtle background styling.
  - Native modal folder removal confirmation dialog (`dialog.remove_folder.*`).
  - Library Rescan button ("Neu einlesen") with SF Symbol `arrow.clockwise`.
  - Path hints (`~/...`) in the folder picker dropdown.
- **Milestone 4 Interoperability, Reliability & Security (`M4`):**
  - Client compatibility quirk engine and device matrix (`INT-100`..`INT-104`).
  - Stored M-SEARCH replay diagnostics (`INT-103`).
  - Parser fuzzing and containment bounds (`SEC-060`..`SEC-068`).
  - Long-haul soak test and crash recovery verification (`REL-001`..`REL-006`).

---

## [0.1.0-alpha.2] - 2026-11-23

Second Alpha Release of PhairDLNA — direct server steering, window lifecycle management, streamlined native typography, and native settings editing without web UI.

### Added

- **Direct Server Steering (`DEC-080`, `UX-019`):**
  - Direct graphical Start/Stop button in the overview window with dynamic status reflection.
  - Native service start/stop controls in application menus and status item.
  - Interactive folder management directly in the overview window (add/remove media roots).
- **Primary Settings in Main Window:**
  - Direct viewing and editing of Server Name (`device_name`) and Port (`http_port`) in the main window.
  - Validated and atomic persistence to `config.toml` without routing through preferences.
- **macOS Window Lifecycle & Reopen (`ADR-0013`):**
  - Window close button (red "X" or `Cmd+W`) hides/minimizes the window (`orderOut:`) while keeping the application and DLNA server running in the background.
  - Seamless window restoration when clicking the Dock icon (`applicationShouldHandleReopen:hasVisibleWindows:`) or choosing "PhairDLNA Übersicht" (`Cmd+1`).
  - Native multi-language quit confirmation dialog ensuring the server is not stopped unintentionally.
- **Streamlined Typography:**
  - Removed obsolete manual text scaling controls (100%/150%/200%) in favor of native macOS system typography and Dynamic Type compatibility.
- **Diagnostics & About:**
  - Native file-save panel for exporting sanitized diagnostics bundles.
  - Dedicated About tab with project version, license, and repository links.

### Changed

- Complete retirement of the legacy Web Admin UI in favor of pure-native macOS AppKit controls (`DEC-075`).
- Refactored `OverviewViews` and responsive vertical reflow geometry.

---

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
