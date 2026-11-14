# F-07 — Client compatibility profiles

Status: Draft Owner role: Product/Requirements Version/date: 0.1 / 2026-11-14 Requirements:
INT-100, INT-101, INT-102, INT-103, INT-104

## 1. Purpose and user story

A **control point** is the client application that browses a DLNA server — the DLNA app on a
television, a phone app, VLC. **DMR** is the device that plays what the control point selects.
Control points and renderers differ: some are strict about the metadata, some are lenient, some
stop at the first thing they do not recognise. Each such difference is a **quirk**: a documented
deviation of a client from the specification.

A **workaround** is what the server does about a quirk. A **device profile** is the declarative
record that says "when this client asks, do that". The mechanism has to satisfy three rules at
once: it may only change behaviour for the client that needs it, it may never break a normative
requirement for anybody else, and the user must be able to switch it off without recompiling.

**User story.** As a user, I want my television to be able to browse my library even though that
television's DLNA app deviates from the specification, so that I do not have to buy new hardware;
and I want to be able to turn that special handling off, so that a future firmware update does not
leave me with a server that only works for one device.

## 2. Scope

**In scope** — the declarative profile store and its schema; the rules that decide which profile
  applies to a request; the evidence rule that a profile may only exist with a recorded
  justification; the switch that disables a workaround through configuration; and the diagnostics
  command that replays a stored discovery request.

**Explicitly out of scope** — the device-test programme itself and the contents of the
  interoperability matrix (`../05-quality/INTEROPERABILITY-MATRIX.md` owns both; INT-102 covers the
  recording duty only); the *specific* workarounds for specific clients, because none is known yet
  (see §12); the DIDL-Lite generation the workarounds influence (F-05); the SSDP answer builder
  (F-01); the media HTTP layer (F-06); the access-control list (CFG-009).

## 3. Functional requirements

**INT-100** — The server shall maintain a declarative profile of known client quirks, each with
  the client family, the observed symptom, the workaround and the evidence that justified it.
  [DESIGN] The profile store is **data, not code**. Each record carries: `id`, `client_family`,
  `match` (the observable signal that identifies the client), `symptom`, `workaround` (a named
  adjustment drawn from an enumerated vocabulary), `evidence` (a pointer to a row or entry in the
  interoperability matrix) and `since`. The columns of the quirk table in the matrix are exactly
  client family, symptom, workaround and evidence, and the matrix states that a quirk needs all four
  (INTEROPERABILITY-MATRIX §5). [DESIGN] The shipped profile set is **empty at the baseline**,
  because no device test has been performed yet (INTEROPERABILITY-MATRIX §4.2, §5); the mechanism is
  specified now so that the first device round has somewhere to write.

**INT-101** — A quirk workaround shall never violate a normative requirement of the UPnP or DLNA
  specifications for other clients. [DESIGN] A workaround may only (a) change output that no
  specification makes mandatory, or (b) narrow the change to the requests that match the quirk's own
  client. It shall not remove a required response header, weaken a MUST of the UPnP Device
  Architecture or the AV specifications for clients that do not match, switch off a security control
  (SEC-001 … SEC-008, SEC-070 … SEC-072), or alter the transport, the discovery behaviour or the
  access-control list. [DESIGN] The rule is enforced mechanically: every profile declares the
  requirement IDs its adjustment touches, and a profile that would touch a MUST is rejected when the
  store is loaded. [SPEC] The DLNA layer stays **additive and separable**: plain
  `http-get:*:<mime>:*` remains producible and plain UPnP clients keep working in both modes
  (standards §6.6, §8.8 N9; INT-078).

**INT-102** — The server shall record every device-test result — device model, firmware version,
  client application, media format and observed outcome — in the interoperability matrix. [DESIGN]
  The matrix defines the device row as device, firmware, client, format, outcome, notes, date and
  by, and states that a row without a device is not a row (INTEROPERABILITY-MATRIX §4, §6). This
  requirement is a duty of *process*, not a quota: the baseline matrix contains no device row, and
  that is the correct state until a device round happens. [DESIGN] The server contributes the
  artefacts a test session needs: the diagnostics report (F-14), the replay corpus (§4) and the
  per-request log with redacted client identity (OPS-007, SEC-094).

