# Dependency, licence and supply-chain review — PhairDLNA

Status: **In Review (v1.3 draft, 2026-11-19)** · Owner role: Security & Privacy Engineer
Version/date: 1.3 / 2026-11-19
Purpose: Record the licence, security and maintenance assessment of dependencies and candidates. The resolved graph inventory is being reconciled; this review is not yet complete.
References: `../00-governance/PROJECT-STANDARDS.md`, `../05-quality/QUALITY-GATES.md`, `../07-project/STATUS.md`
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

## 2. Candidate and direct-dependency review

The metadata below was researched on 2026-11-14. A candidate verdict is not evidence
that a crate is used. Direct and transitive membership is determined by the committed
`Cargo.lock`; its current membership was checked on 2026-11-19. Rows marked **not
adopted** are proposals only and are not part of the product's dependency graph.

Download counts are crates.io totals and are shown only as a rough indicator of
adoption, not as a quality judgement.

| Crate | Version | Licence | MSRV | Downloads | Updated | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| `tokio` | 1.54.1 | MIT | 1.85 | 1 193 567 705 | 2026-10-21 | **Adopt** — async runtime |
| `hyper` | 1.12.0 | MIT | 1.63 | 1 129 663 767 | 2026-10-06 | **Adopt** — HTTP/1.1 server |
| `axum` | 0.8.9 | MIT | 1.80 | 592 932 422 | 2026-04-14 | **Adopt** — routing, service layer |
| `tower` | 0.5.3 | MIT | — | 841 258 187 | 2026-01-12 | **Adopt** — middleware |
| `tower-http` | 0.7.1 | MIT | 1.65 | 621 132 531 | 2026-08-31 | **Adopted in current lock by DEC-078** — timeout feature is wired around handler futures; tracing integration and its privacy policy remain under review |
| `quick-xml` | 0.42.0 | MIT | 1.86 | 548 849 079 | 2026-08-22 | **Adopted in current lock by DEC-078** — bounded event adapter; project retains DTD/entity policy, tree construction and resource bounds |
| `roxmltree` | 0.21.2 | MIT OR Apache-2.0 | 1.60 | 106 279 045 | 2026-10-24 | **Alternative** — DOM; resolves DTD entities unless disabled |
| `rusqlite` | 0.42.0 | MIT | 1.89 | 154 994 331 | 2026-11-11 | **Adopt** — SQLite, bundled, extensions off |
| `sqlx` | 0.9.0 | MIT OR Apache-2.0 | 1.94 | 189 841 591 | 2026-05-01 | **Not adopted** — heavier, async-first; `rusqlite` is enough |
| `symphonia` | 0.6.1 | **MPL-2.0** | 1.85 | 21 837 379 | 2026-08-13 | **ADR-0008 target, not in current lock** — conditional MPL assessment is in §3; current M2.2 deviation is F-10 D-01 / ADR-0011 |
| `lofty` | 0.25.5 | MIT OR Apache-2.0 | 1.89 | 1 568 758 | 2026-10-25 | **Not adopted** — audio-tag and artwork candidate; absent from the current lock |
| `notify` | 8.2.2 | CC0-1.0 | 1.88 | 198 269 121 | 2026-11-09 | **Adopt** — stable 8.2.2 line with native FSEvents (macOS) and inotify (Linux) backends for FUN-025 automatic filesystem change monitoring |
| `ignore` | 0.4.34 | Unlicense OR MIT | 1.88 | 213 708 619 | 2026-10-27 | **Not adopted** — candidate for gitignore-style filtering; absent from the current lock |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 | 1.77 | — | 2026 | **Adopt** — the error types |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | 1.63 | 997 650 245 | 2026-03-11 | **Adopt** — temporary directories in tests (dev-dependency) |
| `walkdir` | 2.5.0 | MIT | — | 774 949 224 | 2024-03-01 | **Not adopted** — candidate traversal primitive; absent from the current lock |
| `clap` | 4.6.7 | MIT OR Apache-2.0 | 1.85 | 1 340 276 208 | 2026-09-14 | **Not adopted** — candidate CLI parser; absent from the current lock |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | 1.56 | 1 709 177 081 | 2026-07-18 | **Adopt** — serialisation |
| `toml` | 1.2.1+spec-1.1.0 | MIT OR Apache-2.0 | 1.85 | 1 141 698 378 | 2026-11-06 | **Adopt** — configuration format; exact lock version |
| `unicode-normalization` | 0.1.26 | (MIT OR Apache-2.0) AND Unicode-3.0 | 1.36 | 683 092 567 | 2026-10-16 | **Adopt** — NFC normalisation of path components |
| `unicase` | 2.10.0 | MIT OR Apache-2.0 | — | 492 183 614 | 2026-10-05 | **Adopt** — full Unicode case folding for the path identity |
| `tracing` | 0.1.44 | MIT | 1.65 | 1 050 071 464 | 2025-12-18 | **Not adopted** — candidate structured-logging crate; absent from the current lock |
| `uuid` | 1.29.2 | Apache-2.0 OR MIT | 1.89 | 992 221 000 | 2026-11-09 | **Adopt** — device UDN (v4) |
| `blake3` | 1.8.7 | CC0-1.0 OR Apache-2.0 (LLVM exception) | — | 227 947 308 | 2026-08-20 | **Adopt** — the object id hash of the catalogue (M1.9, DEC-027); the `pure` feature keeps the build free of a C compiler |
| `cpufeatures` | 0.3.1 | MIT OR Apache-2.0 | 1.85 | — | — | **Adopt** — resolved by the current `blake3` dependency; download and update metadata for this lock version not re-researched |
| `muda` | 0.21.2 | Apache-2.0 OR MIT | 1.90 | 51 346 421 | 2026-10-08 | **Evaluate** — macOS menu bar |
| `tray-icon` | 0.26.1 | MIT OR Apache-2.0 | 1.90 | 44 016 004 | 2026-10-07 | **Evaluate** — tray/status item |
| `objc2` | 0.6.5 | MIT | 1.71 | 162 826 258 | 2026-10-05 | **Adopted (macOS only, M2.6)** — modern Objective-C bindings; the runtime of the classes that receive the actions of the menu (ADR-0013) |
| `objc2-app-kit` | 0.3.2 | Zlib OR Apache-2.0 OR MIT | 1.71 | 89 033 565 | 2025-10-04 | **Adopted (macOS only, M2.6)** — `NSStatusItem`, `NSMenu`, `NSWorkspace`, preferences panels and the primary `NSWindow` overview |
| `objc2-foundation` | 0.3.2 | MIT | 1.71 | — | 2025-10-04 | **Adopted (macOS only, M2.6)** — the texts, the URLs and the timer of the application; new with M2.6 |
| `mp4` | 0.14.0 | MIT | — | — | 2023-08-01 | **Not adopted** — unmaintained since 2023 |
| `matroska` | 0.30.1 | MIT OR Apache-2.0 | — | — | 2026-04-15 | **Evaluate** — Matroska/WebM container parsing |
| `image` | 0.25.10 | MIT OR Apache-2.0 | 1.88 | — | 2026-03-10 | **Adopt** — the thumbnail of a photo (M2.2, § 2.4); only the `jpeg` and the `png` feature are compiled in |
| `infer` | 0.22.0 | MIT | 1.74 | — | 2026-07-15 | **Not adopted** — content-sniffing candidate; absent from the current lock |
| `mime_guess` | 2.0.5 | MIT | — | — | 2024-06-29 | **Not adopted** — extension-to-MIME candidate; absent from the current lock |
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

