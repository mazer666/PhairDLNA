# Security model — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Security & Privacy Engineer
Requirements: `SEC-001` … `SEC-096`, `PRD-001` … `PRD-014`
Companions: `PRIVACY.md`, `DEPENDENCY-LICENSE-REVIEW.md`, `../05-quality/QUALITY-GATES.md`

---

## 1. The threat model in one paragraph

The server listens on a **local network**, speaks two protocols that were designed in
2002 for a trusted home, parses XML and HTTP that arrive from strangers, and reads a
file system that may be shared, synchronised or hostile. The realistic attacker is
**not** a remote hacker: it is a laptop on the same café Wi-Fi, a compromised smart
device, a malicious media file that arrived through a sync client, or a web page the
user opened. Everything in this document follows from that.

Two structural facts shape the whole model:

1. **The network is the attack surface, not the internet.** The server must never be
   reachable from outside the LAN, and must never become a reflector for someone
   else's attack.
2. **The filesystem is the second attack surface.** A media file is a *file from
   someone else* that happens to be on the user's disk. It must be parsed the way an
   untrusted input is parsed.

---

## 2. Assets

| Asset | Why it matters | Worst case if lost |
| --- | --- | --- |
| Media contents | The user's private library | Irreversible personal harm |
| File system paths | Reveals the user's name and folder layout | Privacy loss, targeted attacks |
| Configuration and credentials | Controls the whole server | Full control of the service |
| Host resources | The server runs on the user's personal machine | The user's machine becomes unusable |
| Local network | The server is a door into the LAN | Pivot to other devices |
| Catalogue metadata | Reveals what the user watches and listens to | Privacy loss, profiling |
| The binary itself | Supply chain | Arbitrary code as the user |

---

## 3. Trust boundaries

```
┌────────────────────────────────────────────────────────────────────┐
│  UNTRUSTED: the LAN                                                │
│  · SSDP datagrams (M-SEARCH, hostile NOTIFY)                      │
│  · HTTP requests (headers, Range, method, body)                   │
│  · SOAP bodies (XML, arbitrary depth)                              │
└────────────────────────────┬───────────────────────────────────────┘
                             │  boundary A: the wire
┌────────────────────────────▼───────────────────────────────────────┐
│  TRUSTED: the server process                                       │
│  · parsers (XML, HTTP, DIDL)  — bounded, no DTD, no external ent.  │
│  · request router — allowlist of actions, no shell, no IGD        │
│  · catalogue (SQLite) — read-only queries, no string interpolation │
└────────────────────────────┬───────────────────────────────────────┘
                             │  boundary B: the file system
┌────────────────────────────▼───────────────────────────────────────┐
│  SEMI-TRUSTED: the user's media                                    │
│  · file names, metadata, container structures — untrusted content  │
│  · parsed OUT OF PROCESS in a sandboxed child (SEC-066)           │
└────────────────────────────────────────────────────────────────────┘

┌────────────────────────────────────────────────────────────────────┐
│  LOOPBACK-ONLY: the administration surface                         │
│  · loopback is NOT a boundary: DNS rebinding reaches it.           │
│  · therefore: Host allowlist + CSRF + custom header + bearer token│
└────────────────────────────────────────────────────────────────────┘
```

---

## 4. The eight normative mitigations

Every threat below maps to one of these. They are the project's security baseline.

