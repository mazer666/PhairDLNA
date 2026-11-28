# Performance budgets and measurement protocol — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: QA / Performance Engineer
Requirements: `PERF-001` … `PERF-015`
Gate: `../05-quality/QUALITY-GATES.md` (QG-21, QG-15), `TEST-STRATEGY.md`

---

## 1. What a performance number means here

A performance number without a method is a rumour. Every measurement in this project
is reported with:

1. **the hardware profile** — model, CPU, RAM, storage type, file system, OS version;
2. **the method** — the command, the tool, the warm-up, the number of iterations;
3. **the statistic** — median and 95th percentile, never a single best run;
4. **the date and the revision** — the commit the measurement was taken on.

A number that lacks any of these is reported as `INCONCLUSIVE`, not as a result.

**Nothing in this document is a measurement.** Every value is a **target**: a budget
the implementation is measured against. The first measurement will be recorded in
`docs/07-project/STATUS.md` with its evidence.

---

## 2. The reference hardware profile

Because results only mean something relative to a machine, the project defines the
*shape* of a reference profile below. Any measurement on other hardware must state
its own profile.

**The reference hardware profile is decided (OD-11, DEC-081):** Apple Silicon (arm64, Apple M-Series, ≥16 GB RAM, NVMe SSD, release build `--release`) is the primary tier for all performance targets. Secondary profiles apply to Linux x86_64 and Windows x86_64.

The block below is the **shape** every performance report embeds. The filled-in
version lives in `../09-research/qa-performance.md` §5.3.

| Property | Reference value |
| --- | --- |
| Platform | macOS on Apple Silicon (primary, DEC-081), macOS on Intel, Linux x86_64, Windows x86_64 |
| Memory | ≥ 16 GB |
| Storage | NVMe SSD for the index; the media library on a spinning disk or a NAS is a *separate* profile and must be labelled as such |
| Library size | 100 000 objects unless stated otherwise |
| Report shape | The `hardware:` block from `../09-research/qa-performance.md` §5.3, embedded verbatim |
| Server mode | Headless daemon, release build (`--release`) |

Network conditions (where relevant) are named explicitly: wired 1 GbE, or a defined
Wi-Fi class. A measurement over an unspecified network is worthless.

---

## 3. Budgets

### 3.1 Idle and start-up

| ID | Budget | Target | Measured on | Notes |
| --- | --- | --- | --- | --- |
| PERF-001 | Resident memory while idle with 10 000 indexed objects | < 50 MB | — | The catalogue lives on disk; only the hot pages are in memory |
| PERF-002 | Time from process start to "ready to answer discovery" | < 2 s | — | Measured from process start to the first successful SSDP advertisement |
| PERF-013 | Average CPU while idle over one minute | < 1 % | — | Excludes the periodic advertisement |
| PERF-015 | Discovery latency seen by a client | < 10 s at p95 over 30 trials | — | On a documented single-subnet LAN; a television that waits too long assumes no server exists |
| PERF-012 | Behaviour beyond the tested envelope | Responsive, no memory exhaustion | — | A catalogue larger than 100 000 objects must still browse |

### 3.2 Scanning and indexing

| ID | Budget | Target | Measured on | Notes |
| --- | --- | --- | --- | --- |
| PERF-003 | Incremental rescan of 10 000 unchanged files | No file content is re-read | — | Verified by instrumenting the read path, not by timing |
| PERF-004 | Indexing throughput with metadata extraction disabled | ≥ 200 files/s | — | Metadata extraction is measured separately |
| PERF-005 | First page of a `Browse` over a container with 10 000 children | < 100 ms | — | Measured at the SOAP handler, excluding network time |
| PERF-008 | Directory scans during a browse | 0 | — | A structural property, verified by an audit hook, not a timing |
| PERF-009 | The catalogue database with 100 000 objects | A page query stays within a documented budget and the plan does not degrade into a full table scan | — | The exact budget is set by the first measurement; the requirement is that the plan is bounded |

