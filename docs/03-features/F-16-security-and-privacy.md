# F-16 — Security hardening and privacy guarantees

Status: Draft

Owner role: Product/Requirements

Version/date: 0.1 / 2026-11-14

Requirements: INT-057, INT-058, INT-059, SEC-010, SEC-011, SEC-012, SEC-013, SEC-014, SEC-015, SEC-016, SEC-017, SEC-018, SEC-060, SEC-061, SEC-062, SEC-063, SEC-064, SEC-065, SEC-066, SEC-067, SEC-068, SEC-069, SEC-070, SEC-071, SEC-072, SEC-090, SEC-091, SEC-092, SEC-093, SEC-094, SEC-095, SEC-096
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved (see `docs/09-research/`).
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.

## 1. Purpose and user story

This feature is the **promise the product makes about the user's data and the user's machine**, written down so that an implementation can be measured
against it and an auditor can check it.

The server listens on a **local network**, speaks two protocols designed in 2002 for a trusted home, parses XML and HTTP that arrive from strangers,
and reads a file system that may be shared, synchronised or hostile. The realistic attacker is not a remote hacker: it is a laptop on the same café
Wi-Fi, a compromised smart device, a malicious media file that arrived through a sync client, or a web page the user opened (SECURITY-MODEL §1).

Three commitments follow, and this specification states each of them as a requirement with a test that fails without it:

1. **The server is hard to attack from the network.** Every parser is bounded, every framing rule is enforced by us rather than by the framework, and
   no input can make the server reach outside itself.
2. **The server is private.** It stores only what it needs locally, it talks to nothing but the local network, and it ships no analytics, no
   advertising identifier and no third-party service.
3. **The server is honest about its licence.** AGPL §13 obliges a program that users interact with over a network to offer its corresponding source. A
   LAN is a network, so the running server serves the source of the exact build it is running, without authentication and at no charge.

**User story.** As a user, I want my media server to be safe to leave running on a shared network and safe to run on a machine that holds my personal
files; I want it to make no outbound connections I did not ask for; and I want anybody on my network to be able to download the source of the version
I am running, because the project is free software.

## 2. Scope

**In scope** — the network and parsing hardening requirements; the resource bounds; the isolation of untrusted media parsing; the privacy guarantees
(no telemetry, no egress, no third-party service, minimal storage, redaction, file permissions); the three protocol refusals (GENA eventing, Internet
Gateway Device, network self-update); the AGPL source offer as a product feature; and the mapping of every mitigation and every threat row in
`../04-security/SECURITY-MODEL.md` to a requirement and to a **planned negative test**.

**Explicitly out of scope** — filesystem containment rules `SEC-001` … `SEC-008`, which belong to F-08 and are referenced here as dependencies; the
administration surface's token, CSRF and `Host` rules `OPS-020` … `OPS-027`, which belong to F-14 (this specification states the security *guarantee*
and the negative test, and F-14 owns the implementation); the HTTP delivery semantics `INT-080` … `INT-099` (F-06); the protocol surface itself (F-01
… F-05); the dependency and licence *tooling*, which is F-19 (`SUP-001` … `SUP-009`); the code-signing and notarisation plan (`SUP-009`, F-19); and
the privacy *narrative*, which lives in `../04-security/PRIVACY.md`.

## 3. Functional requirements

**SEC-010** — The XML parser shall not process a DTD, shall not resolve external entities and shall bound the nesting depth and the total document
size. [DESIGN] The decision is ours, not the parser's: `quick-xml` *surfaces* `DocType` and `GeneralRef` events, so the server rejects any document
containing a doctype and any general entity reference (DEPENDENCY-LICENSE-REVIEW §5.2; M-02). The byte limit is enforced **before** the parser sees
the data, because the parser performs internal allocations first.

**SEC-011** — The server shall bound the size of every accepted HTTP request line, header block and body. [DESIGN] Each bound is a configured value
with a documented default and a documented failure behaviour (SEC-060); exceeding a bound is a rejection, never a truncation.

**SEC-012** — The server shall bound the number of concurrent connections and the number of concurrent transfers. [DESIGN] Both bounds are
configurable (CFG-008); exceeding the connection bound closes the connection, exceeding the transfer bound refuses the transfer with a clear error
(FUN-043).

**SEC-013** — The server shall answer an unauthenticated or malformed request with a generic error that reveals no internal path, version detail or
stack trace. [DESIGN] One error formatter produces the response; the detailed cause goes to the log, redacted (SEC-094), never to the wire.

**SEC-014** — The server shall not follow a redirect to a non-local address, and shall reject a callback URL that is not a validated local address.
[DESIGN] The redirect rule applies to every outbound request the server could make; the callback rule is the GENA validation of INT-057.

**SEC-015** — The server shall validate and normalise every URL path segment it accepts, and shall decode percent-encoding exactly once. [DESIGN]
Decoding twice is the classic double-encoding bypass (`%252e%252e%252f`); the rule is decode once, then reject `..`, both separator characters, NUL
and control characters (M-05).

**SEC-016** — The server shall enforce a read and write timeout on every connection. [DESIGN] Three deadlines: header, whole request, and idle between
writes. A connection that misses one is closed (SEC-TH-14).