| ID | Mitigation |
| --- | --- |
| **M-01** | **SSDP responder hardening.** Answer only `M-SEARCH` for our own targets; ignore every other method and every inbound `NOTIFY`; rate-limit per source address and globally; cap outstanding responses; small IP TTL; never answer a source address that is not on a plausible local network; never bind SSDP or HTTP to a non-LAN interface without explicit opt-in; never include library contents in a reply. |
| **M-02** | **Untrusted XML and SOAP.** Reject any document containing a DTD; never resolve external entities; the entity resolver always fails; bound document size, nesting depth and node count; allowlist the advertised SOAP actions; an unknown action returns an identical error regardless of the input. |
| **M-03** | **No shell, ever.** No command is ever built from a SOAP argument, a file name, a client name or any other input. No Internet Gateway Device action is implemented. The only process the server ever starts is the isolated media-inspection child, with an argument array and never a shell. |
| **M-04** | **Strict HTTP framing.** Reject conflicting framing headers (`Content-Length` together with `Transfer-Encoding`, duplicated `Transfer-Encoding`, a `Transfer-Encoding` that is not `chunked`); reject CR, LF and NUL in header values; cap the header block; enforce read, header and whole-request deadlines; cap connections, connections per source and transfers in flight; parse `Range` overflow-safe and support a single range only. |
| **M-05** | **Path containment.** Decode a URL path exactly once; reject `..`, both separator characters, NUL and control characters; join, canonicalise, then verify containment; re-verify containment *after* resolving symbolic links and aliases; open with "no follow" semantics where the platform allows it. |
| **M-06** | **Isolation of untrusted work.** Parse media metadata and generate thumbnails in a separate, sandboxed process with no network access, a bounded memory and CPU budget, a time limit and a quarantine path. The administration surface requires a `Host` allowlist, a CSRF token, a custom request header, a bearer token in a `0600` file and `SameSite=Strict`; it sends no CORS headers; media bytes are never served from the administration origin. |
| **M-07** | **Bounded resources.** Every parser has an input bound; every queue, worker pool and channel has a capacity; memory allocation never scales with an unvalidated length field; the cache is size-bounded and evictable; file descriptors are bounded and released on cancellation. |
| **M-08** | **Supply chain and provenance.** Commit `Cargo.lock` and review it like source; run the licence, advisory, ban and source checks on every change; treat every dependency with a `build.rs` or a proc-macro as code we have read; ship no FFmpeg; do not self-update over the network; offer the corresponding source of the running build without authentication. |

---

## 5. Threat table

Severity is the *impact to the user*, not the theoretical impact. Every row names the
verification that will prove the mitigation works — a negative test that must fail
without the mitigation.

