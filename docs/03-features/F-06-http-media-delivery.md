# F-06 — HTTP media delivery, byte ranges and seeking

Status: Draft
Owner role: Product/Requirements
Version/date: 0.1 / 2026-11-14
Requirements: FUN-040, FUN-041, FUN-042, FUN-043, FUN-044, FUN-045, FUN-046, FUN-050, INT-080,
Claim tags: `[SPEC]` from a specification · `[OBSERVED]` documented real-world behaviour · `[DESIGN]` our own decision · `[OPEN]` unresolved (see `docs/09-research/`).
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.
INT-081, INT-082, INT-083, INT-084, INT-085, INT-086, INT-087, INT-088, INT-089, INT-090, INT-091, INT-092, INT-093,
INT-094, INT-095, INT-096, INT-097, INT-098, INT-099

## 1. Purpose and user story

Everything the user watches or listens to travels over this feature. The DIDL-Lite metadata (F-05) tells a television
*where* a file is; the **HTTP media delivery** layer sends the bytes. It is an ordinary HTTP/1.1 file server with the
parts a player needs: byte ranges (so the user can seek), correct headers (so the player knows what it received) and
the DLNA extensions that renderers expect.

**User story.** As a user, I want to press "play" on my television and have the film start
immediately, be able to drag the timeline to jump to a scene, and have the picture continue after a pause — without
the television buffering the whole file or the server stalling when two rooms watch at once.

## 2. Scope

**In scope**

- `GET` and `HEAD` for media objects over HTTP/1.1, with the response headers of RFC 9110:
  single byte ranges, `206` / `416`, `Content-Range`, `Accept-Ranges`, no `multipart/byteranges`,
  `If-Range`, `Content-Type`, `Content-Length`, keep-alive and no chunked encoding.
- The DLNA request and response headers `transferMode.dlna.org`, `contentFeatures.dlna.org`,
  `getcontentFeatures.dlna.org` and `realTimeInfo.dlna.org`.
- The media URL scheme, its containment check inside the configured roots, and the mapping from an
  object identifier to a file; transfer accounting: concurrent transfers, bandwidth limits,
  bounded buffers, cancellation.

**Explicitly out of scope**

- The SOAP control channel and the device description (F-01, F-02); the DIDL-Lite document (F-05).
- Transcoding, format conversion and every protocol except HTTP/1.1 over plain TCP (F-17); RTSP,
  HTTP/2, HTTPS and any public or internet-facing exposure (PRD-004, SEC-091).
- The optional administration surface, which is a separate port (F-14); the AVTransport service
  itself — this feature only provides the position data it needs (FUN-045).

## 3. Functional requirements

**FUN-040** — The server shall serve media by byte range from the original file without transcoding whenever the
client accepts the format. [DESIGN] The bytes come from the file on disk; no conversion happens on this path
(ARCHITECTURE §6).

**FUN-041** — The server shall report the container and codec of a file in a way a client can use to decide whether it
can play it. [DESIGN] The MIME type in the third protocolInfo field (F-05) and the `upnp:class` are the client-visible
report; the codec is part of the MIME type and of the DLNA profile token (standards §6.2).

**FUN-042** — The server shall support multiple simultaneous direct-play streams, bounded by a configurable limit.
[DESIGN] The limit comes from the configuration (CFG-008); the server refuses rather than degrades (FUN-043).

**FUN-043** — The server shall bound the number of concurrent transfers and shall refuse further transfers with a
clear error rather than exhausting resources. [DESIGN] A refusal is `503 Service Unavailable` with a `Retry-After`
header and a log line that names the reason.

**FUN-044** — The server shall apply a per-stream bandwidth limit when configured, and shall document that the limit
applies to the whole transfer, not to a rate. [OPEN] The wording is ambiguous — see §12.

**FUN-045** — The server shall report the position and duration of the active transfer through the AVTransport service
when that service is implemented. [DESIGN] The HTTP layer publishes "bytes sent" and "length" on the event bus; an
AVTransport service turns them into a position.

**FUN-046** — The server shall cancel a transfer cleanly when the client closes the connection. [SPEC] The write error
or the closed socket ends the transfer; the file descriptor and the transfer slot are released (SEC-063).

**FUN-050** — The server shall keep the buffer size small and bounded so that a slow client cannot consume unbounded
memory. [DESIGN] One fixed-size buffer per transfer; the value is a configuration default proposed in §12.

## 4. Technical requirements