This is a reviewed candidate list, not a claim that every tool is installed or in
`Cargo.lock`. Of the crates shown, `tempfile` is currently a workspace dev-dependency;
the test-runner and audit commands are separately installed developer tools.

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
| `tokio-macros` | 2.7.2 | MIT | 1.71 | the macros of `tokio` |
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

### 2.4 The crates the picture decoder brings with it (M2.2)

The milestone M2.2 added the thumbnail of a photo. The decoder is the `image` crate,
and the project compiles **only** its `jpeg` and its `png` feature: a server that
serves JPEG and PNG does not need a decoder for GIF, TIFF, WebP or AVIF, and every
format that is left out is a decoder that is left out of the attack surface
(`SEC-067`). The following crates are **new** in the lock file because of it. Every
one of them is allowed by `deny.toml` and is covered by `cargo deny check licenses`
and by `cargo audit` like every other crate.

| Crate | Version | Licence | MSRV | Purpose |
| --- | --- | --- | --- | --- |
| `image` | 0.25.10 | MIT OR Apache-2.0 | 1.88 | the decoder and the encoder of the thumbnail |
| `png` | 0.18.1 | MIT OR Apache-2.0 | 1.88 | the PNG decoder |
| `zune-core` | 0.5.3 | MIT OR Apache-2.0 | — | the shared types of the JPEG decoder |
| `zune-jpeg` | 0.5.15 | MIT OR Apache-2.0 | — | the JPEG decoder |
| `bytemuck` | 1.25.2 | MIT OR Apache-2.0 | 1.60 | the byte casts of the decoded picture |
| `byteorder-lite` | 0.1.0 | MIT OR Apache-2.0 | — | the reading of the big-endian numbers |
| `moxcms` | 0.8.1 | MIT OR Apache-2.0 | — | the colour management of the pictures |
| `pxfm` | 0.1.30 | MIT OR Apache-2.0 | — | the pixel format helpers |
| `num-traits` | 0.2.19 | MIT OR Apache-2.0 | 1.60 | the numeric traits of the colour management |
| `crc32fast` | 1.5.5 | MIT OR Apache-2.0 | 1.65 | the checksum of the PNG chunks |
| `bitflags` | 2.13.2 | MIT OR Apache-2.0 | 1.56 | the flags of the PNG chunks |
| `flate2` | 1.1.11 | MIT OR Apache-2.0 | 1.63 | the decompression of the PNG data |
| `miniz_oxide` | 0.8.9 / 0.9.1 | MIT OR Zlib OR Apache-2.0 | 1.63 | Both versions are required by incompatible upstream bounds: `png` needs `^0.8`, while `flate2` needs `^0.9`; both dependency paths arrive through `image` |
| `adler2` | 2.0.1 | MIT OR Zlib OR Apache-2.0 | — | the checksum of the compressed data |
| `simd-adler32` | 0.3.10 | MIT OR Apache-2.0 | — | the fast checksum of the compressed data |
| `fdeflate` | 0.3.7 | MIT OR Apache-2.0 | — | the fast decompressor of the PNG data |
| `zlib-rs` | 0.6.8 | MIT OR Apache-2.0 | — | the alternative backend of `flate2` |
| `cfg-if` | 1.0.5 | MIT OR Apache-2.0 | — | the conditional compilation |
| `autocfg` | 1.5.1 | MIT OR Apache-2.0 | — | the build script of `num-traits` |

