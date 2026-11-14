# Product requirements — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Product Owner
Source: the project owner's mission statement and the product decisions in
`../00-governance/DECISION-LOG.md`.
Traceability: every ID below appears in `REQUIREMENTS-INDEX.md` with feature, milestone
and verification method.

These are the **product-level** requirements: what the product guarantees to the person
using it. Feature-level requirements live in `../03-features/`.

---

## 1. Product goal

A best-in-class, privacy-first, configurable and highly compatible DLNA media server
with a macOS-first native experience and a portable Rust core — free of charge,
advertising, tracking and vendor accounts, for as long as the project exists.

---

## 2. Non-negotiables

These are `MUST`-level commitments. A release that violates one of them is not
acceptable, regardless of what else it achieves.

| ID | Requirement | Priority | Verification |
| --- | --- | --- | --- |
| PRD-001 | The project shall be public and licensed AGPL-3.0-or-later, with the complete unmodified licence text shipped in `LICENSE`. | MUST | Repository inspection + licence audit (SUP-001) |
| PRD-002 | The product shall be free of charge, with no advertising, no telemetry, no analytics and no vendor account. | MUST | Code audit + network-observation test (SEC-090) |
| PRD-003 | Normal operation shall not require an internet connection; no component shall contact a third-party service without an explicit, documented, per-action user consent. | MUST | Offline-run test + outbound-connection audit (SEC-091) |
| PRD-004 | The product shall never expose the host to the public internet automatically; no router configuration (port mapping, UPnP IGD) shall be attempted. | MUST | Code audit + network test (SEC-092) |
| PRD-005 | The core shall be implemented in Rust and shall be free of platform-specific code; platform integration shall be isolated behind adapters. | MUST | Architecture gate + dependency direction check (PLT-001) |
| PRD-006 | The headless server shall provide the complete product function; no capability shall require the graphical application to be present or running. | MUST | Headless-only test (UX-001) |
| PRD-007 | Media files shall be opened read-only by default; the server shall never modify, move, rename or delete a user's media without explicit, per-action consent. | MUST | Filesystem-permission test + code audit (SEC-010) |
| PRD-008 | The server shall run on macOS (Apple Silicon and Intel) as the first supported platform, and the architecture shall support Linux and Windows from the first release of the core. | MUST | Platform build matrix (PLT-002) |
| PRD-009 | The product shall use sensible, safe defaults: a new installation shall serve the LAN without configuration, without opening a non-loopback administration port and without contacting the internet. | MUST | First-run test on a clean machine (UX-002) |
| PRD-010 | The product shall remain usable end-to-end for a person who has never used a DLNA server, guided by a first-run experience and by error messages that name the cause and the next step. | MUST | Usability protocol (UX-010) |
| PRD-011 | Documentation shall be part of the product: every feature shall have a specification, every requirement an ID, and a navigation file shall point a newcomer to the exact file to change. | MUST | Documentation gate (DOC-001) |
| PRD-012 | Every claim about performance, compatibility or security shall be backed by a recorded measurement or a named test; unverified claims shall be marked as such in the documentation. | MUST | Documentation review (DOC-002) |
| PRD-013 | The product shall not contain advertising, "recommended" content, telemetry endpoints or promotional network calls of any kind. | MUST | Outbound-connection audit (SEC-093) |
| PRD-014 | The server shall expose its own status, diagnostics and configuration locally to the person who installed it, without requiring a cloud service or a remote support channel. | MUST | Admin-surface test (OPS-001) |
| PRD-015 | Direct play shall be the default and preferred playback path; transcoding shall be optional, off by default and never silently applied. | MUST | Protocol trace test (FUN-070) |

---

## 3. User stories

Each story is a promise the product makes. The acceptance criteria live in the
corresponding feature specification.