**SEC-017** — The server shall never bind a non-loopback port other than the media port without an explicit configuration entry. [DESIGN] The media
port is the one exception, because the product's purpose is to be reachable by a television; every other listener — the administration surface in
particular — is loopback-only unless the user has written down an override (CFG-005, `OPS-020`).

**SEC-018** — The server shall treat an SSDP or SOAP peer as untrusted and shall perform no action on its behalf other than answering it. [DESIGN] A
peer never causes a file access, a catalogue read, an outbound connection or a state change. This is the boundary that keeps discovery from becoming
an attack surface.

**SEC-060** — Every parser in the server shall have an explicit input size bound and a documented failure behaviour. [DESIGN] The bound and the
failure behaviour are part of the parser's public contract and are stated in its doc comment; a parser without them does not merge (PROJECT-STANDARDS
§2.2).

**SEC-061** — The server shall bound the number of entries in a single DIDL-Lite response and the size of the generated XML document. [DESIGN] A
`Browse` page is capped at the configured page size and the generated document at a configured byte size, whichever is reached first (INT-032;
SEC-TH-22).

**SEC-062** — The server shall run metadata extraction with a time limit per file and shall abandon a file that exceeds it. [DESIGN] The limit is per
file, configurable (CFG-008), and the abandoned file is reported rather than silently skipped.

**SEC-063** — The server shall bound the number of open file descriptors used by transfers and shall release them on cancellation. [DESIGN] A transfer
occupies one slot in a bounded table; cancelling the transfer, or the client closing the connection, releases the descriptor (INT-089).

**SEC-064** — The server shall bound the size of its on-disk cache and shall evict entries when the limit is exceeded. [DESIGN] The cache is
disposable and evictable (ARCHITECTURE §9); eviction is least-recently-used and never touches a media file (FUN-018).

**SEC-065** — The server shall never allocate memory proportional to a client-supplied length field without validating it against the actual file
size. [DESIGN] A `Range` header, a `Content-Length` and a declared media length are all checked against the real size before a buffer exists.

**SEC-066** — The server shall parse media metadata and generate thumbnails in a separate, sandboxed process with no network access, a bounded memory
and CPU budget, a time limit, and shall quarantine a file whose parsing fails. [DESIGN] A media file is "a file from someone else that happens to be
on the user's disk". The child process has no network access, a memory and CPU ceiling and a time limit; a file that makes it crash, hang or exceed
its budget is quarantined and the parent is unaffected (M-06; SEC-TH-19, SEC-TH-20).

**SEC-067** — The server shall bound the declared dimensions of an image before allocating any buffer for it, so that a file declaring enormous
dimensions cannot exhaust memory. [DESIGN] The declared dimensions are read from the header, compared against the configured pixel ceiling, and
refused before any buffer is sized.

**SEC-068** — The server shall treat playlist entries as untrusted paths, shall resolve and contain them, and shall refuse entries that use a network
scheme. [DESIGN] A playlist is a file from someone else: its entries go through the same containment rule as a media URL, and a `http://` or `rtsp://`
entry is dropped, not fetched (M-05; SEC-TH-21).

**SEC-069** — The server shall not resolve or follow a macOS Finder alias as if it were a symbolic link; aliases shall be treated according to the
same containment rules as symbolic links. [DESIGN] A Finder alias is a regular file to the kernel, so canonicalising the path does not resolve it; it
is resolved explicitly and then contained by the same rule. *Closes when:* the macOS fixture described in F-08 §12 item 3 exists.

**SEC-070** — The server shall offer the corresponding source of the running build, including its dependency tree and build recipe, over HTTP without
authentication, as required by the AGPL network-use clause. [DESIGN] The route is served from the media origin, is unauthenticated because an
authenticated offer is not an offer, and serves a fixed artefact; no request parameter influences what is served. The offer includes the build
scripts, the lock file and the exact build recipe — AGPL §13 requires "scripts to control those activities" (security-landscape §9.4). Implementation:
F-14 (`OPS-016`, `OPS-017`).

**SEC-071** — The server shall accept only a `Host` header that matches a configured local host name, so that a DNS-rebinding attack from a web page
cannot reach the server. [DESIGN] Loopback is not a boundary: a name that resolves to `127.0.0.1` and then to the LAN address reaches the
administration surface unless the `Host` header is checked against an allowlist (SEC-TH-23). Implementation: F-14 (`OPS-022`).

**SEC-072** — The server shall require a custom, non-standard request header on every state-changing administration request, in addition to the CSRF
token. [DESIGN] A custom header cannot be set by a cross-origin HTML form; combined with the CSRF token it makes a cross-site state change require
both a token and a header the browser will not add (M-06; SEC-TH-24). Implementation: F-14 (`OPS-023`).

**SEC-090** — The product shall contain no telemetry, no analytics and no crash reporting to any third party. [DESIGN] The rule extends to the
dependency graph: a dependency that opens a network connection on its own is prohibited (PRIVACY §8).

**SEC-091** — The product shall make no outbound network connection other than the SSDP multicast announcements and responses, and the media
connections it serves, unless the user explicitly starts a feature that requires internet access. [DESIGN] This is checked by an
**outbound-connection audit** — the server runs with egress blocked except the local network, and a normal browse-and-play session must produce zero
unexpected connections (PRIVACY §4). The eventing opt-in of INT-057 is the only feature in the baseline that can create an outbound connection, and it
is off by default.

