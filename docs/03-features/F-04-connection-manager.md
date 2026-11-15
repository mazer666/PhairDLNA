# F-04 — ConnectionManager and transfer protocols

Status: Draft
Owner role: Product/Requirements
Version/date: 0.1 / 2026-11-14
Requirements: INT-050, INT-051, INT-052, INT-053, INT-054, INT-055, INT-056
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved (see `docs/09-research/`).
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.

## 1. Purpose and user story

A **DLNA media server** (the "DMS" — Digital Media Server) holds the user's films, music and photos. A
**control point** is the client application that browses and plays them (a television's DLNA app, a
phone app). A **DMR** (Digital Media Renderer) is the device that decodes and shows the media.

Before a control point can play anything, it must learn *how* the server is willing to send media.
That negotiation is the job of the **ConnectionManager** service
(`urn:schemas-upnp-org:service:ConnectionManager:1`): "Which transfer protocols and media types can
you send me, and what is happening on the connections you have open?"

The answer is expressed as **protocolInfo** — a four-field string
`<transportProtocol>:<network>:<contentFormat>:<additionalInfo>`, for example `http-get:*:video/mp4:*`
(standards §1.3, §4.4): how it is transported (`http-get`), over which network (`*` = any), in which
content format (the MIME type), and with which additional parameters (DLNA tokens or `*`).

**User story.** As a user, I want my television's DLNA app to ask my server what it can
play and receive an *honest* answer, so that my television chooses a format it can decode and starts
playing without a trial-and-error exchange of failed requests.

## 2. Scope

**In scope**

- The `ConnectionManager:1` service description: actions, arguments, state variables with their allowed
  value ranges.
- The five normative actions of ConnectionManager:1 (standards §1.3): `GetProtocolInfo`,
  `PrepareForConnection`, `ConnectionComplete`, `GetCurrentConnectionIDs`, `GetCurrentConnectionInfo`.
- The protocolInfo format, the `Source` / `Sink` lists, the connection table and the rule that no
  connection may remain *pending*.
- The decision to advertise `PrepareForConnection` and `ConnectionComplete` as **optional** actions.

**Explicitly out of scope**

- AVTransport service instances and per-connection AVTransport state (F-06 owns the
  position/duration reporting of an active transfer).
- RTSP, `http-subscribe`, HTTP push and every other transport except `http-get`
  (INT-056); transcoding and format conversion (F-17).
- The SOAP envelope, the SCPD generation machinery and the device description (F-02);
  the ContentDirectory browse semantics (F-03).
- GENA event delivery for the three evented state variables — the baseline contains no
  requirement ID for it (see §12).

## 3. Functional requirements

**INT-050** — The server shall implement `GetProtocolInfo` and return at least the
`http-get` transfer protocol with the MIME types it can actually serve. [SPEC] `GetProtocolInfo`
returns `Source` and `Sink`, each a comma-separated list of protocolInfo strings (standards §1.3, §8.4
M1–M2). The `Source` list shall contain at least one `http-get` entry naming MIME types the catalogue
can currently produce (standards §4.4).

**INT-051** — The `GetProtocolInfo` result shall not advertise a container or MIME type
that the server cannot deliver. [SPEC] A renderer must be able to play a MIME type a DMS advertises
without a DLNA profile ID, and a DMS that advertises a DLNA profile ID must actually serve content
that conforms to it (standards §6.2, PROT-01 / PROT-05). [DESIGN] The `Source` list is derived from
one shared MIME registry — the same table that decides `Content-Type` in F-06 and the third
protocolInfo field in F-05.

**INT-052** — The server shall implement `GetCurrentConnectionIDs` and return an empty
list when no transfer is active. [SPEC] The action is required (R) in the Annex A table (standards
§1.3, §8.4 M4). [OBSERVED] MiniDLNA answers with the literal token `0`; INT-052 asks for an empty list
instead, because a client that counts entries must not count a phantom connection.

**INT-053** — The server shall implement `PrepareForConnection`, `ConnectionComplete`
and `GetCurrentConnectionInfo` in a way that never leaves a connection in a permanently "pending"
state. [SPEC] `Direction` and `Status` have closed value sets — `Input` / `Output` and `OK` /
`ContentFormatMismatch` / `InsufficientBandwidth` / `UnreliableChannel` / `Unknown` (standards §1.3,
§8.4 M6); there is no "pending" status, so a connection must never be reported in a state the client
cannot interpret. [DESIGN] Every `ConnectionID` handed out by `PrepareForConnection` enters the
connection table; `ConnectionComplete`, the end of the HTTP transfer and the shutdown path each remove
it; a reaper removes entries whose transfer ended; an unknown `ConnectionID` yields the UPnP error
`701 No Such Object`. [OBSERVED] MiniDLNA answers `701` for an unknown id and does not implement the
two actions at all.