**INT-103** — The server shall provide a diagnostics command that replays a stored `M-SEARCH`
  request and prints the response it would produce. [DESIGN] The command takes a stored request — a
  fixture file or a captured datagram — feeds it through **the same request-parsing and
  answer-building path the socket uses**, and prints the request and the answer byte for byte as
  they would be sent. It is the counterpart of the SSDP header dump of INT-015, which starts from
  the answer; here the answer is derived from a real request. [DESIGN] The stored-request corpus
  starts empty and grows with every real client observed (INTEROPERABILITY-MATRIX §2); the command
  therefore has to say so clearly when the corpus is empty rather than fail obscurely. [DESIGN] The
  replayed answer is produced *as the server would answer now*, with the current profile set and the
  current configuration applied, so the command is a diagnosis of the running configuration.

**INT-104** — The server shall allow the user to disable a quirk workaround globally without
  editing source code. [DESIGN] The configuration file carries a `quirks` section that names the
  profile ids that are disabled. An unknown key is an error (CFG-013) and an unknown profile id is a
  configuration error naming the id and the file (CFG-002), so that a typo can never silently
  re-enable a workaround. Disabling a profile restores the plain, specification-conforming behaviour
  for that client; it does not disable the client, the format or the feature the profile was about.
  [DESIGN] "Globally" means for every request that would otherwise match the profile, not "for one
  address": the switch is a server-wide setting, not a per-client override.

## 4. Technical requirements

- **Profile store.** [DESIGN] Declarative records, one per quirk, in a form the configuration schema can
  validate (typed, versioned, unknown keys rejected — CFG-002, CFG-003, CFG-013). The store is read at start-up
  and re-read when the configuration changes; a malformed store aborts start-up with a message naming the file,
  the field and the expected type (UX-004, CFG-002).
- **Where the logic lives.** [DESIGN] The profile *data* belongs with the other declarative configuration; the
  *selection* logic is consulted by the HTTP layer once per request and the resulting profile is passed down to
  the DIDL-Lite writer (F-05) and, where relevant, to the discovery answer builder (F-01). The selection logic
  never changes the transport, the socket, the timeout or the access check. [OPEN] The workspace layout in
  ARCHITECTURE §2 does not list a module for this; see §12.
- **Profile record.** `id` (stable, lowercase, hyphenated), `client_family` (free text from the matrix, e.g. a
  television family), `match` (an expression over observable request signals), `symptom` (what the user sees),
  `workaround` (an identifier in the adjustment vocabulary), `evidence` (a pointer into the interoperability
  matrix), `touches` (the requirement IDs the adjustment touches), `since` (the version that introduced it).
- **Adjustment vocabulary.** [DESIGN] The vocabulary is closed and small, so that a profile cannot smuggle in
  arbitrary behaviour: each adjustment is a named, documented change to *output shaping* (for example the fourth
  `protocolInfo` field, or the presence of an extra `res` element). The vocabulary is defined by the features
  that own the affected output (F-05 owns DIDL-Lite; F-04 and F-06 own the transfer signalling), so that F-07
  never reaches into their internals.
- **Matching.** [DESIGN] A `match` expression reads only signals the server already has: the request's
  `User-Agent` (the discovery request carries one — standards §2.2) and the peer's address. It is evaluated
  against a bounded set of profiles: a linear scan over a small, ordered list with substring comparison on a
  length-bounded header value (SEC-007, SEC-011). Most specific match wins; when two matches are equally
  specific the **plain** behaviour wins, because the conservative answer is the one that cannot break a
  conforming client.
- **Evidence gate.** [DESIGN] Loading the store rejects any profile without an `evidence` pointer, any profile
  whose `touches` list names an unknown requirement ID, and any profile whose adjustment is not in the
  vocabulary. The rejected entry is named in the error message. This is the mechanical form of INT-100 and
  INT-101.
- **Disable switch.** [DESIGN] The configuration key is a list of profile ids. A disabled profile is skipped
  during matching, so the affected clients receive the plain behaviour. The configuration module owns the key and
  rejects unknown ids at validation time (CFG-002).