**SEC-092** — The product shall not attempt to configure the user's router, and shall not speak the UPnP Internet Gateway Device protocol. [DESIGN]
The server never opens a port forwarding, never a tunnel, never a relay, and never learns the WAN address (INT-058, SEC-TH-10).

**SEC-093** — The product shall store no personal data beyond local configuration, the catalogue index and local logs. [DESIGN] The index stores only
what a browse response uses — identity, path, size, modification time, media type and display metadata — and no hash of the file content, no play
count, no "last played" (PRIVACY §5). There is no behavioural profile.

**SEC-094** — Logs and diagnostics shall be redactable, and the redaction shall apply to file system paths, client addresses and device identifiers.
[DESIGN] Redaction replaces each value with a stable pseudonym so that a log can be shared without exposing the library layout. A redacted log
contains no absolute path (OPS-007, UX-011).

**SEC-095** — The catalogue database shall be a local file with user-only permissions, and the server shall not create it inside a shared or
world-readable location by default. [DESIGN] The index lives in the user's own data directory with owner-only permissions (ARCHITECTURE §9).

**SEC-096** — The product shall ship no third-party service, no advertising identifier and no device fingerprint. [DESIGN] The device identity is a
random UUID generated locally, not derived from hardware (PRIVACY §3).

**INT-057** — The server shall reject every GENA `SUBSCRIBE` request unless the user has explicitly enabled eventing, and shall reject a callback URL
that is not a validated address on the local network. [DESIGN] GENA eventing is **off by default**, because the `SUBSCRIBE` callback is a
request-forgery gadget: the "CallStranger" class (CVE-2020-12695) showed that a UPnP device can be made to open a connection to an arbitrary URL,
including on the internet, and CERT/CC's guidance to manufacturers is to disable subscription by default and require an explicit opt-in
(security-landscape §5.6; SEC-TH-05). When the user does enable it, the callback URL is parsed as an absolute `http`/`https` URL, resolved to a
literal IP at subscription time and pinned, refused if the address is loopback, link-local (`169.254.0.0/16`, `fe80::/10`), unique-local, multicast,
unspecified or outside the served local networks, refused if it uses a non-default port or userinfo; the number of subscriptions and the event rate
per subscription are capped; and a redirect during delivery is a failure, never a follow.

**INT-058** — The server shall never implement the UPnP Internet Gateway Device actions (`AddPortMapping`, `DeletePortMapping`,
`GetExternalIPAddress`, `NewInternalClient`) and shall answer them with an error. [DESIGN] The dispatcher is an allowlist; an action that is not on it
is an error, whatever it is named. IGD is a different product that happens to share the discovery protocol, and its history includes a
remote-code-execution vulnerability (CVE-2014-8361) (security-landscape §3.1; SEC-TH-10).

**INT-059** — The server shall not implement a network self-update mechanism; updates shall be installed by the user through their platform's package
manager. [DESIGN] A self-update channel is a supply-chain vector and each applied update is a new "modified version" with a fresh AGPL §13 obligation
(security-landscape §9.4; SEC-TH-27).

**[OPEN] SEC-019 is assigned to this feature but is not defined in the baseline.** The approved catalogue's network-and-parsing block runs `SEC-010` …
`SEC-018`; there is no `SEC-019` anywhere in the baseline, so there is no requirement text to elaborate here. This specification therefore covers
`SEC-010` … `SEC-018` and leaves `SEC-019` unresolved rather than inventing a requirement for it. *Closes when:* the owner either withdraws the
identifier or adds a defined requirement with that ID to the catalogue. See §12 item 9.

## 4. Technical requirements

- **Boundaries.** Three, as in the security model: the wire (untrusted LAN), the file system (semi-trusted media), and the loopback administration
  surface (not a boundary on its own — hence `Host` allowlist plus CSRF plus custom header plus bearer token).
- **Bounded before parsed.** Every input is size-checked before a parser sees it, because a streaming parser performs internal allocations before the
  consumer observes the event (DEPENDENCY-LICENSE-REVIEW §5.2).
- **One error formatter.** The shape required by UX-004 is produced in one place; the client-visible form is generic (SEC-013) and the internal form
  is redacted (SEC-094).
- **Process isolation.** The only process the server ever starts is the media-inspection child, invoked with an argument array and never a shell
  (M-03, SEC-066).
- **The source offer.** Served from the media origin, unauthenticated, as a fixed artefact. It contains the source of the running revision, the
  dependency tree and the build recipe (SEC-070, OPS-016, OPS-017, M-08).

**Mitigation coverage.** Every mitigation of `../04-security/SECURITY-MODEL.md` §4 maps to requirements owned here or elsewhere, and to a planned
negative test.