**INT-054** — The server shall return the same `Source` value in
`GetCurrentConnectionInfo` as the `res` URI it serves for the same object. [SPEC] A connection record
lets a control point correlate an open transfer with the object it browsed. [DESIGN] The URI recorded
for a connection is produced by the same function that writes the `res` element in F-05, so the two
cannot drift; the URL served by F-06 for that URI is byte-for-byte the same URL (INT-090). The wording
of INT-054 is ambiguous about which "Source" is meant — raised in §12.

**INT-055** — The server shall advertise the `http-get` protocol with the `*` wildcard
for the additional header field only when it does not depend on request-specific headers. [SPEC] The
fourth field of protocolInfo is the "additional info" field (standards §1.3, §4.4); `*` states that no
additional header is required. [DESIGN] The plain UPnP form `http-get:*:<mime>:*` is the default; the
DLNA-flavoured form with `DLNA.ORG_*` tokens is an additive variant owned by F-05 (INT-078).
`http-get` delivery here depends on no request-specific header, so `*` is truthful in the plain form.

**INT-056** — The server shall not implement the `http-subscribe` transfer protocol in
the baseline. [SPEC] `http-subscribe` is the UPnP AV push protocol; no action in this feature may
advertise it in the `Source` list.

## 4. Technical requirements

- **Service identity.** `serviceType` `urn:schemas-upnp-org:service:ConnectionManager:1`,
  `serviceId` `urn:upnp-org:serviceId:ConnectionManager` (standards §3.1; the same
  spelling in every shipped implementation). The device description is F-02's
  (INT-021, INT-029, INT-044).
- **Action set in the SCPD.** The SCPD describes every action with its arguments and
  every state variable with its data type and allowed value range (INT-023). Required:
  `GetProtocolInfo`, `GetCurrentConnectionIDs`, `GetCurrentConnectionInfo`. Optional:
  `PrepareForConnection`, `ConnectionComplete` (standards §1.3 Annex A). [DESIGN] We mark
  the two optional actions optional in our SCPD — the choice MiniDLNA and ReadyMedia make
  (standards §1.3, §8.4 M5) — and still answer them if called, because the SOAP
  dispatcher (F-02) must not fail on an unexpected action.
- **protocolInfo grammar.** Exactly four colon-separated fields:
  `<transportProtocol>:<network>:<contentFormat>:<additionalInfo>` (standards §1.3,
  §4.4). Field 1 is `http-get`; field 2 is `*`; field 3 is a MIME type from the shared
  registry; field 4 is `*` (plain mode) or a `;`-separated list of DLNA tokens (DLNA
  mode, F-05). No field may contain a colon.
- **List format and `Sink`.** `Source` and `Sink` are comma-separated lists of
  protocolInfo strings without surrounding whitespace (standards §1.3). The `Sink` list
  may be empty because a server that never receives media has nothing to receive
  (standards §8.4 M3); [OBSERVED] MiniDLNA returns an empty `<Sink></Sink>`, the
  correct shape for a server that only sends media. [DESIGN] We emit an empty `Sink`
  element rather than omitting it; the argument is required in the response.
- **Connection table.** An in-memory, bounded table of records: `ConnectionID`, source
  URI, protocolInfo, `Direction`, `Status`, and the timestamps needed to reap a finished
  connection. It lives in the `upnp` module of `phairdlna-core` and obeys the project's
  resource limits (SEC-012, SEC-061).
- **Direction and Status.** `Direction` ∈ {`Input`, `Output`}; `Status` ∈ {`OK`,
  `ContentFormatMismatch`, `InsufficientBandwidth`, `UnreliableChannel`, `Unknown`}
  (standards §1.3, §8.4 M6). A connection opened to deliver media uses `Output`;
  `Status` is `OK` while the transfer runs and `Unknown` for a connection never bound
  to a transfer.
- **Module boundary.** The SOAP dispatcher lives in `phairdlna-core/src/upnp/` and
  dispatches to plain action handlers with typed request and response values
  (ARCHITECTURE §4.3). The connection table is not reachable from the catalogue, the
  HTTP layer or the filesystem; it holds URIs, never paths.
- **Evented state variables.** `SourceProtocolInfo`, `SinkProtocolInfo` and
  `CurrentConnectionIDs` are evented variables (standards §1.3, §8.4 M7). The baseline
  has no requirement ID for GENA delivery; see §12.

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.

