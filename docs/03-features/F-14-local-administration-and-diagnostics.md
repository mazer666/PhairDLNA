# F-14 — Local administration and diagnostics

Status: Draft
Owner role: Product/Requirements
Version/date: 0.1 / 2026-11-14
Requirements:
OPS-016, OPS-017, OPS-020, OPS-021, OPS-022, OPS-023, OPS-024, OPS-025, OPS-026, OPS-027, UX-011,
UX-012, SEC-070, SEC-071, SEC-072
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.

## 1. Purpose and user story

The **administration surface** is a small HTTP interface, reachable only from the machine the
server runs on, that shows what the server is doing and lets the user steer it: rescan a folder,
pause a scan, read the configuration. It is optional — the media server is complete without it.

The **diagnostics report** is a structured, redacted snapshot of the running server that a user
can paste into a bug report.

The **source offer** is different in kind: it is a licence obligation, not a convenience. The
project is AGPL-3.0-or-later, and AGPL §13 says that a modified version offered to users
interacting with it over a network must give those users an opportunity to receive the
corresponding source, at no charge, from a network server. A DLNA server on a LAN interacts with
users over a network, so the source offer is a product feature: anybody on the network who can
reach the server can fetch the source of the exact build that is running.

**User story.** As a user, I want a way to see what the server is doing and to ask it to re-read a
folder without juggling command-line flags; I want that interface to be reachable only from my own
machine, even though my browser visits plenty of pages; and because this software is free, I want
anyone who talks to my server to be able to download the source of the version I am running.

## 2. Scope

**In scope** — the loopback administration listener and its request rules (token, CSRF token,
  custom header, `Host` allowlist, rate and size limits); the first-run explanation; the set of
  control actions the surface offers; the diagnostics report; the display of connected clients; the
  source offer route and the reporting of the source revision.

**Explicitly out of scope** — the macOS menu bar application and any graphical interface (a
  separate feature); remote administration of any kind; serving media bytes over the administration
  origin (OPS-024); the UPnP and DLNA protocol surface, which never mentions the administration
  surface (F-01, F-02); telemetry, analytics and crash reporting of any kind (SEC-090); and the
  licence and supply-chain tooling itself (SUP-001 … SUP-008).

## 3. Functional requirements

**OPS-016** — The server shall expose an unauthenticated route that serves the corresponding
  source of the exact running build, so that a network user can obtain it at no charge as the
  licence requires. [DESIGN] The route is served from the **media origin** — the listener that is
  already reachable on the network — and it is unauthenticated, because an offer that requires
  authentication is not an offer (security-landscape §9.4). The route serves a fixed, immutable
  artefact; no request parameter influences what is served.

**OPS-017** — The server shall report the exact source revision it was built from in its status
  output and in the source offer. [DESIGN] The revision is embedded at build time, so the running
  binary and the status output cannot disagree. The source offer names the revision it corresponds
  to, which is what makes "the corresponding source of your version" verifiable.

**OPS-020** — The administration surface shall bind to the loopback interface only by default and
  shall refuse to start when configured otherwise without an explicit override. [DESIGN] Two
  listeners are used — `127.0.0.1` and `::1` — on a port that is not the media port. A non-loopback
  address requires an explicit configuration entry; without it the server refuses to start (CFG-005
  says the same thing about the administration interface).

**OPS-021** — The administration surface shall require a per-installation random token for every
  state-changing request. [DESIGN] The token is generated on first use, stored in a file with
  owner-only permissions inside the user's own data directory, and never appears in the
  configuration file (CFG-014), in a command line, in a URL or in a log record. It is compared in
  constant time, and a failed comparison is reported without revealing which part was wrong
  (security-landscape §6.2).

**OPS-022** — The administration surface shall validate the `Host` header against the loopback
  names to resist DNS-rebinding attacks. [DESIGN] The allowlist is `localhost`, `127.0.0.1`, `[::1]`
  and a configured local host name; any other `Host` is refused **before routing**. This is the
  defence against a page in the user's browser that rebinds its own host name to the server's
  address and then issues same-origin requests to it (security-landscape §6.1, §6.2 rule 2).

**OPS-023** — State-changing administration requests shall require a CSRF token bound to the
  session. [DESIGN] The token is bound to the session that obtained it and is validated in constant
  time; a request without a valid token is refused without touching any state.