| Mitigation | Requirements (owner) | Planned negative test |
| --- | --- | --- |
| M-01 SSDP responder hardening | INT-001 … INT-003, INT-009 … INT-011, INT-016, INT-018 (F-01); SEC-018 | `crates/phairdlna-core/tests/ssdp_hardening.rs` |
| M-02 Untrusted XML and SOAP | SEC-010, SEC-015, SEC-060, SEC-061; INT-025 (F-02) | `crates/phairdlna-core/tests/xml_hardening.rs` |
| M-03 No shell, ever | SEC-005 (F-08), SEC-066, INT-058, INT-059 | `crates/phairdlna-core/tests/no_process_spawn.rs` |
| M-04 Strict HTTP framing | SEC-011 … SEC-017 | `crates/phairdlna-core/tests/http_framing.rs` |
| M-05 Path containment | SEC-001 … SEC-004 (F-08); SEC-015, SEC-068, SEC-069 | `crates/phairdlna-core/tests/path_containment.rs` |
| M-06 Isolation of untrusted work | SEC-066, SEC-067, SEC-071, SEC-072; `OPS-021` … `OPS-027` (F-14) | `crates/phairdlna-core/tests/sandbox_isolation.rs`, `crates/phairdlna-admin/tests/admin_api.rs` |
| M-07 Bounded resources | SEC-011, SEC-012, SEC-060 … SEC-065 | `crates/phairdlna-core/tests/resource_bounds.rs` |
| M-08 Supply chain and provenance | INT-059, SEC-070, SEC-090, SEC-096; `SUP-001` … `SUP-008` (F-19) | `scripts/quality-gates.sh`, `crates/phairdlna-cli/tests/source_offer.rs` |

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.

- [DESIGN] Target: the server stays responsive to a legitimate client while 5 000 sockets are open, and the descriptor count stays under its
  configured cap (SEC-TH-15). Method: connection-flood harness with a concurrent legitimate request, descriptor count sampled.
- [DESIGN] Target: peak memory during a hostile-input run stays flat, and the parser byte limit is enforced before allocation (SEC-TH-07, SEC-TH-22).
  Method: RSS sampling over the fuzz and negative-test corpora.
- [DESIGN] Target: an idle server uses less than 1 % CPU averaged over one minute (PERF-013). Method: `top` sampling over 60 s.
- [DESIGN] Target: the media-inspection child is killed at its time limit and never outlives its budget (SEC-062, SEC-066). Method: a fixture that
  makes the child hang; the parent's memory is sampled throughout.

## 6. Usability and accessibility

- Security that the user cannot understand is security the user disables. Every error the user meets follows the "what happened / why / what to do"
  shape (UX-004) and names the setting or the action, not an error code (PLT-017). The redaction toggle is a user-facing choice (SEC-094, UX-011):
  redacted by default for anything that leaves the machine, unredacted only when the user deliberately asks. The source offer is a *feature*, not an
  obligation hidden in a licence file: the diagnostics report and the status output name the revision and how to fetch the source (SEC-070, OPS-017).
  No requirement in this feature may make the product unusable without a graphical interface (UX-001); the security state is visible in the CLI too.
- [DOC-005] *DTD*, *external entity*, *SSRF*, *DNS rebinding*, *CSRF*, *redaction*, *quarantine* and *GENA* are explained at first use.

## 7. Security and privacy

**Threat coverage.** Every row of the threat table in `../04-security/SECURITY-MODEL.md` §5 is mapped below to the requirement that addresses it and
to the **planned negative test** that proves the mitigation works. No test exists yet; the paths are planned.