- [DESIGN] Target: `GetProtocolInfo` performs no filesystem and no catalogue access — it answers from
  the in-memory MIME registry and a cached protocol list, and the length of the `Source` list is bounded
  by the number of distinct MIME types the catalogue knows, not by the number of files. Method: code
  review plus an automated test that asserts no catalogue handle is opened.
- [DESIGN] Target: the SOAP response for `GetProtocolInfo` is produced in under 10 ms median on the
  reference hardware documented in qa-performance §5.3, measured with `criterion` (qa-performance §5.2)
  over ≥ 30 iterations, median and p95 reported.

## 6. Usability and accessibility

- The feature has no graphical surface. Its user-visible effect is the list of formats that appears in a
  television's DLNA app; that list is the honest answer of INT-050 and INT-051.
- [UX-010] The user-facing status output (OPS-002) shall be able to show the advertised `Source` list in
  plain language ("formats this server can send"), without protocol jargon in the onboarding text.
- [UX-004] When a client requests a transfer mode or content format the server cannot
  deliver, the error the user eventually sees shall be traceable to a log line naming
  the requested format and the supported ones, with paths redacted (OPS-007, SEC-094).
- [UX-001] Everything here is operable and observable without a GUI. [DOC-005] A reader
  who has never heard of UPnP must be able to follow this document: *control point*,
  *renderer*, *server*, *protocolInfo* and *connection* are explained at first use.

## 7. Security and privacy

- The `Source` list contains MIME types only. It shall never contain a file name, a
  directory name or anything derived from the user's disk layout (PLT-008, SEC-093).
- `PeerConnectionManager` and `PeerConnectionID` are client-supplied strings, treated
  as untrusted input (SEC-018): they are stored, never parsed into a URL, a path or a
  command (SEC-005), and they are bounded in length like every other request-supplied
  value (SEC-011, SEC-007).
- The connection table is bounded; a client that never calls `ConnectionComplete` must
  not be able to grow it without limit (SEC-012, SEC-061).
- No data about connections leaves the machine except in the SOAP response to the
  client that asked (SEC-091); this feature causes no outbound connection. Connection
  logs use the redaction rules of OPS-007.

## 8. Interoperability

- Normative: UPnP AV ConnectionManager:1 and its Annex A action table (standards §1.3);
  the protocolInfo four-field format (standards §1.3, §4.4); the Direction and Status
  value sets (standards §1.3, §8.4 M6).
- [OBSERVED] MiniDLNA and ReadyMedia ship `PrepareForConnection` and
  `ConnectionComplete` as optional (standards §1.3, §8.4 M5); a client that requires
  them would therefore be unusual.
- [OBSERVED] MiniDLNA's `GetCurrentConnectionInfo` returns `RcsID` `-1`,
  `AVTransportID` `-1`, `Direction` `Output` and `Status` `Unknown` for connection `0`;
  our empty-connection behaviour is the same shape with an empty protocolInfo.
- [STANDARD, secondary] The DLNA layer expects the fourth field to carry the DLNA
  profile tokens, while the Windows Media Center guidance asks for `*` in plain UPnP
  mode (standards §6.6). Both modes must be producible from one code path (INT-078).
- [STANDARD, secondary] A renderer must be able to play a MIME type the server
  advertises without a DLNA profile ID (PROT-05) — the server-side reading of INT-051.

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| Unknown SOAP action on this service | UPnP fault `401 Invalid Action` | INT-025 (F-02) |
| Missing or malformed argument | UPnP fault `402 Invalid Args` | INT-025 (F-02) |
| `GetCurrentConnectionInfo` with an unknown `ConnectionID` | UPnP fault `701 No Such Object` | INT-053, INT-039 pattern |
| `PrepareForConnection` with a `Direction` outside {`Input`,`Output`} | UPnP fault `402 Invalid Args` | standards §1.3 |
| `PrepareForConnection` with a `RemoteProtocolInfo` the server cannot source | UPnP fault `402 Invalid Args`; no connection is created | INT-051 |
| Client never calls `ConnectionComplete` | the entry is reaped when its transfer ends or the server shuts down; the table stays bounded | INT-053, SEC-012 |
| Server shutting down with connections open | connections are closed within the bounded shutdown time (OPS-008) | ARCHITECTURE §3 |

No failure of this feature may take down the HTTP task or the SSDP task: a bad SOAP request produces
a fault, never a panic (INT-025, UX-005).

## 10. Acceptance criteria and traceability

