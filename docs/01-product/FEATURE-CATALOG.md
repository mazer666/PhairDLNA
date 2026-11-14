# Feature catalog — proposed v1 baseline

Status: Draft, requires owner approval.

## P0: v1 essentials
- F-DISC: SSDP/UPnP discovery, description, lifecycle and network selection.
- F-CAT: ContentDirectory browse, stable IDs, correct DIDL-Lite metadata and pagination.
- F-STREAM: HTTP direct play, HEAD, byte ranges, seek, cancellation and backpressure.
- F-LIB: multiple local and mounted media roots, incremental index, recovery and media metadata.
- F-VIDEO: video browsing, MIME/protocolInfo, sidecar subtitles where client-supported.
- F-AUDIO: albums, artists, genres, cover art, playlists and metadata.
- F-PHOTO: image folders, thumbnails and orientation.
- F-DEVICE: client compatibility profiles and diagnostic traces.
- F-MAC: native macOS menu-bar UI, setup, status, login start and headless server lifecycle.
- F-ADMIN: simple/advanced settings, local admin and diagnostics.
- F-SEC: safe local defaults, media read-only, bounded parsing and no telemetry.
- F-OPS: packaging, updates policy, backup/restore, log privacy and recovery.

## P1: post-v1 candidates
- Optional bounded transcoding and hardware acceleration.
- Advanced subtitle conversion and client workarounds.
- Enhanced duplicate detection, smart collections and remote admin (opt-in only).
- Native Linux/Windows UX beyond headless support, per roadmap.

Priority is subject to feasibility and actual client interoperability. Do not equate file recognition with playable client support.