**The assessment.** Every licence of the set is MIT, Apache-2.0, Zlib or a combination
of those — all of them are allowed by `deny.toml` and all of them are compatible with
AGPL-3.0. The set contains no copyleft licence that would reach the work of the
project, no source-available licence, and nothing that phones home: the decoder reads
the bytes of a file and gives back the pixels.

**What the decoder is not allowed to do.** The decoder is the one part of the project
that turns bytes of a stranger into memory of the server. Two rules follow, and both
are enforced in `crates/phairdlna-core/src/media/thumbnail.rs`:

1. The **declared** size of the picture is checked before the picture is decoded, so
   that a file that claims enormous dimensions is refused before a single pixel is
   allocated (`SEC-067`).
2. The decoder is only reached for a picture that the library already holds inside a
   root. A file that lies outside the root never reaches it (`SEC-003`).

The decoder is a **pure-Rust** crate: it needs no C compiler, no system library and no
external program, so the core stays portable and the build matrix is unaffected
(ADR-0001, `QG-28`).

**Where the decoder runs.** Until the sandbox child of ADR-0008 exists (M-06), the decoder
runs **in the server process**. The picture pipeline reads only files that lie inside a
root, bounds every read and every allocation, and refuses a picture that claims enormous
dimensions before a pixel is allocated. The deviation from ADR-0008 is recorded in ADR-0011
and in DEC-042, and it ends with the child — the decoder was chosen so that it can move
into that child without a change of its interface.

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

