# ADR-0002 — Runtime, HTTP server and XML parser

Status: **Accepted**
Date: 2026-11-14
Owner role: Architect
Requirements: `SEC-010`, `SEC-018`, `INT-024`, `INT-080` … `INT-099`, `FUN-050`

## Context

The server needs an async runtime, an HTTP/1.1 server that speaks byte ranges
correctly, and an XML parser that can be given hostile input safely. All three are
places where a wrong choice is expensive: the HTTP layer is the most attacked surface
of the product, and the XML layer is where a parser bug becomes a remote code
execution.

The licence data and the maintenance status were retrieved from the crates.io API on
2026-11-14 and are recorded in
`../../04-security/DEPENDENCY-LICENSE-REVIEW.md` § 2.

### What the research found

- **Hyper** has the deepest published history of request-smuggling, chunked-encoding
  and header-injection defects of any Rust HTTP implementation. That is a liability
  in general and an *asset* here: the class of bug we care about most is one the
  framework has already been fighting in public.
- **`quick-xml` is a streaming parser**: memory use is proportional to the current
  element, not to the document. It *surfaces* `DocType` and `GeneralRef` events to the
  caller, so the decision to reject a DTD is ours to make and to test.
- **A streaming parser still allocates internally before the consumer sees the
  event.** Bounding the input is therefore necessary but not sufficient: the byte
  limit has to be applied *before* the parser sees the data.
- **`roxmltree` builds a whole DOM** and resolves DTD entities unless `allow_dtd` is
  disabled — the wrong shape for hostile network input.

## Decision

- **Runtime:** Tokio 1.x.
- **HTTP:** Hyper 1.x underneath, with Axum 0.8 for routing, Tower for middleware and
  `tower-http` for tracing and timeouts.
- **XML:** `quick-xml` for parsing; no XML is *written* by hand — a small writer
  module owns the escaping rules.
- **SQLite:** `rusqlite` with the `bundled` feature and `loadable_extension`
  disabled.

Three conditions attach to the HTTP choice:

1. **The framing policy is ours.** Hyper must be configured to reject conflicting
   framing headers (`Content-Length` together with `Transfer-Encoding`, a duplicated
   `Transfer-Encoding`, a `Transfer-Encoding` that is not `chunked`), a `GET` with a
   body, and a `Content-Length` with a sign. A framework that silently "fixes"
   framing is a security problem, so the policy is stated, configured and covered by
   negative tests (`SEC-018`, threat row `SEC-TH-12`).
2. **The media path never chunks.** Media responses always carry a `Content-Length`
   (`INT-088`), because no documented source states whether a certified renderer
   refuses a chunked body — and the safe choice is the one that works either way.
3. **The XML policy is enforced at the parser boundary.** A document containing a
   doctype is rejected before any entity is resolved; the entity resolver always
   fails; the document size, nesting depth and node count are bounded
   (`SEC-010`, threat rows `SEC-TH-06` … `SEC-TH-08`).

`rusqlite` is bundled rather than linked against the system library so that the
server does not inherit whatever SQLite version a machine happens to have, and so
that `loadable_extension` can be disabled by construction. The SQLite sources are
public domain; the binding is MIT.

## Consequences

- The most attacked surface runs on the most battle-tested implementation available in
  the ecosystem, with our policy on top of it rather than a framework's default.
- The XML rules are ours, so they are ours to test — but a mistake in the writer
  module would produce malformed DIDL-Lite for every client, so the writer has a
  round-trip property test and the output is validated against the DIDL-Lite XSD
  (gate `QG-17`).
- The dependency tree is larger than a hand-written HTTP server would be, and
  smaller than a framework stack with its own XML and storage layers.
- Tokio's MSRV is 1.85 and `rusqlite`'s is 1.89, so the workspace's minimum
  supported Rust version is **1.89**.
- The bundling of SQLite increases binary size. That is accepted; the size budget is
  gate `QG-15`.

## Alternatives considered

- **An async runtime other than Tokio** (async-std, smol): rejected — Tokio is the
  ecosystem's default and every auxiliary tool assumes it.
- **Actix Web**: rejected — it is a whole framework rather than a layer, and its
  actor model makes the bounded-concurrency rules of `RES-002` harder to enforce.
- **A hand-written HTTP/1.1 server**: rejected outright. HTTP has a long security
  history and this project has no reason to add to it.
- **`roxmltree` for XML**: rejected — a DOM over hostile input, and it resolves DTD
  entities unless told not to.
- **`sqlx` instead of `rusqlite`**: rejected — an async-first, compile-time-checked
  layer over a workload that is dominated by simple, synchronous index reads.
- **The system SQLite**: rejected — the version and patch level would be inherited
  from the machine.

## Revisit condition

Revisit if Hyper changes its framing defaults, if `quick-xml` changes its entity
handling, or if the measured memory cost of the bundled SQLite becomes significant.
