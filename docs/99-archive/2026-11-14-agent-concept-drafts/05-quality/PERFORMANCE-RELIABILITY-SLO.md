# Performance and reliability — measurable targets

Status: Proposed targets, not measured guarantees.

## Design priorities
Correctness and safety > reliable playback > responsiveness > background throughput > cosmetic work. No performance optimization may bypass validation or security controls.

## Proposed SLOs
PERF-001 Headless idle RSS <50 MiB on documented reference hardware, indexed library, no active scan/stream; report median and p95 over 30 min.
PERF-002 Headless idle CPU <1% of one reference logical core over 30 min; exclude system-wide percentages.
PERF-003 Warm server readiness <2 s on defined macOS hardware with prebuilt index; 30 trials, report p50/p95.
PERF-004 Local UI standard interactions p95 <100 ms, excluding explicit I/O operations; measured on reference Mac.
PERF-005 10 simultaneous direct-play streams on reference wired LAN and storage, with no server-side underruns attributable to PhairDLNA.
PERF-006 100k-item library indexed with bounded RAM and reproducible scan rate; set exact budget after baseline.
PERF-007 Under playback load, indexer reduces its own priority and bounded concurrency; streaming latency regression threshold to be determined.

REL-001 Simulated abrupt termination never modifies media originals; after restart catalog is consistent or safely rebuildable.
REL-002 Offline NAS/removable drive is reported as unavailable, not silently removed; reconnect triggers bounded recovery.
REL-003 Network interface change, sleep/wake and client disconnect do not leak sockets/tasks; recover within measured, defined intervals.
REL-004 Seven-day soak on defined mixed workload without crashes or unbounded RSS/file-descriptor growth.
REL-005 Config changes are validated, atomic and recoverable; invalid config cannot silently change exposure.
REL-006 Active streams survive UI mode switches; headless service remains independent of GUI lifecycle.

## Benchmark discipline
Record OS, CPU, memory, storage, filesystem, network, media fixture sizes/bitrates, clients, version/commit, sample count, cache state and measurement tools. Report p50/p95/max and failures. Compare with at least two competitors under equivalent conditions. Distinguish not-tested, pass, fail and blocked. Revisit thresholds with evidence, not wishful claims.