| Threat | Mitigation | Requirement | Planned negative test |
| --- | --- | --- | --- |
| SEC-TH-01 SSDP reflector/amplifier | M-01 | INT-010, INT-011, SEC-018 | `crates/phairdlna-core/tests/ssdp_hardening.rs` |
| SEC-TH-02 SSDP advertisement forgery | M-01 | INT-010, SEC-018 | `crates/phairdlna-core/tests/ssdp_hardening.rs` |
| SEC-TH-03 SSDP discovery flooding | M-01, M-07 | INT-011, SEC-012 | `crates/phairdlna-core/tests/ssdp_hardening.rs` |
| SEC-TH-04 Multicast membership abuse | M-01 | SEC-012, SEC-063 | `crates/phairdlna-core/tests/ssdp_hardening.rs` |
| SEC-TH-05 GENA callback SSRF (CVE-2020-12695) | M-02, M-01 | INT-057, SEC-014 | `crates/phairdlna-core/tests/eventing.rs` |
| SEC-TH-06 XXE / DTD in SOAP or DIDL | M-02 | SEC-010 | `crates/phairdlna-core/tests/xml_hardening.rs` |
| SEC-TH-07 Entity expansion ("billion laughs") | M-02, M-07 | SEC-010, SEC-060 | `crates/phairdlna-core/tests/xml_hardening.rs` |
| SEC-TH-08 Deeply nested XML | M-02 | SEC-010 | `crates/phairdlna-core/tests/xml_hardening.rs` |
| SEC-TH-09 SOAP action routing abuse | M-02 | SEC-010, SEC-013 | `crates/phairdlna-core/tests/xml_hardening.rs` |
| SEC-TH-10 IGD-style action on a DMS | M-03 | INT-058, SEC-092 | `crates/phairdlna-core/tests/no_process_spawn.rs` |
| SEC-TH-11 Command injection via a SOAP argument | M-03 | SEC-005, SEC-060 | `crates/phairdlna-core/tests/no_process_spawn.rs` |
| SEC-TH-12 HTTP request smuggling | M-04 | SEC-011, SEC-013 | `crates/phairdlna-core/tests/http_framing.rs` |
| SEC-TH-13 Header injection / oversized headers | M-04 | SEC-011, SEC-015 | `crates/phairdlna-core/tests/http_framing.rs` |
| SEC-TH-14 Slowloris | M-04, M-07 | SEC-016, SEC-012 | `crates/phairdlna-core/tests/http_framing.rs` |
| SEC-TH-15 Unbounded concurrency / descriptor exhaustion | M-04, M-07 | SEC-012, SEC-063 | `crates/phairdlna-core/tests/http_framing.rs` |
| SEC-TH-16 Range-header amplification | M-04, M-07 | SEC-065, SEC-061 | `crates/phairdlna-core/tests/http_framing.rs` |
| SEC-TH-17 Path traversal in the HTTP path | M-05 | SEC-015, SEC-001 | `crates/phairdlna-core/tests/path_containment.rs` |
| SEC-TH-18 Symbolic link / alias escape | M-05 | SEC-069, SEC-003 | `crates/phairdlna-core/tests/path_containment.rs` |
| SEC-TH-19 Malformed media metadata | M-06 | SEC-066, SEC-062 | `crates/phairdlna-core/tests/sandbox_isolation.rs` |
| SEC-TH-20 Image dimension bomb | M-06, M-07 | SEC-067, SEC-066 | `crates/phairdlna-core/tests/sandbox_isolation.rs` |
| SEC-TH-21 Playlist path injection | M-05, M-06 | SEC-068, SEC-001 | `crates/phairdlna-core/tests/path_containment.rs` |
| SEC-TH-22 Oversized browse page | M-02, M-07 | SEC-061, SEC-065 | `crates/phairdlna-core/tests/resource_bounds.rs` |
| SEC-TH-23 DNS rebinding | M-06 | SEC-071, SEC-013 | `crates/phairdlna-admin/tests/admin_api.rs` |
| SEC-TH-24 CSRF against the administration API | M-06 | SEC-072 | `crates/phairdlna-admin/tests/admin_api.rs` |
| SEC-TH-25 Administration token disclosure | M-06 | SEC-072, SEC-093 | `crates/phairdlna-admin/tests/admin_api.rs` |
| SEC-TH-26 Signing or notarisation failure | M-08 | SUP-009 (F-19) | `scripts/quality-gates.sh` |
| SEC-TH-27 Dependency supply-chain compromise | M-08 | INT-059, SUP-002, SUP-008 (F-19) | `scripts/quality-gates.sh` |
| SEC-TH-28 AGPL source-offer omission | M-08 | SEC-070, OPS-016, OPS-017 | `crates/phairdlna-cli/tests/source_offer.rs` |

**Privacy properties.**

- **Egress.** The only traffic the server generates is SSDP multicast and unicast replies, the HTTP responses it serves, and — only if the user turns
  it on — GENA event delivery (SEC-091, INT-057). No domain is resolved, no crash report is sent, no "first run" identifier leaves the machine
  (PRIVACY §4).
- **Storage.** The catalogue index, the configuration, the device identity and the logs, and nothing else (SEC-093). The index holds no content hash
  and no viewing history.
- **Identifiers.** The device `UDN` is a locally generated random UUID (SEC-096).
- **Redaction.** Paths, client addresses and device identifiers are replaceable by stable pseudonyms (SEC-094).
- **Permissions.** The catalogue database is user-only (SEC-095); the administration token lives in a `0600` file in a `0700` directory and never in
  the configuration file, a log line or a command-line argument (CFG-014, PRIVACY §3).
- **Source offer.** Unauthenticated, on the media origin, at no charge (SEC-070).

## 8. Interoperability

- Hardening must not break a conforming client. `INT-101` forbids a quirk workaround that violates a normative requirement for other clients, and the
  same rule applies to security policy: a bound that rejects a legal request is a defect, not a protection. The bounds are therefore *configurable
  within a safe range* and their defaults are documented (CFG-004, CFG-008). `SEC-018` and `SEC-013` shape the observable protocol: a malformed
  request gets a generic error, and a legal one gets a normal answer (PC-24, PC-25 in the interoperability matrix). The source offer is an extra route
  on the media origin. It must not be confused with the administration surface and must not expose media bytes (OPS-024).
- [OPEN] Whether a bound low enough to stop a flood is high enough for a legitimate client is an empirical question that needs device evidence.

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| Request exceeds a size bound | rejected with a generic error; the connection is closed | SEC-011, SEC-013 |
| Conflicting framing headers | rejected with `400`; the connection is closed | SEC-011, M-04 |
| A request exceeds its deadline | the connection is closed by the server | SEC-016 |
| The connection cap is reached | the new connection is refused; existing ones continue | SEC-012 |
| The transfer cap is reached | the transfer is refused with a clear error | SEC-012, FUN-043 |
| A `SUBSCRIBE` arrives while eventing is off | rejected; no outbound connection is attempted | INT-057 |
| A `SUBSCRIBE` names a non-local callback | rejected; the egress counter stays at zero | INT-057, SEC-014 |
| An IGD action arrives | answered with an error; no state changes | INT-058, SEC-092 |
| A SOAP body contains a DTD or an entity | rejected before parsing continues; no file is opened | SEC-010 |
| A media file cannot be parsed | the child is killed at its limit; the file is quarantined; the server continues | SEC-066 |
| A file exceeds its declared size | rejected before a buffer is allocated | SEC-065, SEC-067 |
| The cache exceeds its size limit | entries are evicted; the media files are untouched | SEC-064 |
| The catalogue database would be created in a shared location | the server refuses that location | SEC-095 |
| A redirect points outside the local network | the redirect is not followed | SEC-014 |

