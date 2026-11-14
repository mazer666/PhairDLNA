# Competitive benchmark protocol

Status: Draft. No measurements performed.

## Comparators
At minimum: VuIO (direct Rust competitor), Spritz (zero-config comparator), Gerbera or Serviio (mature DLNA comparator). Optionally Jellyfin/Plex as broader ecosystem references.

## Environment
Record exact versions/commits, licenses, hardware, OS, filesystem, library dataset, network topology, client model/firmware, server configuration and measurement tools. Run isolated repeat trials with cache/warmup explicitly controlled.

## Scenarios
1. Time to install and share first folder (novice user study).
2. SSDP discovery p50/p95, reconnection after sleep/wake/VPN/interface changes.
3. Browse latency p50/p95 for 1k, 10k, 100k items; correct pagination.
4. Idle/warm RSS, CPU, cold/warm startup and scan throughput.
5. Direct-play startup/seek success, 1/5/10 concurrent streams, client disconnect.
6. Subtitles, audio tracks, metadata and codec compatibility matrix.
7. Offline NAS/removable media, malformed metadata, crash/index recovery.
8. Security defaults: exposed ports, admin authentication, file traversal, telemetry and outbound traffic.
9. Accessibility and ability of novices to diagnose a scripted failure.

## Measurement
Use repeated measurements, report distributions not cherry-picked single numbers, define acceptable variance, collect logs with redaction, preserve fixtures and scripts. Record *unsupported* and *not tested* distinctly. Distinguish verified data from vendor claims.

## Decision
PhairDLNA is 'better' only for an explicitly defined user outcome and workload where repeatable evidence demonstrates advantage or comparable quality with materially simpler/safer UX. Never claim universal superiority.
