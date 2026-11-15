# F-18 — Performance and reliability budgets

Status: Draft
Owner role: QA / Performance Engineer
Version/date: 0.1 / 2026-11-14
Requirements: PERF-001 … PERF-015, REL-001 … REL-006, RES-001 … RES-009
(`PERF-003` … `PERF-005`, `PERF-008`, `PERF-009` and `PERF-012` are owned by F-09;
`PERF-004` is shared with F-09 and F-10, and the index assigns it to F-09.)
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved (see `docs/09-research/`).
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.

## 1. Purpose and user story

A media server lives on a person's desk, next to their music and their films. It is
expected to be **invisible** while it works and **boring** when it does not: small,
quiet, predictable, and able to run for a week without anyone noticing it.

**User story.** As someone who leaves the server running on a laptop under a desk, I
want it to use almost no memory and almost no processor while idle, to find my files
quickly when the television asks, to keep playing smoothly while the library is
being re-indexed, and to survive being unplugged, put to sleep or killed without
losing anything — so that I never have to think about it.

This feature covers two related concerns:

- **Performance** — how fast the server answers, and how much it costs to answer.
- **Reliability** — what happens when the world misbehaves: a volume disappears, the
  network changes, the machine sleeps, the process is killed.

Both are specified together because they share one instrument: the **measurement
protocol**. A number without a method is not a result.

---

## 2. Scope

**In scope**

- The performance budgets in `PERF-001` … `PERF-015` and the measurement protocol in
  `../05-quality/PERFORMANCE-BUDGETS.md`.
- The reliability targets in `REL-001` … `REL-006` and the fault-injection scenarios
  that prove them.
- The adaptive resource management in `RES-001` … `RES-009`: one priority scheduler,
  bounded pools, and a scanner that yields when the machine is busy.
- The soak test and the signals it records.

**Explicitly out of scope**

- The *content* of the budgets: changing a budget is an owner decision recorded in
  `../07-project/OPEN-DECISIONS.md` (OD-11).
- The comparative benchmark against other servers: that is a plan in
  `../05-quality/COMPETITIVE-BENCHMARK-PLAN.md`, not a requirement.
- Hardware tuning, filesystem tuning and operating-system tuning: the server must
  behave well on an ordinary machine, not an optimised one.

---

## 3. Functional requirements

**PERF-001** — The headless server shall keep its resident memory below 50 MB while
idle with 10 000 indexed objects. [DESIGN] "Idle" means: no scan running, no transfer
active, at least one minute after start-up.

**PERF-002** — The server shall reach the state "ready to answer discovery" within
2 seconds of process start. [DESIGN] The measurement point is the first successful
SSDP advertisement, not the first log line — a log line can be written long before the
socket is bound.

**PERF-003** — An incremental rescan of a tree with 10 000 unchanged files shall
complete without re-reading any file content. [DESIGN] This is verified structurally,
by instrumenting the read path, not by timing: a scan that is fast because it skipped
the work and a scan that is fast because the disk is fast must be distinguishable.

**PERF-004** — The server shall index at least 200 media files per second with
metadata extraction disabled. [DESIGN] Metadata extraction is measured separately,
because it is dominated by the media format and not by the index.

**PERF-005** — A `Browse` request over a container with 10 000 children shall return
its first page within 100 ms. [DESIGN] Measured at the SOAP handler, excluding
network time, so that the number describes the server rather than the LAN.

**PERF-006** — The server shall sustain 10 simultaneous direct-play streams without unbounded memory
growth. [DESIGN] Ten clients each receive a byte range from their own file at the same time,
without transcoding (FUN-040); the resident memory does not grow without bound as the number
of streams rises, and the memory attributable to one stream stays inside the bound that
PERF-007 documents. The measurement is described in §5.

**PERF-008** — A browse request shall never trigger a directory scan. [DESIGN] A
structural property, verified with an audit hook that counts filesystem operations
during a browse.

**PERF-009** — The catalogue database shall support 100 000 objects with a bounded
index scan. [DESIGN] "Bounded" means the query plan does not degrade into a full
table scan per page.

**PERF-012** — The server shall degrade gracefully beyond the tested envelope: a
catalogue larger than 100 000 objects shall remain browsable and shall not exhaust
memory.

**PERF-013** — The average processor usage of an idle server shall be below 1 % of one
logical core, measured over one minute and excluding the periodic advertisement.

**PERF-015** — A client shall discover the server within 10 seconds at the 95th
percentile over 30 trials on a documented single-subnet LAN.