| ID | Surface | Attacker | Entry point | Impact | Sev. | Mitigation | Verification | Status |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| SEC-TH-01 | SSDP reflector/amplifier | Internet attacker via a compromised host, or LAN attacker | UDP 1900 `M-SEARCH` with a spoofed source | Our host becomes an unwitting amplifier; LAN-wide storm | Medium | M-01 | A spoofed-source `M-SEARCH` produces **zero** response packets, and the per-source response counter never exceeds its cap | Planned |
| SEC-TH-02 | SSDP advertisement forgery | LAN attacker | Inbound `NOTIFY` with a forged `LOCATION`/`USN` | Renderers connect to the attacker; content substitution | Medium | M-01 | An injected `NOTIFY` produces no reaction and never changes our advertised `USN` | Planned |
| SEC-TH-03 | SSDP discovery flooding | LAN attacker | High-rate `M-SEARCH` | CPU burn; discovery unusable | Medium | M-01, M-07 | 10 000 `M-SEARCH`/s for 60 s: CPU stays under the ceiling, memory is flat, the *drop* counter rises while the *work* counter does not | Planned |
| SEC-TH-04 | Multicast membership abuse | Local process, flaky network | Interface churn | Descriptor leak; unwanted interfaces stay reachable | Low | M-01 | 100 interface up/down cycles: one membership entry per active interface, constant descriptor count | Open |
| SEC-TH-05 | GENA callback SSRF ("CallStranger" class, CVE-2020-12695) | LAN attacker | `SUBSCRIBE` with an external callback URL | Our server becomes a request proxy; exfiltration; amplification | Medium | M-02, M-01 | A `SUBSCRIBE` towards a routable public address and towards `169.254.169.254` is rejected with **no outbound connection attempt** (egress counter) | Planned |
| SEC-TH-06 | XXE / DTD in SOAP or DIDL | LAN attacker | SOAP body with an internal DTD subset | Local file read; SSRF; denial of service | Low | M-02 | A body with an external entity is rejected, **no file is opened**, no connection is attempted | Planned |
| SEC-TH-07 | Entity expansion ("billion laughs") | LAN attacker | Nested entity definitions | Memory exhaustion | Low–Med | M-02, M-07 | The nested-entity document fails within the byte and time budget; peak memory stays under the cap | Planned |
| SEC-TH-08 | Deeply nested XML | LAN attacker | 100 000 nested elements | Stack exhaustion, crash | Low | M-02 | Rejection at the configured depth; no stack overflow | Planned |
| SEC-TH-09 | SOAP action routing abuse | LAN attacker | Random `SOAPAction` values | Dispatch into the wrong handler; log flooding | Medium | M-02 | 10 000 random actions are rejected with identical status, length and timing | Planned |
| SEC-TH-10 | IGD-style action on a DMS | LAN attacker | `AddPortMapping` and friends | Port exposure; state mutation; historically RCE (CVE-2014-8361) | Low | M-03 | The actions return an error; the open-port set and the process-spawn counter are unchanged | Planned |
| SEC-TH-11 | Command injection via a SOAP argument | LAN attacker | Metacharacters in an argument | Code execution as the server user | Low | M-03 | Fuzzing every argument with shell metacharacters keeps the process-spawn counter at zero and creates no canary file | Planned |
| SEC-TH-12 | HTTP request smuggling | LAN attacker, adjacent proxy | Conflicting framing headers | Request desync; cache poisoning; auth bypass | Medium | M-04 | `CL`+`TE`, duplicated `TE`, a non-`chunked` `TE`, `Content-Length: +5` and a `GET` with a body are each rejected with 400 and the connection is closed | Planned |
| SEC-TH-13 | Header injection / oversized headers | LAN attacker | CRLF in a header; 1 MiB header block | Request splitting; response header injection | Medium | M-04 | The request is rejected, no second request is read from the stream, and no injected header appears in the response | Planned |
| SEC-TH-14 | Slowloris | LAN attacker | Drip-fed request | Connection table exhaustion | Medium | M-04, M-07 | 200 connections sending one byte per second are closed within the idle timeout; the accepted-connection count never exceeds the cap | Planned |
| SEC-TH-15 | Unbounded concurrency / descriptor exhaustion | LAN attacker | Connection flood | The whole host becomes unusable | Medium | M-04, M-07 | 5 000 open sockets: the server stays responsive to a legitimate client and the descriptor count stays under the cap | Planned |
| SEC-TH-16 | Range-header amplification | LAN attacker | 100 overlapping ranges | Memory and CPU exhaustion | Low–Med | M-04, M-07 | A single response, never a multipart body; memory flat | Planned |
| SEC-TH-17 | Path traversal in the HTTP path | LAN attacker | `..%2f..%2f`, double-encoded, absolute path | Read of files outside the root | Medium | M-05 | `..%2f..%2fetc/passwd`, `%252e%252e%252f`, an absolute path and a NUL byte are all rejected, and an `open` audit hook shows **no file outside the root** was opened | Planned |
| SEC-TH-18 | Symbolic link / alias escape | Local user, synced folder | A link inside a shared folder pointing outside | Read of arbitrary user files (e.g. `~/.ssh`) | Medium | M-05 | A symbolic link *and* a Finder alias to `~/.ssh` inside the library are both refused | Open — macOS alias resolution semantics not yet verified |
| SEC-TH-19 | Malformed media metadata | Anyone who can place a file | A malformed ID3/MP4/Matroska/EXIF file | Memory unsafety or denial of service in the server | Med–High | M-06 | A malformed file does not affect the server process; the child is killed at the timeout; the file is quarantined | Planned |
| SEC-TH-20 | Image dimension bomb | Anyone who can place a file | A header declaring enormous dimensions | Host out of memory | Medium | M-06, M-07 | The child is killed and the parent's memory never exceeds the cap | Planned |
| SEC-TH-21 | Playlist path injection | File supplier | A playlist entry pointing outside the root, or using a network scheme | Read outside the root; SSRF | Low | M-05, M-06 | `../../etc/passwd` and `http://evil/x` entries are dropped and no outbound request is made | Planned |
| SEC-TH-22 | Oversized browse page | LAN attacker | `RequestedCount = 2^31-1` | Memory growth, latency | Medium | M-02, M-07 | The response is capped at the configured page size; memory is flat | Planned |
| SEC-TH-23 | DNS rebinding | Web attacker + victim browser | A hostname that resolves to the LAN address | Media exfiltration through the browser (as in CVE-2022-26505) | Medium | M-06 | A page whose hostname resolves to loopback and then to the LAN address is blocked, and the server logs a `Host` mismatch | Planned |
| SEC-TH-24 | CSRF against the administration API | Any web page the user visits | Cross-site form or fetch | Rescan, configuration change | Medium | M-06 | A state-changing request from a foreign origin without a CSRF token is rejected with 403 and changes nothing | Planned |
| SEC-TH-25 | Administration token disclosure | Local user, backup, log shipper | Reading the token file | Full administrative control | Low | M-06 | The token file mode is `0600` in a `0700` directory; the token never appears in a log line | Planned |
| SEC-TH-26 | Signing or notarisation failure | User, downstream packager | An unsigned or un-notarised binary | Gatekeeper block; users disabling the protection; supply-chain compromise | Medium | M-08 | `spctl --assess --type execute` and `codesign --verify --deep --strict` both pass on the release artefact | Planned |
| SEC-TH-27 | Dependency supply-chain compromise | Attacker on crates.io | A typosquat or a malicious `build.rs` | Code execution at build time | Med | M-08 | `Cargo.lock` is committed and diff-reviewed; every dependency with a `build.rs` or a proc-macro is recorded in the dependency review with a named reviewer | Planned |
| SEC-TH-28 | AGPL source-offer omission | User, auditor | A build without the source offer | Licence violation; loss of trust | Med | M-08 | The running server serves its corresponding source without authentication, and the offer names the exact revision (`OPS-016`, `OPS-017`) | Planned |