**FUN-028** — The media server shall serve a file larger than 4 GiB with correct byte
offsets across the 4 GiB boundary and at the end of the file. [DESIGN] A 32-bit offset
silently truncates at 4 GiB, which is the classic DLNA failure: the device plays the
first four gigabytes and then stops. Every offset in the path — the file length, the
range, the `Content-Range` header — is 64-bit.

**INT-080** — The media server shall answer `GET` and `HEAD` requests for media objects with `HTTP/1.1 200 OK`, the
correct `Content-Type`, and a `Content-Length` equal to the file size. [SPEC] RFC 9110 §8.3, §8.6 (standards §5.1).

**INT-081** — The media server shall support a single byte range and answer with `HTTP/1.1 206 Partial Content`, a
correct `Content-Range: bytes <first>-<last>/<total>` and a `Content-Length` equal to the range length. [SPEC] RFC 9110 §14.2 (standards §5.1).

**INT-082** — The media server shall answer a range request whose start is beyond the end of the file with `HTTP/1.1
416 Range Not Satisfiable` and a `Content-Range: bytes */<size>` header. [SPEC] RFC 9110 §15.5.17 (standards §5.1).

**INT-083** — The media server shall answer a syntactically invalid or unsatisfiable `Range` header with `HTTP/1.1
416` rather than with a partial content response, and shall never answer with a malformed body. [OPEN] See §12.

**INT-084** — The media server shall advertise `Accept-Ranges: bytes` on media responses. [SPEC] RFC 9110 §14.3
(standards §5.1); [OBSERVED] MiniDLNA does the same.

**INT-085** — The media server shall not implement `multipart/byteranges`; for a multi-range request it shall serve
the first satisfiable range as a single `206` or ignore the `Range` header and answer `200`. [DESIGN] We serve the
first satisfiable range (standards §5.1, §8.6 H4).

**INT-086** — The media server shall answer a `HEAD` request with the same headers as the corresponding `GET`,
including `Content-Length` and `Accept-Ranges`, and without a body. [SPEC] RFC 9110 §9.3.2 (standards §5.4).

**INT-087** — The media server shall use the extension of the file, refined by the sniffed content, to determine the
`Content-Type`; when both are unknown it shall fall back to `application/octet-stream`. [DESIGN] The type equals the protocolInfo field (F-05).

**INT-088** — The media server shall not use chunked transfer encoding for media objects and shall always send a
`Content-Length`. [SPEC] RFC 9112 §6.1 (standards §5.3); [OBSERVED] no media server the research read chunks a body.

**INT-089** — The media server shall close a connection that a client abandoned mid-transfer without leaking the
transfer slot. [DESIGN] Slot, file descriptor and buffer are released (SEC-063).

**INT-090** — The media server shall serve the media URI that it advertised in the DIDL-Lite `res` element,
byte-for-byte, for the lifetime of the catalogue entry. [DESIGN] One function builds that URI for metadata and router
(F-05).

**INT-091** — The media server shall keep the file open for the duration of the transfer when possible, and shall
reopen it if it was removed and restored. [DESIGN] A vanished file is a `404`; a restored file is opened again.

**INT-092** — The media server shall ignore a `Range` header on a method other than `GET`, and shall ignore a range unit it does not support. [SPEC] RFC 9110 §14.2 (standards §5.1).

**INT-093** — The media server shall honour `If-Range`: when its condition does not hold, it shall ignore the `Range`
header and answer `200` with the full representation. [SPEC] RFC 9110 §13.1.5 (standards §8.6 H10).

**INT-094** — The media server shall answer `404` for an unknown media identifier, `405` for an unsupported method and
`400` for a malformed request line or header block. [DESIGN] The error body never contains a path (SEC-013).

**INT-095** — The media server shall support HTTP/1.1 persistent connections, and every response shall carry either a
`Content-Length` or `Connection: close`. [SPEC] RFC 9110 §9.6 (standards §5.4); [DESIGN] keep-alive is the default.

**INT-096** — The media server shall emit the DLNA response headers `transferMode.dlna.org` and
`contentFeatures.dlna.org` on media responses, and shall answer `406` when the client requests a transfer mode the server does not support for that content. [OBSERVED] MiniDLNA emits both and answers `406` for `Streaming` on an image (standards §5.2).

**INT-097** — The media server shall answer a request carrying `getcontentFeatures.dlna.org: 1` with the `contentFeatures.dlna.org` header and shall reject any other value of that header with `400`. [OBSERVED] MiniDLNA treats any value but `1` as invalid (§8.8 N6).