---

## 4. Technical requirements

**RES-001** — The server shall arbitrate streaming, discovery, metadata reading,
indexing, artwork generation, optional transcoding and maintenance through **one
scheduler** with explicit priority classes. [DESIGN] Streaming correctness and
control-plane responsiveness take precedence over background indexing. A second,
competing arbitration mechanism would make the priority model unverifiable.

**RES-002** — Every worker pool, queue, cache and connection count shall have a
documented bound, a cancellation policy and a defined overload behaviour. [DESIGN]
"Unbounded" is not an acceptable answer anywhere in the server: an unbounded queue
is a denial of service waiting to happen.

**RES-003** — Indexing shall yield when streaming latency, processor usage, disk
input/output or memory pressure crosses a documented threshold, and shall use
**hysteresis** so that it does not oscillate between running and stopping. [DESIGN]
Without hysteresis the scanner flaps, and a flapping scanner is worse for playback
than a scanner that never starts.

**RES-004** — A slow client shall not be mistaken for server overload. [DESIGN] The
scheduler must distinguish "this client is not reading" from "the machine is
overloaded". A slow reader must slow down only its own stream.

**RES-005** — The resource policy shall live in the shared core, not in a user
interface; platform adapters shall supply operating-system hints, not policy
decisions. [DESIGN] Otherwise the policy cannot be tested, and the headless server
would behave differently from the graphical one.

**RES-006** — Local resource metrics shall be opt-in to view and export, shall be
privacy-preserving, and shall never be transmitted off the machine.

**RES-007** — When resources are insufficient, the server shall fail gracefully with
a meaningful explanation; it shall never silently drop an active stream, corrupt
state or alter a source file.

**RES-008** — On a battery-powered or thermally constrained machine the server may
reduce background work, but it shall never silently change which media is exposed and
never disable an active playback. [DESIGN] Reducing background work is allowed;
changing the product's behaviour is not.

**RES-009** — Start-up, scanning, sleep/wake and shutdown shall release workers and
file handles predictably.

**PERF-014** — The server shall survive a scan of a root that contains a broken or slow
network share without blocking browse requests. [DESIGN] The scan runs in its own
bounded workers; a share that stops answering must not hold the request path.

**REL-001** — An abrupt termination shall never modify a media original; after a
restart the catalogue shall be consistent or safely rebuildable.

**REL-002** — A media root that is offline shall be reported as unavailable and shall
not be silently removed from the configuration or the catalogue.

**REL-003** — A network interface change, a sleep/wake cycle and a client disconnect
shall not leak sockets or tasks, and the server shall recover within a documented
interval.

**REL-004** — The server shall survive a seven-day soak on a defined mixed workload
without a crash, an unbounded memory growth or an unbounded descriptor growth.

**REL-005** — A configuration change shall be validated, applied atomically and
recoverable; an invalid configuration shall never silently change the network
exposure.

**REL-006** — Active transfers shall survive a change of the presentation mode, and
the headless service shall remain independent of the life cycle of any user
interface.

---

## 5. Performance requirements

The budgets are the normative part of `../05-quality/PERFORMANCE-BUDGETS.md`. The
rules that make them meaningful:

| Rule | Requirement |
| --- | --- |
| A measurement names its hardware profile | `PERF-010` |
| A measurement reports method, iterations and statistic | `PERF-011` |
| A budget without a machine is not comparable | `PERF-010` |
| Memory per stream is bounded and documented | `PERF-007` |

**PERF-007** — The memory consumed by one active transfer shall be bounded by a
documented value. [DESIGN] The value is not fixed here: it is measured first and
written down. Until then the requirement is "bounded", and the bound is enforced by
the buffer limit in the transfer path.

**PERF-006** is measured with ten simultaneous direct-play clients against the reference
profile: the resident set is sampled before the streams start, while they are running and
after they have been closed, and the growth between the first and the last sample is the
number the requirement is about. The memory attributable to a single stream is the number
PERF-007 documents. A growth that does not flatten is a defect to be explained, in the same
way as a non-flat trend in the soak test.

### The measurement rules

1. Release builds only. A debug build is never a measurement.
2. At least three discarded warm-up runs, then at least 30 measured runs for
   anything under a second.
3. The **median** is the headline number; the 95th percentile is always reported
   next to it. The minimum is never reported as "the" number.
4. The raw output is archived next to the summary, so a reviewer can recompute the
   statistic.
5. A measurement on unspecified hardware is `INCONCLUSIVE`, never `PASS`.