**OPS-024** — The administration surface shall not expose media file contents, only metadata and
  control actions. [DESIGN] The administration origin serves no media bytes at all: a media URL on
  it is answered with a generic "not found". The separation exists so that a rebinding attack cannot
  use the administration origin to exfiltrate the user's media (security-landscape §6.2 rule 8).

**OPS-025** — The administration surface shall be optional; the server shall run completely
  without it. [DESIGN] The surface lives in its own optional crate, `phairdlna-admin` (ARCHITECTURE
  §2). Nothing in discovery, browsing or media delivery depends on it, and the server starts and
  serves media with it absent or disabled.

**OPS-026** — The administration surface shall expose a clear "what is this, who can see it, how
  to turn it off" explanation in the first-run output. [DESIGN] The explanation answers those three
  questions in plain language; when the surface is disabled, the first-run output says so and says
  where to enable it.

**OPS-027** — The administration surface shall apply the same rate limiting and request-size
  limits as the media surface. [DESIGN] Both surfaces read the same limit table, so there is one
  place where a limit is defined and one place where it is changed (SEC-011, SEC-012, SEC-016).

**UX-011** — Diagnostics shall be copyable in one action and shall not contain personal data
  unless the user enables redaction off deliberately. [DESIGN] The report is produced by a single
  command whose output is plain text, so it can be copied with the terminal's own copy command; the
  macOS application offers the same report as a "copy diagnostics" action. By default the report is
  redacted — paths, client addresses and identifiers appear as stable pseudonyms (OPS-007, SEC-094);
  turning redaction off is an explicit act by the user.

**UX-012** — The server shall tell the user which client is currently connected and what it is
  streaming. [DESIGN] The information comes from the connection table (F-04) and from the media
  layer: the client's identity, the address it connected from and the title of the object it is
  playing. Addresses are shown in the status output for the user and redacted in the log (SEC-094).

**SEC-070** — The server shall offer the corresponding source of the running build, including its
  dependency tree and build recipe, over HTTP without authentication, as required by the AGPL
  network-use clause. [DESIGN] The offer contains the source of the exact revision the binary was
  built from, the dependency tree (`Cargo.lock` and the embedded tree produced by `cargo auditable`,
  security-landscape §9.1) and the build recipe, because "the Corresponding Source … means all the
  source code needed to generate, install, and (for an executable work) run the object code and to
  modify it, including scripts to control those activities" (AGPL-3.0 §0, quoted in
  security-landscape §9.4). [DESIGN] The clause itself: AGPL-3.0 §13 — *"if you modify the Program,
  your modified version must prominently offer all users interacting with it remotely through a
  computer network (if your version supports such interaction) an opportunity to receive the
  Corresponding Source of your version by providing access to the Corresponding Source from a
  network server at no charge, through some standard or customary means of facilitating copying of
  software."*

**SEC-071** — The server shall accept only a `Host` header that matches a configured local host
  name, so that a DNS-rebinding attack from a web page cannot reach the server. [DESIGN] Same
  allowlist as OPS-022; the check is the first thing a request meets, before any routing and before
  the body is read.

**SEC-072** — The server shall require a custom, non-standard request header on every
  state-changing administration request, in addition to the CSRF token. [DESIGN] A cross-origin page
  can issue a "simple" request without a preflight, but it cannot add a custom header without one;
  because the server sends no CORS headers at all, the preflight fails and the real request never
  happens (security-landscape §6.2 rule 3).

## 4. Technical requirements

- **Crate.** `phairdlna-admin` is an optional crate that depends on `phairdlna-cli`, which depends on
  `phairdlna-core`; the dependency direction is one-way and is checked by the crate-boundary gate
  (ARCHITECTURE §2). The core knows nothing about it.
- **Listeners.** The administration surface opens its own listeners on `127.0.0.1` and `::1`, on a port
  distinct from the media port. It is not advertised: it appears in no device description and in no SSDP
  answer (security-landscape §6.2 rule 10). The surface is a separate origin from the media surface, so the
  browser's same-origin rules keep the two apart.
- **Request pipeline.** [DESIGN] The order is fixed: (1) `Host` allowlist; (2) method and size limits; (3)
  route lookup; (4) for a state-changing method — bearer token, CSRF token, custom header; (5) the action
  itself; (6) a generic error renderer. Nothing in step 4 or 5 runs before step 1 passes.