**INT-098** — The media server shall serve the media URL only when it resolves inside one of the configured roots.
[OBSERVED] MiniDLNA rejects a "wide link" (§8.6 H11); [DESIGN] we re-check after link resolution (SEC-003).

**INT-099** — The media server shall emit `realTimeInfo.dlna.org` with `DLNA.ORG_TLAG=*` on media responses.
[OBSERVED] MiniDLNA emits it on every response (standards §6.4, §8.8 N4).

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.

- [DESIGN] Target: a single stream sustains the file's bitrate over loopback with no more than one
  core of CPU per stream. Method: qa-performance §5.4 — per-stream CPU from `ps` / `time -l`
  deltas over a 60 s window, reported as median and p95.
- [DESIGN] Target: memory per active stream stays bounded and is reported as MiB/stream from the
  slope of ΔRSS against the number of streams (qa-performance §5.4, §5.1). This is the measurable
  form of FUN-050 and PERF-007.
- [DESIGN] Target: the time to the first byte of a media response is dominated by the open and the
  seek, not by buffering; there is one fixed-size buffer per transfer (FUN-050). The bandwidth
  limit (FUN-044) is measured as sustained throughput per stream (qa-performance §5.1).

## 6. Usability and accessibility

- The user sees the effect of this feature as "the film plays", "the seek works" and "the picture
  does not buffer": a range request that answers `206` is what makes a timeline drag possible.
- [UX-004] A refused transfer (FUN-043) shall say what happened and what the user can do; the
  client-facing body carries the reason, the log line carries the detail.
- [UX-012] The status output (OPS-002) shall name the client that is connected and what it is
  streaming. [UX-001] Nothing in this feature requires a graphical interface.

## 7. Security and privacy

- The media URL is the trust boundary: only identifiers that resolve inside a configured root are
  served, and the check is repeated after symbolic-link resolution (INT-098, SEC-001, SEC-003).
- A request may never make the server read outside the roots, follow a link out of them, or
  expose a path in an error (SEC-013); the wide-link rejection is logged, not answered with
  details.
- The request line, the header block and the number of concurrent connections are bounded
  (SEC-011, SEC-012); the transfer table is bounded (SEC-061, SEC-063).
- Media is opened read-only (SEC-008); the server never writes to a media root (FUN-018).
- No connection leaves the LAN except to answer the client that asked (SEC-091).

## 8. Interoperability

- Normative: RFC 9110 (semantics) and RFC 9112 (framing) — `Range`, `Accept-Ranges`,
  `Content-Range`, `Content-Length`, `If-Range`, `206`, `416`, `HEAD` parity, persistent
  connections (standards §5.1–§5.4).
- [OBSERVED] MiniDLNA implements only single-range `bytes=start-end`, answers `416` for an
  out-of-bounds range, sends `Accept-Ranges: bytes`, always sends `Content-Length`, emits
  `transferMode.dlna.org`, `contentFeatures.dlna.org` and `realTimeInfo.dlna.org`, answers `406` for
  an unsupported transfer mode and rejects a wide link (standards §5.1, §5.2).
- [OBSERVED] rclone and MiniDLNA both send `Connection: close` on every response, and real
  renderers work with that (standards §5.4). [STANDARD/secondary] The DLNA layer expects these four
  headers on media responses (standards §6.4, §8.8 N4–N6).
- [OPEN] Whether a DLNA-certified renderer refuses a chunked body or a body without
  `Content-Length` is not publicly documented (standards §5.3, §9 Q1); see §12.

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| Unknown media identifier | `404 Not Found` with a short body | INT-094 |
| Method other than `GET`/`HEAD` on a media URL | `405 Method Not Allowed` | INT-094 |
| Malformed request line or header block | `400 Bad Request` | INT-094 |
| `Range` start beyond the end of the file | `416` + `Content-Range: bytes */<size>` | INT-082 |
| `Range` syntactically invalid or unsatisfiable | `416`, never a partial body | INT-083 |
| `If-Range` condition does not hold | `200` with the full representation | INT-093 |
| Unsupported `transferMode.dlna.org` for this content | `406 Not Acceptable` | INT-096 |
| The file was deleted while the client was reading | the transfer ends; the slot is released | INT-089, FUN-046 |
| The transfer limit is reached | `503 Service Unavailable` + `Retry-After` | FUN-043 |
| The client disconnected mid-body | the transfer is cancelled, the slot released | INT-089, FUN-046 |

