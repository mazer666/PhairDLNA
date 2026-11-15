# Dependency, licence and supply-chain review — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Security & Privacy Engineer
Requirements: `SUP-001` … `SUP-008`, `SEC-090` … `SEC-096`
Gate: `../05-quality/QUALITY-GATES.md` (QG-08, QG-09, QG-10)
Method: crates.io API (`https://crates.io/api/v1/crates/<name>`), retrieved 2026-11-14.

---

## 1. Policy

1. **No dependency without a record here.** Name, version, licence, purpose, maintenance
   status, alternatives considered, and a named reviewer.
2. **Permissive licences only, by default.** MIT, Apache-2.0, BSD-2/3, ISC, Unicode-3.0,
   Zlib, CC0-1.0. Everything else needs an explicit compatibility assessment.
3. **File-level copyleft (MPL-2.0) is allowed** when the compatibility analysis is
   recorded, because MPL-2.0 permits combining its files into a larger work that is
   licensed under other terms. The MPL files keep their own licence and their source
   must be available.
4. **Copyleft (GPL, AGPL, LGPL, CDDL) requires an owner decision** before adoption.
5. **No new dependency with a `build.rs` or a proc-macro** without reading it. Build
   scripts run arbitrary code on every contributor's and every user's machine.
6. **No native toolchain dependencies in v1.** Anything that needs a C compiler, a
   system library or an external binary is an owner decision.
7. **No shipped FFmpeg.** A GPL FFmpeg inside an AGPL product is a licensing trap: the
   combined work would be governed by GPLv3, not AGPLv3, and the AGPL network-source
   obligation would not attach to it. If transcoding is ever added, it must be an
   LGPL-only build, dynamically linked, with its exact source and configure line in
   the same source offer — and reviewed by counsel.
8. **`Cargo.lock` is committed** and diff-reviewed like source.

---

## 2. Version and licence data (verified 2026-11-14)

Download counts are crates.io totals and are shown only as a rough indicator of
adoption, not as a quality judgement.

