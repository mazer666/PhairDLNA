# Cross-cutting resilience and usability acceptance plan

Status: Draft. All results NOT RUN.

## Release evidence bundles
Each acceptance case has stable ID, setup, fixtures, reference hardware, expected result, actual result, logs/traces, commit, reviewer and status (NOT RUN/PASS/FAIL/BLOCKED). No design intent counts as PASS.

### RES-A01 Streaming priority
10 direct streams + full scan; measure start/seek latency, underruns, CPU/RSS and scan backoff relative to stream-only baseline. Target thresholds require calibrated reference measurements.

### RES-A02 Bounded overload
Slow readers, disconnect storms and malformed requests; memory, sockets and tasks remain bounded; clients get protocol-correct errors.

### LIFE-A01 Network transition
Wi-Fi/Ethernet/IP/VPN transitions; listener/discovery match explicitly allowed interfaces, never widen exposure.

### LIFE-A02 Sleep/wake
Real macOS sleep/wake trials plus simulated 100-cycle test; recover to correct state with no unbounded CPU/FD growth.

### LIFE-A03 Media offline
NAS/removable volume lost and restored during scan/stream; preserve configuration, no destructive catalog cleanup, accurate unavailable state.

### LIFE-A04 Crash recovery
SIGKILL during index writes and config changes; safe reopen/repair and unchanged source-media hashes.

### MB-A01 Novice setup
At least 10 novices; >=90% succeed within 3 min on documented setup. Record completion distribution and usability issues.

### MB-A02 Accessible controls
Keyboard and VoiceOver complete critical flows; independent reviewer documents violations and fixes.

### MB-A03 UI isolation
100 Simple/Advanced toggles while streaming; no playback interruptions or effective server configuration changes.

## Comparative benchmark
Run relevant workloads against VuIO, Spritz and one mature DLNA server on identical conditions. Never claim universal superiority or measured outcomes without reproducible data.