## 10. Acceptance criteria and traceability

**Five of the thirteen criteria are implemented, three partly, and five are not.** The criteria
that need a network runtime — the transfer table, the buffer and the event bus — cannot be
implemented before F-13 and F-18, because the module under test does not exist yet. Every
criterion that the *pure* layer can answer has a test; the state of each one is written below.

**AC-01** — *Given* a media file of 1 000 bytes, *when* a client sends `GET` and then `HEAD`, *then* the responses are
`200 OK` with `Content-Type` equal to the MIME type of the `res` element, `Content-Length: 1000` and `Accept-Ranges:
bytes`, and the `HEAD` body is empty. Requirements: INT-080, INT-084, INT-086, INT-064 · Verification: protocol
conformance test · Location: `crates/phairdlna-core/tests/http_media.rs` — **implemented**

**AC-02** — *Given* the same file, *when* a client sends `GET` with `Range: bytes=100-199`, *then* the response is
`206 Partial Content` with exactly 100 body bytes, `Content-Range: bytes 100-199/1000` and `Content-Length: 100`.
Requirements: INT-081 · Verification: protocol conformance test · Location:
`crates/phairdlna-core/tests/http_range.rs` — **implemented**

**AC-03** — *Given* the same file, *when* a client sends `Range: bytes=1000-`, `Range: bytes=5000-6000` and a
multi-range `Range: bytes=0-1,5-6`, *then* the first two are answered `416` with `Content-Range: bytes */1000`, and
the third is a single `206` that is never `multipart/byteranges`. Requirements: INT-082, INT-083, INT-085 ·
Verification: protocol conformance test · Location: `crates/phairdlna-core/tests/http_range.rs` —
**implemented**

**AC-04** — *Given* a `Range` header on a method other than `GET`, an unsupported range unit and an `If-Range` whose
condition does not hold, *when* they are answered, *then* the `Range` header is ignored and the full representation is
returned. Requirements: INT-092, INT-093 · Verification: protocol conformance test · Location:
`crates/phairdlna-core/tests/http_range.rs` — **implemented**

**AC-05** — *Given* a file whose extension and sniffed content disagree, *when* it is requested, *then* the
`Content-Type` is the sniffed type, or `application/octet-stream` when neither is known; and *given* any media
response, *then* it carries a `Content-Length` and no `Transfer-Encoding: chunked`, and a reused connection carries
`Content-Length` or `Connection: close`. Requirements: INT-087, INT-088, INT-095 · Verification: protocol conformance
test · Location: `crates/phairdlna-core/tests/http_media.rs` — **partly implemented**: the type of
the answer, the `Content-Length`, the absence of a chunked body and the connection handling are
implemented; the *sniffing* of the content is F-10's, and the catalogue hands the type over
already refined

**AC-06** — *Given* a client that closes the connection after part of the body, *when* the server notices, *then* the transfer slot, the file descriptor and the buffer are released.
Requirements: INT-089, FUN-046, FUN-050 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/http_cancel.rs` — **planned**: the transfer slot, the file
descriptor and the buffer belong to the runtime (F-13, F-18), which does not exist yet

**AC-07** — *Given* a catalogue entry, *when* the client fetches the URL from the `res` element twice, *then* both requests return the same bytes and the URL keeps its meaning while the entry exists.
Requirements: INT-090, INT-091 · Verification: automated test · Location:
`crates/phairdlna-core/tests/http_media.rs` — **partly implemented**: the URL keeps its meaning
and the same bytes are answered; the reopening of a file that was removed and restored is the
file layer's (F-08, F-13)

**AC-08** — *Given* an unknown media identifier, an unsupported method and a malformed request line, *then* the answers are `404`, `405` and `400`, and no answer contains a path. Requirements: INT-094,
SEC-013 · Verification: protocol conformance test · Location: `crates/phairdlna-core/tests/http_negative.rs`
— **implemented**

**AC-09** — *Given* a media response, *when* it is inspected, *then* it carries `transferMode.dlna.org`,
`contentFeatures.dlna.org` and `realTimeInfo.dlna.org`; *given* an unsupported transfer mode, *then* the answer is
`406`; *given* `getcontentFeatures.dlna.org: 1`, *then* the response carries `contentFeatures.dlna.org`, while the
value `2` is answered `400`. Requirements: INT-096, INT-097, INT-099 · Verification: protocol conformance test ·
Location: `crates/phairdlna-core/tests/http_dlna_headers.rs` — **implemented**

**AC-10** — *Given* a media URL whose identifier resolves outside every configured root, *when* it is requested,
*then* the server refuses to open the file and answers `404` without revealing the path. Requirements: INT-098,
SEC-001 · Verification: automated test · Location: `crates/phairdlna-core/tests/http_path_containment.rs`
— **implemented** for the identifier and for the lexical check; the repetition of the check with
the canonical path, after the symbolic links have been resolved, needs the file system and
belongs to the runtime

**AC-11** — *Given* the transfer limit of the configuration, *when* more clients than the limit request a stream,
*then* the extra request is refused with `503 Service Unavailable` and a `Retry-After` header, and the running
transfers are unaffected. Requirements: FUN-042, FUN-043 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/http_limits.rs` — **planned**: the transfer table is the runtime's
(F-13, F-18)