All criteria below are *planned*. The criteria that the dispatcher can answer without a socket are
**covered** by the tests listed in §11; the criteria that need the HTTP layer or a real transfer are
still **planned**.

**AC-01** — *Given* a catalogue with MP3, MP4 and JPEG files, *when* a client calls `GetProtocolInfo`,
*then* the `Source` list is non-empty, contains an `http-get` entry and every entry has exactly four
colon-separated fields. Requirements: INT-050 · Verification: protocol conformance test · Planned
location: `crates/phairdlna-core/tests/connection_manager.rs`
— **covered** by `crates/phairdlna-core/tests/connection_manager.rs::the_source_list_has_four_fields_per_entry`.

**AC-02** — *Given* the same catalogue, *when* the `Source` list is compared with the MIME types the
catalogue can produce, *then* no advertised MIME type is missing from the catalogue's MIME registry.
Requirements: INT-051, INT-050 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/connection_manager.rs`
— **covered** by `crates/phairdlna-core/tests/connection_manager.rs::every_advertised_mime_type_is_in_the_registry`.

**AC-03** — *Given* no active transfer, *when* a client calls `GetCurrentConnectionIDs`, *then* the
response contains an empty `ConnectionIDs` list. Requirements: INT-052 · Verification: protocol
conformance test · Planned location: `crates/phairdlna-core/tests/connection_manager.rs`
— **covered** by `crates/phairdlna-core/tests/connection_manager.rs::the_list_of_the_running_transfers_is_empty`.

**AC-04** — *Given* no active transfer, *when* a client calls `GetCurrentConnectionInfo` with a
`ConnectionID` that was never issued, *then* the response is the UPnP error `701 No Such Object`.
Requirements: INT-053 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/connection_manager.rs`
— **covered** by `crates/phairdlna-core/tests/connection_manager.rs::an_unknown_transfer_is_the_fault_701`.

**AC-05** — *Given* a client that called `PrepareForConnection` and never called `ConnectionComplete`,
*when* the transfer has ended and the reaper has run, *then* the `ConnectionID` is no longer listed
and the connection table has not grown beyond bound. Requirements: INT-053 · Verification: automated
test · Planned location: `crates/phairdlna-core/tests/connection_manager.rs`
— **covered** by `crates/phairdlna-core/tests/connection_manager.rs::a_preparation_creates_no_transfer`: the server never opens a connection, so there is none that could stay pending.

**AC-06** — *Given* a connection bound to a media object, *when* `GetCurrentConnectionInfo` returns,
*then* the URI it reports equals the `res` URI of that object and the HTTP layer serves that same URI.
Requirements: INT-054, INT-090 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/connection_manager.rs`
— **planned**: a connection bound to a media object needs the `res` URI of F-05 and the HTTP layer of F-06; neither exists yet.

**AC-07** — *Given* the plain UPnP mode, *when* `GetProtocolInfo` is answered, *then* every `Source`
entry has `*` in the fourth field and no entry mentions `DLNA.ORG_`. Requirements: INT-055, INT-078 ·
Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/connection_manager.rs`
— **covered** by `crates/phairdlna-core/tests/connection_manager.rs::every_entry_carries_the_wildcard_and_names_no_profile`.

**AC-08** — *Given* any mode, *when* `GetProtocolInfo` is answered, *then* no entry uses the transport
protocol `http-subscribe`. Requirements: INT-056 · Verification: protocol conformance test · Planned
location: `crates/phairdlna-core/tests/connection_manager.rs`
— **covered** by `crates/phairdlna-core/tests/connection_manager.rs::no_entry_uses_the_push_protocol`.

**AC-09** — *Given* the served SCPD document, *when* it is parsed, *then* the action list contains the
three required actions and marks `PrepareForConnection` and `ConnectionComplete` optional.
Requirements: INT-053, INT-023 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/connection_manager.rs`
— **covered** by `crates/phairdlna-core/tests/connection_manager.rs::the_service_description_marks_the_optional_actions`.

**AC-10** — *Given* a `GetProtocolInfo` call, *when* the response is produced, *then* the `Sink`
element is present and empty, and no file name or path appears in the response. Requirements: INT-051,
PLT-008 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/connection_manager.rs`
— **covered** by `crates/phairdlna-core/tests/connection_manager.rs::the_sink_is_present_and_empty`.

**AC-11** — *Given* a `PrepareForConnection` call with a `Direction` outside {`Input`,`Output`},
*when* the request is processed, *then* the response is the UPnP error `402 Invalid Args` and no
connection is created. Requirements: INT-053 · Verification: protocol conformance test · Planned
location: `crates/phairdlna-core/tests/connection_manager.rs`
— **covered** by `crates/phairdlna-core/tests/connection_manager.rs::a_wrong_direction_is_the_fault_402`.

