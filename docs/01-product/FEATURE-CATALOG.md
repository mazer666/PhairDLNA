# Feature catalog — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Product Owner

This is the **index of features**. The normative content of a feature is in its
specification under `../03-features/`; the requirement IDs it owns are listed in
`../01-product/REQUIREMENTS-INDEX.md`.

A feature specification exists for every feature below. The product areas on the left
are the vocabulary used in the strategy documents; the canonical identifiers on the
right are the ones that appear in the requirements index, the traceability matrix and
the code.

| Product area | Feature | Specification | Milestone | Priority |
| --- | --- | --- | --- | --- |
| Discovery | F-01 SSDP discovery and advertisement | `F-01-ssdp-discovery.md` | M1 | MUST |
| Description | F-02 UPnP device and service description | `F-02-upnp-device-description.md` | M1 | MUST |
| Browsing | F-03 ContentDirectory browsing and search | `F-03-content-directory.md` | M1 | MUST |
| Transfer | F-04 ConnectionManager and transfer protocols | `F-04-connection-manager.md` | M1 | MUST |
| Metadata | F-05 DIDL-Lite metadata generation | `F-05-didl-lite-metadata.md` | M1 | MUST |
| Playback | F-06 HTTP media delivery, byte ranges and seeking | `F-06-http-media-delivery.md` | M1 | MUST |
| Compatibility | F-07 Client compatibility profiles | `F-07-client-compatibility.md` | M4 | MUST |
| Library | F-08 Media roots and filesystem safety | `F-08-media-roots-and-filesystem.md` | M1 | MUST |
| Library | F-09 Catalogue index and incremental scanning | `F-09-catalogue-index.md` | M1 | MUST |
| Library | F-10 Metadata extraction and artwork | `F-10-metadata-and-artwork.md` | M2 | MUST |
| Library | F-11 Playlists, subtitles and organisation | `F-11-playlists-and-subtitles.md` | M2 | SHOULD |
| Configuration | F-12 Configuration | `F-12-configuration.md` | M2 | MUST |
| Operations | F-13 Command line and service lifecycle | `F-13-cli-and-service-lifecycle.md` | M0 | MUST |
| Administration | F-14 Local administration and diagnostics | `F-14-local-administration-and-diagnostics.md` | M2 | MUST |
| User experience | F-15 Native macOS experience | `F-15-macos-native-experience.md` | M2 | MUST |
| Security | F-16 Security hardening and privacy guarantees | `F-16-security-and-privacy.md` | M1 | MUST |
| Optional | F-17 Optional transcoding | `F-17-optional-transcoding.md` | M5 | COULD |
| Quality | F-18 Performance and reliability budgets | — (see `../05-quality/`) | M1 | MUST |
| Quality | F-19 Quality gates and developer experience | — (see `../05-quality/`) | M0 | MUST |
| Proposed | F-20 Compatibility Inspector (proposed, deferred) | `F-20-compatibility-inspector.md` | — | COULD |
| Proposed | F-21 Playback Doctor (proposed, deferred) | `F-21-playback-doctor.md` | — | COULD |

## Scope summary

**In the baseline (M0–M3):** discovery, description, browsing, transfer negotiation,
metadata, direct play with seeking, the library with indexing, configuration, the
headless daemon, the local administration surface, the native macOS application, and
the security and privacy guarantees.

**Deferred (M4):** the client compatibility profiles and the real-device matrix.

**Deferred (M5):** transcoding.

**Proposed beyond the baseline:** the Compatibility Inspector and the Playback
Doctor, described in `DIFFERENTIATION-STRATEGY.md`.

**Explicitly out of scope:** internet-facing streaming, cloud accounts, remote
administration by default, DRM, advertising, telemetry and writing to the user's
media.
