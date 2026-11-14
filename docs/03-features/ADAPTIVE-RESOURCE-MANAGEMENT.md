# F-RESOURCE — Adaptive resource management

Status: Draft requirements for owner/agent review. No implementation or benchmarks claimed.

## User outcome
Playback stays smooth while library scans, thumbnails and diagnostics remain unobtrusive. Users need not tune threads, queues or CPU limits to get a good default experience.

## Requirements
RES-001 Single Rust-core scheduler arbitrates stream-serving, discovery, metadata reads, indexing, artwork, optional transcoding and maintenance with explicit priority classes. Streaming correctness and control-plane responsiveness take precedence over background indexing.
RES-002 Every worker pool, queue, cache and connection count has a documented bound, cancellation policy and overload behavior. Avoid unbounded tasks or memory.
RES-003 Indexing dynamically yields when streaming latency, CPU, disk I/O or memory pressure crosses documented thresholds. Use hysteresis to prevent oscillation. Do not mistake network client slowness for server overload.
RES-004 Simple mode uses adaptive defaults. Advanced exposes safe, validated resource budgets and a reset-to-default path; changing UI mode never changes effective budgets.
RES-005 Optional transcoding is isolated and resource-capped; it must not starve direct-play clients or discovery.
RES-006 Metrics are local, opt-in to view/export, privacy-preserving and labeled with measurement provenance. No telemetry.
RES-007 When resources are insufficient, fail gracefully with a meaningful explanation; never silently drop active streams, corrupt state or alter source files.
RES-008 Battery/thermal-aware behavior on macOS may reduce background work, but cannot silently change media exposure or disable active playback; document OS signals and fallback.
RES-009 Startup, scan, sleep/wake and shutdown must release workers and file handles predictably.

## Acceptance evidence
- Test concurrent 1/5/10 direct-play clients with indexing on/off, on documented storage and network. Compare startup latency, seek success, throughput, CPU, RSS, disk utilization, open FDs; define acceptable regression thresholds from baseline.
- Trigger synthetic CPU/memory/disk pressure and assert indexing backs off while streams remain served, or reports capacity limits accurately.
- Prove bounded task/queue/cache growth under slow readers, disconnect storms and hostile input.
- Repeat 100 Simple/Advanced view switches during a stream; no change in scheduling configuration or playback interruptions.
- Measure sustained 7-day mixed workload with memory/FD trend and error counts.
- For each test record hardware, OS, commit, fixtures, tools, sample count, p50/p95/max, failures and raw artifacts.

## Architecture boundaries
Resource policy belongs in shared core, not UI. Platform adapters supply OS hints, not policy decisions. Scheduling and backpressure must be testable with deterministic synthetic clocks/events where feasible.

## Open decisions
Reference hardware and storage; limits per device; transcoding in v1; battery policy; threshold calibration. Agents should propose defaults with evidence rather than invent measurements.