**AC-12** — *Given* a configured bandwidth limit, *when* a stream runs, *then* its sustained throughput does not
exceed the limit and the other streams are not starved; *given* a slow client that stops reading, *then* resident
memory grows by at most one buffer per active transfer. Requirements: FUN-044, FUN-050, PERF-007 · Verification:
benchmark · Planned location: `qa/perf/scripts/run-bandwidth-limit.sh`, `run-slow-client.sh` —
**planned**: a benchmark needs a runtime to measure

**AC-13** — *Given* an active transfer, *when* the position and duration are read through the event bus, *then* they match the bytes sent and the length of the file, and the MIME type is the one in the `res` element.
Requirements: FUN-045, FUN-041 · Verification: automated test · Planned location:
`crates/phairdlna-core/tests/http_transfer_state.rs` — **planned**: the event bus is the runtime's
(ARCHITECTURE §3)

## 11. Test strategy and dependencies

**Test locations.** The five rows below exist today; the three rows after them are still planned.

| Purpose | Path | State |
| --- | --- | --- |
| Media answers and headers | `crates/phairdlna-core/tests/http_media.rs` | **exists** |
| Range semantics (the table of qa-performance §5.4) | `crates/phairdlna-core/tests/http_range.rs` | **exists** |
| Negative requests and error codes | `crates/phairdlna-core/tests/http_negative.rs` | **exists** |
| DLNA headers | `crates/phairdlna-core/tests/http_dlna_headers.rs` | **exists** |
| Path containment | `crates/phairdlna-core/tests/http_path_containment.rs` | **exists** |
| Cancellation and transfer limits | `crates/phairdlna-core/tests/http_cancel.rs`, `http_limits.rs` | planned |
| Range-header and header-block fuzzing | `fuzz/fuzz_targets/fuzz_range_header.rs`, `fuzz_http_headers.rs` (qa-performance §4.2) | planned |
| Throughput, memory and device behaviour | `qa/perf/scripts/run-*.sh`; `qa/device-matrix.toml` | planned |

**Method.** The HTTP layer reads its bytes through a `MediaSource` trait, so the range, header and
cancellation behaviour is testable against an in-memory source without a disk (ARCHITECTURE §4.7).
The `Range` table of qa-performance §5.4 is the row-by-row test plan; the fuzz target checks that no
`Range` header ever produces a range outside the resource.

**What the tests prove.** The evaluator is pure: it reads no socket and no file, it decides the
status, the headers and the slice of the file to read. A test therefore compares what the server
promised with what the client receives — the fixture holds the bytes of the file, and the test reads
exactly the slice the answer names (`crates/phairdlna-core/tests/support/media.rs`).

**Dependencies.** F-05 (the `res` URI and the MIME type), F-08 and F-09 (the roots and the
catalogue entry behind an identifier), F-12 (the limits in the configuration), F-16 (path
containment and the request bounds).

## 12. Open questions and decisions

### 12.1 Decisions taken by the implementation (WP4)

These decisions are implemented and covered by a test. The ones marked **[owner decision]** change a
documented behaviour and are reported to the owner with this work package.

**D-01 — A range that cannot be read is answered with `416` (INT-083).** A `Range` header whose unit is
`bytes` and whose value cannot be read — `bytes=abc-def`, `bytes=200-100`, `bytes=-0` — is answered
with `416` and `Content-Range: bytes */<size>`, never with a partial body. This follows INT-083, which
is a MUST; question 1 below proposes the opposite and is still open. A header that names another
*unit* (`items=0-10`) is ignored and the whole representation is returned, as INT-092 asks. *Implemented.*