No hostile input may take down the HTTP or SSDP tasks: a bad request produces a rejected request, never a panic (UX-005).

## 10. Acceptance criteria and traceability

All criteria below are *planned*; no test file exists yet.

**AC-01** — *Given* a SOAP body containing an internal DTD subset and one containing an external entity, *when* each is sent, *then* both are
rejected, no file is opened and no connection is attempted. Requirements: SEC-010, SEC-013 · Verification: negative test · Planned location:
`crates/phairdlna-core/tests/xml_hardening.rs`

**AC-02** — *Given* a document with nested entity definitions and one with 100 000 nested elements, *when* each is parsed, *then* each fails inside
the configured byte and time budget, peak memory stays under the cap and no stack overflow occurs. Requirements: SEC-010, SEC-060, SEC-065 ·
Verification: negative test · Planned location: `crates/phairdlna-core/tests/xml_hardening.rs`

**AC-03** — *Given* 10 000 random `SOAPAction` values, *when* they are sent, *then* each is rejected with an identical status, body length and timing
class, and no handler other than the allowlisted ones is reached. Requirements: SEC-013, SEC-010 · Verification: negative test · Planned location:
`crates/phairdlna-core/tests/xml_hardening.rs`

**AC-04** — *Given* a request carrying `Content-Length` together with `Transfer-Encoding`, a duplicated `Transfer-Encoding`, a `Transfer-Encoding`
that is not `chunked`, a `Content-Length: +5` and a `GET` with a body, *when* each is sent, *then* each is rejected with `400` and the connection is
closed. Requirements: SEC-011, SEC-013 · Verification: negative test · Planned location: `crates/phairdlna-core/tests/http_framing.rs`

**AC-05** — *Given* a request whose header value contains CR, LF or NUL and a request with a 1 MiB header block, *when* each is sent, *then* each is
rejected, no second request is read from the stream and no injected header appears in the response. Requirements: SEC-011, SEC-015 · Verification:
negative test · Planned location: `crates/phairdlna-core/tests/http_framing.rs`

**AC-06** — *Given* 200 connections that each send one byte per second, *when* the idle timeout elapses, *then* every connection is closed and the
accepted-connection count never exceeds its cap. Requirements: SEC-016, SEC-012 · Verification: negative test · Planned location:
`crates/phairdlna-core/tests/http_framing.rs`

**AC-07** — *Given* 5 000 open sockets, *when* a legitimate client makes a request, *then* the server answers it and the descriptor count stays under
the configured cap. Requirements: SEC-012, SEC-063 · Verification: negative test · Planned location: `crates/phairdlna-core/tests/http_framing.rs`

**AC-08** — *Given* 100 overlapping ranges in one request, *when* it is answered, *then* a single response is produced, never a multipart body, and
memory stays flat. Requirements: SEC-065, SEC-061 · Verification: negative test · Planned location: `crates/phairdlna-core/tests/http_framing.rs`

**AC-09** — *Given* a request for `..%2f..%2fetc/passwd`, `%252e%252e%252f`, an absolute path and a NUL byte, *when* each is sent, *then* each is
rejected before any filesystem call and an open-audit hook shows that no file outside the root was opened. Requirements: SEC-015, SEC-001 ·
Verification: negative test · Planned location: `crates/phairdlna-core/tests/path_containment.rs`

**AC-10** — *Given* a symbolic link and a Finder alias inside the library that point at `~/.ssh`, *when* the library is scanned and the targets are
requested, *then* both are refused. Requirements: SEC-069, SEC-003 · Verification: negative test · Planned location:
`crates/phairdlna-core/tests/path_containment.rs`

**AC-11** — *Given* a playlist containing `../../etc/passwd` and `http://evil/x` entries, *when* the playlist is read, *then* both entries are
dropped, no outbound request is made and the remaining entries play. Requirements: SEC-068, SEC-091 · Verification: negative test · Planned location:
`crates/phairdlna-core/tests/path_containment.rs`

**AC-12** — *Given* a malformed media file and a file declaring enormous image dimensions, *when* they are inspected, *then* the parent process is
unaffected, the child is killed at its time limit, the parent's memory never exceeds its cap and both files are quarantined. Requirements: SEC-066,
SEC-067, SEC-062 · Verification: negative test · Planned location: `crates/phairdlna-core/tests/sandbox_isolation.rs`

**AC-13** — *Given* a `SUBSCRIBE` request whose callback is a routable public address and one whose callback is `169.254.169.254`, *when* both are
sent with eventing disabled, *then* both are rejected and the egress counter shows no outbound connection attempt. Requirements: INT-057, SEC-014,
SEC-091 · Verification: negative test · Planned location: `crates/phairdlna-core/tests/eventing.rs`

**AC-14** — *Given* eventing explicitly enabled and a `SUBSCRIBE` whose callback resolves to a loopback, link-local or non-local address, *when* it is
sent, *then* it is rejected and the address is pinned and re-validated on delivery; and *given* a delivery that answers with a redirect, *then* the
redirect is not followed. Requirements: INT-057, SEC-014 · Verification: negative test · Planned location: `crates/phairdlna-core/tests/eventing.rs`