- **Credentials.** The bearer token arrives in an `Authorization` header, never in the query string
  (security-landscape §6.2 rule 6). The CSRF token arrives in a header as well. The custom header is a fixed
  non-standard name, for example `X-Phair-Admin`. All three comparisons are constant-time. A log record about a
  rejected request names the reason and a truncated hash of the token, never the token itself.
- **Token file.** The token file has mode `0600` inside a directory with mode `0700`, in the per-user data
  directory of the platform (ADR-0003, `UserEnvironment`). The server checks the mode at start-up and refuses to
  serve the administration surface from a file that other users can read, explaining why (CFG-014, SEC-013).
- **No CORS.** The server sends no `Access-Control-Allow-Origin`, no `Access-Control-Allow-Headers` and no
  `Access-Control-Allow-Credentials` header, ever. A preflight request is answered without them.
- **Limits.** The administration surface reads the same limit table as the media surface: request-head size,
  body size, concurrent connections, in-flight requests, read and write timeouts (SEC-011, SEC-012, SEC-016,
  security-landscape §8).
- **Control actions.** [DESIGN] The action set is closed and small: report status, start a scan, stop a scan,
  list the roots, list the quarantined subtrees, read and write a small set of configuration values. No action
  executes a command, opens a file by an arbitrary path or reaches outside the catalogue (SEC-005, SEC-001).
- **Source offer.** [DESIGN] The offer is a single immutable archive produced at build time and embedded in the
  binary, so that the running build always offers the source of *itself* and never of a checkout that happens to
  be lying around. The route serves it with a `Content-Length` and no `Content-Type` invention. The revision
  is embedded at build time and reported by the status command and by the offer itself. The route ignores every
  request parameter; the artefact it serves never depends on the request.
- **Diagnostics report.** [DESIGN] The report is a structured document containing: the version and the source
  revision; the platform and the detected per-root behaviours; the configured root names with pseudonymised
  paths; the catalogue size and the result of the last scan; the active transfers with their client identities;
  the quarantined subtrees; the list of enabled compatibility profiles; and the configuration with secrets
  removed. It is produced by one command and by one action in the macOS application.
- **Connected clients.** [DESIGN] The status output and the diagnostics report list each open transfer with the
  client's identity and the title of the object being streamed. The media layer records the `User-Agent` of the
  request that started the transfer; the client address is shown to the user in the status output and
  pseudonymised in the log (OPS-007, SEC-094).
- **Errors.** The administration surface answers with generic messages that reveal no path, no version detail
  and no stack trace (SEC-013), and the internal error chain never reaches the client.

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.

- [DESIGN] Target: an administration request is answered in under **50 ms** median on the reference hardware of
  qa-performance §5.3. Method: `criterion` benchmark over ≥ 30 iterations, median and p95 reported (PERF-011).
- [DESIGN] Target: the source offer is served without building an archive per request. Method: code review
  plus an assertion that no archive is created while serving the route.
- [DESIGN] Target: the diagnostics report is produced in under **1 s** for a catalogue of 100 000 objects.
  Method: benchmark over a generated catalogue, median and p95 reported.
- [DESIGN] Target: an idle server with the administration surface enabled stays below 1 % average CPU over one
  minute (PERF-013). Method: `top` sampling over 60 s.

## 6. Usability and accessibility

- The feature has no graphical surface of its own; the macOS application may surface it later, but the command
  line is complete on its own (UX-001).
- [UX-010] The first-run explanation shall use plain language: "this page is only reachable from this
  computer", not "loopback binding" or "CSRF".
- [UX-004] Every refusal the user can meet — a wrong token, a wrong host, a denied folder — shall say what
  happened, why and what to do next.
- [UX-011] The diagnostics report is one command and one copy action; it is the artefact a user attaches to a
  bug report.
- [UX-012] The status output names the connected client and the title it is streaming, in one line per
  transfer.
- [UX-006] Every administration subcommand has `--help`, and the source-offer route is printed by the status
  command so that a user can find it.
- [OPS-026] The first-run output answers the three questions — what is this, who can see it, how to turn it off
  — before the user has to ask.
- [DOC-005] *Loopback*, *rebinding*, *CSRF*, *bearer token* and *corresponding source* are explained at first
  use.