- **Replay command.** [DESIGN] The command reads a stored request from the corpus directory, runs it through the
  discovery request parser and the answer builder, and prints the request, the selected profile and the answer.
  The corpus directory is empty at the baseline; the command prints an explicit "no stored requests" message and
  exits with the "nothing to do" exit code (OPS-014).
- **Privacy.** [DESIGN] The `User-Agent` value used for matching is held in memory for the request only, is
  never written to the index, and is redacted in log records like every other client identifier (OPS-007,
  SEC-094, SEC-093).

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.

- [DESIGN] Target: profile selection adds no measurable latency to a request — budget **≤ 100 µs** per request
  on the reference hardware documented in qa-performance §5.3. Method: `criterion` benchmark over ≥ 30
  iterations, median and p95 reported, with and without a matching profile (PERF-011).
- [DESIGN] Target: the profile store is loaded once per configuration change, not per request. Method: code
  review plus an assertion that no store file is opened while serving a request.
- [DESIGN] Target: the replay command answers within **2 s** for a single stored request. Method: wall-clock
  measurement over ≥ 30 invocations, median and p95 reported.

## 6. Usability and accessibility

- The feature has no graphical surface. Its user-visible effect is that one television works.
- [UX-010] User-facing text shall describe the mechanism in plain language ("this TV needs a different answer")
  and shall not use the words *quirk*, *profile* or *workaround* in the onboarding or status output.
- [UX-004] When a profile is rejected at start-up, the message shall say what happened, why, and what the user
  can do (fix the entry, or remove it), and shall name the profile id.
- [UX-001] Everything here is operable without a GUI: the disable switch is a configuration key, and the
  diagnosis is a command.
- [DOC-005] *Control point*, *renderer*, *quirk*, *workaround*, *device profile* and *User-Agent* are explained
  at first use.

## 7. Security and privacy

- The `User-Agent` header and the peer address are untrusted input (SEC-018). They are compared, never
  interpreted, never logged in raw form and never used to build a path, a URL or a command (SEC-005, SEC-007).
- A profile may not weaken a security control. Path containment (SEC-001 … SEC-008), the loopback-only
  administration surface and its token rules (OPS-020, OPS-021, SEC-070 … SEC-072) are outside the vocabulary
  of adjustments; a profile naming them is rejected by the evidence gate (INT-101).
- A profile may not become an authorisation mechanism. The access-control list is by address (CFG-009) and the
  administration token is independent of it (OPS-021); matching a profile never grants anything.
- The profile store contains no personal data (SEC-093): a client family and a symptom are not personal data.
  A device-specific value that would identify a user (a hostname, a serial number) shall not be stored in a
  profile.
- This feature causes no outbound connection (SEC-091).

## 8. Interoperability

- Normative: the additive and separable DLNA layer — plain `http-get:*:<mime>:*` and the DLNA-flavoured form
  from one code path, plain UPnP clients keep working (standards §6.6, §8.8 N9).
- Normative: the discovery request shape the replay command consumes, including `MAN: "ssdp:discover"` and `MX`
  (standards §2.2, §8.1 D2–D3).
- The interoperability matrix is the project's evidence of compatibility: the quirk table is deliberately empty
  because no device test has been performed (INTEROPERABILITY-MATRIX §4.2, §5). **No client quirk is asserted in
  this document.**
- [OPEN] Which fourth `protocolInfo` field a given client family expects. The Windows Media Center compatibility
  guidance asks for `*` (standards §6.6); our target television families have not been tested. *Closes when:* a
  device round records what each family accepts.
- [OPEN] Whether any target client needs a change at all. *Closes when:* the first device round completes.

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| Profile store is malformed | start-up aborts with a message naming the file, the entry and the field | CFG-002, UX-004 |
| Profile has no evidence pointer | the entry is rejected and named in the start-up error | INT-100 |
| Profile names an unknown requirement ID or an unknown adjustment | the entry is rejected and named | INT-100, INT-101 |
| Configuration disables an unknown profile id | configuration validation error naming the id | INT-104, CFG-002 |
| Configuration contains an unknown key | configuration error, never ignored | CFG-013 |
| Request has no `User-Agent` | the plain behaviour applies | INT-101 |
| Two profiles match equally specifically | the plain behaviour wins | INT-101 |
| Replay corpus is empty | the command prints "no stored requests" and exits with the "nothing to do" code | INT-103, OPS-014 |
| A stored request is unparseable | the command reports the file and the parse error, and exits with the runtime-failure code | UX-005, OPS-014 |