**AC-15** — *Given* the IGD actions `AddPortMapping`, `DeletePortMapping`, `GetExternalIPAddress` and `NewInternalClient`, *when* each is sent, *then*
each returns an error, the set of open ports is unchanged and the process-spawn counter stays at zero. Requirements: INT-058, SEC-092 · Verification:
negative test · Planned location: `crates/phairdlna-core/tests/no_process_spawn.rs`

**AC-16** — *Given* every SOAP argument and every file name is fuzzed with shell metacharacters, *when* the requests are made, *then* the
process-spawn counter stays at zero and no canary file is created. Requirements: SEC-005, SEC-060 · Verification: negative test · Planned location:
`crates/phairdlna-core/tests/no_process_spawn.rs`

**AC-17** — *Given* a spoofed-source `M-SEARCH`, an injected inbound `NOTIFY` and a high-rate flood, *when* they are delivered, *then* the spoofed
search produces zero response packets, the injected `NOTIFY` produces no reaction and changes nothing, and the drop counter rises while the work
counter does not. Requirements: INT-011, INT-010, SEC-018 · Verification: negative test · Planned location:
`crates/phairdlna-core/tests/ssdp_hardening.rs`

**AC-18** — *Given* a page request with `RequestedCount = 2^31-1`, *when* it is answered, *then* the response is capped at the configured page size
and memory stays flat. Requirements: SEC-061, SEC-065 · Verification: negative test · Planned location:
`crates/phairdlna-core/tests/resource_bounds.rs`

**AC-19** — *Given* the administration surface, *when* a request carries a `Host` header that resolves to loopback and then to the LAN address, *then*
the request is blocked and a `Host` mismatch is logged; and *given* a cross-origin state-changing request without a CSRF token, *then* it is refused
with `403` and changes nothing; and *given* a request without the custom header, *then* it is refused. Requirements: SEC-071, SEC-072, SEC-013 ·
Verification: negative test · Planned location: `crates/phairdlna-admin/tests/admin_api.rs`

**AC-20** — *Given* the token file, *when* its permissions are inspected, *then* the file mode is `0600` inside a `0700` directory and the token never
appears in a log line. Requirements: SEC-093, SEC-072, CFG-014 · Verification: automated test · Planned location:
`crates/phairdlna-admin/tests/admin_api.rs`

**AC-21** — *Given* the running server, *when* a normal browse-and-play session runs with egress blocked except the local network, *then* zero
outbound connections are observed; and *given* the tree and the dependency graph, *then* no telemetry, analytics or crash-reporting code is present.
Requirements: SEC-090, SEC-091, SEC-096 · Verification: negative test · Planned location: `scripts/outbound-audit.sh`

**AC-22** — *Given* a log produced with redaction enabled, *when* it is inspected, *then* it contains no absolute path, no client address and no
device identifier in the clear, and each replaced value maps to the same pseudonym on every occurrence. Requirements: SEC-094, OPS-007 · Verification:
automated test · Planned location: `crates/phairdlna-core/tests/redaction.rs`

**AC-23** — *Given* the catalogue database, *when* it is created, *then* it is created in the user's own data directory with owner-only permissions,
and the server refuses a configuration that would place it in a shared or world-readable location. Requirements: SEC-095, SEC-093 · Verification:
automated test · Planned location: `crates/phairdlna-core/tests/resource_bounds.rs`

**AC-24** — *Given* the running server, *when* the source-offer route is fetched without authentication, *then* it returns the source of the exact
running revision, including the dependency tree and the build recipe, and the revision it names is the revision the status output reports.
Requirements: SEC-070, OPS-016, OPS-017 · Verification: automated test · Planned location: `crates/phairdlna-cli/tests/source_offer.rs`

**AC-25** — *Given* the release artefact, *when* `spctl --assess --type execute` and `codesign --verify --deep --strict` are run, *then* both pass;
and *given* the dependency graph, *then* `cargo audit` and `cargo deny` report no un-waived advisory and no incompatible licence. Requirements:
SUP-003, SUP-004, SUP-009 · Verification: licence and advisory gate · Planned location: `scripts/quality-gates.sh`