## 7. Security and privacy

- **DNS rebinding** is the threat the administration surface is most exposed to: a page in the user's browser
  can resolve its own host name to the server's address and issue same-origin requests. This class of defect
  exfiltrated media from a shipped media server — "a DNS rebinding issue in ReadyMedia … allows a remote web
  server to exfiltrate media files" (CVE-2022-26505, verified; security-landscape §4.1, §6.1). The defences are
  the `Host` allowlist (OPS-022, SEC-071), the custom header plus CSRF token (OPS-023, SEC-072), the absence of
  CORS headers, and the fact that no media bytes are served from the administration origin (OPS-024).
- **Token handling.** Owner-only file, constant-time comparison, never in a URL, an environment variable, a
  command line or a log record (OPS-021, CFG-014, SEC-TH-25).
- **Resource limits.** The administration surface shares the media surface's limits, so it cannot be used to
  exhaust the host (OPS-027, SEC-012).
- **Least privilege.** The control actions cannot reach outside the catalogue and cannot execute anything
  (SEC-005). The surface is optional and its absence changes nothing about the media server (OPS-025).
- **Generic errors.** No path, version or stack trace leaves the process (SEC-013).
- **The source offer is public by design.** Because it is unauthenticated, it serves a fixed, read-only,
  immutable artefact; no request input influences it, and it exposes nothing about the user's media.
- **Privacy.** No personal data is stored beyond local configuration, the catalogue index and local logs
  (SEC-093); the diagnostics report is redacted unless the user deliberately turns redaction off (UX-011,
  SEC-094); the product makes no outbound connection (SEC-091) and contains no telemetry (SEC-090).

## 8. Interoperability

- The administration surface is **not** part of the UPnP device: it is not in the device description (F-02),
  not advertised over SSDP (F-01), not part of any service description (F-03, F-04), and a DLNA client never
  sees it. The separation is deliberate (security-landscape §6.2 rule 10).
- The source offer is an ordinary HTTP/1.1 `GET` on the media origin and follows the same body rules as the
  rest of that surface: a `Content-Length`, no chunked encoding (standards §8.6 H6, INT-088).
- The administration surface speaks no UPnP protocol; its output is plain text and JSON for the machine
  consumers and human-readable text for the user.

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| `Host` header not in the allowlist | refused before routing, with a generic body; the mismatch is logged | OPS-022, SEC-071 |
| State-changing request without a bearer token | `401` with a generic body; no state is touched | OPS-021 |
| Bearer token present, CSRF token missing or wrong | `403` with a generic body | OPS-023 |
| Bearer and CSRF tokens valid, custom header missing | `403` with a generic body | SEC-072 |
| Token file readable by other users | the administration surface does not start; the reason is explained | OPS-021, CFG-014 |
| Request body larger than the limit | `413` | OPS-027, SEC-011 |
| Too many requests from one address | `429`, the excess dropped | OPS-027, INT-011 |
| Unknown administration route | generic `404` with no path, version or stack detail | SEC-013 |
| Media URL requested on the administration origin | generic `404`; no media byte is served | OPS-024 |
| Source revision missing from the build | the source route answers `503` and the status output explains why | OPS-017, UX-004 |
| Preflight request (`OPTIONS`) | answered without any `Access-Control-*` header | SEC-072 |

No failure of this feature may take down the media server: the administration surface is a
separate, optional component, and an error in it produces a diagnostic, never a panic (UX-005).

## 10. Acceptance criteria and traceability

All criteria below are *planned*; no test file exists yet.

**AC-01** — *Given* a running server, *when* the listening sockets are inspected, *then* the
  administration surface listens on `127.0.0.1` and `::1` only, on a port distinct from the media
  port, and no socket is bound to a LAN address. Requirements: OPS-020 · Verification: automated
  test · Planned location: `crates/phairdlna-admin/tests/admin_api.rs`

**AC-02** — *Given* a configuration that sets the administration surface to a non-loopback address
  without the explicit override, *when* the server starts, *then* it refuses to start and the
  message names the setting. Requirements: OPS-020, CFG-005 · Verification: automated test · Planned location: `crates/phairdlna-cli/tests/cli_config.rs`