No failure of this feature may take down the HTTP task: a profile that cannot be applied is a
request-level condition, never a panic (UX-005).

## 10. Acceptance criteria and traceability

All criteria below are *planned*; no test file exists yet.

**AC-01** — *Given* the shipped profile store, *when* it is loaded, *then* it is empty, every
  entry that does exist carries `client_family`, `symptom`, `workaround` and `evidence`, and no
  entry exists without an evidence pointer. Requirements: INT-100 · Verification: automated test ·
  Planned location: `crates/phairdlna-core/tests/compat_profiles.rs`

**AC-02** — *Given* a profile record whose `evidence` names nothing in the interoperability
  matrix, *when* the store is loaded, *then* the entry is rejected and the start-up message names
  the profile id and the missing evidence. Requirements: INT-100 · Verification: automated test ·
  Planned location: `crates/phairdlna-core/tests/compat_profiles.rs`

**AC-03** — *Given* a profile whose adjustment would touch a MUST-level requirement, *when* the
  store is loaded, *then* the entry is rejected, and the error names the requirement id.
  Requirements: INT-101 · Verification: automated test · Planned location:
  `crates/phairdlna-core/tests/compat_profiles.rs`

**AC-04** — *Given* a request from a client that matches no profile, *when* the DIDL-Lite document
  is produced, *then* the fourth `protocolInfo` field is the plain `*` and no DLNA-only token
  appears, and the same request answered with every profile enabled produces byte-identical output
  to the request answered with the plain profile. Requirements: INT-101, INT-078 · Verification:
  protocol conformance test · Planned location: `crates/phairdlna-core/tests/didl_lite.rs`

**AC-05** — *Given* a configuration that disables a profile by id, *when* the server starts and a
  request arrives from the client that profile matched, *then* the request receives the plain
  behaviour and the server was not rebuilt. Requirements: INT-104 · Verification: automated test ·
  Planned location: `crates/phairdlna-cli/tests/cli_config.rs`

**AC-06** — *Given* a configuration that disables a profile id that does not exist, *when* the
  configuration is validated, *then* validation fails with a message naming the id, and *given* an
  unknown configuration key, *then* validation fails as well and the key is not ignored.
  Requirements: INT-104, CFG-002, CFG-013 · Verification: automated test · Planned location:
  `crates/phairdlna-cli/tests/cli_config.rs`

**AC-07** — *Given* a stored `M-SEARCH` request in the corpus, *when* the replay command is
  invoked, *then* it prints the request and the response the server would produce, and the response
  is byte-for-byte the answer the socket would send for that request. Requirements: INT-103 ·
  Verification: protocol fixture · Planned location: `crates/phairdlna-core/tests/ssdp_replay.rs`

**AC-08** — *Given* an empty corpus, *when* the replay command is invoked, *then* it prints an
  explicit message that no stored request is available and exits with the "nothing to do" exit code.
  Requirements: INT-103, OPS-014 · Verification: automated test · Planned location:
  `crates/phairdlna-cli/tests/cli_diagnostics.rs`

**AC-09** — *Given* a completed device test, *when* the result is recorded, *then* the matrix row
  carries device model, firmware version, client application, media format and observed outcome, and
  the outcome is one of the documented status values. Requirements: INT-102 · Verification: manual
  protocol · Planned location: `qa/device-matrix.toml`

**AC-10** — *Given* a request whose `User-Agent` is absent, *when* it is served, *then* the plain
  behaviour applies and no error is raised. Requirements: INT-101 · Verification: protocol
  conformance test · Planned location: `crates/phairdlna-core/tests/compat_profiles.rs`

**AC-11** — *Given* a request from an address that the access-control list refuses, *when* it
  arrives, *then* it is refused exactly as before, regardless of any profile it would match.
  Requirements: INT-101, CFG-009 · Verification: protocol conformance test · Planned location:
  `crates/phairdlna-core/tests/compat_profiles.rs`