### The soak test signals

`REL-004` is verified by a seven-day run that samples every minute: resident memory,
open file descriptors, thread count, index file size, catalogue object count, failed
and refused requests, and log size. Any trend that is not flat is a defect to be
explained, not a footnote.

---

## 6. Usability and accessibility

- The status command reports what the user would otherwise have to guess: memory
  use, the number of indexed objects, the scan state and the active transfers
  (`OPS-002`).
- A resource limit that the user has hit shall be reported in plain language with
  the setting that would change it (`UX-004`).
- The user interface shall never block while a measurement or a scan runs
  (`MB-002`).
- The diagnostics report shall distinguish "the server is slow" from "the network is
  slow" from "this client is incompatible" — three different problems that a single
  "playback failed" message hides.

---

## 7. Security and privacy

- Resource limits are a security control, not only a performance one: an unbounded
  queue is a denial-of-service vector (`SEC-060` … `SEC-065`, threat rows
  `SEC-TH-03`, `SEC-TH-14`, `SEC-TH-15`, `SEC-TH-16`, `SEC-TH-22`).
- Resource metrics may reveal what the user is playing (`RES-006`); they are
  therefore opt-in, local and redacted.
- The soak and the fault-injection tests must never modify a media original
  (`REL-001`); the media checksums are compared before and after.
- The outbound-connection audit runs as part of the soak, so that a background
  feature that starts talking to the internet is caught.

---

## 8. Interoperability

- A client that abandons a transfer must not leave a slot behind (`INT-089`,
  `SEC-TH-14`). The soak measures the descriptor count for exactly this reason.
- The server must never answer a request more slowly because another client is
  streaming (`RES-004`); a household with several televisions is the normal case,
  not the exception.
- Discovery latency (`PERF-015`) is a client-visible property: a television that
  waits too long for the server assumes that no server exists.

---

## 9. Error and recovery behaviour

| Situation | Expected behaviour |
| --- | --- |
| A volume disappears during a scan | The affected subtree is marked unavailable, the scan continues elsewhere, the reason is reported (`REL-002`) |
| A client stops reading | The transfer is cancelled after the timeout; its buffer is released; no other client is affected (`RES-004`) |
| Memory pressure | Background work yields; active streams continue; if the limit is reached, the server explains it instead of failing silently (`RES-007`) |
| The process is killed | On restart the catalogue is consistent or safely rebuilt; media originals are untouched (`REL-001`) |
| The machine sleeps and wakes | Interfaces are refreshed, listeners rebound, discovery re-advertised on permitted interfaces only; no listener leak (`REL-003`) |
| The configuration is invalid | The server refuses to start and names the field; it never starts with a partially applied configuration (`REL-005`) |

---

## 10. Acceptance criteria and traceability