## 3. Conditional MPL-2.0 assessment (`symphonia` candidate)

ADR-0008 names `symphonia` as the planned pure-Rust audio parser. It is **not in the
current `Cargo.lock`**: F-10 D-01 and ADR-0011 explicitly record the temporary M2.2
deviation, where the bounded picture parser runs in the server process before the
sandboxed child is implemented. The accepted architecture does not mean the crate is
already shipped. Its MPL-2.0 file-level copyleft must be assessed before M-06 adds it.

**Compatibility:** MPL-2.0 §3 permits combining MPL-covered files with other files
under different terms, provided the MPL-covered files remain under MPL-2.0 and their
source is made available. An AGPL-3.0-or-later work that links `symphonia` is
therefore compatible, provided:

- the `symphonia` files keep their MPL-2.0 notices and are not relicensed,
- the corresponding source of the MPL-covered files is part of the source offer
  (`OPS-016`, `SEC-070`), which the AGPL §13 offer already requires to be complete.

**Decision:** retain this compatibility analysis for M-06. Before adding the crate,
update the dependency review and add the licence to `deny.toml` in the same change.

**Condition retained from the original proposal:** if `symphonia` is considered again,
the feature owner must define how metadata remains readable without it. Neither
`symphonia` nor `lofty` is in the current `Cargo.lock`.

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

**Accepted target (ADR-0002):** Tokio 1.x as the runtime, Hyper 1.x as the HTTP/1.1
implementation, Axum 0.8 for routing, Tower for middleware, and `tower-http` for
tracing and timeouts. **Current implementation:** `tower-http` 0.7.1 is in the lock and
its timeout feature wraps the handler future in both routers. The independent
`DeadlineIo` still enforces reset-on-I/O read and write silence deadlines. The accepted
tracing role is not yet integrated; adding request logging requires an explicit
redaction-safe policy and review before M-02 can close.

Rationale: Hyper has the deepest battle history against request smuggling and
header-injection defects — the exact class of bug that matters most here — and the
whole Rust ecosystem converges on it. The decision is written up as ADR-0002.

**Constraint:** the HTTP layer must reject conflicting framing headers itself
(M-04). A framework that "fixes" framing silently is a security problem, so the
framing policy is ours and is covered by negative tests.

### 5.2 XML

**Accepted target (DEC-019 and ADR-0002):** `quick-xml` for parsing, because it is a
**streaming** parser: memory use is proportional to the current element, not to the
document. `roxmltree` builds a whole DOM and resolves DTD entities unless `allow_dtd`
is disabled, which makes it the wrong shape for hostile network input.

**Current implementation:** `quick-xml` 0.42.0 is in the lock and backs the bounded
adapter in `crates/phairdlna-core/src/upnp/xml_reader.rs`. The project still owns the
element tree, namespace resolution, entity allowlist, and byte/depth/node/text bounds.
DOCTYPE and custom entity references are rejected; predefined and legal XML 1.0 numeric
references are accepted. The 22-case XML reader suite passes, including 11 isolated
truncation cases with recovery and a positive depth-boundary case. The adapter builds a
complete tree, so its total application-level memory is proportional to the document;
the configured byte bound caps that allocation. Full hostile-input and allocation review
remains open.

**Two facts that shape the implementation:**

