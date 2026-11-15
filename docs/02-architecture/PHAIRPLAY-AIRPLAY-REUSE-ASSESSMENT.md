# PhairPlay cross-project reuse and AirPlay feasibility

Status: Research and architecture proposal — **not** an approval to copy code, implement AirPlay, or claim feature support.
Source reviewed: https://github.com/mazer666/PhairPlay (main, 2026-11-15). Inspect commit SHA and current source before porting.

## Key architectural distinction

PhairPlay is primarily an **AirPlay 2 receiver** on Android TV/Fire TV; PhairDLNA is a **DLNA MediaServer** for macOS/Linux/Windows. AirPlay output to Apple TV/compatible receivers requires an **AirPlay sender/client**, which is not obtained by dropping PhairPlay's receiving RTSP server into PhairDLNA. PhairPlay also implements UPnP/DLNA **MediaRenderer**, not the same role as PhairDLNA's MediaServer. Reuse protocol expertise, defensive patterns and tests where relevant; implement only the missing role with protocol-correct behavior.

## Confirmed repository findings (from project files, not device testing)

- Project readme: https://github.com/mazer666/PhairPlay/blob/main/README.md — AirPlay receiver, mDNS, RTSP, pairing, FairPlay, audio/video, DACP and DLNA renderer described.
- AirPlay source: https://github.com/mazer666/PhairPlay/tree/main/app/src/main/kotlin/com/phairplay/airplay — session, URL mode, remote, discovery, codecs and protocol parsing.
- DLNA renderer: https://github.com/mazer666/PhairPlay/tree/main/app/src/main/kotlin/com/phairplay/dlna — SSDP, SOAP, GENA, renderer controls, network and URI logic.
- Diagnostics: https://github.com/mazer666/PhairPlay/tree/main/app/src/main/kotlin/com/phairplay/diagnostics — local redaction, self-tests, CPU/resource assessment.
- Lifecycle: https://github.com/mazer666/PhairPlay/tree/main/app/src/main/kotlin/com/phairplay/service — network recovery and supervision.
- License: https://github.com/mazer666/PhairPlay/blob/main/LICENSE — Apache License 2.0; PhairDLNA proposed AGPL-3.0-or-later.
- README explicitly says **0 real-TV device tests** performed as of inspected version; described build/test status is project-reported, not independently verified in this review.

## Candidate reuse matrix

| Component / knowledge | PhairPlay evidence | PhairDLNA opportunity | Decision |
|---|---|---|---|
| mDNS advertising and service lifecycle | airplay/MdnsService*, MdnsRegistrationTracker | Optional AirPlay sender discovery using mDNS/DNS-SD; macOS Bonjour adapter and portable Rust equivalent | STUDY / REDESIGN |
| RTSP messages/session parsing | airplay/Rtsp* | Evaluate sender-side RTSP controls where relevant | STUDY / NEW SENDER |
| Device/pairing security | airplay/handshake/* | Threat modeling and compatibility test fixtures; do not assume receiver pairing maps to sender | STUDY / LEGAL CHECK |
| Timing, codec negotiation and stream feedback | Audio*, TimingHandler, StreamStats | Capability scoring and adaptive delivery only if applicable to sender protocol | STUDY |
| Playback metadata / remote control | NowPlaying*, DacpClient | Optional casting controls and now-playing diagnostics | STUDY |
| DLNA SSDP, SOAP, GENA | dlna/* | Reuse standards knowledge, parser tests and negative cases; renderer vs server roles differ | TEST REFERENCE |
| URI validation, SOAP XML, redaction | DLNA/security/diagnostics modules | Common defensive requirements for Rust implementations | PORT IDEAS AND TESTS |
| Network recovery/service supervision | service/NetworkRecoveryMonitor etc. | macOS sleep/wake, interface and advertisement recovery policies | ADAPT DESIGN |
| Kotlin/Android UI and codecs | Android UI / C++ | No direct dependency in Rust core; use as behavioral reference | DO NOT IMPORT BLINDLY |
| FairPlay-related native code | app/src/main/cpp/playfair/* | High legal, cryptographic and maintenance sensitivity | EXCLUDE pending independent legal/security/necessity review |

## Licensing, provenance and security gates

Apache-2.0 is generally compatible with AGPLv3 in the appropriate direction, but source-level integration requires actual ownership/provenance verification, notices, attribution and license review. A repository's top-level Apache license does not automatically prove every included third-party/native source file shares that license. Check nested headers, origin, dependencies, cryptographic implementations, SDK terms and codec patents. AGPL compliance for networked software remains a separate obligation. No automatic transplantation of FairPlay or proprietary/undocumented pairing code. Do not present this as legal clearance.

Before reuse, demand review of: input validation, pair/authentication, URL SSRF prevention, outbound network opt-in, local discovery permissions, privacy-preserving logs, bounded CPU/RAM/socket use, race/cancellation/recovery, fuzz/protocol tests, compatibility with actual Apple TV and AirPlay-capable devices, and export/cryptography licensing where relevant.

## Proposed AirPlay feature boundary

AIR-001: Optional AirPlay sender integration is disabled unless user explicitly enables it; no extra advertiser/listener or external dependency in core when off.
AIR-002: Source files/media and device discovery remain local. Show actual destination and media/metadata disclosure before first cast.
AIR-003: Detect compatible targets; distinguish supported/unknown/unsupported, AirPlay generation, pairing and codec capabilities. Don't claim automatic universal AirPlay 2 coverage.
AIR-004: Implement sender-side controls (start/pause/seek/stop) only for tested receivers; error states and fallback are explicit.
AIR-005: Resource budgets, stream arbitration and safe abort; AirPlay use must not regress existing DLNA streaming.
AIR-006: No forced Apple account, cloud access, DRM bypass or proprietary key extraction.
AIR-007: Simple mode displays 'Cast to device' only when enabled and supported; Advanced gives profiles, discovery/network permissions and codec/fallback choices.
AIR-008: Acceptance includes Apple TV and at least one independently implemented AirPlay receiver on real hardware with documented version, codec, auth, transport, seek, subtitle and reconnection cases; all results NOT RUN until tested.
AIR-009: Independent security, provenance and codec/license audit before code acceptance.

## Implementation order

1. Stabilize primary DLNA MediaServer and native Mac experience.
2. Define protocol-neutral `CastingTarget`, `CastingSession` and `CapabilityEvidence` interfaces without importing receiver code.
3. Research and independently validate AirPlay sender feasibility, protocol/SDK/legal constraints, real receiver tests and resource envelope.
4. Port selected reusable **ideas/test fixtures**, with source attribution and explicit security review, to small documented Rust modules where justified.
5. Implement optional sender module after baseline owner approval; test against real Apple TV/receivers and compare CPU/RAM with AirPlay off.
6. Only then advertise AirPlay support in GitHub topics and README as implemented.

## Agent task

Cross-reference PhairPlay's actual files and tests, enumerate a file-level reuse candidate inventory with license origin and role mismatch; independently review architecture/security/legal/QA; propose meaningful user outcome and acceptance tests. Produce a go/no-go decision for sender capability. No AirPlay feature development before requirements approval; no tags/releases; GitHub Actions manual only.