**AC-03** — *Given* a state-changing request, *when* it is sent without the bearer token, *then*
  it is answered `401`; *given* a valid bearer token and no CSRF token, *then* it is answered `403`;
*given* both tokens and no custom header, *then* it is answered `403`; and in no case is any state
changed. Requirements: OPS-021, OPS-023, SEC-072 · Verification: automated test · Planned location: `crates/phairdlna-admin/tests/admin_api.rs`

**AC-04** — *Given* a request whose `Host` header names a host that is not in the allowlist,
*when* it arrives, *then* it is refused before routing and before the body is read, and the
response body carries no internal detail. Requirements: OPS-022, SEC-071, SEC-013 · Verification:
negative test · Planned location: `crates/phairdlna-admin/tests/admin_api.rs`

**AC-05** — *Given* any response from the administration surface, *when* its headers are
  inspected, *then* no `Access-Control-Allow-Origin`, `Access-Control-Allow-Headers` or
  `Access-Control-Allow-Credentials` header is present. Requirements: SEC-072 · Verification:
  automated test · Planned location: `crates/phairdlna-admin/tests/admin_api.rs`

**AC-06** — *Given* a media URL requested on the administration origin, *when* it is served,
*then* the answer is a generic `404` and no byte of the media file is returned. Requirements:
OPS-024 · Verification: negative test · Planned location:
`crates/phairdlna-admin/tests/admin_api.rs`

**AC-07** — *Given* a build in which the administration crate is absent or the surface is
  disabled, *when* the server starts, *then* discovery, browsing and media delivery behave exactly
  as they do with the surface enabled. Requirements: OPS-025 · Verification: protocol conformance
  test · Planned location: `crates/phairdlna-core/tests/ssdp_discovery.rs`

**AC-08** — *Given* a first run with the administration surface enabled, *when* the first-run
  output is read, *then* it explains what the surface is, who can reach it and how to turn it off;
  and *given* a first run with the surface disabled, *then* the output says that it is disabled.
  Requirements: OPS-026, UX-010 · Verification: automated test · Planned location:
  `crates/phairdlna-cli/tests/cli_diagnostics.rs`

**AC-09** — *Given* a request body larger than the configured maximum, *when* it is sent to the
  administration surface, *then* it is answered `413`, and the limit in force is the same value the
  media surface uses. Requirements: OPS-027, SEC-011 · Verification: automated test · Planned location: `crates/phairdlna-admin/tests/admin_api.rs`

**AC-10** — *Given* the source-offer route is requested without any credential, *when* it is
  answered, *then* it returns the source archive of the running build. Requirements: OPS-016,
  SEC-070 · Verification: automated test · Planned location:
  `crates/phairdlna-admin/tests/source_offer.rs`

**AC-11** — *Given* the source archive, *when* it is opened, *then* it contains the project source
  of the exact revision, the dependency tree and the build recipe. Requirements: SEC-070 ·
  Verification: automated test · Planned location: `crates/phairdlna-admin/tests/source_offer.rs`

**AC-12** — *Given* the status output and the source offer, *when* both name a source revision,
*then* they name the same revision. Requirements: OPS-017 · Verification: automated test · Planned location: `crates/phairdlna-cli/tests/cli_diagnostics.rs`

**AC-13** — *Given* the diagnostics command, *when* it is run, *then* the output is a single block
  of text that contains no unredacted filesystem path, no client address and no device identifier.
  Requirements: UX-011, SEC-094, OPS-007 · Verification: automated test · Planned location:
  `crates/phairdlna-cli/tests/cli_diagnostics.rs`

**AC-14** — *Given* the user has deliberately switched redaction off, *when* the diagnostics
  command is run, *then* the report contains the real paths, and the command states in its output
  that redaction is off. Requirements: UX-011, SEC-094 · Verification: automated test · Planned location: `crates/phairdlna-cli/tests/cli_diagnostics.rs`

**AC-15** — *Given* a client that is streaming an item, *when* the status output is produced,
*then* it names that client and the title of the item it is streaming. Requirements: UX-012 ·
Verification: protocol conformance test · Planned location:
`crates/phairdlna-cli/tests/cli_diagnostics.rs`

**AC-16** — *Given* the token file, *when* its mode is inspected, *then* it is `0600` inside a
  `0700` directory, and the token itself appears in no log record and in no process argument list.
  Requirements: OPS-021, CFG-014 · Verification: automated test · Planned location:
  `crates/phairdlna-admin/tests/admin_api.rs`