1. `quick-xml` *surfaces* `DocType` and `GeneralRef` events to the caller. The
   server therefore rejects any document containing a doctype and any custom entity
   reference; only the five predefined and valid numeric references are accepted — the
   decision is ours, not the parser's (`SEC-010`, M-02).
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

**What the decision costs, and who owns it.** The `bundled` feature compiles the SQLite
amalgamation with a **C compiler**, so the build of the workspace needs one. That is in
tension with the policy of § 1 rule 6 ("no native toolchain dependencies in v1"), with
`PLT-009` ("shall not require a C toolchain beyond what the platform already provides")
and with the description of the workspace that the core is free of a C compiler. The rule
of § 1 rule 6 says that a native dependency is an **owner decision**, and the decision
was taken for SQLite in ADR-0006 and in § 5.3 of this document; the rule was therefore
not broken silently, but it was bent. The dependency was **adopted in M2.9** with the
version `0.42.0` — the version of the table in § 2, whose MSRV of 1.89 is the MSRV of
the workspace — and with the features `bundled` and without `loadable_extension`; the
support of extensions stays switched off, so nothing can load code into the index of the
server.

The consequences that are still open are named here so that the owner may overrule the
decision: the build of the workspace needs a C compiler, the binary grows by the size of
the SQLite amalgamation, and the claim of the workspace that the core is free of a C
compiler is no longer true. The index of the catalogue is the only user of SQLite in the
project, and it lives in `phairdlna-core` — the crate that was meant to stay portable
(`QG-28`).

---

## 6. Supply-chain controls

### 6.1 Current resolved graph check

The committed lock file was inspected with `cargo metadata --locked --offline
--all-features --format-version 1` on 2026-11-19. Cargo reported **126 package
entries: 4 workspace packages and 122 registry packages, representing 121 distinct
third-party crate names**. This metadata includes target-specific and optional
packages; it is not evidence that every package ships in every build. The configured
licence gate, `cargo deny check licenses`, passed on the same date (`licenses ok`).

That gate proves that the resolved licence expressions satisfy `deny.toml`; it does
not prove that every resolved crate has the policy-required purpose, maintenance,
alternative and named-reviewer assessment. Sections 2.1–2.4 record the reviewed direct
dependencies and selected transitive families. The table below adds the 47 graph crate
names that those sections did not name, with their locked version, declared licence and
shortest route from a workspace crate. Together, the sections name all 121 distinct
third-party crate names in this graph. This is an inventory cross-check, not a completed
per-crate assessment: maintenance, alternatives and review evidence remain incomplete
for transitive packages. The project policy therefore remains open before M-02 closes.
Target-only, development and build-time crates must not be mistaken for runtime product
dependencies; the routes are provenance, not proof that a package is compiled into each
artifact. Each distinct version of a duplicate must also be explained. Do not treat the
passing licence gate as completion of the dependency review.

The shortest paths are derived from the locked, all-features graph. `normal`, `dev` and
`build` edges can differ by target; use `cargo tree --target <triple>` to confirm the
artifact-specific graph before making a runtime claim.