| ID | Scenario | Requirement | Verification | Planned test |
| --- | --- | --- | --- | --- |
| **AC-01** | Given 10 000 indexed objects and an idle server, when the memory is sampled after one minute, then the resident set is below 50 MB | `PERF-001` | Benchmark | `crates/phairdlna-core/benches/idle_memory.rs` |
| **AC-02** | Given a cold start with a prebuilt index, when the process is started, then it is ready to answer discovery within 2 seconds, measured over 30 trials at p50 and p95 | `PERF-002` | Benchmark | `crates/phairdlna-core/benches/startup.rs` |
| **AC-03** | Given a tree of 10 000 unchanged files, when an incremental rescan runs, then no file content is read | `PERF-003` | Automated test with an instrumented reader | `crates/phairdlna-core/tests/incremental_scan.rs` |
| **AC-04** | Given metadata extraction disabled, when 1 000 files are indexed, then at least 200 files per second are indexed | `PERF-004` | Benchmark | `crates/phairdlna-core/benches/index_throughput.rs` |
| **AC-05** | Given a container with 10 000 children, when it is browsed, then the first page is returned within 100 ms | `PERF-005` | Benchmark | `crates/phairdlna-core/benches/browse_latency.rs` |
| **AC-06** | Given any browse request, when it is served, then the filesystem is not touched | `PERF-008` | Automated test with an audit hook | `crates/phairdlna-core/tests/browse_does_not_scan.rs` |
| **AC-07** | Given ten concurrent direct-play clients and a full scan running, when the clients seek, then the seek latency stays within the agreed regression threshold of the stream-only baseline | `RES-001`, `RES-003` | Benchmark | `crates/phairdlna-core/benches/streaming_under_scan.rs` |
| **AC-08** | Given the scanner is yielding, when the load drops, then it resumes without oscillating more than once within the hysteresis band | `RES-003` | Automated test with a synthetic clock | `crates/phairdlna-core/tests/scheduler_hysteresis.rs` |
| **AC-09** | Given one client that stops reading, when another client streams, then the second client's throughput is unchanged | `RES-004` | Integration test | `crates/phairdlna-core/tests/slow_client_isolation.rs` |
| **AC-10** | Given an exhausted worker pool, when a request arrives, then the server refuses it with a clear error instead of allocating without limit | `RES-002`, `RES-007` | Negative test | `crates/phairdlna-core/tests/resource_exhaustion.rs` |
| **AC-11** | Given the server is killed during an index write, when it restarts, then the catalogue is consistent or rebuildable and the media checksums are unchanged | `REL-001` | Fault injection | `crates/phairdlna-cli/tests/crash_recovery.rs` |
| **AC-12** | Given a network share disappears during a scan, when the scan continues, then the root is reported as unavailable and is not deleted from the configuration | `REL-002` | Integration test | `crates/phairdlna-core/tests/offline_volume.rs` |
| **AC-13** | Given a seven-day soak with a mixed workload, when the signals are sampled, then memory, descriptors and thread count are flat and no crash occurred | `REL-004` | Soak test | `scripts/soak.sh` |
| **AC-14** | Given an invalid configuration, when the server starts, then it refuses to start and names the offending field, and the previous configuration is unchanged | `REL-005` | Negative test | `crates/phairdlna-cli/tests/invalid_configuration.rs` |
| **AC-15** | Given an active transfer, when the user switches the presentation mode, then the transfer continues uninterrupted | `REL-006` | Integration test | `crates/phairdlna-mac/tests/mode_switch.rs` |
| **AC-16** | Given the same workload on two machines with different storage, when both are measured with the same method, then the numbers are comparable because the hardware profile is recorded | `PERF-010`, `PERF-011` | Manual review | `docs/05-quality/PERFORMANCE-BUDGETS.md` |
| **AC-17** | Given a root that contains a network share which stops answering, when a browse request arrives for a folder on another root, then the request is served within the browse budget and the stuck scan is reported | `PERF-014`, `RES-003` | Integration test with a stalling share | `crates/phairdlna-core/tests/slow_share_does_not_block.rs` |
| **AC-17** | Given ten simultaneous direct-play clients, when the streams run and are then closed, then the resident memory does not grow without bound and the memory attributable to one stream stays within the documented bound | `PERF-006`, `PERF-007` | Benchmark | `crates/phairdlna-core/benches/ten_streams.rs` |

---

## 11. Test strategy and dependencies

**Dependencies**

- The scheduler must exist before `RES-001` … `RES-009` can be implemented; it is a
  core module, not an optimisation layer added later.
- The soak script and the fault-injection harness are prerequisites for `REL-001` …
  `REL-006`.
- The reference hardware profile (OD-11) must be named before the first measurement.

**Layers**

| Layer | What it covers |
| --- | --- |
| Unit | The scheduler's priority rules, the hysteresis, the buffer bound |
| Property | The scheduler never lets a background task starve a foreground one; a bounded pool never allocates beyond its bound |
| Integration | Streaming under scan, slow clients, disconnect storms, exhausted pools |
| Fault injection | `SIGKILL` during index writes and configuration changes |
| Soak | Seven days, flat trends, outbound-connection audit |
| Benchmark | Every budget in `../05-quality/PERFORMANCE-BUDGETS.md` |

**Dependencies on other features:** F-06 (the transfer path), F-09 (the index), F-12
(configuration), F-13 (the service), F-15 (the interface).

---

## 12. Open questions and decisions

| ID | Question | What would close it |
| --- | --- | --- |
| F18-Q-01 | What are the exact hysteresis thresholds, and how are they calibrated? | A measurement on the reference hardware; until then the values stay in the configuration as documented defaults |
| F18-Q-02 | What is the memory bound per active transfer (`PERF-007`)? | The first measurement; the value is then written into the budget document |
| F18-Q-03 | Which reference hardware defines the release targets? | Owner decision OD-11 |
| F18-Q-04 | Should the server reduce background work on battery power, and how does it learn the power state? | A platform capability decision; the macOS implementation can read the power source, but the policy is ours |
| F18-Q-05 | Is the seven-day soak the right duration, or is seven days a floor rather than a target? | Evidence from the first soak: if the leak rate is below the detection threshold, a shorter soak proves nothing |