**AC-17** — *Given* an unknown administration route, *when* it is requested, *then* the answer is
  a generic `404` that reveals no path, no version detail and no stack trace. Requirements: SEC-013
  · Verification: negative test · Planned location: `crates/phairdlna-admin/tests/admin_api.rs`

**AC-18** — *Given* the device description and the SSDP answers, *when* they are inspected, *then*
  no administration URL, port or route appears in either. Requirements: OPS-025, SEC-071 ·
  Verification: protocol conformance test · Planned location:
  `crates/phairdlna-core/tests/ssdp_discovery.rs`

## 11. Test strategy and dependencies

**Planned test locations** (none of these files exists yet):

| Purpose | Planned path |
| --- | --- |
| Request pipeline, token, CSRF, custom header, `Host` allowlist, limits | `crates/phairdlna-admin/tests/admin_api.rs` |
| Source offer: archive contents, revision, unauthenticated access | `crates/phairdlna-admin/tests/source_offer.rs` |
| Diagnostics report and status output | `crates/phairdlna-cli/tests/cli_diagnostics.rs` |
| Configuration refusal for a non-loopback administration address | `crates/phairdlna-cli/tests/cli_config.rs` |
| Absence of the administration surface from the UPnP surface | `crates/phairdlna-core/tests/ssdp_discovery.rs` |

**Method.** The administration surface is driven over the loopback interface with a hand-written
HTTP client, so that the `Host` header, the credentials and the custom header can be controlled
exactly. The source archive is inspected as an archive in the test, not as a downloaded file, so
the test asserts on its contents. The "survives without the administration crate" criterion is
checked by a build of the core and the CLI without the optional crate — the crate-boundary gate
(ARCHITECTURE §2) keeps that build honest.

**Dependencies.** F-02 (the device description the administration surface must not appear in),
F-04 (the connection table that names the connected clients), F-06 (the media surface whose limits
are shared), F-07 (the enabled compatibility profiles the diagnostics report lists), F-08 (the
root names and the quarantined subtrees the report contains), the build pipeline that embeds the
revision, and the licence obligations of AGPL-3.0 §13.

## 12. Open questions and decisions

1. **[OPEN] The token's lifetime.** OPS-021 calls it a "per-installation random token", which reads as stable
   for the life of the installation, while the security research recommends rotating it "on restart or on
   explicit request" (security-landscape §6.2 rule 5). A stable token is easier to use; a rotating one limits
   the value of a leaked file. *Proposed decision:* generate per installation, rotate on explicit request, and
   re-generate when the file is replaced. *Closes when:* the owner decides.
2. **[OPEN] How the source archive is produced.** The requirement is that the offer corresponds to the *exact
   running build*. The archive could be produced at build time and embedded in the binary, or produced at
   start-up from an installed source tree. The first is exact by construction; the second is cheaper to build
   but can drift. *Proposed decision:* build-time production, with the artefact and its checksum recorded.
   *Closes when:* the owner confirms it and the build pipeline is designed.
3. **[OPEN] Which listener carries the source offer.** The offer must be reachable by a network user, so it
   cannot live on the loopback-only administration origin; the media origin is the natural home. That is a
   deliberate asymmetry — the offer is public, the administration surface is not — and it should be stated in the
   user-facing text. *Closes when:* the owner confirms the placement.
4. **[OPEN] Whether a browser session is ever involved.** A CSRF token "bound to the session" implies a session
   concept. If the administration surface is only a JSON API consumed by the command line and by the macOS
   application, the session can be the token itself; if a browser interface is added later, a cookie with
   `SameSite=Strict` may be needed. *Closes when:* the administration interface is designed.
5. **[OPEN] How long a disconnected client stays visible.** UX-012 asks which client is connected; a client that
   disconnected a second ago is not connected. *Proposed decision:* the status output lists the open transfers
   and, separately, the last client that disconnected, with a timestamp. *Closes when:* the owner decides.
6. **[OPEN] Whether the diagnostics report lists the compatibility profiles.** It is useful and it names client
   families, which are not personal data (SEC-093), but it does tell a reader which devices use this server.
   *Closes when:* the owner decides whether the list is included by default or only in an extended report.
