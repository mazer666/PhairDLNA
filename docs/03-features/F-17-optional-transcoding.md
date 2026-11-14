# F-17 — Optional transcoding

Status: Draft · **Deferred feature**
Owner role: Product/Requirements
Version/date: 0.1 / 2026-11-14
Requirements: FUN-047, FUN-048, FUN-049

## 1. Purpose and user story

**Transcoding** means re-encoding a media file while the server streams it — changing its
codec, its container or its resolution because a client cannot play the original.

PhairDLNA does **not** do this by default, and this feature says so three times over.
The architecture's first design rules say "direct play by default: nothing is
transcoded unless the user asked for it" (ARCHITECTURE §1 rule 6). `FUN-040` requires the
server to serve media by byte range from the original file whenever the client accepts the
format. And `FUN-049` — a **MUST** — requires that the server never silently fall back to
transcoding: a file that cannot be played directly is *reported* as incompatible.

The reason is not technical squeamishness. A transcoder is the single most expensive
thing this product could do: it burns CPU that a direct-play server does not, it breaks
byte-range seeking unless the output is seekable, it multiplies the "what does this file
look like" questions, and — in the usual implementation — it drags a large GPL-licensed
toolchain into a project that promises to ship no FFmpeg and to link no GPL code.

**User story.** As a user, I want my television to be able to play a file that is in a
format it does not support, but only if I have explicitly asked for that, only if I have
chosen what it should be converted to, and I want the server to tell me plainly when a
file cannot be played rather than silently burning my CPU trying.