| ID | As a … | I want … | So that … |
| --- | --- | --- | --- |
| PRD-020 | first-time user | to point the app at a folder and see it on my TV without reading a manual | my media works in minutes, not hours |
| PRD-021 | collector | several folders — local, external drive and NAS — to appear as one coherent library | I do not reorganise my files for the server |
| PRD-022 | collector | the index to update incrementally | a rescan of 100 000 files does not take hours |
| PRD-023 | household member | accurate titles, artwork and folder structure on the TV | browsing feels like a media library, not a file dump |
| PRD-024 | viewer | to press play and have the file start without a long buffering pause | the evening is not spent watching a progress bar |
| PRD-025 | viewer | to seek reliably in a long film | I do not restart the film after a mistake |
| PRD-026 | user with subtitles | subtitle files to be offered where the client supports them | I can watch with subtitles |
| PRD-027 | person with a failed playback | a log entry that says *what* failed, *why* and *what to do* | I can fix it without filing an issue |
| PRD-028 | privacy-conscious user | certainty that nothing leaves my network | I can trust the software with my media |
| PRD-029 | tinkerer | a validated, documented configuration file and a CLI that reports problems clearly | I can script the server |
| PRD-030 | tinkerer | to export my configuration and restore it on another machine | moving machines is not a re-setup project |
| PRD-031 | person with an unplugged drive | the server to keep running and to report the missing volume instead of crashing | one unplugged cable does not break the household |
| PRD-032 | person with a corrupt index | automatic recovery instead of a manual rebuild | a power cut does not cost me an evening |
| PRD-033 | homelab operator | the server to start at login on macOS, as a systemd user service on Linux and as a Windows service on Windows | it is always there |
| PRD-034 | developer | a codebase with small modules, documented public APIs and a test for every requirement | I can contribute without fear |
| PRD-035 | developer | automatic quality gates that fail when a rule is broken | green means green |
| PRD-036 | security-conscious user | the server bound to my LAN only, with a loopback-only administration surface | a guest device cannot reconfigure my server |

---

## 4. Scope

### In scope for the baseline (M0–M3)

- SSDP discovery and advertisement, UPnP AV MediaServer device and service description.
- ContentDirectory browsing (and search where the client asks for it).
- ConnectionManager with an honest `GetProtocolInfo`.
- DIDL-Lite metadata for audio, video, images and playlists.
- HTTP/1.1 media delivery with byte ranges, HEAD and reliable seeking.
- Media library with multiple roots, external volumes, NAS mounts, incremental indexing,
  metadata extraction and artwork.
- Headless daemon, CLI, local administration and diagnostics.
- Native macOS menu bar application.

### Deferred / optional

- Transcoding (explicitly optional, resource-bounded, off by default).
- Hardware-accelerated transcoding.
- Internet metadata scraping.
- Multi-user accounts with per-user libraries.
- DLNA certification (the project states compatibility with evidence instead).

### Out of scope

- Internet-facing streaming and remote administration by default.
- Cloud accounts, subscriptions, DRM circumvention, proprietary media catalogues.
- Advertising, telemetry, third-party analytics of any kind.
- Writing to or reorganising the user's media library.

---

## 5. Success criteria

A milestone is successful when its acceptance criteria are met **and** the evidence is
recorded. Success of the product overall requires all of:

1. **Interoperability** — a published device matrix with observed outcomes, including
   failures, across at least the client families named in
   `../05-quality/INTEROPERABILITY-MATRIX.md`.
2. **Performance** — measured budgets on documented hardware
   (`../05-quality/PERFORMANCE-BUDGETS.md`), reported with method and machine.
3. **Reliability** — a seven-day soak without crash, leak or wedged worker, plus
   crash-recovery and index-corruption recovery tests.
4. **Security** — no open critical or high finding at release; every hardening has a
   negative test that would fail without it.
5. **Automation** — every quality gate executable and observed, with a negative test
   proving the gate fails when it should.

Targets stated before measurement are **proposals**, not results. This document makes no
performance or compatibility claim that is not backed by an evidence record.