---

## 6. Platform-specific notes

### 6.1 macOS

- **TCC (privacy prompts)** protects Desktop, Documents, Downloads, network volumes and
  removable volumes. Full Disk Access must be granted by the user and can never be
  circumvented. A denied folder is a *consent* problem, not an error: the server
  explains it and continues with the rest of the library (`PLT-017`).
- **Gatekeeper** blocks an unsigned binary that was downloaded. An ad-hoc signature
  works on Apple Silicon (a signature is mandatory to execute at all), but users must
  clear the quarantine attribute once. The distribution decision is an open owner
  decision (`../07-project/OPEN-DECISIONS.md`, OD-01).
- **The application firewall** is per-application. An unsigned or ad-hoc binary
  triggers a prompt, and until the user answers, incoming connections are refused.
  This is the largest macOS-specific *device-test* risk and must be exercised on a
  real machine.
- **launchd**: a daemon (system context) and an agent (user context) behave
  differently with respect to the local-network permission — daemons are exempt,
  agents are not. The always-on server therefore runs as a daemon, the user interface
  as an agent (`PLT-013`).
- **Aliases versus symbolic links**: a Finder alias is not a POSIX symbolic link. Both
  must be resolved and contained (`SEC-069`).

### 6.2 Linux

- The service runs unprivileged. Privileged operations (mounting a share) go through
  polkit, not through running as root.
- `inotify` has per-user watch limits; the scanner must handle the overflow event by
  rescanning rather than by failing.
- No sandboxed package format (Flatpak) is the primary distribution channel, because
  its filesystem sandbox prevents access to arbitrary media folders.

### 6.3 Windows

- The Windows firewall blocks inbound connections by default; the installer creates
  the rule.
- Long paths require both the manifest flag and the system-wide policy.
- Creating symbolic links requires Developer Mode or elevation, so symlink-heavy
  libraries behave differently under a service account — another reason for the
  explicit symlink policy.

---

## 7. Verification strategy

Security is only credible if every mitigation has a test that fails without it.

| Layer | What it proves | How |
| --- | --- | --- |
| **Unit** | The primitive is correct | Path containment, escaping, header validation, the range parser, the redaction filter |
| **Property** | The invariant holds for all inputs | `proptest`: a canonicalised path never escapes its root; DIDL escaping round-trips; a `Range` parser never overflows |
| **Fuzz** | The parser survives hostile input | `cargo-fuzz` targets for XML, HTTP headers, `Range`, playlists and media headers — each with a committed seed corpus |
| **Integration** | The whole surface behaves | A test harness that starts the server and attacks it over real sockets |
| **Sandbox** | The isolation holds | A canary file outside the root that must never be opened; an egress counter that must stay at zero |
| **Review** | A human looked | Independent security review before each release gate |

The gate `QG-19` (fuzz smoke) reports `NO CRASH FOUND (not a pass)` — never "passed".

---

## 8. Release criteria

A release requires:

- no open critical or high finding in this table,
- every mitigation above has its negative test implemented and observed,
- every documented exception is recorded in `../00-governance/EXCEPTIONS.md` with an
  approver and a review date,
- the dependency and licence review is current,
- the threat model has been reviewed against the current implementation.

Until then the project is **not releasable** — see `../07-project/RELEASE-BLOCKERS.md`.
