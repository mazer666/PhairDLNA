# Test strategy — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: QA / Performance Engineer
Requirements: `DOC-002`, `DOC-011`, `SEC-001` … `SEC-096`, `INT-001` … `INT-104`
Gates: `../05-quality/QUALITY-GATES.md` · Protocol evidence:
`../05-quality/INTEROPERABILITY-MATRIX.md`

---

## 1. The shape of the suite

| Layer | What it covers | How it runs | Rough share |
| --- | --- | --- | --- |
| **Unit** | Domain logic: escaping, identifiers, the range parser, path containment, redaction, configuration validation | Pure functions, no I/O | The largest share |
| **Property** | Invariants that must hold for *all* inputs | `proptest` | Small, but the highest-value tests in the project |
| **Integration** | SSDP, SOAP, ContentDirectory, HTTP delivery, the catalogue, the CLI | Real sockets, a temporary directory, a real SQLite file | Substantial |
| **Protocol conformance** | The specification's requirements, one test per obligation | Fixtures from the conformance checklist | One test per `PC-nn` row |
| **Negative / security** | The attacks in the threat model | The attack is actually performed | One test per `SEC-TH-nn` row |
| **Fuzz** | Parser robustness | `cargo-fuzz`, nightly toolchain, committed seed corpora | Continuous |
| **Device** | Real renderers | A manual protocol, results in the interoperability matrix | Per release |
| **Benchmark** | Budgets | `criterion` + `hyperfine` | Continuous |
| **Soak** | Long-run stability | 7 days | Before release |

---

## 2. Property tests — the highest-value tests

Two invariants deserve property-based testing, because a counterexample is a bug
report that arrives before a user finds it.

| ID | Property | Why it matters |
| --- | --- | --- |
| P-1 | Escaping round-trips: for any title, `escape(unescape(title)) == title`, and the escaped form never contains a raw `<` or `&` that would break the XML | A single unescaped character breaks every client that browses that folder |
| P-2 | A canonicalised path never escapes its root: for any path and any root, `canonicalise(join(root, path))` is inside `root`, or the operation is rejected | The classic path-traversal class; the property is stronger than a list of test cases |

Additional candidates, once the modules exist: the `Range` parser never overflows and
never returns a range outside the file; the object-identifier function is
deterministic and injective for distinct identities; the redaction filter removes
every absolute path from a log record.

---

## 3. Negative and recovery tests

Every hardening in `../04-security/SECURITY-MODEL.md` needs a test that performs the
attack. The catalogue of required cases:

**Discovery** — malformed `M-SEARCH`; `M-SEARCH` without `MX`; a request for an
unknown `ST`; a flood from one source; a spoofed source address; an inbound
`NOTIFY`; a search on a disabled interface.

**XML and SOAP** — a body with a doctype; a body with an external entity; a nested
entity bomb; a 100 000-deep document; an oversized body; an unknown `SOAPAction`;
10 000 random `SOAPAction` values; an action argument containing shell
metacharacters; an Internet Gateway Device action.

**HTTP** — conflicting framing headers; a duplicated `Transfer-Encoding`; a
`Content-Length` with a `+` sign; a `GET` with a body; a CRLF in a header value; a
1 MiB header block; 200 slow connections; 5 000 concurrent connections; 100
overlapping ranges; an unsatisfiable range; a malformed range; a path traversal in
the URL (`..%2f`, `%252e%252e%252f`, an absolute path, a NUL byte); a request for an
unknown media id; a `Host` header that does not match.

**Filesystem** — a symbolic link pointing outside the root; a macOS Finder alias
pointing outside the root; a permission-denied subtree; a volume that disappears
mid-scan; a file that is replaced while it is being read; a corrupt index database;
a file whose metadata is malformed; a file declaring enormous image dimensions; a
playlist containing `../../etc/passwd` and `http://evil/x`.

**Privacy** — a DNS-rebinding attempt against the administration surface; a
cross-site state-changing request without a CSRF token; a request without the custom
header; an attempt to read the token file's permissions; an outbound-connection
audit during a normal session.

---

## 4. Protocol conformance

The conformance checklist in `INTEROPERABILITY-MATRIX.md` §2 lists 26 checks
(`PC-01` … `PC-26`). Each becomes an automated test against the real server, using
fixtures — not mocks — so that the wire format is what is tested.

**The SSDP corpus must be captured by the project.** The only public SSDP packet
capture available contains no `M-SEARCH` requests at all, so the corpus starts empty
and grows with every real client observed. Each captured request is stored as a
fixture and answered by the server in a test.

**The DIDL-Lite XSD is public** (`upnp.org/schemas/av/didl-lite.xsd`) and is vendored
into the repository with rewritten `schemaLocation` references so that the check runs
offline. There is **no** official XSD for a UPnP device description — every
`upnp.org/schemas/device/*` path answers with HTML — so the device description is
validated against hand-written fixtures instead.

---

## 5. Device testing

Manual, structured, recorded. The protocol is in
`INTEROPERABILITY-MATRIX.md` §6. The rules:

- Record the device model exactly, the firmware, the client application and version.
- Use a controlled media set: one file per format plus a subtitle sidecar.
- Perform at least four operations: discovery, browse, start playback, seek.
- Record the outcome per operation, and capture the server log for the session.
- A failure is published with the same care as a success.

---

## 6. Fuzzing

Six targets, each with a committed seed corpus:

| Target | Input |
| --- | --- |
| `fuzz_soap` | A SOAP request body |
| `fuzz_device_description` | The generated description, round-tripped |
| `fuzz_http_request` | A raw HTTP request |
| `fuzz_range` | A `Range` header value |
| `fuzz_playlist` | An M3U/PLS/XSPF file |
| `fuzz_media_header` | The first bytes of a media file |

`cargo-fuzz` requires a pinned nightly toolchain, so it is a separate job from the
stable-toolchain tests. A fuzzing result is reported as
`NO CRASH FOUND (not a pass)` or `CRASH FOUND (defect)` — never as a pass.

---

## 7. Test data

- **No personal media.** Every fixture is either generated by the test or a tiny
  synthesised file.
- **No real paths.** Fixtures use a temporary directory; the project name of the
  user never appears in a test.
- **Generated media.** Small media files are synthesised in the test setup rather
  than committed, except for a handful of deliberately malformed files whose exact
  bytes matter.
- **Determinism.** A test that depends on timing, on the machine's locale or on the
  order of a directory listing is a broken test; the filesystem is sorted explicitly
  where order matters.

---

## 8. Coverage

Coverage is measured with `cargo llvm-cov` and reported as **information**
(`QG-26`), never as a quality claim. A coverage number does not tell you whether the
important paths are tested; the requirement-to-test mapping in
`../07-project/TRACEABILITY-MATRIX.md` does.

The real completeness question is: *for every requirement, is there a test that
would fail if the requirement were broken?* The traceability matrix answers it, and a
requirement without a verification method stays `Draft`.

---

## 9. Recording a test result

Every test run that is cited anywhere carries:

- the exact command,
- the summary line (counts, failures, duration),
- the toolchain versions,
- the revision.

A run that was not performed is written as `NOT RUN — <reason>`. A skipped test is
written as `SKIPPED (exit 77) — <reason>`. Neither is a pass.