**AC-12** — *Given* a request that is logged, *when* the log record is produced, *then* the
  `User-Agent` value appears redacted, like any other client identifier. Requirements: INT-102,
  SEC-094, OPS-007 · Verification: automated test · Planned location:
  `crates/phairdlna-core/tests/compat_profiles.rs`

**AC-13** — *Given* the server with a profile store in which every profile is disabled, *when* a
  request is served, *then* the output is identical to the output of a build with an empty profile
  store. Requirements: INT-104, INT-101 · Verification: automated test · Planned location:
  `crates/phairdlna-core/tests/compat_profiles.rs`

**AC-14** — *Given* two profiles that match a request equally specifically, *when* the request is
  served, *then* the plain behaviour applies. Requirements: INT-101 · Verification: automated test ·
  Planned location: `crates/phairdlna-core/tests/compat_profiles.rs`

## 11. Test strategy and dependencies

**Planned test locations** (none of these files exists yet):

| Purpose | Planned path |
| --- | --- |
| Profile store validation, evidence gate, matching | `crates/phairdlna-core/tests/compat_profiles.rs` |
| Selection logic and vocabulary unit tests | `crates/phairdlna-core/src/compat/` (inline `#[cfg(test)]` module) — *module name pending the decision in §12* |
| Replay of a stored discovery request | `crates/phairdlna-core/tests/ssdp_replay.rs` |
| Configuration switch behaviour | `crates/phairdlna-cli/tests/cli_config.rs` |
| Device behaviour per client family | `qa/device-matrix.toml` |

**Method.** The profile store, the matching rules and the adjustment vocabulary are pure data and
pure functions, so they are exercised without a socket and without a device. The replay command is
driven with a hand-written stored `M-SEARCH` fixture; its output is compared with the answer the
socket path produces for the same request, so the two cannot drift. A device round is the only way
to populate the profile store, and it is recorded in the matrix (INTEROPERABILITY-MATRIX §6).

**Dependencies.** F-01 (the discovery request and answer this feature replays and may shape), F-05
(the DIDL-Lite output a profile may adjust), F-04 and F-06 (the transfer signalling a profile may
adjust), the configuration module (CFG-002, CFG-013), the interoperability matrix, and the `qa`
device matrix.

## 12. Open questions and decisions

1. **[OPEN] The quirk table is empty, and that is deliberate.** No device test has been performed
   (INTEROPERABILITY-MATRIX §4.2, §5), so no client quirk is asserted anywhere in this document. *Closes when:* the
   first device round is recorded and the first evidence-backed profile exists.
2. **[OPEN] How a client is identified.** A `User-Agent` substring, the peer address, or both — and what happens
   when a client changes its `User-Agent` between firmware versions. The matrix requires the client application
   *and version* to be recorded (INTEROPERABILITY-MATRIX §4, §6), which suggests the version must be part of the
   match. *Closes when:* the first device round provides the `User-Agent` strings of the tested clients.
3. **[OPEN] Where the profile store lives and who may edit it.** INT-104 requires *disabling* a workaround
   without editing source code, which a configuration override satisfies. Whether a user may also *add* a profile
   of their own — turning the store into a user-editable file — is a separate decision with a support cost.
   *Closes when:* the owner decides.
4. **[OPEN] The module for this feature.** ARCHITECTURE §2 lists the modules of `phairdlna-core` and has no entry
   for compatibility profiles; the logic fits into `config` (data) plus `http` (selection), or needs a new
   module and an amendment to the workspace layout. *Closes when:* the architect records the decision; the
   ADR-0003 rule "small orthogonal modules" argues for a narrow one.
5. **[OPEN] Plain or DLNA fourth `protocolInfo` field per client family.** The Windows Media Center guidance
   asks for `*` (standards §6.6); the DLNA layer asks for the token set (standards §6.4). *Closes when:* a device
   round shows what each family accepts. Until then the plain form is the default, because it is the one that
   cannot break a conforming client (INT-101).
6. **[OPEN] Where the stored-request corpus lives.** The matrix states that the corpus starts empty and grows
   with every real client observed (INTEROPERABILITY-MATRIX §2), but not whether the files are committed to the
   repository or kept in a user-supplied directory. *Closes when:* the owner fixes the location and the licensing
   of captured client requests.