**AC-26** — *Given* a media file that a client cannot play directly, *when* the client requests it, *then* the server serves the original bytes and
reports the file as incompatible; it never silently transcodes. Requirements: FUN-049, FUN-047 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/transcode_policy.rs`

## 11. Test strategy and dependencies

**Planned test locations** (none of these files exists yet):

| Purpose | Planned path |
| --- | --- |
| SSDP hardening: spoofing, flooding, forged `NOTIFY` | `crates/phairdlna-core/tests/ssdp_hardening.rs` |
| XML/SOAP hardening: DTD, entities, depth, action routing | `crates/phairdlna-core/tests/xml_hardening.rs` |
| HTTP framing, headers, timeouts, concurrency, ranges | `crates/phairdlna-core/tests/http_framing.rs` |
| Containment, alias, playlist entries | `crates/phairdlna-core/tests/path_containment.rs` |
| Sandbox isolation, image bombs, quarantine | `crates/phairdlna-core/tests/sandbox_isolation.rs` |
| Resource bounds: pages, memory, descriptors, cache | `crates/phairdlna-core/tests/resource_bounds.rs` |
| No process spawn; IGD refusal | `crates/phairdlna-core/tests/no_process_spawn.rs` |
| GENA eventing refusal and callback validation | `crates/phairdlna-core/tests/eventing.rs` |
| Log redaction | `crates/phairdlna-core/tests/redaction.rs` |
| Administration surface attacks | `crates/phairdlna-admin/tests/admin_api.rs` |
| Source offer | `crates/phairdlna-cli/tests/source_offer.rs` |
| Outbound-connection audit | `scripts/outbound-audit.sh` |
| Licence, advisory, signing gates | `scripts/quality-gates.sh` |
| Fuzz targets (SOAP, HTTP, range, playlist, media header) | `crates/phairdlna-core/fuzz/` |

**Method.** Every requirement here needs a test that *fails without the mitigation* (SECURITY-MODEL §7). The method for each is: perform the attack,
and assert on a mechanism rather than on the response alone — an open-audit hook for the filesystem, an egress counter for the network, a
process-spawn counter for the shell rule, a descriptor count for the resource bounds. A fuzzing result is reported as `NO CRASH FOUND (not a pass)`
and never as a pass (QUALITY-GATES §1 rule 3).

**Dependencies.** F-01 (SSDP behaviour this hardens), F-02 and F-03 (the SOAP surface and the browse page), F-06 (media delivery, ranges and framing),
F-08 (filesystem containment, `SEC-001` … `SEC-008`), F-09 and F-10 (metadata and artwork in the sandboxed child), F-11 (playlists), F-14 (the
administration surface), F-19 (`SUP-001` … `SUP-009`, the supply-chain gates), and `../04-security/SECURITY-MODEL.md`, `../04-security/PRIVACY.md`,
`../04-security/DEPENDENCY-LICENSE-REVIEW.md`.

## 12. Open questions and decisions

1. **[OPEN] Ownership overlap with F-14.** `SEC-070`, `SEC-071` and `SEC-072` are assigned to F-14 in `../01-product/REQUIREMENTS-INDEX.md` and to
   F-16 by the current assignment. *Proposed reading, used here:* the requirement text and its negative test live in F-16; the implementation (the
   route, the `Host` allowlist, the custom header) lives in F-14. *Closes when:* the owner records one owner per requirement.
2. **[OPEN] SEC-066, SEC-067, SEC-068, SEC-069, SEC-071, SEC-072, INT-057, INT-058, INT-059 and OPS-016 / OPS-017 have no feature in the requirements
   index.** The index shows `—` for them. This specification claims them per the current assignment; the index needs the same update. *Closes when:*
   the index is regenerated.
3. **[OPEN] The eventing opt-in surface.** INT-057 requires an explicit user opt-in for GENA eventing but does not say where that switch lives, nor
   what "a validated address on the local network" means for a multi-subnet home. *Proposed reading, used here:* the validation algorithm of
   security-landscape §5.6 — resolve to a literal IP at subscription time, pin it, re-validate on every delivery, and allow only addresses on the
   networks the server already serves. *Closes when:* the owner confirms and the configuration reference carries the exact rule.
4. **[OPEN] The bounds themselves.** The baseline fixes no number for the request-line limit, the header-block limit, the connection cap, the transfer
   cap, the cache size, the image pixel ceiling or the metadata time limit. *Proposed decision, not in the baseline:* every bound is a named
   configuration value with a documented default, and the defaults are chosen from the hostile-input corpora rather than from the happy path, so that
   a legitimate client never hits them. *Closes when:* the owner fixes the defaults in the configuration reference (F-12).
5. **[OPEN] The macOS alias semantics.** SEC-069 requires aliases to be contained like symbolic links, but the research could not establish what
   `canonicalize` and `O_NOFOLLOW` do to an alias (security-landscape §7.4; F-08 §12 item 3). The threat row SEC-TH-18 is *Open* for that reason.
   *Closes when:* a macOS fixture establishes the behaviour.
6. **[OPEN] The sandbox mechanism.** SEC-066 requires "a separate, sandboxed process with no network access, a bounded memory and CPU budget". On
   macOS the natural mechanism is the `sandbox_init` / Seatbelt profile; on Linux it is a namespace or a bubblewrap profile. The baseline does not
   name a mechanism and this document does not either. *Closes when:* an ADR selects the mechanism per platform.
7. **[OPEN] Whether the source offer may be rate-limited.** SEC-070 requires the offer to be served to any network user without authentication;
   SEC-012 requires a connection cap. A cap that a network user can trip by downloading the archive repeatedly would make the offer unreliable.
   *Proposed reading, used here:* the offer route is exempt from the *media* throughput limits but participates in the connection cap. *Closes when:*
   the owner decides.
8. **[OPEN] Redaction default.** SEC-094 requires redaction to be *possible*; it does not say whether it is on by default. The privacy model describes
   redaction as a user choice. *Proposed reading, used here:* redaction is on for anything the user exports and off for the local log, which the user
   alone can read. *Closes when:* the owner fixes the default.
9. **[OPEN] SEC-019 has no requirement text.** The identifier is assigned to this feature, but the approved baseline defines no `SEC-019` — the
   network-and-parsing block runs `SEC-010` … `SEC-018`. *Proposed reading, used here:* the identifier is left unelaborated rather than filled with an
   invented requirement. *Closes when:* the owner withdraws the identifier or defines a requirement for it in the catalogue.