### 3.3 Streaming

| ID | Budget | Target | Measured on | Notes |
| --- | --- | --- | --- | --- |
| PERF-006 | 10 simultaneous direct-play streams | No unbounded memory growth | — | Memory must stay flat over the run |
| PERF-007 | Memory per active stream | Bounded and documented | — | The value is filled in by the first measurement; until then the requirement is "bounded" |
| PERF-014 | Scan of a root containing a slow network share | Browse stays responsive | — | The scan must not block the request path |

### 3.4 Method rules

| ID | Rule |
| --- | --- |
| PERF-010 | Every measurement names its hardware profile. |
| PERF-011 | Every measurement reports the method, the iteration count and the statistic (median and p95). |

---

## 4. Measurement protocol

1. **Build**: `cargo build --release --workspace`. Never measure a debug build.
2. **Warm-up**: at least 3 warm-up runs that are discarded. A cold cache is measured
   separately and labelled as such.
3. **Iterations**: at least 30 measured runs for anything under one second; at least
   5 for anything longer.
4. **Statistic**: the **median** is the headline number; the **95th percentile** is
   always reported next to it. The minimum is never reported as "the" number.
5. **Isolation**: no other workload on the machine; the machine's power settings are
   noted; on macOS, thermal state is noted.
6. **Raw data**: the raw measurement output is archived next to the summary, so that
   a reviewer can recompute the statistic.
7. **Tools**:
   - micro-benchmarks: `criterion`
   - end-to-end CLI timing: `hyperfine`
   - memory: `time -l` on macOS, `/usr/bin/time -v` on Linux (the `-v` flag is
     GNU-specific), plus `ps`/`lsof` sampling
   - profiling on macOS: `xctrace` / `powermetrics` (they need elevated privileges
     and must be requested explicitly); `perf` is Linux-only
8. **Baseline**: benchmarks are compared against a stored `criterion` baseline
   (`QG-21`). A regression beyond the agreed threshold fails the gate.

---

## 5. Measuring the things that are easy to fake

| Claim | How it is actually measured |
| --- | --- |
| "Starts fast" | Process spawn to the first SSDP advertisement, not to the first log line |
| "Uses little memory" | Resident set size after a steady-state workload, sampled over time — not the peak |
| "Handles a large library" | 100 000 objects in the index, browse latency at several depths |
| "Streams smoothly" | Sustained transfer with the client reading as fast as possible, memory sampled every second, no stall > 1 s |
| "Survives a long run" | The soak test: 7 days, memory, descriptor count, thread count, index size and dropped connections sampled and graphed |

---

## 6. The soak test

Part of the release gate (`QG-22`). It runs for seven days with a scripted client
that continuously browses and streams, and records every minute:

| Signal | Why |
| --- | --- |
| Resident memory | Detects a leak in the catalogue or the transfer path |
| Open file descriptors | Detects a leak in the transfer or watcher path |
| Thread count | Detects a task that never finishes |
| Index file size | Detects an unbounded write |
| Catalogue object count | Detects a scanner that duplicates entries |
| Failed and refused requests | Detects an error path that is quietly accumulating |
| Log size | Detects a log flood |

Any trend that is not flat is a defect to be explained before release. "It probably
leaks a little but it is fine" is not an acceptable result — it is a defect with a
known rate.

---

## 7. Recording a result

```
### <requirement id> — <what was measured>

* **Machine**: <model, CPU, RAM, storage, OS>
* **Method**: <command, tool, warm-up, iterations>
* **Target**: <value>
* **Measured**: median <x> ms, p95 <y> ms
* **Verdict**: PASS / FAIL / INCONCLUSIVE
* **Evidence**: <path to the raw output>
* **Date / revision**: <date>, <commit>
```

The template lives in this file so that a result can be copied verbatim into the
feature specification or the status report without re-inventing the format.