**Milestone.** This feature is **deferred to milestone M5** ("Optional transcoding:
resource-bounded codec integration, hardware acceleration where feasible, subtitle
optimization", ROADMAP). `FUN-047` and `FUN-048` carry the priority **COULD**, which the
requirements method defines as "explicitly outside the current baseline" (§3). **No
requirement in this feature blocks the baseline**: the baseline server ships with
transcoding absent, and that absence satisfies every requirement below.

## 2. Scope

**In scope** — the opt-in switch and the target-format selection; the invocation of an
external program out of process with an argument array; the resource budget, the bounded
queue and the cancellation path; the reporting of a file as incompatible instead of
transcoding it; and the interaction with the advertised protocol information.

**Explicitly out of scope** — shipping, bundling, downloading or linking **any** codec
library or transcoder, including FFmpeg in any form (DEPENDENCY-LICENSE-REVIEW §1 rule 7);
the in-process re-encoding of media in Rust (this feature is about invoking an *external*
program, not writing an encoder); hardware acceleration (a separate M5 item named in the
roadmap, not a requirement in this baseline); subtitle burning and optimisation; the
choice of a *specific* transcoder program — the project ships none and endorses none
(§12 item 1); the audio/video filters and the target-format catalogue, which belong to
the configuration surface (F-12); and the direct-play path itself (F-06, `FUN-040`).

## 3. Functional requirements

**FUN-047** — The server shall implement optional transcoding only when the user has
explicitly enabled it and selected a target format. [DESIGN] Transcoding is **off by
default** and a *target format* is part of the opt-in, not a separate default: enabling
the feature without naming a target is a configuration error, not a request for a
default. The switch is a configuration value of the same typed, validated, versioned
schema as everything else (CFG-001 … CFG-004) and it is off in the shipped defaults, so
an empty configuration file yields a direct-play server (CFG-004). With the feature off,
the server contains no code path that can start a transcoder at all.

**FUN-048** — The transcoder, if enabled, shall run in a separate process with an
explicit resource budget, a queue bound and a cancellation path. [DESIGN] Four properties,
each separately checkable:

- **Separate process.** The transcoder is an external program the user supplies; it is
  started with an argument array and **never through a shell** (SEC-005, M-03). No
  `sh -c`, no string concatenation of a file name into a command line, no
  `Command::new("sh")`.
- **Explicit resource budget.** The child is given a memory ceiling and a CPU limit, a
  wall-clock time limit per job, and a limit on the number of concurrent transcoder
  processes. The budget is configuration, and its defaults are documented (CFG-008).
- **Bounded queue.** Jobs wait in a bounded queue. When the queue is full, the request is
  refused with a clear error; it is never queued without limit and never blocks the
  event loop. The scheduler arbitrates transcoding against streaming and indexing with
  explicit priority classes, and streaming correctness and control-plane responsiveness
  take precedence over it (RES-001, RES-002).
- **Cancellation path.** When the client closes the connection, when the user cancels from
  the interface, or when the time limit is reached, the child process is terminated, its
  resources are released and the transfer slot is freed (SEC-063, INT-089, UX-013).

**FUN-049** — The server shall never silently fall back to transcoding; a file that cannot
be played directly shall be reported as incompatible. [DESIGN] This requirement is
binding in the baseline and is satisfied there by the *absence* of transcoding: with the
feature off, every file is served as it is, and a client that cannot play it receives an
incompatible-media response rather than a conversion it did not ask for. When the feature
is on, the same rule holds with one explicit exception the user configured: a file that
matches a *user-written* rule is transcoded on purpose, and every other file that the
client cannot play is reported as incompatible. The decision is recorded in the
catalogue, and the user interface names the reason (UX-004). No automatic profile
detection, no "helpful" fallback, no heuristic.

## 4. Technical requirements

- **No shipped transcoder.** The project ships **no FFmpeg** and links no GPL code
  (DEPENDENCY-LICENSE-REVIEW §1 rule 7; security-landscape §9.3). The only acceptable
  shape is: *a user-supplied external binary, invoked out of process with an argument
  array, never a shell.* The server does not download it, does not install it, does not
  verify its licence and does not ship it in a release artefact.
- **Invocation.** The configuration names the program path and an argument *template*
  with fixed placeholders for the input path, the output destination and the target
  format. The server substitutes arguments as separate `argv` entries — never a shell
  string — and the placeholder values are quoted by the operating system, not by us. A
  file name containing a space, a quote or a semicolon is one argument, always.
- **Output.** [DESIGN] Two output shapes are possible and the choice matters for seeking:
  a **pipe** (the transcoder writes to the HTTP response as it runs — cheap, but the
  output is not seekable) or a **cached file** (the transcoder writes into the bounded
  on-disk cache — costs disk, but the result is seekable and can be reused). Because
  `INT-079` permits `DLNA.ORG_OP=01` (byte-range seeking advertised) **only** when
  seeking is actually supported, a piped transcoded resource must be advertised without
  that flag. The default is the cached file, because seeking is the more commonly needed
  feature; the choice is an open question (§12 item 2).
- **Protocol information.** `INT-050` and `INT-051` require `GetProtocolInfo` to list only
  what the server can actually serve, and `INT-064` requires the `res` `protocolInfo` to
  name a MIME type the server really serves. With transcoding off, the advertised
  protocol information is direct play only. With it on, the advertised set changes only
  for the formats the user configured, and a client is never told that a transcoded
  container is available when no transcoder is configured. `INT-029` still requires the
  `ConnectionManager` service to be advertised even when no transcoding is available,
  because clients use it to learn the transfer protocols.
- **Scheduling.** Transcoding jobs go through the shared scheduler with an explicit
  priority class below streaming (RES-001). A slow transcode must never make the server
  look overloaded to another client (RES-004), and a resource shortage must produce a
  meaningful explanation rather than a silently dropped stream (RES-007).
- **The child's own safety.** The transcoder is a user-supplied program, so unlike the
  media-inspection child of `SEC-066` we do not control its code — we control only its
  *invocation*: the argument array, the working directory, the environment, the standard
  streams, the resource limits and the kill path. A transcoder that is killed for
  exceeding its budget leaves no partial file in the catalogue and no half-served
  response.
- **Interaction with the cache.** A transcoded artefact is a **disposable cache** entry
  (ARCHITECTURE §9): bounded in size (SEC-064), evictable, never a user-owned artefact,
  never written inside a media root except through the explicitly configured cache
  directory (FUN-018).

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value, and
none is binding before M5.

- [DESIGN] Target: with transcoding disabled, the server's throughput and memory are
  identical to a direct-play server, and no child process is ever started. Method: assert
  the process-spawn counter stays at zero during a browse-and-play session.
- [DESIGN] Target: the transcoder queue never exceeds its configured bound and a request
  arriving at a full queue is refused within a documented time rather than queued
  indefinitely. Method: instrument the queue depth in a synthetic overload run.
- [DESIGN] Target: cancelling a transfer terminates the transcoder child and releases
  its resources within the documented cancellation budget. Method: timestamp the cancel
  and the child's exit.
- [DESIGN] Target: the server's memory growth per transcoded stream is bounded and
  documented (PERF-007). Method: RSS sampling over a run with the configured number of
  concurrent transcodes.

## 6. Usability and accessibility

- The feature is off by default, so the default experience contains no trace of it
  (UX-001): the interface never shows a transcoding control that does nothing.
- The opt-in is a deliberate act with a stated consequence: the interface names the
  program that will be used, the target format, the resource budget and the fact that
  the server will spend CPU on it.
- A file that cannot be played is reported as **incompatible**, in the user's terms, with
  the reason and what to do next (UX-004, UX-010). The word "transcode" is a technical
  term; the simple mode says "this TV cannot play this file format".
- Long transcoding operations show progress and can be cancelled (UX-013, MB-002).
- [DOC-005] *Transcoding*, *direct play*, *codec*, *container*, *bitrate* and *pipe* are
  explained at first use.

## 7. Security and privacy

- **No shell, ever.** A transcoder invocation is an argument array; no command line is
  ever built from a file name or a metadata value (SEC-005, M-03). A file whose name
  contains shell metacharacters is one argument and cannot become a command.
- **The user-supplied program is outside our trust boundary.** It is invoked with a
  minimal environment, a defined working directory and bounded resources. The server
  does not assume it is well behaved; it bounds it.
- **No egress.** A transcoder that needs the internet is the user's business, not the
  server's: the product makes no outbound connection other than SSDP and the media it
  serves (SEC-091), and the transcoder is started with no network need of its own.
- **The media roots stay read-only.** Transcoded output goes to the disposable cache
  directory, never into the user's library (FUN-018, SEC-064).
- **No silent behaviour.** Because a conversion is expensive and visible on the network,
  the absence of a silent fallback is also a privacy property: the server never spends
  the user's CPU on something they did not ask for (FUN-049).
- **Licence.** Because the project ships no transcoder and links no codec library, the
  AGPL source offer and the dependency audit are not affected by this feature
  (SEC-070, SUP-002).

## 8. Interoperability

- When transcoding is off, the server's interoperability profile is exactly the
  direct-play profile of F-06 and F-07: the advertised `protocolInfo` names the real
  container and MIME type of the file.
- When transcoding is on, the server advertises the transcoded form **only** for the
  formats the user configured, and the DLNA signalling stays truthful:
  `DLNA.ORG_OP=01` is emitted only when byte-range seeking is genuinely supported
  (`INT-079`), and `DLNA.ORG_PN` is emitted only when the content really conforms to
  that DLNA Media Format Profile (`INT-079`).
- A client that insists on seeking may refuse a non-seekable transcoded resource. That
  is a client behaviour, and the honest response is the correct `DLNA.ORG_OP` value and a
  clear report — not a fake capability.
- The interoperability matrix records, per client family, whether transcoding was used
  in the device test; a row without that note is a direct-play row (INT-102).

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| Transcoding disabled (default) | the file is served as it is; no child process exists | FUN-047, FUN-040 |
| Enabled but no target format selected | configuration error at start-up naming the field | FUN-047, CFG-002 |
| The configured program does not exist or is not executable | the job fails with a readable error naming the program; the server keeps running | UX-004, UX-005 |
| The client cannot play the original file and no rule matches | the file is reported as incompatible; nothing is converted | FUN-049 |
| The queue is full | the request is refused with a clear error; nothing is queued without limit | FUN-048, RES-002 |
| The transcoder exceeds its memory or CPU budget | the child is terminated; the transfer ends with an error; no partial artefact is kept | FUN-048, RES-007 |
| The transcoder exceeds its wall-clock limit | the child is terminated and the job is reported as timed out | FUN-048, SEC-062 |
| The client disconnects mid-transcode | the child is terminated and the transfer slot is released | FUN-048, SEC-063, INT-089 |
| The cache is full | entries are evicted; an active job is not evicted | SEC-064 |
| The transcoder writes a malformed output | the output is validated before it is advertised; a malformed one is discarded and the file is reported as incompatible | FUN-049 |

No transcoding failure may crash the server, corrupt the catalogue or alter a source
file (REL-001, RES-007).

## 10. Acceptance criteria and traceability

All criteria below are *planned*; no test file exists yet, and they are **not binding
before M5**.

**AC-01** — *Given* the shipped default configuration, *when* a client plays a file whose
format the client cannot handle, *then* the server serves the original bytes, no external
program is started, the process-spawn counter stays at zero and the file is reported as
incompatible. Requirements: FUN-047, FUN-049, FUN-040 · Verification: automated test ·
Planned location: `crates/phairdlna-core/tests/transcode_policy.rs`

**AC-02** — *Given* a configuration that enables transcoding but names no target format,
*when* the configuration is validated, *then* start-up aborts with a message naming the
field, the file and the accepted values, and no transcoder is started. Requirements:
FUN-047, CFG-002 · Verification: automated test · Planned location:
`crates/phairdlna-cli/tests/cli_config.rs`

**AC-03** — *Given* transcoding enabled with a target format, *when* a file matching a
user-written rule is requested, *then* the configured external program is started with
the input path, the output destination and the target format as **separate arguments**,
and the response body is the program's output. Requirements: FUN-047, FUN-048 ·
Verification: automated test with a stub program · Planned location:
`crates/phairdlna-core/tests/transcode_policy.rs`

**AC-04** — *Given* a file name containing a space, a quote and a semicolon, *when* it is
transcoded, *then* the program receives it as exactly one argument and no shell is
involved. Requirements: FUN-048, SEC-005 · Verification: negative test · Planned
location: `crates/phairdlna-core/tests/transcode_policy.rs`

**AC-05** — *Given* transcoding enabled, *when* more jobs arrive than the queue bound
allows, *then* the excess requests are refused with a clear error, the queue depth never
exceeds its bound and the server keeps answering browse requests. Requirements: FUN-048,
RES-002 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/transcode_budget.rs`

**AC-06** — *Given* a transcoder that exceeds its memory or CPU budget, *when* the limit
is reached, *then* the child is terminated, the transfer ends with an error, the server
keeps running and no partial artefact is kept. Requirements: FUN-048, RES-007 ·
Verification: negative test · Planned location:
`crates/phairdlna-core/tests/transcode_budget.rs`

**AC-07** — *Given* a client that disconnects during a transcoded transfer, *when* the
connection closes, *then* the child process is terminated, the transfer slot is released
and the descriptor count returns to its pre-transfer value. Requirements: FUN-048,
SEC-063, INT-089 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/transcode_budget.rs`

**AC-08** — *Given* transcoding enabled and a client that cannot play the original file
of a file that matches no user rule, *when* the client requests it, *then* no
transcoder is started, the file is reported as incompatible and the reason is visible to
the user. Requirements: FUN-049, UX-004 · Verification: automated test · Planned
location: `crates/phairdlna-core/tests/transcode_policy.rs`

**AC-09** — *Given* transcoding enabled, *when* `GetProtocolInfo` is read, *then* it
lists only the containers and MIME types the server can actually deliver in its current
configuration, and the `res` `protocolInfo` of a transcoded item names the delivered
type. Requirements: INT-050, INT-051, INT-064, FUN-047 · Verification: protocol
conformance test · Planned location: `crates/phairdlna-core/tests/connection_manager.rs`

**AC-10** — *Given* a transcoded resource served from a pipe, *when* its `res` element is
rendered, *then* `DLNA.ORG_OP=01` is **not** emitted, and a seekable cached resource
does emit it. Requirements: INT-079, FUN-048 · Verification: protocol conformance test ·
Planned location: `crates/phairdlna-core/tests/didl_lite.rs`

**AC-11** — *Given* a release artefact of any milestone, *when* it is inspected, *then* it
contains no transcoder binary, no codec library and no reference to a bundled FFmpeg, and
the dependency list contains no codec crate. Requirements: SUP-002, SUP-003, SUP-007 ·
Verification: licence and advisory gate · Planned location: `scripts/quality-gates.sh`

## 11. Test strategy and dependencies

**Planned test locations** (none of these files exists yet):

| Purpose | Planned path |
| --- | --- |
| Opt-in policy, incompatible reporting, argument array | `crates/phairdlna-core/tests/transcode_policy.rs` |
| Queue bound, resource budget, cancellation | `crates/phairdlna-core/tests/transcode_budget.rs` |
| Configuration validation for the opt-in | `crates/phairdlna-cli/tests/cli_config.rs` |
| Protocol information with and without transcoding | `crates/phairdlna-core/tests/connection_manager.rs` |
| DLNA signalling for transcoded resources | `crates/phairdlna-core/tests/didl_lite.rs` |
| No transcoder in a release artefact | `scripts/quality-gates.sh` |

**Method.** The transcoder is an **external program**, so the tests use a **stub
program** written for the test — one that echoes its arguments, one that produces a known
output, one that allocates without bound, one that never exits, and one that exits with a
failure. Testing against a stub keeps the test deterministic and keeps a real codec out
of the repository. The stub is a test fixture, not a shipped artefact.

**Dependencies.** F-06 (the delivery path that a transcoded stream replaces),
F-12 (the configuration that carries the opt-in and the budget), F-14 (the control
actions that cancel a job), F-16 (the no-shell rule, the resource bounds and the
outbound-connection audit), F-19 (`SUP-002`, `SUP-003` and the release-artefact check),
and the scheduler of RES-001.

## 12. Open questions and decisions

1. **[OPEN] Which external programs, if any, are documented as known-good.** The project
   ships none, and it should not *endorse* a specific one without a licence review of
   that program in its default build — the research records that FFmpeg's own LGPL
   compliance depends on the configure line and on optional GPL parts being disabled
   (security-landscape §9.3). *Proposed decision, not in the baseline:* the project
   documents the **command-template mechanism** and leaves the choice of program to the
   user, with a note in the documentation that the user is responsible for the licence of
   the program they run. *Closes when:* the owner decides whether a "known-good programs"
   page is acceptable.
2. **[OPEN] Pipe or cached output.** A pipe is cheap but not seekable; a cached file
   costs disk but can be sought and reused. `INT-079` makes the consequence visible to
   clients. *Proposed decision, not in the baseline:* cached file by default, pipe
   available for users who prefer to spend no disk. *Closes when:* the owner picks one
   and the configuration reference carries it.
3. **[OPEN] How a "user-written rule" selects a file.** `FUN-049` forbids a silent
   fallback but permits an explicit rule; the baseline does not say what a rule matches
   on. *Proposed reading, used here:* a rule names a source container/codec, a target
   format and an optional client profile; a file that matches no rule is reported as
   incompatible. *Closes when:* the owner fixes the rule shape in the configuration
   reference.
4. **[OPEN] The priority of FUN-049 versus the milestone of its feature.** `FUN-049` is a
   **MUST** but is scheduled for M5, and the requirements method says a milestone cannot
   be accepted while a MUST requirement of that milestone is open (§3). *Proposed
   reading, used here:* `FUN-049` is binding from the baseline, because a server that
   never transcodes satisfies it; only `FUN-047` and `FUN-048` are deferred to M5.
   *Closes when:* the owner confirms, or the index marks `FUN-049` as satisfied by the
   baseline.
5. **[OPEN] Interaction with the cache budget.** A cached transcoded artefact competes
   with thumbnails and artwork for the same bounded cache (SEC-064). Whether transcoded
   artefacts get their own budget, and what happens to them when it is full, is not
   decided. *Closes when:* the owner fixes the cache policy.
6. **[OPEN] Hardware acceleration.** The roadmap names it as an M5 item, but a
   hardware-accelerated path is usually a vendor library with a native toolchain, which
   `SUP-006` makes an owner decision. *Closes when:* the owner decides whether an
   owner-approved native dependency is ever in scope.