| Crate | Lock version | Licence metadata | Shortest dependency path from a workspace crate |
| --- | --- | --- | --- |
| `atomic-waker` | `1.1.2` | `Apache-2.0 OR MIT` | `phairdlna-admin → axum → hyper → atomic-waker` |
| `block2` | `0.6.2` | `MIT` | `phairdlna-mac → objc2-app-kit → block2` |
| `bumpalo` | `3.20.4` | `MIT OR Apache-2.0` | `phairdlna-admin → uuid → wasm-bindgen → wasm-bindgen-macro → wasm-bindgen-macro-support → bumpalo` |
| `dispatch2` | `0.3.1` | `Zlib OR Apache-2.0 OR MIT` | `phairdlna-mac → objc2-app-kit → objc2-core-foundation → dispatch2` |
| `equivalent` | `1.0.3` | `Apache-2.0 OR MIT` | `phairdlna-core → toml → indexmap → equivalent` |
| `fallible-iterator` | `0.3.0` | `MIT/Apache-2.0` | `phairdlna-core → rusqlite → fallible-iterator` |
| `fallible-streaming-iterator` | `0.1.9` | `MIT/Apache-2.0` | `phairdlna-core → rusqlite → fallible-streaming-iterator` |
| `getrandom` | `0.4.3` | `MIT OR Apache-2.0` | `phairdlna-admin → tempfile → getrandom` |
| `hashbrown` | `0.17.1` | `MIT OR Apache-2.0` | `phairdlna-core → toml → indexmap → hashbrown` |
| `indexmap` | `2.14.2` | `Apache-2.0 OR MIT` | `phairdlna-core → toml → indexmap` |
| `js-sys` | `0.3.107` | `MIT OR Apache-2.0` | `phairdlna-admin → uuid → js-sys` |
| `linux-raw-sys` | `0.12.1` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | `phairdlna-admin → tempfile → rustix → linux-raw-sys` |
| `objc2-cloud-kit` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | `phairdlna-mac → objc2-app-kit → objc2-cloud-kit` |
| `objc2-core-data` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | `phairdlna-mac → objc2-app-kit → objc2-core-data` |
| `objc2-core-foundation` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | `phairdlna-mac → objc2-app-kit → objc2-core-foundation` |
| `objc2-core-graphics` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | `phairdlna-mac → objc2-app-kit → objc2-core-graphics` |
| `objc2-core-image` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | `phairdlna-mac → objc2-app-kit → objc2-core-image` |
| `objc2-core-text` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | `phairdlna-mac → objc2-app-kit → objc2-core-text` |
| `objc2-core-video` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | `phairdlna-mac → objc2-app-kit → objc2-core-video` |
| `objc2-encode` | `4.1.0` | `MIT` | `phairdlna-mac → objc2 → objc2-encode` |
| `objc2-io-surface` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | `phairdlna-mac → objc2-app-kit → objc2-core-graphics → objc2-io-surface` |
| `objc2-quartz-core` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | `phairdlna-mac → objc2-app-kit → objc2-quartz-core` |
| `pkg-config` | `0.3.34` | `MIT OR Apache-2.0` | `phairdlna-core → rusqlite → libsqlite3-sys → pkg-config` |
| `proc-macro2` | `1.0.107` | `MIT OR Apache-2.0` | `phairdlna-admin → tokio → tokio-macros → proc-macro2` |
| `quote` | `1.0.47` | `MIT OR Apache-2.0` | `phairdlna-admin → tokio → tokio-macros → quote` |
| `r-efi` | `6.0.0` | `MIT OR Apache-2.0 OR LGPL-2.1-or-later` | `phairdlna-admin → tempfile → getrandom → r-efi` |
| `rustversion` | `1.0.23` | `MIT OR Apache-2.0` | `phairdlna-admin → uuid → wasm-bindgen → rustversion` |
| `serde_core` | `1.0.229` | `MIT OR Apache-2.0` | `phairdlna-admin → axum → serde_core` |
| `serde_derive` | `1.0.229` | `MIT OR Apache-2.0` | `phairdlna-core → serde → serde_derive` |
| `serde_spanned` | `1.1.3` | `MIT OR Apache-2.0` | `phairdlna-core → toml → serde_spanned` |
| `smallvec` | `1.16.5` | `MIT OR Apache-2.0` | `phairdlna-core → rusqlite → smallvec` |
| `syn` | `3.0.10` | `MIT OR Apache-2.0` | `phairdlna-admin → tokio → tokio-macros → syn` |
| `thiserror-impl` | `2.0.21` | `MIT OR Apache-2.0` | `phairdlna-core → thiserror → thiserror-impl` |
| `tinyvec` | `1.13.3` | `Zlib OR Apache-2.0 OR MIT` | `phairdlna-core → unicode-normalization → tinyvec` |
| `toml_datetime` | `1.1.3+spec-1.1.0` | `MIT OR Apache-2.0` | `phairdlna-core → toml → toml_datetime` |
| `toml_parser` | `1.1.5+spec-1.1.0` | `MIT OR Apache-2.0` | `phairdlna-core → toml → toml_parser` |
| `toml_writer` | `1.2.0+spec-1.1.0` | `MIT OR Apache-2.0` | `phairdlna-core → toml → toml_writer` |
| `unicode-ident` | `1.0.27` | `(MIT OR Apache-2.0) AND Unicode-3.0` | `phairdlna-admin → tokio → tokio-macros → proc-macro2 → unicode-ident` |
| `vcpkg` | `0.2.15` | `MIT/Apache-2.0` | `phairdlna-core → rusqlite → libsqlite3-sys → vcpkg` |
| `wasi` | `0.11.1+wasi-snapshot-preview1` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | `phairdlna-admin → tokio → mio → wasi` |
| `wasm-bindgen` | `0.2.130` | `MIT OR Apache-2.0` | `phairdlna-admin → uuid → wasm-bindgen` |
| `wasm-bindgen-macro` | `0.2.130` | `MIT OR Apache-2.0` | `phairdlna-admin → uuid → wasm-bindgen → wasm-bindgen-macro` |
| `wasm-bindgen-macro-support` | `0.2.130` | `MIT OR Apache-2.0` | `phairdlna-admin → uuid → wasm-bindgen → wasm-bindgen-macro → wasm-bindgen-macro-support` |
| `wasm-bindgen-shared` | `0.2.130` | `MIT OR Apache-2.0` | `phairdlna-admin → uuid → wasm-bindgen → wasm-bindgen-shared` |
| `windows-link` | `0.2.1` | `MIT OR Apache-2.0` | `phairdlna-admin → tempfile → windows-sys → windows-link` |
| `windows-sys` | `0.61.2` | `MIT OR Apache-2.0` | `phairdlna-admin → tempfile → windows-sys` |
| `winnow` | `1.0.5` | `MIT` | `phairdlna-core → toml → winnow` |