**D-02 — A set of ranges is served as one range (INT-085).** `bytes=0-1,5-6` is answered with `206` and
the first range that the file can satisfy — never with a `multipart/byteranges` body. *Implemented.*

**D-03 — The transfer mode of a content.** An image is delivered interactively and in no other way; a
renderer that asks to stream an image gets `406`, as the reference implementation does (standards
§5.2). Sound and pictures are streamed, and a background request is served like a streaming one. The
answer always names the mode the server uses. *Implemented; the proposed decision of question 3 below
still needs the owner's confirmation.*

**D-04 — The identifier of a media object is an opaque token.** The identifier in the URL may not carry
a separator, a dot or a control character, so a request that tries to walk out of the catalogue is
answered like an unknown object and never reveals the layout of the disk (SEC-013). A target whose
percent-encoding is broken is answered with `400`, because the request line itself is broken
(INT-094). *Implemented.*

**D-05 — The containment of a resolved path is a lexical check.** `is_inside_roots` normalises the path
and compares it with the roots, so a path that walks out of a root with `..` does not pass. The check
cannot see a symbolic link: the caller resolves the links first and repeats the check with the
canonical path (INT-098, SEC-003). *Implemented.*

**D-06 — The answer carries no `ETag` today.** `If-Range` is honoured when the object has a validator
and the condition is an entity tag that matches it; the server does not emit an `ETag` of its own yet,
because the validator of a file is the catalogue's business (F-08). A date cannot be compared, so a
date condition does not hold and the range is ignored (INT-093). *Implemented.*

**D-07 — The body of an error answer is a fixed text.** The answers `400`, `404`, `405`, `406` and `416`
carry a short text that names no path (SEC-013). A `HEAD` answer carries the headers of the `GET`
answer and no body (INT-086). *Implemented.*

**D-08 — The connection is kept alive unless the client asks to close it.** HTTP/1.1 makes the
persistent connection the default, so the answer carries no `Connection` header unless the request
asked for `close` (INT-095). Every answer carries a `Content-Length` in any case (INT-088). *Implemented.*

### 12.2 Open questions

1. **[OPEN] `416` or `400` for an invalid `Range`?** INT-083 answers `416` for a syntactically invalid or
unsatisfiable range; INT-094 reserves `400` for a malformed request line or header block. [OBSERVED] MiniDLNA answers
`400` when the start is greater than the end and `416` when the range is outside the file; qa-performance §5.4
proposes ignoring the header and answering `200` for `Range: bytes=abc`. *Proposed decision:* one table — unknown unit
or unparsable value → ignore and answer `200`; a well-formed but unsatisfiable range → `416`; and amend INT-083.
*Closes when:* the owner confirms the table. **The implementation follows INT-083 and answers `416`
for an unparsable value (D-01); the proposal is not implemented.**

2. **[OPEN] Chunked bodies and certified renderers.** Whether a DLNA-certified renderer refuses a chunked body or a body without `Content-Length` is not publicly documented (standards §5.3, §9 Q1). *Closes when:* a device test answers it; until then INT-088 keeps media bodies unchunked, the low-risk choice.

3. **[OPEN] Multi-range, content-type and transfer-mode policies.** INT-085 allows "first satisfiable range as a
single `206`" or "ignore `Range` and answer `200`"; this document chooses the first, because a renderer that asks for
two ranges usually wants the first one. INT-087 says "the extension of the file, refined by the sniffed content"; this
document reads that as "the sniffed type wins". INT-096 needs a per-content mode: *proposed decision:* `Streaming` for
audio and video, `Interactive` for images, `406` for a mode that does not fit. *Closes when:* the owner confirms.

4. **[OPEN] The meaning of the bandwidth limit (FUN-044).** The baseline says the limit "applies to the whole
transfer, not to a rate", which reads like a cap on the total bytes rather than a rate. *Proposed decision:* the limit
is a cap on bytes per second applied to the whole transfer, and the documentation states that it slows the transfer
rather than truncating it. *Closes when:* the owner confirms and CFG-008 gains the setting.

5. **[OPEN] The buffer size (FUN-050) and "byte-for-byte" (INT-090).** The research gives no figure for the buffer:
*proposed decision:* one fixed buffer of 64 KiB per transfer, recorded as a configuration default (CFG-004). If the
file changes on disk after indexing, the catalogue entry is refreshed by the scanner (FUN-006): *proposed reading:*
the URI keeps its meaning for the lifetime of the catalogue entry, while the bytes follow the file on disk. *Closes
when:* the owner confirms both.