| Crate | Version | Licence | MSRV | Downloads | Updated | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| `tokio` | 1.54.1 | MIT | 1.85 | 1 193 567 705 | 2026-10-21 | **Adopt** — async runtime |
| `hyper` | 1.12.0 | MIT | 1.63 | 1 129 663 767 | 2026-10-06 | **Adopt** — HTTP/1.1 server |
| `axum` | 0.8.9 | MIT | 1.80 | 592 932 422 | 2026-04-14 | **Adopt** — routing, service layer |
| `tower` | 0.5.3 | MIT | — | 841 258 187 | 2026-01-12 | **Adopt** — middleware |
| `tower-http` | 0.7.1 | MIT | 1.65 | 621 132 531 | 2026-08-31 | **Evaluate** — tracing, timeouts; range support must be verified |
| `quick-xml` | 0.42.0 | MIT | 1.86 | 548 849 079 | 2026-08-22 | **Adopt** — streaming XML; DTD/entity events surfaced to the caller |
| `roxmltree` | 0.21.2 | MIT OR Apache-2.0 | 1.60 | 106 279 045 | 2026-10-24 | **Alternative** — DOM; resolves DTD entities unless disabled |
| `rusqlite` | 0.42.0 | MIT | 1.89 | 154 994 331 | 2026-11-11 | **Adopt** — SQLite, bundled, extensions off |
| `sqlx` | 0.9.0 | MIT OR Apache-2.0 | 1.94 | 189 841 591 | 2026-05-01 | **Not adopted** — heavier, async-first; `rusqlite` is enough |
| `symphonia` | 0.6.1 | **MPL-2.0** | 1.85 | 21 837 379 | 2026-08-13 | **Adopt with assessment** — pure-Rust audio decoding; MPL-2.0 is file-level copyleft |
| `lofty` | 0.25.5 | MIT OR Apache-2.0 | 1.89 | 1 568 758 | 2026-10-25 | **Adopt** — audio tags, artwork |
| `notify` | 9.0.0-rc.8 | CC0-1.0 | 1.91 | 198 269 121 | 2026-11-09 | **Adopt the stable 8.x line** — do not depend on a release candidate |
| `ignore` | 0.4.34 | Unlicense OR MIT | 1.88 | 213 708 619 | 2026-10-27 | **Adopt** — gitignore-style filtering during scans |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 | 1.77 | — | 2026 | **Adopt** — the error types |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | 1.63 | 997 650 245 | 2026-03-11 | **Adopt** — temporary directories in tests (dev-dependency) |
| `walkdir` | 2.5.0 | MIT | — | 774 949 224 | 2024-03-01 | **Adopt** — traversal primitive |
| `clap` | 4.6.7 | MIT OR Apache-2.0 | 1.85 | 1 340 276 208 | 2026-09-14 | **Adopt** — CLI parsing |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | 1.56 | 1 709 177 081 | 2026-07-18 | **Adopt** — serialisation |
| `toml` | 1.2.1 | MIT OR Apache-2.0 | 1.85 | 1 141 698 378 | 2026-11-06 | **Adopt** — configuration format |
| `unicode-normalization` | 0.1.26 | (MIT OR Apache-2.0) AND Unicode-3.0 | 1.36 | 683 092 567 | 2026-10-16 | **Adopt** — NFC normalisation of path components |
| `unicase` | 2.10.0 | MIT OR Apache-2.0 | — | 492 183 614 | 2026-10-05 | **Adopt** — full Unicode case folding for the path identity |
| `tracing` | 0.1.44 | MIT | 1.65 | 1 050 071 464 | 2025-12-18 | **Adopt** — structured logging |
| `uuid` | 1.29.2 | Apache-2.0 OR MIT | 1.89 | 992 221 000 | 2026-11-09 | **Adopt** — device UDN (v4) |
| `blake3` | 1.8.7 | CC0-1.0 OR Apache-2.0 (LLVM exception) | — | 227 947 308 | 2026-08-20 | **Adopt** — the object id hash of the catalogue (M1.9, DEC-027); the `pure` feature keeps the build free of a C compiler |
| `cpufeatures` | 0.2.17 | MIT OR Apache-2.0 | 1.60 | 302 118 751 | 2026-03-24 | **Adopt** — new with `blake3`; reads the CPU flags for the assembly backend |
| `muda` | 0.21.2 | Apache-2.0 OR MIT | 1.90 | 51 346 421 | 2026-10-08 | **Evaluate** — macOS menu bar |
| `tray-icon` | 0.26.1 | MIT OR Apache-2.0 | 1.90 | 44 016 004 | 2026-10-07 | **Evaluate** — tray/status item |
| `objc2` | 0.6.5 | MIT | 1.71 | 162 826 258 | 2026-10-05 | **Adopt (macOS only)** — modern Objective-C bindings |
| `objc2-app-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | 1.71 | 89 033 565 | 2025-10-04 | **Adopt (macOS only)** — `NSStatusItem` and friends |
| `mp4` | 0.14.0 | MIT | — | — | 2023-08-01 | **Not adopted** — unmaintained since 2023 |
| `matroska` | 0.30.1 | MIT OR Apache-2.0 | — | — | 2026-04-15 | **Evaluate** — Matroska/WebM container parsing |
| `image` | 0.25.10 | MIT OR Apache-2.0 | 1.88 | — | 2026-03-10 | **Evaluate** — dimensions and thumbnails; decoding happens in the sandbox |
| `infer` | 0.22.0 | MIT | 1.74 | — | 2026-07-15 | **Adopt** — content sniffing |
| `mime_guess` | 2.0.5 | MIT | — | — | 2024-06-29 | **Adopt** — extension → MIME |
| `futures-core` | 0.3.34 | MIT OR Apache-2.0 | 1.36 | 305 000 000 | 2025-12-01 | **Adopt** — the `Stream` trait of the media body (M1.11) |

### 2.1 The crates `blake3` brings with it

`blake3` is the first dependency the core takes for a cryptographic hash. Three crates
are **new** in the lock file because of it: `arrayvec` (MIT OR Apache-2.0),
`constant_time_eq` (Apache-2.0 OR MIT) and `cpufeatures` (MIT OR Apache-2.0) — the last
one reads the CPU flags of the machine, and it is used by the assembly backend that the
`pure` feature of the project does not compile. The build dependency `cc` and its
helpers (`shlex`, `find-msvc-tools`) are new as well; they belong to the same optional
backend. Every one of them is allowed by `deny.toml` and is covered by
`cargo deny check licenses` like every other crate.

The licence of `blake3` itself is CC0-1.0 OR Apache-2.0 (with the LLVM exception), and
`deny.toml` allows all three expressions. CC0-1.0 is a public-domain dedication, which is
the weakest copyleft obligation of any licence in the set: nothing in the project inherits
an obligation from it.

### 2.2 Test and quality tooling (dev-dependencies only)

| Crate | Version | Licence | Purpose |
| --- | --- | --- | --- |
| `cargo-nextest` | 0.9.x | MIT OR Apache-2.0 | Test runner |
| `assert_cmd` | 2.2.2 | MIT OR Apache-2.0 | CLI integration tests |
| `predicates` | 3.1.x | MIT OR Apache-2.0 | Assertions |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | Temporary directories |
| `wiremock` | 0.6.5 | MIT/Apache-2.0 | HTTP mocking |
| `insta` | 1.49.0 | Apache-2.0 | Snapshot tests |
| `proptest` | 1.12.0 | MIT OR Apache-2.0 | Property tests |
| `criterion` | 0.8.2 | Apache-2.0 OR MIT | Benchmarks |
| `xmlschema` | 0.0.14 | MIT OR Apache-2.0 | XSD validation of DIDL-Lite — **low adoption (≈4 800 downloads), dev-only, risk flagged** |
| `cargo-deny` | 0.20.2 | MIT OR Apache-2.0 | Licence, advisory, ban and source checks |
| `cargo-audit` | 0.23.0 | MIT OR Apache-2.0 | RustSec advisories |

### 2.3 The crates the network stack brings with it (M1.11)

The milestone M1.11 added the runtime and the HTTP server to `phairdlna-cli`. The core
takes **no** new dependency: it stays portable and free of a runtime (DEC-033). The
following crates are **new** in the lock file because of the network stack. Every one of
them is allowed by `deny.toml` and is covered by `cargo deny check licenses` and by
`cargo audit` like every other crate.

| Crate | Version | Licence | MSRV | Purpose |
| --- | --- | --- | --- | --- |
| `axum` | 0.8.9 | MIT | 1.80 | the router and the HTTP server of the daemon |
| `axum-core` | 0.5.6 | MIT | 1.78 | the types of the router |
| `tower` | 0.5.3 | MIT | — | the service layer |
| `tower-layer` | 0.3.3 | MIT | — | the layers of the service |
| `tower-service` | 0.3.3 | MIT | — | the service trait |
| `hyper` | 1.12.0 | MIT | 1.63 | the HTTP/1.1 implementation under `axum` |
| `hyper-util` | 0.1.21 | MIT | 1.63 | the helpers of `hyper` |
| `http` | 1.5.0 | MIT OR Apache-2.0 | 1.57 | the header and status types |
| `http-body` | 1.1.0 | MIT | 1.61 | the body trait |
| `http-body-util` | 0.1.5 | MIT | 1.61 | the helpers of the body trait |
| `httparse` | 1.10.1 | MIT OR Apache-2.0 | — | the parser of the request line and the headers |
| `httpdate` | 1.0.3 | MIT OR Apache-2.0 | 1.56 | the date header |
| `bytes` | 1.12.1 | MIT | 1.57 | the byte buffers of the body |
| `mime` | 0.3.18 | MIT OR Apache-2.0 | — | the media types |
| `matchit` | 0.8.4 | MIT AND BSD-3-Clause | — | the router of the paths |
| `percent-encoding` | 2.3.2 | MIT OR Apache-2.0 | 1.51 | the escaping of the paths |
| `tokio` | 1.54.1 | MIT | 1.85 | the runtime, the sockets and the signals |
| `tokio-macros` | 2.7.1 | MIT | 1.71 | the macros of `tokio` |
| `mio` | 1.2.5 | MIT | 1.71 | the readiness of the sockets |
| `socket2` | 0.6.5 | MIT OR Apache-2.0 | 1.70 | the socket options |
| `rustix` | 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | 1.65 | the system calls of the sockets |
| `errno` | 0.3.15 | MIT OR Apache-2.0 | 1.56 | the error numbers |
| `libc` | 0.2.191 | MIT OR Apache-2.0 | 1.65 | the platform calls |
| `signal-hook-registry` | 1.4.8 | MIT OR Apache-2.0 | 1.26 | the registration of the signals |
| `futures-core` | 0.3.34 | MIT OR Apache-2.0 | 1.36 | the `Stream` trait |
| `futures-util` | 0.3.34 | MIT OR Apache-2.0 | 1.71 | the helpers of the futures |
| `futures-task` | 0.3.34 | MIT OR Apache-2.0 | 1.71 | the waking of the tasks |
| `pin-project-lite` | 0.2.17 | Apache-2.0 OR MIT | 1.37 | the pinning of the futures |
| `slab` | 0.4.12 | MIT | 1.51 | the arenas of the runtime |
| `once_cell` | 1.21.4 | MIT OR Apache-2.0 | 1.65 | the one-time initialisation |
| `fastrand` | 2.5.0 | Apache-2.0 OR MIT | 1.63 | the jitter of the answer delay |
| `itoa` | 1.0.19 | MIT OR Apache-2.0 | 1.68 | the conversion of the numbers |
| `memchr` | 2.8.3 | Unlicense OR MIT | 1.61 | the search in the bytes |
| `sync_wrapper` | 1.0.2 | Apache-2.0 | — | the shared access to the socket |

**The assessment.** Every licence of the set is MIT, Apache-2.0, BSD-3-Clause, the
Unlicense, or a combination of those. The two that deserve a word:

- `rustix` carries the **LLVM exception**, which is the exception of the Apache-2.0
  licence for the compiler runtimes; the project uses it as an ordinary Apache-2.0 crate.
  `deny.toml` allows the three expressions of the crate.
- `sync_wrapper` is **Apache-2.0 alone** (no MIT alternative). Apache-2.0 is a
  permissive licence with an explicit patent grant, and it is compatible with AGPL-3.0.
  `deny.toml` allows it.

No crate of the set is a copyleft licence that would reach the work of the project
(`GPL`, `LGPL`, `AGPL`, `SSPL`), no crate is a source-available licence, and none of
them phones home: the network stack opens the sockets of the server and nothing else.

---

## 2a. Why the path identity needs two Unicode crates

The catalogue identity of a path is the normalised and case-folded form of its
components (ADR-0007). Two crates are needed for that, and the reason for each is
different:

- **`unicode-normalization`** provides NFC. macOS preserves whichever normalisation
  form a file was written in, so two spellings of "café" can reach the server as
  different byte sequences for the same file.
- **`unicase`** provides the *full* Unicode case folding. A simple lowercase
  mapping would treat `Straße` and `STRASSE` as two different names, although a
  case-insensitive volume treats them as one file — and then the catalogue would
  show the film twice.

Both are permissively licensed; `unicode-normalization` additionally carries the
`Unicode-3.0` licence, which the allow-list in `deny.toml` permits.

## 3. The MPL-2.0 assessment (`symphonia`)

`symphonia` is the pure-Rust audio decoding library we intend to use. It is
**MPL-2.0**, which is file-level copyleft.

**Compatibility:** MPL-2.0 §3 permits combining MPL-covered files with other files
under different terms, provided the MPL-covered files remain under MPL-2.0 and their
source is made available. An AGPL-3.0-or-later work that links `symphonia` is
therefore compatible, provided:

- the `symphonia` files keep their MPL-2.0 notices and are not relicensed,
- the corresponding source of the MPL-covered files is part of the source offer
  (`OPS-016`, `SEC-070`), which the AGPL §13 offer already requires to be complete.

**Decision:** adopt `symphonia` with the conditions above, and list it in the
`cargo-deny` allow-list as `MPL-2.0` with the reason recorded here.

**Condition:** the `symphonia` dependency must not be the *only* way to read audio
metadata. The tag-reading path uses `lofty` (MIT/Apache-2.0) so that a licence
question about metadata extraction can never block the catalogue.

---

## 4. Crates we deliberately do **not** use

| Crate / component | Reason |
| --- | --- |
| `cargo-lint` | Does not exist in a usable form: a crates.io stub from 2020 whose repository is gone. |
| `rumperd` | The commonly cited Rust UPnP stack is unmaintained and oriented at the IGD/embedded use case, not at a media server. We implement SSDP ourselves (ADR-0005). |
| `cocoa` | Legacy: built on the old `objc` crate, only a small fraction documented, platform metadata omits `aarch64-apple-darwin`. `objc2` supersedes it. |
| `dispatch` | Unreleased since 2020; no stable crate for the Objective-C block/dispatch model. |
| `mp4` | Unmaintained since 2023. |
| `sqlx` | Heavier than needed; `rusqlite` covers the access pattern with fewer moving parts. |
| `rust-code-analysis-cli` | Stale (last release 2023) and MPL-2.0; used at most as advisory information, never as a gating check. |
| `ffmpeg-next` / `ffmpeg-sys` / any bundled FFmpeg | Links GPL/LGPL code into an AGPL product and drags a native toolchain into every build and every distribution. See §1 rule 7. |
| `libupnp` / `pupnp` bindings | C library with its own licence and behaviour; our protocol surface is small enough to own (ADR-0005). |
| `notify` 9.0.0-rc.x | A release candidate. Pin the stable 8.x line. |
| Any dependency with an unreviewed `build.rs` or proc-macro | Build scripts execute arbitrary code on every machine that builds the project. |

---

## 5. Runtime and XML: the decisions that matter

### 5.1 Async runtime and HTTP

**Decision:** Tokio 1.x as the runtime, Hyper 1.x as the HTTP/1.1 implementation,
Axum 0.8 for routing, Tower for middleware, `tower-http` for tracing and timeouts.

Rationale: Hyper has the deepest battle history against request smuggling and
header-injection defects — the exact class of bug that matters most here — and the
whole Rust ecosystem converges on it. The decision is written up as ADR-0002.

**Constraint:** the HTTP layer must reject conflicting framing headers itself
(M-04). A framework that "fixes" framing silently is a security problem, so the
framing policy is ours and is covered by negative tests.

### 5.2 XML

**Decision:** `quick-xml` for parsing, because it is a **streaming** parser: memory
use is proportional to the current element, not to the document. `roxmltree` builds a
whole DOM and resolves DTD entities unless `allow_dtd` is disabled, which makes it the
wrong shape for a hostile network input.

**Two facts that shape the implementation:**

1. `quick-xml` *surfaces* `DocType` and `GeneralRef` events to the caller. The
   server therefore rejects any document containing a doctype and any general entity
   reference — the decision is ours, not the parser's (`SEC-010`, M-02).
2. A streaming parser still performs **crate-internal allocations before the consumer
   sees the event**. Bounding the input is therefore necessary but not sufficient:
   the byte limit must be enforced *before* the parser sees the data.

### 5.3 SQLite

**Decision:** `rusqlite` with the `bundled` feature (a pinned SQLite compiled into the
binary, so the server does not depend on the system's SQLite version) and
`loadable_extension` **disabled**.

`rusqlite` is MIT. Its `libsqlite3-sys` dependency builds SQLite itself, which is
public domain. This avoids the question of a system SQLite with unknown version or
patch level, and it keeps the Windows and Linux builds free of a system dependency.

---

## 6. Supply-chain controls

| Control | Tool | Rule |
| --- | --- | --- |
| Licences | `cargo deny check licenses` | Allow `AGPL-3.0-or-later`, `Apache-2.0`, `MIT`, `BSD-2/3`, `ISC`, `Unicode-3.0`, `Zlib`, `CC0-1.0`, `MPL-2.0`; deny everything else; `unused-allowed-license = warn` |
| Advisories | `cargo audit` | Fail on any un-waived RustSec advisory; a waiver needs a record with an expiry date |
| Bans | `cargo deny check bans` | Banned: the crates in §4, plus any crate with an unreviewed `build.rs` |
| Sources | `cargo deny check sources` | Only crates.io and the project's own registry; a new source needs a review |
| Duplicates | `cargo deny check bans` with `multiple-versions` | Warn, and record why a duplicate is acceptable |
| Lock file | `git diff Cargo.lock` | Every change is reviewed like source |
| Auditable binaries | `cargo auditable` | Release builds embed the dependency tree |
| Typosquatting | manual review | Prefer crates with several maintainers, a `SECURITY.md` and a non-trivial download count; treat an unfamiliar name as suspicious |

The typosquatting risk is real and documented: crates such as `rustdecimal`,
`postgress` and `faster_log` were published on crates.io to steal credentials and
exfiltrate host data. The mitigation is not a scanner but a **process**: every new
dependency is read, recorded and reviewed by a named person.

---

## 7. Review record

| Item | Status | Reviewed by | Date |
| --- | --- | --- | --- |
| Runtime and HTTP stack | Proposed in ADR-0002 | Architect | 2026-11-14 |
| XML parser | Proposed in ADR-0002 | Architect | 2026-11-14 |
| SQLite binding | Proposed in ADR-0002 | Architect | 2026-11-14 |
| Media metadata stack | Proposed in ADR-0008 | Architect | 2026-11-14 |
| macOS platform crates | Proposed in ADR-0002 | Architect | 2026-11-14 |
| `symphonia` MPL-2.0 | **Accepted with conditions** (§3) | Security | 2026-11-14 |
| FFmpeg exclusion | **Accepted** (§1 rule 7) | Security | 2026-11-14 |

Every proposal becomes an ADR before the dependency is added to `Cargo.toml`.