| Control | Tool | Rule |
| --- | --- | --- |
| Licences | `cargo deny check licenses` | Allow `AGPL-3.0-or-later`, `Apache-2.0`, `Apache-2.0 WITH LLVM-exception`, `MIT`, `BSD-3-Clause`, `Unicode-3.0`, `Zlib`, `CC0-1.0`, `Unlicense`; deny everything else; add a licence only when an assessed dependency requires it |
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
| `symphonia` MPL-2.0 | **ADR-0008 target; not in current lock; conditional review for M-06** (§3) | Security | 2026-11-14 |
| FFmpeg exclusion | **Accepted** (§1 rule 7) | Security | 2026-11-14 |
| `image` (the thumbnail decoder) | **Accepted with conditions** (§ 2.4) | Security | 2026-11-16 |
| `rusqlite` (the index of the catalogue) | **Adopted** in M2.9 — the decision of § 5.3, the version `0.42.0`, `bundled` without `loadable_extension`; the C compiler that the build needs is an owner decision that is recorded and may be overruled | Security | 2026-11-17 |

Every proposal becomes an ADR before the dependency is added to `Cargo.toml`.

## 8. Revision history

| Version/date | Change |
| --- | --- |
| 1.3 / 2026-11-19 | Records the all-features locked graph size and passing licence gate; makes explicit that the per-crate purpose, maintenance, alternatives and reviewer inventory remains incomplete. |
| 1.2 / 2026-11-19 | Records the DEC-078 direct dependencies and targeted test evidence; keeps tracing integration, full parser security review and M-02 closure open. |
| 1.1 / 2026-11-19 | Separates reviewed candidates from the actual lock graph, records the ADR-0002 implementation discrepancies and `miniz_oxide` version paths, reconciles resolved crate versions, and removes unused licence allowances from the active gate configuration. |