**AC-12** — *Given* a client that sends a `PeerConnectionManager` value of several kilobytes, *when*
the request is processed, *then* it is rejected without an unbounded allocation and without writing
the value to a log in raw form. Requirements: INT-053, SEC-011, SEC-007 · Verification: automated test
· Planned location: `crates/phairdlna-core/tests/connection_manager.rs`
— **covered** by `crates/phairdlna-core/tests/connection_manager.rs::a_huge_peer_value_is_refused_without_an_unbounded_allocation`.

## 11. Test strategy and dependencies

**Test locations**:

| Purpose | Path | State |
| --- | --- | --- |
| Action-level conformance of this feature | `crates/phairdlna-core/tests/connection_manager.rs` | **exists** |
| ProtocolInfo grammar and the faults of the actions without a transfer | `crates/phairdlna-core/src/upnp/connection_manager.rs` (inline `#[cfg(test)]` module) | **exists** — there is no connection table, because the server opens no connection |
| Cross-check of the advertised MIME types against the HTTP layer | `crates/phairdlna-core/tests/protocol_info_consistency.rs` | planned — the HTTP layer of F-06 does not exist yet |
| SOAP-level negative cases | `crates/phairdlna-core/tests/soap_negative.rs` (qa-performance §2.5) | **exists** |
| Device behaviour | `qa/device-matrix.toml` (qa-performance §3.2) | planned |

**Method.** The action handlers are plain functions with typed inputs and outputs, so the SOAP layer
can be exercised without a socket; a hand-written SOAP client (qa-performance §2.5) drives the
dispatcher. The honesty criterion (AC-02) is checked against the same MIME registry the HTTP layer
uses, so it fails as soon as they drift.

**Dependencies.** F-02 (SCPD generation, SOAP dispatch, fault codes), F-05 (the fourth protocolInfo
field and the `res` URI), F-06 (the served media URL), F-07 (the client profile that may select the
DLNA-flavoured protocolInfo).

## 12. Open questions and decisions

> **The HTTP status of a request that is too large.**
> The control path answers a request that is too large with the SOAP fault `402 Invalid Args`, because that is what the UPnP specification offers. At the
> HTTP layer the status `413 Request Entity Too Large` says the same thing more directly. The HTTP layer does not exist yet, so the choice belongs to
> F-06. *Closes when:* the HTTP layer maps the failure to a status.

1. **[OPEN] Five actions, not six.** standards.md §1.3 enumerates exactly five actions for
   ConnectionManager:1, and the baseline owns only INT-050…INT-056; the research identifies no
   sixth normative action (ConnectionManager:2/3 exist but are effectively unused, standards §7.3).
   *Closes when:* the owner confirms the intended surface, or adds a requirement ID.
2. **[OPEN] The word "Source" in INT-054.** `GetCurrentConnectionInfo` has no argument called
   `Source` (its arguments are `RcsID`, `AVTransportID`, `ProtocolInfo`,
   `PeerConnectionManager`, `PeerConnectionID`, `Direction`, `Status` — standards §1.3).
   *Proposed reading, used throughout this document:* the connection's source URI, reported in the
   `ProtocolInfo` argument or a sibling field. *Closes when:* the owner confirms the reading.
3. **[OPEN] Eventing has no requirement ID.** `SourceProtocolInfo`, `SinkProtocolInfo` and
   `CurrentConnectionIDs` are evented variables (standards §1.3, §8.4 M7), but the baseline has no
   requirement for GENA `SUBSCRIBE` / `UNSUBSCRIBE` / `NOTIFY`. [PROPOSED — not in the baseline]
   `INT-057 — The server shall deliver GENA notifications for the evented ConnectionManager state
   variables.` Rationale: a service that declares evented variables it never notifies contradicts the
   device architecture contract that F-02 relies on.
4. **[OPEN] Does a pure server need a non-empty `Sink`?** standards.md §8.4 M3 allows an empty
   `Sink` and MiniDLNA ships one empty, but the DLNA certification text (standards §6.2) speaks
   about the *renderer's* `SinkProtocolInfo`. *Closes when:* a device test shows whether an empty
   `Sink` is accepted.
5. **[OPEN] `PrepareForConnection` semantics for a pull-only server.** The action sets up a peer
   connection in which the server *sends* to a renderer; this server only pulls media over HTTP.
   *Closes when:* the owner decides whether the action should fault for `Direction=Output` (as
   this document assumes) or allocate a connection record that the HTTP layer later closes.
