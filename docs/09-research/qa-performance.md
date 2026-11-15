# PhairDLNA — QA & Performance Test Strategy (Requirements Phase)

**Document status:** requirements-phase design; no implementation exists yet, so nothing here is a measured result.
**Author:** QA & Performance Engineer.
**Date written:** 2026-11-14 (all crates.io retrieval dates in this document: **2026-11-14 UTC**).
**Scope:** new-from-scratch, pure-Rust DLNA/UPnP AV media server, macOS-first, AGPL-3.0, manual-only GitHub Actions (i.e. workflows are triggered by `workflow_dispatch`, never automatically on push/PR).

---

## 0. How to read this document

Every external claim carries a label and the URL that was actually fetched.

| Label | Meaning |
|---|---|
| **VERIFIED** | I fetched the URL during this research pass (2026-11-14) and observed the claim directly. All `crates.io` numbers come from the crates.io JSON API, e.g. `https://crates.io/api/v1/crates/assert_cmd` and `https://crates.io/api/v1/crates/<name>/versions`. |
| **ASSUMPTION** | A reasonable inference from VERIFIED evidence, but not directly confirmed by the cited source. Every ASSUMPTION that a later implementation depends on is listed again in §9 (Open Questions). |
| **OPEN QUESTION** | Requires a project decision or an experiment before it can be a gate. |

Three further conventions:

1. **"Scriptable"** in this document means: the tool has a non-interactive CLI, exits with a meaningful exit code, and can be run as one step of a `workflow_dispatch` GitHub Actions job without a TTY. This is the deciding factor for a *manual-only* pipeline.
2. **"Linux-only" / "macOS-only"** claims are always qualified with the evidence, because the common shorthand is often wrong (see the `valgrind` finding in §6 and the `/usr/bin/time -v` correction in §5).
3. Numbers reported by crates.io (download counts) are **popularity signals, not quality signals**. They are recorded here so a reviewer can see how established a tool is; they are never used as a selection criterion on their own.

---

## 1. Test framework landscape

### 1.1 The runner: `cargo test` vs `cargo-nextest`

#### `cargo test` (built into the toolchain) — VERIFIED capabilities

`cargo test` can: run unit tests, integration tests (every file in `tests/`), doc tests (`cargo test --doc`), and benchmarks in `benches/`; filter by name (`cargo test foo`), run exactly (`--exact`), run ignored tests (`--ignored`), list tests (`--list`), compile without running (`--no-run`), and support custom test harnesses.

What `cargo test` cannot do (this list is the reason a second runner exists):

* **No cross-binary parallel scheduling.** Test *binaries* run one after another; parallelism exists only *inside* a binary (threads). A workspace with many integration-test binaries therefore serialises at the binary level.
* **No per-test process isolation.** A test that aborts the process (panic in a `#[should_panic]`-less context, `abort()`, OOM, signal) kills the remaining tests in that binary, and the runner reports one "failure", not "which tests never ran".
* **No run-state persistence, no retries, no slow-test classification, no JUnit output** built in.
* **No "did any test actually run?" guard.** An empty filter that matches nothing still exits 0 — a silent pass. (nextest exposes this directly as `NEXTEST_NO_TESTS=auto|pass|warn|fail`, default `auto` — VERIFIED, `https://nexte.st/docs/running/`.)
* No test-ordering/filter-set semantics, no per-test timeout default beyond the harness' own.

#### `cargo-nextest` — VERIFIED

* Runs **each test in its own process** (documented under "Why process-per-test?", `https://nexte.st/` navigation; the design is described at `https://nexte.st/docs/design/why-process-per-test/`).
* Schedules tests across binaries and cores → the wall-clock win comes from parallelism *and* from removing per-binary serialisation, not from making individual tests faster. For a project with many integration-test binaries (one per protocol surface), this is the single biggest CI-time lever.
* Features that matter for this project: filtersets (`--filter-expr`), retries (`--retries`), slow-test marking (tests > 60 s by default are marked `SLOW`), per-test-process recording/replay (`record-replay`), JUnit output for archiving, and `--no-tests` with `auto|pass|warn|fail`.
* **Limitation (VERIFIED, important):** "Doctests are currently not supported because of limitations in stable Rust. For now, run doctests in a separate step with `cargo test --doc`." — `https://nexte.st/docs/running/`. So a nextest-based pipeline **must** have a separate doctest step, otherwise doc tests silently never run. This is a prime candidate for a NOT-RUN-vs-PASS confusion and is handled by gate QG-05 and the labelling policy in §8.
* Installation: pre-built binaries are distributed via an install script (documented at `https://nexte.st/docs/installation/pre-built-binaries/`), so CI does not need to compile it — but compiling from source is also supported.
* **Naming trap (VERIFIED):** the crates.io crate named `nextest` is a *stub* — "Stub crate, you likely want cargo-nextest instead", version 0.1.1, 31,246 total downloads (`https://crates.io/api/v1/crates/nextest`). The real crate is **`cargo-nextest`**, version **0.9.152**, 13,877,198 downloads, Apache-2.0 OR MIT (`https://crates.io/api/v1/crates/cargo-nextest`). Anyone installing "the nextest crate" from crates.io has installed the wrong thing.

**Recommendation:** `cargo test` remains the developer-loop runner (fastest to type, no extra binary), `cargo-nextest` is the CI runner for unit+integration tests, with an explicit `cargo test --doc` step for doctests.

### 1.2 Tool inventory (crates.io data retrieved 2026-11-14)

`max_version` and license are the values returned by the crates.io API for that version; downloads are the crate's lifetime total and 90-day figures as returned by the same API.

| Tool (crate) | Version | License | Downloads (total / 90d) | Platforms | Scriptable in manual GHA | Role in PhairDLNA |
|---|---|---|---|---|---|---|
| `cargo-nextest` | 0.9.152 | Apache-2.0 OR MIT | 13,877,198 / 1,899,135 | macOS, Linux, Windows (prebuilt binaries — VERIFIED, nexte.st) | Yes | CI test runner |
| `assert_cmd` | 2.2.2 | MIT OR Apache-2.0 | 98,640,291 / 21,654,423 | all (pure Rust) — ASSUMPTION | Yes | CLI integration tests of the `phairdlna` binary |
| `predicates` | 3.1.4 | MIT OR Apache-2.0 | 258,781,289 / 49,239,399 | all — ASSUMPTION | Yes | Assertion predicates (`contains`, `predicate::str::similar`) |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | 997,650,245 / 256,912,994 | all — ASSUMPTION | Yes | Temp dirs/files for library tests (indexer, path handling) |
| `mockall` | 0.15.0 | MIT OR Apache-2.0 | 193,300,361 / 35,836,688 | all — ASSUMPTION | Yes | Trait mocks (used sparingly; see below) |
| `wiremock` | 0.6.5 | MIT/Apache-2.0 | 105,193,209 / 36,036,221 | all — ASSUMPTION | Yes | **Preferred** HTTP mocking (declarative, async-friendly) |
| `httpmock` | 0.9.0 | MIT | 34,405,560 / 7,063,331 | all — ASSUMPTION | Yes | Alternative HTTP mocking (server + client API) |
| `mockito` | 1.7.2 | MIT | 63,258,208 / 10,598,895 | all — ASSUMPTION | Yes | Lightweight HTTP mocking (good for small client-side tests) |
| `insta` | 1.49.0 | Apache-2.0 | 129,270,361 / 40,136,768 | all — ASSUMPTION | Yes (+ `cargo-insta` CLI) | **DIDL-Lite XML golden snapshots** |
| `proptest` | 1.12.0 | MIT OR Apache-2.0 | 246,433,063 / 77,274,656 | all — ASSUMPTION | Yes | Property-based tests (escaping round-trip, path containment) |
| `quickcheck` | 1.1.0 | Unlicense OR MIT | 79,851,745 / 15,856,637 | all — ASSUMPTION | Yes | Fallback property testing (no proc-macro, simpler) |
| `cargo-fuzz` | 0.13.2 | MIT OR Apache-2.0 | 6,426,351 / 2,326,754 | x86-64 Linux, x86-64 macOS, aarch64 macOS, Windows (MSVC ASan) — VERIFIED | Yes (needs nightly toolchain — VERIFIED) | Fuzzing |
| `cargo-llvm-cov` | 0.9.1 | Apache-2.0 OR MIT | 9,202,160 / 1,987,520 | all (needs `llvm-tools-preview`) — VERIFIED (README) | Yes | Coverage reporting |
| `criterion` | 0.8.2 | Apache-2.0 OR MIT | 340,224,609 / 80,838,965 | all — ASSUMPTION | Yes (bench harness) | Micro-benchmarks with statistics |
| `cargo-deny` | 0.20.2 | MIT OR Apache-2.0 | 7,528,212 / 2,467,011 | all — ASSUMPTION | Yes | Licence / advisory / bans / sources checks |
| `cargo-audit` | 0.23.0 | Apache-2.0 OR MIT | 15,029,368 / 4,453,410 | all — ASSUMPTION | Yes | RustSec advisory check |
| `cargo-outdated` | 0.19.0 | MIT | 1,024,737 / 72,626 | all — ASSUMPTION | Yes (non-interactive flags) | Dependency-freshness report (**advisory only**) |
| `cargo-semver-checks` | 0.51.0 | Apache-2.0 OR MIT | 738,803 / 177,360 | all — ASSUMPTION | Yes | Public-API semver breakage gate (release) |
| `typos` | 0.10.44 | MIT OR Apache-2.0 | 718,014 / 133,785 | all — ASSUMPTION | Yes | Spelling/typo gate |
| `tokei` | 15.0.0 | MIT OR Apache-2.0 | 1,766,555 / 394,932 | all — ASSUMPTION | Yes | LOC census (input to the file-size gate) |
| `rust-code-analysis` | 0.0.25 | MPL-2.0 | 180,759 / 132,754 | Linux, macOS, Windows — VERIFIED (project docs) | Yes | **Complexity metrics** (COGNITIVE, CC, NOM, LOC) — library |
| `rust-code-analysis-cli` | 0.0.25 | MPL-2.0 | 88,722 / (see library) | Linux, macOS, Windows — VERIFIED | Yes | The CLI front-end for the metrics above |
| `cargo-bloat` | 0.12.1 | MIT | 442,439 / 63,684 | ELF, Mach-O, PE (no WASM) — VERIFIED (README) | Yes | Binary-size attribution |
| `lychee` | 0.24.2 | Apache-2.0 OR MIT | 255,300 / 80,252 | all — ASSUMPTION | Yes | Documentation link check |
| `cargo-mutants` | 27.1.0 | MIT | 928,323 / 454,326 | all — ASSUMPTION | Yes | Optional mutation testing (runs inside nextest) |
| `cargo-vet` | 0.10.3 | Apache-2.0/MIT | 733,852 / 140,621 | all — ASSUMPTION | Yes | Optional supply-chain attestations |
| `cargo-hack` | 0.6.46 | Apache-2.0 OR MIT | 2,014,197 / 210,959 | all — ASSUMPTION | Yes | Feature-matrix builds (`--feature-powerset`) |
| `cargo-public-api` | 0.52.0 | MIT | 407,003 / 172,738 | all — ASSUMPTION | Yes | API surface diffing (companion to semver-checks) |
| `libxml` | 0.3.23 | MIT | 2,630,490 / 614,065 | needs libxml2 C library — ASSUMPTION | Yes | **Rejected**: C bindings, `unsafe`, breaks pure-Rust policy (§2.4) |
| `xmlschema` | 0.0.14 | MIT OR Apache-2.0 | 4,835 / 2,178 | all (pure Rust, `#![forbid(unsafe_code)]`) — VERIFIED (README) | Yes | **Pure-Rust XSD validator** for DIDL-Lite |
| `xsd` | 0.5.0 | Unlicense | 188,427 / 117,972 | all — ASSUMPTION | Yes | **Codegen, not validation** (see §2.4) |

### 1.3 Per-tool notes (what each one is actually good for here)

**`assert_cmd` + `predicates`** — assert_cmd drives a binary as a subprocess and asserts on stdout/stderr/exit status; predicates provides the matcher vocabulary (`contains`, `starts_with`, regex-ish `similar`). Together they are the right tool for the `phairdlna` CLI surface (`phairdlna serve --config …`, `phairdlna index --root …`, `phairdlna doctor`). Docs: `https://docs.rs/assert_cmd/latest/assert_cmd/`, `https://docs.rs/predicates/latest/predicates/`.

**`tempfile`** — required for library tests that touch the filesystem (indexer scans, path canonicalisation, permissions). Use `TempDir` and its `into_path`/`keep` only where a test genuinely needs a surviving directory; otherwise the temp dir is removed on drop, which keeps the machine clean during long soaks.

**Mocking: `wiremock` vs `httpmock` vs `mockito` vs `mockall`**

* `wiremock` (0.6.5) — a declarative HTTP mock server: match on method/path/headers/body, respond with a template. Best fit for testing *our* HTTP server (ContentDirectory, AVTransport, file serving) from the client side, and for testing outbound HTTP clients we may add (e.g. metadata fetches).
* `httpmock` (0.9.0) — server + client, supports more protocols and a standalone binary mode; heavier than needed.
* `mockito` (1.7.2) — smallest surface; good for one-off client tests.
* `mockall` (0.15.0) — Rust-native trait mocking. Use only where a real implementation cannot be constructed cheaply; prefer fakes over mocks for stateful traits (index store, clock). Mock-heavy suites rot quickly because the mock encodes the implementation, not the contract.

**Decision:** `wiremock` for HTTP, hand-written fakes for internal traits, `mockall` only as an escape hatch.

**`insta` (1.49.0) — the DIDL-Lite golden mechanism**

Snapshot testing is the right tool for XML because the failure mode we care about is "the DIDL-Lite document changed shape in a way that breaks a renderer", which unit assertions on individual fields will not catch.

Operational facts (VERIFIED, `https://insta.rs/docs/settings/`, `https://insta.rs/docs/cli/`):

* `insta` writes `.snap` files next to the test; the companion CLI `cargo-insta` provides `review`, `accept`, `reject`, `pending-snapshots`, `test`.
* `INSTA_UPDATE` accepts `auto` (default), `always`, `new`, `unseen`, `no`. **In CI we set `INSTA_UPDATE=no`** so a snapshot difference fails the gate instead of silently rewriting the golden file.
* **`INSTA_FORCE_PASS` exists** (`behavior.force_pass`, default `false`). If that variable is set, a mismatching snapshot can be forced to "pass". Our CI must assert `INSTA_FORCE_PASS` is unset, and the gate record must include the env values used (see §8).
* Snapshot review is a *human* step (`cargo insta review`) → in a manual-only pipeline this is by design: the reviewer looks at the diff and accepts it deliberately.

**`proptest` (1.12.0) vs `quickcheck` (1.1.0)** — proptest is the richer choice (strategies, shrinking, `proptest-regressions/` files that persist failing inputs, `ProptestConfig` for case counts and time budgets). quickcheck is the minimal, dependency-light fallback. We use proptest for the two properties that matter (§2.6) and quickcheck is not adopted — one property framework is better than two.

**`cargo-fuzz` (0.13.2)** — see §4 for the nightly-toolchain constraint (VERIFIED) and target/corpus layout.

**`cargo-llvm-cov` (0.9.1)** — LLVM source-based coverage; README documents the `llvm-tools-preview` rustup component and `CARGO_LLVM_COV_SETUP` to control behaviour when the component is missing (VERIFIED, `https://raw.githubusercontent.com/taiki-e/cargo-llvm-cov/main/README.md`). Coverage is a *visibility* metric, not a gate threshold, in the first milestones; a hard percentage gate on a project without a baseline is theatre.

**`criterion` (0.8.2)** — VERIFIED from `https://docs.rs/criterion/latest/criterion/`: it is "a microbenchmarking library which aims to provide strong statistical confidence in detecting and estimating the size of performance improvements and regressions", it "collects detailed statistics", and its `measurement` module offers pluggable measurements (default `WallTime`, plus processor-time / CPU / GPU counters). Its report format is VERIFIED from `https://bheisler.github.io/criterion.rs/book/user_guide/command_line_output.html`: each run prints `slope`, `R^2`, `mean`, `std. dev.`, `median`, `med. abs. dev.`, outlier counts, and a `change:` line with a p-value ("change: [-38.292% -37.342% -36.524%] (p = 0.00 < 0.05) Performance has improved"); the default warm-up is three seconds and the default sample count is 100 ("Collecting 100 samples in estimated 13.354 s"). Criterion is used for hot-path micro-benchmarks (DIDL serialisation, index lookups, path canonicalisation). It is **not** a product-level benchmark: it cannot tell us how many concurrent streams a TV can pull.

**`cargo-deny` (0.20.2) — AGPL + MPL-2.0 configuration**

Config schema VERIFIED from `https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html`: `[licenses]` takes `allow` (SPDX v2.1 identifiers), `unused-allowed-license`, `confidence-threshold`; `[licenses.private]` with `ignore = true` and `registries`; `[[licenses.exceptions]]`; `[[licenses.clarify]]` for crates with non-SPDX or ambiguous expressions; `license-files` with a `path` and a `hash`.

Proposed `deny.toml` for PhairDLNA (AGPL-3.0 project; MPL-2.0 tolerated because it is file-level copyleft and does not force the whole work to be relicensed):

```toml
# deny.toml — PhairDLNA
[graph]
all-features = true            # license set must hold for every feature combination

[licenses]
# PhairDLNA itself is AGPL-3.0-only. Dependencies may be permissive, or AGPL, or
# file-level copyleft (MPL-2.0) whose file-level obligations we can satisfy.
allow = [
  "AGPL-3.0-only",
  "AGPL-3.0-or-later",
  "MPL-2.0",
  "MIT",
  "Apache-2.0",
  "Apache-2.0 WITH LLVM-exception",
  "BSD-2-Clause",
  "BSD-3-Clause",
  "ISC",
  "Unicode-3.0",
  "Unicode-DFS-2016",
  "Zlib",
  "CC0-1.0",
  "Unlicense",
]
unused-allowed-license = "warn"   # keep the allow-list tight over time
confidence-threshold = 0.93

[licenses.private]
ignore = true                     # do not deny unpublished/workspace crates

[licenses.exceptions]
# Example shape if a dependency needs an explicit exception (prefer a
# [[licenses.exceptions]] entry over widening `allow`):
#   name = "some-crate"
#   allow = ["AGPL-3.0-only"]

[[licenses.clarify]]
name = "ring"
expression = "MIT AND ISC AND OpenSSL"

[bans]
multiple-versions = "warn"
wildcards = "deny"
highlight = "all"

[advisories]
version = 2
yanked = "deny"
ignore = []                       # no blanket advisory ignores

[sources]
unknown-registry = "deny"
unknown-git = "deny"
allow-registry = ["https://github.com/rust-lang/crates.io-index"]
```

Notes:
* `unused-allowed-license = "warn"` keeps the allow-list honest; a `"deny"` there fails the build whenever an allowed licence is no longer used.
* `multiple-versions = "warn"` (not `deny`) — banning duplicate versions of common transitive crates is usually unachievable and produces gate noise; escalates to `deny` per-crate via `[[bans]]` where it matters.
* `MPL-2.0` in the allow-list is a *policy decision*, not a legal opinion: it is safe only if the MPL-covered files are kept separate and modified MPL files are published. **OPEN QUESTION** — confirm with the project's legal reviewer before QG-07 blocks a release.

**`cargo-audit` (0.23.0)** — checks `Cargo.lock` against the RustSec advisory database (`https://github.com/RustSec/advisory-db`, VERIFIED; human-readable at `https://rustsec.org/advisories/`, also exported to OSV). It needs network access to fetch advisories (or a pre-seeded DB) → in a manual-only pipeline, either cache the DB as a workflow artifact or accept the network dependency explicitly; a failed DB fetch must be reported as **NOT RUN / INFRA ERROR**, never as "no advisories found" (§8).

**`cargo-outdated` (0.19.0)** — prints a table of outdated direct/transitive dependencies (`https://raw.githubusercontent.com/kbknapp/cargo-outdated/master/README.md`). **Advisory only**: it must never block a build, because it reports the state of the ecosystem, not the correctness of the code.

**`cargo-semver-checks` (0.51.0)** — VERIFIED from `https://raw.githubusercontent.com/obi1kenobi/cargo-semver-checks/main/README.md`: `cargo semver-checks` (or `check-release`) compares the current public API against a baseline revision and reports each breaking change against the SemVer rules; it supports `--baseline-rev`, `--target <triple>` (important for us: `aarch64-apple-darwin` and `x86_64-unknown-linux-gnu` produce different APIs), and by default checks all features except ones named `unstable`, `nightly`, `bench`, `no_std` or prefixed `_`. For a server binary with a library core, this is the release gate that keeps a published crate's API compatible.

**`typos` (0.10.44)** — spell checker; VERIFIED from `https://raw.githubusercontent.com/crate-ci/typos/master/README.md`: false positives are marked by adding entries to `_typos.toml` (e.g. `[default] extend-ignore-identifiers-re = [...]`). Since DLNA is full of protocol vocabulary (`DIDL-Lite`, `upnp`, `protocolInfo`, `AVTransport`), expect a real allow-list file. Note the tool's own crate is named `typos` while the binary is `typos` (`cargo install typos-cli` installs the binary) — VERIFIED naming detail to avoid a wrong install command.

**`tokei` (15.0.0)** — LOC census per language (`https://raw.githubusercontent.com/XAMPPRocky/tokei/master/README.md`). It **reports**; it does not enforce. It is the data source for our file-size gate (§7.3), not the gate itself.

**`rust-code-analysis` / `rust-code-analysis-cli` (0.0.25, MPL-2.0)** — VERIFIED from `https://mozilla.github.io/rust-code-analysis/print.html`: it computes maintainability metrics including `COGNITIVE` ("Cognitive complexity, measuring how complex it is to understand a unit of code"), `CC` (cyclomatic complexity), `NOM`, `LOC`, `CLOC`, `BLANK`, `HALSTEAD`, and states "rust-code-analysis can run on the most common platforms: Linux, macOS, and Windows". The project is tree-sitter based and the library is used by Firefox's code-quality tooling. **Caveat (VERIFIED):** the crates.io releases are 0.0.25, published 2023-01-13, while the GitHub repository was last pushed 2026-04-06 — i.e. the *published crate* is stale relative to the repository. Use it as an **advisory** complexity metric with a pinned version, and do not let a stale tool silently block releases.

**`cargo-bloat` (0.12.1)** — VERIFIED from `https://raw.githubusercontent.com/RazrFalcon/cargo-bloat/master/README.md`: "Supports ELF (Linux, BSD), Mach-O (macOS) and PE (Windows) binaries. WASM is not supported." Used to attribute release-binary size (e.g. a regression from a new transitive dependency).

**`lychee` (0.24.2)** — async link checker (`https://raw.githubusercontent.com/lycheeverse/lychee/master/README.md`); flags such as `--include-fragments`, `-x/--exclude`, and per-status-code handling are documented at `https://lychee.cli.rs/guides/cli/`.

**`cargo-mutants` (27.1.0)** — mutation testing; documented by nextest itself in its feature list (`https://nexte.st/` navigation → "cargo-mutants"). Optional; use it to test the *test suite* (does a test actually kill the mutant?), not the product.

---

## 2. Protocol conformance testing

### 2.1 What we can and cannot validate against an official schema

**VERIFIED:** the official DIDL-Lite XML Schema *is* publicly available:

* `http://upnp.org/schemas/av/didl-lite.xsd` — fetched 2026-11-14, HTTP 200, 18,456 bytes, `targetNamespace="urn:schemas-upnp-org:metadata-1-0/DIDL-Lite/"`.
* `http://upnp.org/schemas/av/didl-lite-v3-20130331.xsd` — fetched 2026-11-14, HTTP 200, 18,456 bytes (byte-identical size to the "latest" file).
* Normative reference: `ContentDirectory:4` service spec PDF (`https://upnp.org/specs/av/UPnP-av-ContentDirectory-v4-Service-20101231.pdf`, fetched 200, 2,731,236 bytes) lists, in its schema table, `didl-lite-vn-yyyymmdd.xsd`, `didl-lite-vn.xsd`, `didl-lite.xsd` under `http://www.upnp.org/schemas/av/`, and its references section states: *"Available at: http://www.upnp.org/schemas/av/didl-lite-v3-20101231.xsd. Latest version available at: http://www.upnp.org/schemas/av/didl-lite.xsd."* (text extracted locally with `pdftotext`).
* The schema's own imports (verified from the downloaded file):
  * `http://www.upnp.org/schemas/av/upnp.xsd` → fetched 200, 43,622 bytes
  * `http://www.upnp.org/schemas/av/av.xsd` → fetched 200, 31,960 bytes
  * `http://dublincore.org/schemas/xmls/simpledc20021212.xsd` → fetched 200, 2,917 bytes, `application/xml`
  * `http://www.w3.org/2001/03/xml.xsd` (W3C XML namespace schema)

**Plan:** vendor all four files into `crates/phairdlna-core/tests/schema/` at a pinned commit, rewrite the `schemaLocation` attributes to local relative paths (the upstream schema points at absolute `http://` URLs, which makes offline validation and reproducible CI impossible), and record the upstream URLs + SHA-256 of each file in a `schemas/SOURCES.md` so a reviewer can prove we validate against the *official* bytes.

**VERIFIED — the UPnP *device description* has no official XSD.** Every path tried under `http://upnp.org/schemas/device/` (e.g. `UPnP-v1.0-UDPDeviceArchitecture.xsd`, `device-1-0.xsd`, `UPnP-device-architecture-v1.1.xsd`) returned **HTTP 200 with an 84,533-byte HTML page** (the upnp.org "not found" page), not an XSD. The Device Architecture spec (`https://upnp.org/specs/arch/UPnP-arch-DeviceArchitecture-v1.0.pdf`, fetched 200) states that the UPnP Template Language "is written in XML syntax and is derived from XML Schema (Part 1: Structures, Part 2: Datatypes)" and explicitly notes "(Note that upnp-org is used instead of schemas-upnp-org in this case because an XML schema is not defined for each service ID.)"

Consequence for the plan: **device-description conformance cannot be "XSD-validated" — it must be structurally asserted.** We write our own assertion suite for the device description (element order, required attributes, UDN format, URLBase handling, serviceList contents) and, additionally, an *authored* XSD for our own output as a regression contract (clearly labelled "project-authored, not a UPnP standard").

**VERIFIED — upnp.org soft-404s.** A link checker that only inspects the HTTP status will report every missing `upnp.org/schemas/...` URL as "OK". Any docs-link gate touching upnp.org must also check content-type and byte size (see QG-11).

### 2.2 SSDP testing without real devices

**What to test.** A DLNA/UPnP media server must answer these SSDP requests:

| Case | Request shape | Expected server behaviour |
|---|---|---|
| M-SEARCH, ST = `upnp:rootdevice` | `M-SEARCH * HTTP/1.1\r\nHOST: 239.255.255.250:1900\r\nMAN: "ssdp:discover"\r\nMX: 1\r\nST: upnp:rootdevice\r\n\r\n` | respond with the server's root device advertisement |
| M-SEARCH, ST = `ssdp:all` | as above with `ST: ssdp:all` | respond with multiple advertisements |
| M-SEARCH, ST = `urn:schemas-upnp-org:device:MediaServer:1` | as above | respond only if that device type is offered |
| M-SEARCH, ST = `urn:schemas-upnp-org:service:ContentDirectory:1` | as above | respond |
| M-SEARCH, MX = 0 | `MX: 0` | respond immediately (no random delay) |
| M-SEARCH, MX = 5 | `MX: 5` | respond within MX seconds (randomised delay allowed) |
| M-SEARCH, missing `MAN` | no `MAN` header | ignore (RFC 2616 semantics); must not crash |
| M-SEARCH, wrong `MAN` | `MAN: "ssdp:discoverx"` | ignore |
| M-SEARCH, `MAN` without quotes | `MAN: ssdp:discover` | tolerate (be liberal) |
| M-SEARCH, LF-only line endings | `\n` instead of `\r\n` | tolerate or ignore; must not hang |
| M-SEARCH, duplicate `ST` headers | two `ST:` lines | handle deterministically |
| M-SEARCH, `ST` with trailing whitespace / mixed case | `ST: Upnp:rootdevice` | case-insensitive match |
| M-SEARCH, no `ST` and no `MAN` | | ignore |
| M-SEARCH, huge `ST` value | 8 KB `ST` | reject gracefully, no allocation blow-up |
| NOTIFY (server-initiated) | `NOTIFY * HTTP/1.1 ... NTS: ssdp:alive` | our own advertisements must be well-formed and correctly periodic (`ssdp:byebye` on shutdown) |
| Unsolicited / IPv4 vs IPv6 | M-SEARCH to `ff02::c` | respond over IPv6 with the right `HOST`/`LOCATION` |
| Wrong port | M-SEARCH to the server's unicast port | ignore silently |

**Where real captures come from.** VERIFIED: the Wireshark sample-capture repository (`https://wiki.wireshark.org/SampleCaptures`) publishes a capture described as *"Public_nic (libpcap) A bunch of SSDP (Universal Plug and Play protocol) announcements"* at
`https://wiki.wireshark.org/uploads/__moin_import__/attachments/SampleCaptures/Public_nic`
— fetched 2026-11-14, HTTP 200, 59,184 bytes, `pcap capture file, microsecond ts (little-endian) - version 2.4 (Ethernet, capture length 65535)`.
**Honest scope of that capture:** a byte-level scan shows **132 occurrences of `NOTIFY * HTTP/1.1` and 0 occurrences of `M-SEARCH`**. It is therefore a *notification* (server-advertisement) corpus, **not** an M-SEARCH corpus. The broader capture index (`https://www.netresec.com/?page=PcapFiles`) lists many public pcap repositories but is a directory, not an M-SEARCH corpus.

**Conclusion (important for planning):** the golden M-SEARCH corpus must be **captured by us** from real clients, because that is exactly the behaviour we must interoperate with and no public corpus of client M-SEARCH variants exists that we could verify.

### 2.3 Building the golden M-SEARCH corpus

Protocol for creating `tests/corpus/msearch/` (checked into the repo, reviewed like source):

1. **Capture on real clients.** On macOS: `sudo tcpdump -i en0 -s0 -w capture.pcap 'udp port 1900'` (multicast on `en0`). On Linux: `tcpdump -i any -s0 -w capture.pcap 'udp port 1900'`. Each capture must be accompanied by a metadata file recording: client software + version, OS + version, capture date/time, timezone, interface, and whether the capture was taken with the server running.
2. **Extract payloads.** `tshark -r capture.pcap -Y 'udp.port==1900' -T fields -e data.data` → hex → decode → save as raw HTTP text files (`*.msearch`), one file per datagram.
3. **Anonymise.** Strip device-specific values that would leak the lab network (client IPs, UDNs) while preserving the *structure* we care about (header order, casing, whitespace, `MX` values, `MAN` quoting).
4. **Classify** each file with a `cases.toml` entry: `id`, `source_client`, `captured_at`, `purpose` (e.g. `header_order`, `mx_bounds`, `malformed`), `expected_response` (`none` | `ok_rootdevice` | `ok_mediasserver` | `error`), `notes`.
5. **Provenance & licence.** Record where each capture came from and under what licence. *Do not* copy captures from third-party sites without checking terms; captures we generate ourselves are ours.
6. **Regression use.** Every corpus file gets a test that replays it against our SSDP handler and asserts the recorded `expected_response`. New corpus files therefore *require* a human decision — they are not auto-generated noise.

Minimum client coverage for the corpus: at least one UPnP/DLNA control point on each of — Windows (Windows Media Player / "Media Player" legacy UPnP stack), Android (BubbleUPnP / Kodi), iOS (BubbleUPnP / Infuse), macOS (e.g. VLC, Kodi, BubbleUPnP), a smart TV (Samsung Tizen / LG webOS), and a set-top box (Apple TV / Fire TV). We record what we actually have and mark the rest as planned — never as "tested".

### 2.4 Validating our XML: choosing a validator

**VERIFIED — `xsd` is not a validator.** The crates.io crate `xsd` 0.5.0 ("XSD.rs", repository `https://github.com/rust-rdf/rdf.rs`) generates Rust types from an XSD; it does not validate instance documents against it. Using `xsd` as a "DIDL-Lite validator" would be a category error.

**VERIFIED — `xmlschema` is a validator, and it is pure Rust.** `xmlschema` 0.0.14 (MPL-2.0 OR MIT? — **no**: license `MIT OR Apache-2.0`, verified from the crates.io API; repository `https://github.com/sebastienrousseau/xmlschema`) describes itself as "XML Schema (XSD) validation for Rust — the schema member of the oxml suite, with zero unsafe code". Its README (`https://raw.githubusercontent.com/sebastienrousseau/xmlschema/main/README.md`) states a Status table:

* supported: elements/model groups/cardinality/attributes, all 44 built-in simple types + 12 restriction facets, `xs:list`/`xs:union`, `xs:all`/`xs:group`/`xs:attributeGroup`, `xs:any`/`xs:anyAttribute` (namespace and `processContents`), complex-type derivation (extension, restriction, Particle Valid restriction), schema self-validity, `xs:pattern` (own engine, XSD dialect incl. class subtraction), identity constraints, `xs:import`/`xs:include` (caller-supplied `SchemaSource`), diagnostics with paths;
* **not** supported: **substitution groups** — and "an unsupported construct is skipped rather than rejected … **What was skipped is reported**", via `support::unsupported`, so "this document is valid" and "this document was checked" are never confused;
* conformance: **96.1% of the W3C suite's decided tests**, 247 tests plus the conformance suite.

The DIDL-Lite schema uses `xsd:any` with `processContents="strict"` and simple-type facets (e.g. `restricted` as `xsd:boolean`, `childCount` as `xsd:int`) — all in the supported set. The README also states the rationale: "Rust has no pure-Rust XSD validator … `libxml` … is C: it needs a build toolchain, contains `unsafe`, does not work in WebAssembly, and inherits libxml2's CVE stream."

**Decision (with caveats):**
* Adopt `xmlschema` as the DIDL-Lite validator, pinned to `=0.0.14`.
* **Caveat 1 (VERIFIED):** it is a **pre-1.0 release candidate** (`0.0.14`; the README says "This is the 0.0.14 release candidate; registry availability is verified separately before publication"). Pin exactly; re-verify at implementation time.
* **Caveat 2 (VERIFIED):** substitution groups are unsupported — DIDL-Lite does not use them, but if a future schema revision does, the validator will *skip* rather than fail. Our gate must therefore also record the `support::unsupported` audit output and refuse to claim "fully validated" when anything was skipped.
* **Caveat 3:** `xmlschema` requires a matching `oxml` family version (`xmlschema = "=0.0.14"`, `oxml = "=0.0.13"` per the README). Add both as **dev-dependencies** only.
* **Rejected alternative:** `libxml` 0.3.23 (MIT) — C bindings, `unsafe`, external libxml2 requirement: violates the pure-Rust/no-unsafe policy and adds a native build dependency to a macOS-first project.

### 2.5 SOAP action testing with a hand-written client

We implement a **minimal control point inside the test suite** (`crates/phairdlna-testkit`), not as a product feature. It performs the real discovery→description→SOAP sequence over real sockets against a server instance started by the test:

1. Send the recorded M-SEARCH corpus files to the server's SSDP socket; collect responses.
2. Parse the `LOCATION:` URL from the response; `GET` the device description; assert the XML parses and contains our device type, UDN, and `ContentDirectory` service.
3. `GET` the SCPD (`serviceList/service/SCPDURL`); assert the `Browse` action is present with the expected argument names.
4. `POST` the `Browse` SOAP action:
   `SOAPAction: "urn:schemas-upnp-org:service:ContentDirectory:1#Browse"`, body
   `<u:Browse xmlns:u="urn:schemas-upnp-org:service:ContentDirectory:1"><ObjectID>0</ObjectID><BrowseFlag>BrowseDirectChildren</BrowseFlag><Filter>*</Filter><StartingIndex>0</StartingIndex><RequestedCount>0</RequestedCount><SortCriteria></SortCriteria></u:Browse>`
5. Assert the SOAP envelope, the `Result` element, and the DIDL-Lite payload.
6. Validate the DIDL-Lite payload against the vendored official XSD with `xmlschema`.
7. Additionally validate the *device description* with structural assertions (no official XSD exists — §2.1).

The SOAP client must be **hand-written over `tokio`/`std::net`** rather than an HTTP client library, because the point is to control the exact bytes on the wire (this is what makes the test a conformance test rather than a library test). We deliberately do not use an HTTP crate's high-level API for this path. `SOAPAction` header quoting, `Content-Type: text/xml; charset="utf-8"`, and the `CONTENT-LENGTH`/chunked behaviour are all part of the surface we assert.

**Negative SOAP cases** (all must produce a well-formed UPnP error, never a hang or a panic):
unknown action; missing required argument; wrong argument type (`ObjectID` empty vs `0`); `RequestedCount` negative; `Filter` with unknown prefixes; `BrowseFlag` unknown value; `SearchCriteria` with malformed SQL; SOAPAction header missing; `Content-Type` wrong; HTTP chunked body truncated; header with invalid UTF-8 bytes; body larger than the limit; two `Content-Length` headers.

### 2.6 Property-based testing (the two properties that matter)

**Property P1 — DIDL-Lite escaping round-trips.**
For any metadata string `s` (title, artist, album, mime-type-ish fields, upnp:artist, etc.), `escape(s)` then XML-parse then `unescape` yields `s` exactly. Generate `s` with `proptest`'s `string::any()` plus adversarial generators: strings containing `& < > " '`, control characters (`\x00`–`\x1f`), non-BMP Unicode (`\u{1F600}`), combining marks, RTL overrides (`\u{202E}`), and strings that look like XML markup (`</dc:title>`).

Round-tripping must be **encoding-aware**: control characters that XML 1.0 forbids (`\x00`, `\x0b`, `\x0c`, `\x7f`–`\x9f` ranges) must be *rejected or replaced by policy*, not silently passed through — a browser/renderer that receives invalid XML may drop the whole document. The property therefore asserts the *document* still parses and the *value* is preserved or replaced per the documented policy.

**Property P2 — path canonicalisation never escapes the root.**
For any relative path `p` (including `..`, `.`, empty segments, symlink-ish names, NUL bytes, absolute paths, Windows-style drive letters, UNC paths, and trailing-dot names) and any root `r`: `resolve(r, p)` — after normalisation — is either inside `r` or returns an explicit error; it never yields a path whose canonical form starts outside `r`. This is a security property; it is also the kind of bug that a unit test with three hard-coded cases will miss.

Implementation: `proptest` with a shrinking-friendly generator; on failure, proptest writes the failing input into `proptest-regressions/<target>.txt` — these files are committed, so a discovered bug stays fixed. That artefact is also the fuzzing seed corpus (§4).

**Additional properties** (cheap, high value): index key ordering is stable and total; `Range` header parsing never panics and never returns a range outside the resource; playlist parsing preserves item order and rejects absolute URLs outside the media root.

### 2.7 The golden-file corpus for XML

Two complementary mechanisms, deliberately:

* **`insta` snapshots** for *didl-Lite document shape* — a snapshot of the serialised DIDL-Lite for a fixture library (one item per media type, one container, one item with a long description, one item with characters requiring escaping). Because XML attribute order and namespace prefixes are part of what renderers see, the snapshot captures the whole document, not just fields.
* **Explicit schema validation** via `xmlschema` for *standards conformance* — snapshot equality proves "unchanged"; schema validation proves "valid per the official XSD". Neither alone is enough.

Both must run on every PR-equivalent (manual) CI run; a snapshot diff is reviewed by a human before acceptance.

---

## 3. Cross-client testing

### 3.1 Tools that can act as a DLNA client or renderer

| Tool | What it is | Licence | State | Use in PhairDLNA |
|---|---|---|---|---|
| **gupnp-tools** (GNOME) | Universal Control Point, AV Control Point, Network Light, Upload | GPL-2.0 — VERIFIED (`https://raw.githubusercontent.com/GNOME/gupnp-tools/master/README.md`) | read-only GitHub mirror of `gitlab.gnome.org/GNOME/gupnp-tools` | **Primary open control point** for automated discovery/browse checks and manual smoke tests |
| **gupnp / gupnp-dlna** (GNOME) | UPnP framework; `gupnp-dlna` does DLNA media-profile guessing | LGPL-2.1 — VERIFIED (`https://github.com/GNOME/gupnp`) | active | profile-guessing cross-check for our `protocolInfo` values |
| **libupnp / pupnp** | Portable C SDK: control points, devices, bridges; `samples/tv`; Python integration tests; fuzzing targets | BSD-style — VERIFIED (`https://github.com/pupnp`) | active | reference implementation to diff behaviour against; its samples can act as a *server* we point our client at |
| **Platinum** (plutinosoft) | C++ UPnP SDK with `FileMediaServerTest` and `MediaRendererTest` executables | VERIFIED to exist (`https://github.com/plutinosoft/Platinum`) | legacy but buildable | act as a *renderer* to exercise our server's push/AVTransport path |
| **Cling** (4thline) | Java/Android UPnP stack + Workbench + MediaRenderer | LGPL/CDDL — **EOL**: the README states "Cling EOL: This project is no longer actively maintained, code may be outdated." (VERIFIED, `https://github.com/4thline/cling`) | unmaintained | historical reference only; **do not** build a test strategy on it |
| **gmrender-resurrect** | UPnP/DLNA renderer (mentioned in the pupnp README) | — | community | candidate DMR for AVTransport tests |
| **Cast Receiver Simulator** (Google) | Browser-based Cast receiver simulator used with `chrome://inspect` and the Command-and-Control (CaC) tool | Google Cast SDK terms — VERIFIED to exist (`https://developers.google.com/cast/docs/android_tv_receiver/debugging`, `https://github.com/googlecast/CastReceiver`) | vendor-hosted | useful only for DIAL/Cast-adjacent behaviour; **not** a DLNA client |

**No open-source DLNA *certification* tool exists for us to use.** VERIFIED from `https://www.dlna.org/` (fetched 2026-11-14): "The Digital Living Network Alliance ("DLNA") has dissolved and is no longer operating. This website, its contents, and certification and testing services and tools related to compliance with the DLNA Guidelines are now being provided by SpireSpark International, Inc. with permission from DLNA." The DLNA Conformance Test Tool (CTT), Media Capabilities Verification Tool (MCVT) and Link Protection Test Tool (LPTT) are proprietary and gated behind the DLNA/SpireSpark offering; UPnP's own CTT is a UIC-member tool. **Consequence:** we cannot certify. We build our own auditable conformance report and say so plainly in the README (never "DLNA certified").

### 3.2 The device matrix

A reproducible matrix is a **file in the repo**, not a wiki page. Proposed format: `qa/device-matrix.toml` (TOML parses in Rust with zero new dependencies) with one `[[case]]` per row:

```toml
[[case]]
id = "samsung-tizen-2023-browse-root"
client = "Samsung Tizen 2023 (built-in DLNA client)"
client_version = "Tizen 7.0 (firmware 2023-10)"
device_model = "UE43RU7500 (43\" UHD)"
firmware = "Tizen 7.0 / build 2023.10"
os = "Tizen"
network = "wifi 802.11ac, same VLAN as server"
media = "sample/mp4/h264_aac_1080p.mp4"
feature = "browse"          # browse | play | seek | resume | subtitle | transcode
expected = "browse returns 1 container + 3 items; item.res.protocolInfo=http-get:*:video/mp4:*"
observed = ""               # filled in by the tester
result = "not_run"          # pass | fail | blocked | not_run | inconclusive
tested_at = ""              # RFC3339 UTC
tester = ""                 # initials or handle
evidence = "qa/evidence/2026-11-14-samsung-tizen/browse-root.pcap"
sha256 = ""                 # sha256 of the evidence file
notes = ""
```

Rules that make it auditable:

1. `expected` must be written **before** the test run (it is the contract), and any change to `expected` after a failure requires a commit that explains why — otherwise "fix the test" is indistinguishable from "fix the expectation".
2. `observed` must quote the actual protocol data (a DIDL-Lite excerpt, a SOAP response, a byte range response), not paraphrase it.
3. `evidence` points at a stored artefact: a pcap, an HTTP transcript, a DIDL-Lite XML dump, or a screen recording. Artefacts are stored with a SHA-256 in the matrix row.
4. `result` uses the vocabulary of §8. `not_run` is the default and is **not** a pass.
5. A small `xtask` subcommand (`cargo xtask matrix check`) validates the file: every row has all fields, no duplicate ids, every `evidence` path exists, every `sha256` matches, and no row has `result = "pass"` without an `observed` value. It also prints the coverage summary (how many clients, how many firmware versions, how many media profiles) so gaps are visible.

### 3.3 Reproducible hardware runs

* Every physical run records: the server's git commit, the server's `--version` output, the host macOS version and hardware, the client model/firmware, the network topology (wired/wifi, VLAN, AP model), and the media fixture (a single immutable fixture set under `qa/fixtures/`, with per-file SHA-256).
* Fixtures must be small, synthetic and licence-clean: generated test patterns (e.g. `ffmpeg`-produced 5-second colour-bar MP4/MKV with two audio tracks, a WAV, a JPEG, an MP3, a subtitle file). Never ship commercial content.
* Two network topologies are mandatory, because they expose different bugs: **direct Ethernet** (no AP, no NAT) and **via a Wi-Fi AP with multicast filtering enabled/disabled**. Multicast SSDP is the single most common source of "works on my machine" failures.

---

## 4. Fuzzing & negative testing practice

### 4.1 The tool and its hard constraint

**VERIFIED** (`https://rust-fuzz.github.io/book/cargo-fuzz/setup.html`): libFuzzer "needs LLVM sanitizer support; this works on x86-64 Linux, x86-64 macOS and Apple-Silicon (aarch64) macOS, and Windows (thanks to the MSVC AddressSanitizer)"; "Requires a C++ compiler with C++11 support"; and **"This project requires the nightly compiler since it uses the `-Z` compiler flag to provide address sanitization."**

This is the practical issue the requirements phase must settle: **`cargo fuzz` cannot run on stable Rust.** Options:

| Option | Consequence |
|---|---|
| (a) Install a pinned nightly toolchain in the fuzzing workflow job only | ✅ recommended. Fuzzing is a separate manual workflow with `rustup toolchain install nightly-<pinned>` and `cargo +nightly-<pinned> fuzz …`. Stable remains the default toolchain for every other gate, so a nightly regression cannot silently affect normal builds. |
| (b) Make the whole project nightly | ❌ rejected: unstable features become accidental dependencies; MSRV and SemVer promises become meaningless. |
| (c) Skip fuzzing | ❌ rejected: XML/HTTP/Range parsing is exactly where memory-safety and panic bugs live. |

A note on the C++ requirement: cargo-fuzz needs a C++ toolchain for the libFuzzer runtime on the platforms where it is not prebuilt; on macOS, `xcode-select --install` provides clang/libc++. **ASSUMPTION** (not verified in the fetched page) that Apple's clang is sufficient; the setup page only says "Requires a C++ compiler with C++11 support". This must be verified on the target machine during implementation.

**VERIFIED** layout facts from the book (`https://rust-fuzz.github.io/book/print.html`): the default corpus directory is `fuzz/corpus/<target>`; crash artefacts are written under `fuzz/artifacts/<target>/` with names like `crash-<sha>`; and `cargo fuzz coverage <target> [corpus dirs]` exists for source-based coverage of a corpus.

### 4.2 Fuzz targets we will write

| Target | Entry point | Oracle / property | Seed corpus |
|---|---|---|---|
| `fuzz_didl_xml` | parse a DIDL-Lite document | never panics; never allocates unboundedly; either parses or returns a typed error | `qa/corpus/didl/*.xml` — every golden DIDL-Lite from §2.7, plus the official XSD's own examples |
| `fuzz_device_desc_xml` | parse a device description | never panics; required elements either present or a typed error | `qa/corpus/devicedesc/*.xml` — our own descriptions + captured ones from real devices |
| `fuzz_http_headers` | parse an arbitrary header block | never panics; no unbounded allocation from a single header; CRLF injection in a value is rejected, not echoed | `qa/corpus/http/*.bin` — raw header blocks captured from real clients, plus the corpus from `wiremock`/`httpmock` test fixtures |
| `fuzz_range_header` | parse `Range: bytes=a-b,c` | never panics; the returned range is always inside the resource or an explicit 416; `a > b` is rejected | `qa/corpus/range/*.txt` — every Range form in §5's table plus real client Range headers |
| `fuzz_m3u` | parse an M3U/M3U8 playlist | never panics; item count matches the number of parsed entries; no item escapes the media root; no infinite loop on a self-referencing playlist | `qa/corpus/m3u/*.m3u8` — hand-written pathological playlists (recursive `#EXT-X-KEY` chains, negative segment durations, 10k-item playlists) |
| `fuzz_path` | resolve a user-supplied path against the media root | **never** escapes the root (Property P2) | `qa/corpus/path/*.txt` — every path string from the proptest regression files (§2.6) |

**Seed corpus is mandatory, per target, and is committed to the repo.** A fuzz target without a committed seed corpus starts from zero coverage and will "find nothing" while proving nothing. Our rule: a target may only be merged if (i) it has ≥ 5 seeds in its corpus directory, (ii) the seeds are documented with their provenance, and (iii) `cargo fuzz run <target> -- -runs=1000` completes without a crash **and** without the target being optimised away (we assert the target reaches non-trivial code by requiring `cargo fuzz coverage <target>` to report coverage of the target file — VERIFIED that the `coverage` subcommand exists).

### 4.3 Honest fuzzing reporting

A fuzz run produces exactly one of two truthful statements. The project uses these strings verbatim:

* **"NO CRASH FOUND — not a pass."** Full form:
  `FUZZ RESULT: NO CRASH FOUND (not a pass). Target: fuzz_didl_xml. Duration: 60 min. Executions: 41,203,118. Corpus: 812 inputs (12 seeds + 800 generated). Toolchain: nightly-2026-09-15 (x86_64-apple-darwin). Coverage: fuzz/artifacts/coverage/didl_xml.txt (attached). Interpretation: no crash was observed in this run on this machine in this time budget; this is evidence of absence, not proof of absence.`
* **"CRASH FOUND — defect."** Full form:
  `FUZZ RESULT: CRASH FOUND (defect, blocks the milestone it touches). Target: fuzz_range_header. Artefact: fuzz/artifacts/fuzz_range_header/crash-<sha256>. Minimised input: <bytes>. Reproduce: cargo +nightly-2026-09-15 fuzz run fuzz_range_header fuzz/artifacts/fuzz_range_header/crash-<sha256>. Root cause: <one line>. Regression test added: <test path>.`

Additional honesty rules:

* Never write "fuzzing passed". Write "no crash found in <N> minutes". The pass/fail of the *gate* is separate and is about "the target built, the corpus was loaded, and the run completed without a crash" — that is a **smoke test**, exactly as the cargo-fuzz book describes CI usage: "It can be helpful, as a smoke test, to build and run your fuzz targets for a small amount of time in CI" (VERIFIED, `https://rust-fuzz.github.io/book/cargo-fuzz/ci.html`).
* Always record the **time budget** and the **executions count**. "No crash" without a duration is not a claim at all.
* Always record the **toolchain and platform**. A fuzz result on x86-64 Linux says nothing about aarch64 macOS.
* A crash artefact is a defect until it is (a) minimised, (b) reduced to a committed regression test, and (c) the fix is merged. The gate goes red on discovery, not on fix.
* Fuzzing does **not** replace the property tests: proptest gives reproducible, committed regressions; libFuzzer gives coverage-guided exploration. We need both, and we label them differently.

### 4.4 Negative testing beyond fuzzing

Negative tests that fuzzing will never find (because they are semantic, not structural) belong in explicit unit/integration tests:

* SSDP: the 16-case table in §2.2 (each with an expected response class).
* SOAP: the negative case list in §2.5.
* HTTP: `Range` semantics table (§5.4), conditional requests (`If-Modified-Since`, `If-None-Match`), `HEAD` parity with `GET`, `Content-Length` vs actual body mismatch, chunked bodies, `Connection: close` handling, and a request with `Host` mismatch (we are a single-origin server — decide and document the policy).
* Filesystem: permission-denied directories, files deleted between index and serve, symlink loops, symlinks pointing outside the root, files with invalid UTF-8 names, very long names (255-byte limit on macOS APFS), files changing size mid-stream (Range semantics during a live file).
* Indexer: media files that are truncated, non-media content with a media extension, media files with zero duration, duplicate content in multiple folders.

---

## 5. Performance methodology

### 5.1 What we measure, and the exact definition of each metric

| Metric | Definition (must be precise or it is not comparable) | Unit |
|---|---|---|
| **Idle RSS** | Resident set size of the server process after startup, index build finished, **zero** clients connected, sampled after a 60 s settle. | MiB |
| **Warm startup time** | Time from process launch to "server is ready to answer" — defined as: the SSDP socket is bound and the first `GET /description.xml` returns 200. Measured with a warm page cache (second run) and reported separately from cold start. | ms |
| **Index build throughput** | Number of media files indexed per second, measured on a fixed fixture set with a fixed file count, excluding the initial `walk`? No: **including** the walk, because the user waits for the walk. Report both `files/s` and total wall time. | files/s |
| **DIDL Browse latency** | Time to produce a `Browse` SOAP response for a container with N children, measured **server-side** (handler entry → response bytes ready) and **client-side** (request written → full response read). Report both; they answer different questions. | ms (p50/p95) |
| **Streaming throughput** | Bytes served per second to a single client over loopback, per container/codec, at a fixed bitrate budget. Report the *sustained* rate and the *instantaneous* p95 over 60 s. | MB/s |
| **Per-stream CPU** | CPU time consumed by the server process while serving one stream, divided by stream duration: `Δcpu_time / Δwall_time`, i.e. "cores used". Measured from `ps`/`top` deltas or `time -l` output, not from an in-process counter. | cores (1.0 = one core saturated) |
| **Memory per stream** | ΔRSS between "no streams" and "N streams steady state", divided by N. | MiB/stream |
| **Multi-stream behaviour** | Per-stream latency and throughput as a function of concurrent stream count (1, 2, 4, 8). The interesting metric is the *degradation curve*, not a single number. | table |

### 5.2 The measurement tools

**`criterion` (0.8.2)** — for **micro**-benchmarks: DIDL serialisation, index lookups, path canonicalisation, HTTP header parsing. VERIFIED (`https://docs.rs/criterion/latest/criterion/`): it collects detailed statistics "providing strong confidence that changes to performance are real, not measurement noise", supports pluggable measurements (default `WallTime`, plus CPU-time / CPU / GPU counters), and produces charts. Criterion reports mean, median, slope and outlier analysis; our published numbers use **median and p95** (see §5.4), not criterion's default slope-only comparison, and we never compare across machines.

**`hyperfine`** — for **end-to-end CLI timings** (process launch → exit). VERIFIED (`https://raw.githubusercontent.com/sharkdp/hyperfine/master/README.md`): "Statistical analysis across multiple runs", "Warmup runs can be executed before the actual benchmark", "Cache-clearing commands can be set up before each benchmark run", "Statistical outlier detection to detect interference from other programs and caching effects", "Export results to various formats: CSV, JSON, Markdown, AsciiDoc", "Parameterized benchmarks", "Cross-platform". Example:

```sh
hyperfine --warmup 5 --runs 30 --export-json qa/perf/warm-start.json \
  'phairdlna serve --config qa/perf/server.toml --ready-file /tmp/ready'
```
(the server writes the ready file once it can answer requests; hyperfine measures the whole process lifetime).

**`/usr/bin/time` — the `-v` flag is Linux-only; macOS uses `-l`.**
* VERIFIED: GNU `time` documents `-v, --verbose` — "Give very verbose output about all the program knows about", and its report includes `%M Maximum resident set size of the process during its lifetime, in Kbytes` (`https://man7.org/linux/man-pages/man1/time.1.html`). This is the GNU coreutils `time`, what you get on Linux.
* VERIFIED: macOS ships the BSD `time`, whose flag is `-l`: "The contents of the rusage structure are printed as well", with the example output showing "maximum resident set size" (`https://keith.github.io/xcode-man-pages/time.1.html`).
* **Therefore the requirement as written ("use `/usr/bin/time -v`") is wrong for a macOS-first project.** The portable invocation is `time -l` on macOS and `time -v` on Linux; both print maximum RSS. The script must branch on `uname -s`, and the reported figure must state which time(1) produced it.

**`ps` / `top` / `lsof`** — for sampling RSS, thread count, and file-descriptor count of a long-lived process. On macOS: `ps -o pid,rss,vsz,th,%cpu -p $PID`, `top -l 1 -pid $PID -stats pid,cpu,mem,threads`, `lsof -p $PID | wc -l` (fd count). On Linux: `/proc/$PID/status` (`VmRSS`, `Threads`), `/proc/$PID/fd | wc -l`. **ASSUMPTION**: the exact `top -stats` keys differ across macOS versions; verify on the target macOS release.

**`perf`** — VERIFIED: `perf(1)` is documented in the *Linux* manual pages (`https://man7.org/linux/man-pages/man1/perf.1.html`, "top perf - Performance analysis tools for Linux"). There is no `perf` on macOS. On macOS the equivalent is **Instruments**, driven headlessly by `xctrace`:
* VERIFIED (`https://keith.github.io/xcode-man-pages/xctrace.1.html`): `xctrace record [--output path] [--template path|name] [--device name|UDID] [--time-limit time] [--all-processes | --attach pid|name | --launch command [arguments]] [--no-prompt] …` and it "is used to record new Instruments.app .trace files using the specified recording template"; `xctrace export` can convert a trace to XML/other formats.
* `powermetrics` — VERIFIED (`https://keith.github.io/xcode-man-pages/powermetrics.1.html`): `-s samplers, --samplers samplers` (comma-separated sampler list), `-i N` (sample interval), `-o file` (output file). Its own man page warns that some estimates "may be inaccurate - hence they should not be used for any comparison between devices".
* **Permissions:** the fetched `powermetrics` and `xctrace` man pages contain **no** statement about required privileges (VERIFIED by grep: no "root"/"privilege" text). Empirically these tools require elevated privileges on macOS (typically `sudo`) and Instruments additionally needs to be granted "System Instruments" / developer permissions. **ASSUMPTION** — must be verified on the target machine, and the plan must handle the "no sudo in CI" case by using `xctrace` without elevated privileges (Time Profiler attachment to our own process is typically allowed) and by preferring `/usr/bin/time -l` + `ps` for the headline RSS/CPU numbers.
* The macOS man pages for `leaks` and `xctrace` are the same source used for §6.

**Do not present synthetic numbers as product guarantees** — see §5.6.

### 5.3 The measurement protocol (this is the part a reviewer audits)

**Hardware profile** — every performance report embeds this block verbatim:

```
hardware:
  model: MacBook Pro 14" (2023)
  chip: Apple M2 Pro (10-core CPU, 16-core GPU)
  cpu_cores_available: 10
  memory_gb: 16
  storage: APFS, internal SSD, volume "Macintosh HD"
  os: macOS 15.6 (build 24G90)
  power: AC power, Low Power Mode disabled
  thermal_state_at_start: nominal
  network: loopback (127.0.1) unless the row says otherwise
  background: "standardised profile" (no IDE indexers, no backup daemons, screen off)
server:
  commit: <git sha>
  build: cargo build --release --locked
  profile: [profile.release] lto="thin", codegen-units=1, panic="abort" (as committed)
fixtures:
  set: qa/fixtures/v1 (sha256 of the manifest: …)
method:
  warmup: 3 runs discarded, then 30 measured runs
  statistic: median (primary) + p95 (secondary); mean and min recorded but not published as headline
  date_utc: …
```

**Warm-up rule:** discard the first 3 runs of any measurement. Rationale: page cache, first-touch allocation, and lazy initialisation dominate the first runs. For the *streaming* benchmark, warm up by playing the fixture once and then measuring a fresh 60 s window.

**Iterations:** ≥ 30 measured runs for anything we publish (hyperfine's default is 10, which is too few for a p95 claim); criterion's defaults are a 3-second warm-up and 100 samples (VERIFIED, criterion book) — keep them, and additionally require ≥ 30 independent *process-level* repetitions for the headline CLI numbers (hyperfine `--runs 30`).

**Statistic reported:** **median as the headline, p95 as the tail indicator**, and the raw per-run data always archived as JSON (hyperfine `--export-json`, criterion's `target/criterion/**/new/estimates.json`). We never publish a mean without the distribution, and we never publish a p95 without the sample size.

**Repeatability:** results are re-measured on a second day (or at least after a reboot) before being quoted in a release note; if the medians differ by > 10 %, the report says so and both runs are attached.

### 5.4 Streaming-specific methodology

* **Range table** (the correctness surface of the streaming path; each row is also a test case):

  | Request | Expected |
  |---|---|
  | `Range: bytes=0-` | 206, bytes 0..len-1, `Content-Range: bytes 0-<len-1>/<len>` |
  | `Range: bytes=100-199` | 206, exactly 100 bytes, `Content-Range: bytes 100-199/<len>` |
  | `Range: bytes=-100` | 206, last 100 bytes, `Content-Range: bytes <len-100>-<len-1>/<len>` |
  | `Range: bytes=100-` on a 100-byte file | 206, 1 byte (100-99 → invalid ⇒ 416) — decide and document; RFC 9110 says an unsatisfiable range → 416 |
  | `Range: bytes=<len>-` | 416 with `Content-Range: bytes */<len>` |
  | `Range: bytes=0-1,5-6` (multipart) | 206 `multipart/byteranges` — or 200 if we do not support multipart; **must be documented** |
  | `Range: bytes=abc` | ignore the header → 200 (RFC 9110: an unsatisfiable *syntax* is ignored) |
  | `Range: bytes=0-99999999999999999999` | clamp to the end, 206 |

* **Throughput measurement:** client = a purpose-built minimal HTTP client inside `phairdlna-testkit` that reads the body at max speed and reports bytes/s per 1 s window; the server is on loopback for the *server-side* number, and on a 1 GbE link for the *network* number (these are different numbers and must not be conflated).
* **Per-stream CPU:** sample `ps -o %cpu` for the server pid at 1 Hz over the 60 s window, take the median of the per-second values, and report `median_cores = median(%cpu)/100`. Cross-check with the `user+sys` CPU time from `time -l` over the same window divided by wall time.
* **Memory per stream:** RSS at steady state with 0, 1, 2, 4, 8 streams; plot ΔRSS vs N; report the slope as MiB/stream (linear fit through the origin, with R² reported).
* **Multi-stream:** the same fixture, N ∈ {1,2,4,8} streams; report per-stream throughput and p95 latency for each N. The headline is the *point at which per-stream throughput degrades below a stated reference*, not "supports N streams".

### 5.5 Artefact layout

```
qa/perf/
  README.md                 # how to reproduce every number below
  environment/<date>-<host>.json      # hardware profile block
  raw/<date>-<metric>.json            # raw per-run data (hyperfine/criterion JSON)
  summary/<date>.md                   # human-readable summary (median + p95 + n)
  scripts/run-<metric>.sh             # the exact command that produced the raw data
```

Every summary line must link to the raw JSON and the script. A reviewer can re-run the script and compare.

### 5.6 How not to present a synthetic number as a product guarantee

Banned phrasings (and their replacements):

| Banned | Allowed |
|---|---|
| "Supports 8 concurrent streams" | "On a MacBook Pro 14″ M2 Pro (macOS 15.6), serving the fixture set over loopback, 8 concurrent streams sustained 118 MB/s aggregate with a p95 per-stream latency of 42 ms (n=30 runs, raw data: `qa/perf/raw/2026-11-20-multistream.json`)." |
| "Startup is fast" | "Warm startup to first successful `GET /description.xml` was 312 ms median / 388 ms p95 (n=30, `qa/perf/raw/…-warm-start.json`)." |
| "Uses little memory" | "Idle RSS after indexing 12,480 files was 148 MiB (single sample, `qa/perf/raw/…-idle-rss.json`)." |
| "Benchmark: 1.2× faster than the old version" | "Median DIDL serialisation time changed from 41.2 µs to 38.9 µs (n=100 iterations per version, criterion `estimates.json`); this is a micro-benchmark on an M2 Pro and does not predict end-to-end seek latency." |
| "Indexing is 5,000 files/s" | "Indexing 12,480 files (walk + probe) took 2.41 s wall (5,178 files/s) on an NVMe APFS volume, single-threaded probe, macOS 15.6." |

Rules:

1. Every number is accompanied by: hardware, OS, dataset, method, sample size, statistic, date, and a pointer to raw data.
2. Synthetic or emulated workloads are labelled **"synthetic"** in the sentence itself, never only in a footnote.
3. Numbers measured on loopback are labelled **"loopback"** and never presented as network performance.
4. A benchmark result from one machine is presented as a **regression baseline for that machine**, not as a product guarantee. The claim a product guarantee makes ("supports N streams") requires a defined reference hardware profile and is stated as a *conditional* claim with the profile attached.
5. Micro-benchmarks never appear in the same table as end-to-end measurements without a label distinguishing them.

---

## 6. Soak / stability

### 6.1 Running a 7-day soak reproducibly

**Where it runs.** Two supported modes, both reproducible:

* **Local, authoritative** — a script (`qa/soak/run-soak.sh`) plus a config file (`qa/soak/soak.toml`) that pins: git commit, binary path, dataset, port, client script, sample interval, and the soak duration. A soak is bound to a *commit*; if the binary changes mid-soak, the soak is invalid.
* **CI, scheduled but manual** — a GitHub Actions workflow with `workflow_dispatch` only (the project is manual-only). The runner is ephemeral (GitHub-hosted runners are not suitable for a 7-day run because of session limits) → **the CI soak workflow is for short (≤ 60 min) soaks and soak *smoke* checks**; the 7-day soak is a local/macOS-runner job, with the results committed as artefacts.

That distinction must be explicit in the report: "7-day soak executed on hardware X" ≠ "7-day soak executed in CI".

**Workload.** The soak must exercise all the paths that leak: startup, index build, periodic SSDP `NOTIFY` advertisement and `ssdp:byebye` on shutdown, repeated discovery, repeated `Browse` at several depths, repeated `Search`, HTTP `GET` of media with `Range` (including aborted downloads — client disconnects mid-body), and a client that opens and abandons TCP connections (the classic fd leak).

### 6.2 What to record

Sample every 60 s into a CSV (`qa/soak/<date>-<host>.csv`), one row per sample:

| Column | Source (macOS) | Source (Linux) |
|---|---|---|
| `t_s` | seconds since start | same |
| `rss_mib` | `ps -o rss= -p $PID` | `/proc/$PID/status` VmRSS |
| `threads` | `ps -M $PID \| wc -l` | `/proc/$PID/status` Threads |
| `fds_open` | `lsof -p $PID \| wc -l` | `ls /proc/$PID/fd \| wc -l` |
| `sockets_open` | `lsof -p $PID -i \| wc -l` | `ss -tanp \| grep $PID \| wc -l` |
| `index_entries` | server's `/metrics` endpoint | same |
| `index_bytes` | server's `/metrics` endpoint | same |
| `connections_accepted` / `connections_rejected` / `requests_failed` | server counters | same |
| `ssdp_responses_sent` | server counter | same |
| `cpu_pct_median_60s` | `ps -o %cpu=` | same |
| `event_log_tail` | structured log entries since last sample | same |

**Analysis rules (pre-registered, so the conclusion is not chosen after seeing the data):**

* **RSS trend:** linear regression of `rss_mib` against `t_s` over the *last 72 hours* (to exclude warm-up growth). Slope > 0.5 MiB/hour → **FAIL (leak suspected)**; between 0.1 and 0.5 MiB/hour → WARN with a note; ≤ 0.1 MiB/hour → PASS. Report slope, R², and the first/last quartiles' medians.
* **fd trend:** slope > 0.5 fds/hour → FAIL. The absolute fd count must also be compared to a fixed budget (e.g. ≤ 512 for an idle server with 8 clients) — an fd count that is stable but high is still a defect.
* **Thread count:** must return to baseline within 60 s after the load stops (no thread leak). The number of worker threads must be bounded and independent of the number of clients (no thread-per-connection growth).
* **Index growth:** entry count must be constant for a static dataset; a monotone increase is a leak (e.g. a cache that never evicts).
* **Dropped connections:** a non-zero increase in `connections_rejected` or `requests_failed` that correlates with time → FAIL. A small constant number of client-side aborts is expected (the workload deliberately aborts downloads) and must be distinguished from server-side errors by counter name.
* **Restarts:** the server must not crash. Any crash is a FAIL with the panic message and the last 200 log lines attached.

### 6.3 Leak detection: what is actually available

**VERIFIED — `valgrind` is not Linux-only, but it is effectively unavailable on modern macOS.** The Valgrind platform list (`https://valgrind.org/info/platforms.html`) states: "X86/Darwin (10.8 to 10.14), AMD64/Darwin (10.8 to 13.0): supported." — i.e. the macOS port exists but stops at macOS 13.0 and x86-64; there is no arm64/Apple-Silicon Darwin port listed. The project home page (`https://valgrind.org/`) lists the same platform set. So:
* On Linux x86-64: `valgrind --leak-check=full --error-exitcode=1 ./phairdlna ...` is available and is the strongest tool for the leak class.
* On Apple-Silicon macOS: **valgrind is not an option** (no supported port) → use `leaks` (below) or run the same leak check in a Linux CI job.

**VERIFIED — macOS `leaks`.** (`https://keith.github.io/xcode-man-pages/leaks.1.html`): "Search a process's memory for unreferenced malloc buffers"; synopsis `leaks [options] pid | partial-executable-name | memory-graph-file` and `leaks [options] -atExit -- command`; options include `-list`, `-groupByType`, `-nostacks`, `-nosources`, `-quiet`, `-exclude symbol`, `-outputGraph path`. Usage for a server: `leaks --atExit -- ./phairdlna serve --config …` with `MallocStackLogging=1` in the environment to get allocation stacks.

**VERIFIED — `miri`.** (`https://raw.githubusercontent.com/rust-lang/miri/master/README.md`): "Miri is an Undefined Behavior detection tool for Rust. It can run binaries and test suites of cargo projects and detect unsafe code that fails to uphold its safety requirements", detecting out-of-bounds accesses, use-after-free, invalid use of uninitialised data, and violation of intrinsic preconditions; it also warns "Miri does not catch every violation of the Rust specification". Miri requires a nightly toolchain (it interprets MIR rather than executing machine code) — VERIFIED by the tool's own nature and the book's context; **ASSUMPTION** on exact platform support beyond Linux/macOS.

### 6.4 Why the `unsafe` policy matters (justification, not vibes)

The soak/leak strategy above is weak on macOS precisely because the strong tools are missing there. The cheapest, most durable mitigation is a **codebase-level invariant**:

* PhairDLNA's own crates carry `#![forbid(unsafe_code)]` (enforced by a lint gate, QG-03), and `cargo-deny`'s `bans`/`sources` checks constrain what can be pulled in.
* **Consequence 1 — memory-safety bugs in our code are prevented by the compiler, not by a sanitizer.** Out-of-bounds access, use-after-free, use-after-scope and uninitialised reads of Rust-owned data cannot occur in `safe` Rust. ASan (available in the fuzzing builds via `cargo-fuzz`'s nightly sanitizer support) and Miri therefore mostly test *our unsafe blocks and our dependencies*, not our safe code.
* **Consequence 2 — the remaining defect class in safe Rust is logical leaks:** reference cycles (`Rc`/`Arc` cycles), unbounded caches/maps, buffers that grow without eviction, tasks that never finish, and file handles held in long-lived structures. *These* are exactly what the soak measures (RSS trend, fd trend, index growth) — which is why the soak is the primary leak-detection mechanism and not a side activity.
* **Consequence 3 — the leak tooling splits by platform cleanly:** Linux CI runs `valgrind` on a short scenario (fast, deterministic); macOS runs `leaks --atExit` on the same scenario; both run the same soak workload so the results are comparable.
* **Consequence 4 — the policy is auditable:** `#![forbid(unsafe_code)]` is machine-checkable (clippy/rustc lint), so a reviewer can verify the claim without reading the whole codebase. An "we are careful" claim is not auditable.

**OPEN QUESTION:** the dependency tree may contain crates that use `unsafe` internally (most do). `cargo-deny` cannot detect that. If we want a supply-chain-level statement, `cargo-geiger` (not verified here — **UNVERIFIED**, not adopted) or a manual audit of the unsafe-using dependencies would be needed. **UNVERIFIED** is the correct label; the crate was not checked against crates.io in this pass.

---

## 7. Proposed gate matrix

Milestones used by the "blocks" column:

* **M1** — protocol core: SSDP + device description + ContentDirectory skeleton.
* **M2** — media index + full Browse/Search + DIDL-Lite correctness.
* **M3** — streaming: HTTP serving, Range, AVTransport, eventing.
* **M4** — hardening and public release (0.1.0).

Statuses use the vocabulary of §8. "Manual/automatic" describes the *workflow trigger*: in this project every workflow is `workflow_dispatch` (manual), so "Automatic" means "the gate itself decides pass/fail automatically once a human starts the run" — the human must still press the button.

| Gate ID | Gate | Tool / command | Threshold | Platform | Blocks | Manual/Auto | How a reviewer verifies the result |
|---|---|---|---|---|---|---|---|
| QG-01 | Formatting | `cargo fmt --all --check` | Zero diff | macOS, Linux | M1, M2, M3, M4 | Manual trigger / automatic verdict | CI log shows the `cargo fmt --check` exit code 0; reviewer may re-run locally; the diff is empty by construction |
| QG-02 | Clippy (default lints) | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Zero warnings | macOS, Linux | M1, M2, M3, M4 | Manual / automatic | CI log lists 0 warnings; reviewer re-runs the identical command |
| QG-03 | No `unsafe` + forbidden APIs | `#![forbid(unsafe_code)]` per crate, enforced by clippy/rustc lint in QG-02, plus a grep gate for `unsafe ` / `unsafe fn` outside vendored code | Zero occurrences in our crates | macOS, Linux | M1, M2, M3, M4 | Manual / automatic | `cargo clippy` fails on any `unsafe` block (the `forbid` lint); reviewer greps `src/` |
| QG-04 | Build (all targets/features) | `cargo build --workspace --all-targets --all-features --locked` + `cargo build --release --locked` | Compiles with no warnings | macOS (aarch64 + x86_64), Linux | M1, M2, M3, M4 | Manual / automatic | CI artifact lists the produced binaries; reviewer checks `--locked` (no lockfile drift) |
| QG-05 | Unit tests | `cargo nextest run --workspace --all-features` | Zero failures; `NEXTEST_NO_TESTS=fail` so an empty selection is an error | macOS, Linux | M1, M2, M3, M4 | Manual / automatic | nextest JUnit XML artifact shows the executed count; reviewer confirms the executed count > 0 (no "0 tests run") |
| QG-06 | Integration tests | `cargo nextest run --test '*'` (separate nextest invocation, own group in `nextest.toml`) | Zero failures | macOS, Linux | M1, M2, M3, M4 | Manual / automatic | Separate JUnit artifact; integration failures cannot be hidden inside the unit group |
| QG-07 | Doc tests | `cargo test --doc --workspace` | Zero failures; **must run**, since nextest does not support doctests | macOS, Linux | M2, M3, M4 | Manual / automatic | CI shows a non-zero doctest count; a run of 0 doctests is reported as `NOT RUN`, not as a pass |
| QG-08 | Licence check | `cargo deny check licenses` (with the `deny.toml` of §1.3) | Zero unallowed licences; `unused-allowed-license` = warn | macOS, Linux | M4 | Manual / automatic | CI log lists the licence set of every dependency; reviewer confirms AGPL-3.0-only + MPL-2.0 policy intent |
| QG-09 | Advisory check | `cargo audit check` | Zero un-ignored advisories; DB fetch failure ⇒ `NOT RUN / INFRA ERROR` (never "clean") | macOS, Linux | M2, M3, M4 | Manual / automatic | CI log shows the RustSec DB revision it used; reviewer checks the DB was actually fetched |
| QG-10 | Dependency bans & sources | `cargo deny check bans sources` | Zero violations; `wildcards = deny`, unknown registries/git denied | macOS, Linux | M2, M3, M4 | Manual / automatic | CI log lists the banned duplicate versions and any source exception granted |
| QG-11 | Docs link check | `lychee --no-progress --max-retries 2 '**/*.md'` + a content-type/size assertion for `upnp.org` links (soft-404 check) | Zero broken links; soft-404s detected | macOS, Linux | M4 | Manual / automatic | Reviewer clicks the reported URL list; for upnp.org links the check verifies the response is XML, not HTML |
| QG-12 | Spelling | `typos --check` (with `_typos.toml` allow-list) | Zero findings | macOS, Linux | M4 | Manual / automatic | Reviewer inspects `_typos.toml` — every entry is a protocol term, not a suppressed real typo |
| QG-13 | File size / LOC budget | `tokei --output json` + `xtask size-check` (project script; max 500 lines and max 40 KiB per source file, max 200 KiB per test file; `tests/` and `fuzz/` excluded) | No file over budget; new files must not raise the repo LOC baseline | macOS, Linux | M2, M3, M4 | Manual / automatic | The script's output (file, lines, bytes) is attached; reviewer can re-run `cargo xtask size-check`. **No off-the-shelf Rust tool enforces "max file size" — the script is the gate** |
| QG-14 | Function complexity | `rust-code-analysis-cli metrics -l rust src/` (advisory) + clippy `excessive_nesting` / `too_many_lines` (pedantic, promoted to deny) | clippy: zero hits. r-c-a: report only — flagged functions listed in the report, not blocking, until a baseline exists | macOS, Linux | M4 | Manual / automatic | Reviewer sees the metric table; the clippy part is blocking. `clippy::cognitive_complexity` is **not** used — see §7.3 |
| QG-15 | Binary size | `cargo bloat --release -n 20` | Release binary ≤ budget agreed at M2 (recorded in `qa/size-budget.txt`); a >5 % regression blocks | macOS | M4 | Manual / automatic | The bloat table is attached; reviewer checks the total and the biggest contributors |
| QG-16 | Snapshot goldens (DIDL-Lite) | `INSTA_UPDATE=no INSTA_FORCE_PASS=0 cargo insta test --review` (or `cargo nextest run` with the same env) | Zero pending snapshots; `INSTA_FORCE_PASS` must be unset | macOS, Linux | M2, M3, M4 | Manual / automatic | CI lists pending snapshots and **fails**; the reviewer inspects the diff with `cargo insta review` and accepts deliberately |
| QG-17 | XML schema validation | `cargo nextest run -p phairdlna-core --test didl_schema` (uses `xmlschema` + vendored official XSD) | Every DIDL-Lite document valid; the `support::unsupported` audit reports nothing skipped | macOS, Linux | M2, M3, M4 | Manual / automatic | CI attaches the validator report and the SHA-256 manifest of the vendored schemas (`qa/schemas/SOURCES.md`) proving the official bytes were used |
| QG-18 | Property tests | `cargo nextest run --test proptests` with `PROPTEST_CASES` raised in CI | Zero failures; regression files committed | macOS, Linux | M2, M3, M4 | Manual / automatic | The proptest seed/`proptest-regressions/` files are in the diff; reviewer re-runs with the same case count |
| QG-19 | Fuzz smoke | `cargo +nightly-<pinned> fuzz run <target> -- -runs=20000 -max_total_time=60` for each target | Each target builds, loads its committed seed corpus, and completes the budget with no crash | macOS (aarch64/x86_64), Linux | M2, M3, M4 | Manual / automatic | CI attaches the libFuzzer logs with the executions count; the report says "NO CRASH FOUND — not a pass" with the duration, never "fuzzing passed" (§4.3) |
| QG-20 | Miri (unsafe/UB) | `cargo +nightly-<pinned> miri test --workspace` | Zero UB reports | macOS, Linux | M3, M4 | Manual / automatic | Miri output shows the tested crates; with `#![forbid(unsafe_code)]` this is expected to be empty — an empty run is reported as such, and a "0 tests" result is `NOT RUN` |
| QG-21 | Benchmark baseline | `cargo bench --workspace` (criterion) + `hyperfine` for the CLI-level metrics | No statistically significant regression vs the stored baseline (criterion's own comparison; >5 % median regression blocks) | macOS (the reference host) | M4 | Manual / automatic | Criterion's `estimates.json` + comparison output attached; reviewer sees baseline commit id and the delta |
| QG-22 | Soak (7-day) | `qa/soak/run-soak.sh` (local) / short soak workflow | RSS slope ≤ 0.1 MiB/h over the last 72 h; fd slope ≤ 0.5/h; no crashes; index size constant | macOS (authoritative), Linux (valgrind cross-check) | M4 | Manual / automatic | The CSV, the regression output (slope/R²) and the pre-registered analysis are attached; reviewer recomputes the slope from the CSV |
| QG-23 | Cross-client matrix | `cargo xtask matrix check` + the `qa/device-matrix.toml` rows marked `pass` | Every row has evidence + SHA-256; no row claims `pass` without `observed` | macOS + client devices | M3, M4 | Manual / automatic | The reviewer opens the evidence artefact referenced by a row and re-verifies the SHA-256 |
| QG-24 | Semver / public API | `cargo semver-checks check-release --baseline-rev <tag>` (per target triple) | Zero breaking changes | macOS, Linux | M4 | Manual / automatic | The semver-checks report lists each checked item and the baseline revision used |
| QG-25 | Dependency freshness (advisory only) | `cargo outdated --exit-code 0` | **Never blocks.** Reported as a table in the run summary with an explicit "ADVISORY — does not block" label | macOS, Linux | — | Manual / automatic | The reviewer reads it as information; the gate's status in the summary is `INFO`, never `PASS` implying correctness |
| QG-26 | Coverage | `cargo llvm-cov --workspace --all-features --lcov --output-dir qa/coverage` | Reported, not gated, until a baseline exists; a *decrease* in covered lines > 5 % is flagged | macOS, Linux | M4 | Manual / automatic | The LCOV report is attached; the reviewer checks the delta against the stored baseline, not an absolute number |
| QG-27 | Mutation sanity (optional) | `cargo mutants --workspace` (runs under nextest) | Reported; surviving mutants listed as test-suite gaps | macOS, Linux | — | Manual / automatic | The mutant report names which mutants were not caught — these become test tasks |

### 7.1 Gate ordering and cost

Run the cheap deterministic gates first so a human-triggered run fails fast: QG-01 → QG-02 → QG-03 → QG-04 → QG-05/06/07 → QG-08/09/10 → QG-12 → QG-13/14 → QG-16/17/18 → QG-19 → QG-20 → QG-11 → QG-21 → QG-15 → QG-24 → QG-26. QG-22 (soak) and QG-23 (device matrix) run on their own cadence, not in the main workflow.

### 7.2 The gate summary artifact

Every workflow run produces one machine-readable file `qa/results/<run-id>.json`:

```json
{
  "run_id": "2026-11-14T09:31:00Z",
  "commit": "0f1e2d3",
  "host": {"os": "macOS 15.6", "model": "MacBook Pro 14 (M2 Pro)", "arch": "aarch64"},
  "gates": [
    {"gate": "QG-05", "name": "Unit tests", "status": "PASS", "reason": "", "tool_version": "cargo-nextest 0.9.152", "duration_s": 41.2, "executed": 812, "artifacts": ["nextest-junit.xml"]},
    {"gate": "QG-19", "name": "Fuzz smoke", "status": "PASS", "reason": "smoke only: no crash in 60s x 5 targets; NOT a fuzzing verdict", "tool_version": "cargo-fuzz 0.13.2 / nightly-2026-09-15", "duration_s": 312, "executed": 100000, "artifacts": ["fuzz-logs.txt"]},
    {"gate": "QG-22", "name": "Soak", "status": "NOT_RUN", "reason": "7-day soak scheduled separately; last soak 2026-11-07 PASS", "tool_version": "", "duration_s": 0, "artifacts": []}
  ]
}
```

The aggregator rule: **the run summary may print "ALL GATES PASSED" only if every gate's status is exactly `PASS`.** Any `NOT_RUN`, `SKIPPED`, `BLOCKED`, `INCONCLUSIVE` or `FAIL` suppresses that sentence and prints the offending rows instead.

### 7.3 The file-size and complexity checks: what actually exists

This section answers the specific question "does a Rust tool exist for file size / complexity?" honestly.

**Spelling — `typos` 0.10.44 exists and works.** VERIFIED: crates.io API, README (`https://raw.githubusercontent.com/crate-ci/typos/master/README.md`), with a `_typos.toml` configuration file for protocol vocabulary.

**LOC census — `tokei` 15.0.0 exists and works.** VERIFIED: crates.io API, README. It *reports* per-language line counts; it does not enforce limits.

**File size — no dedicated, maintained Rust tool enforces a maximum file size.** VERIFIED by inspection of the candidates: `tokei` has no threshold mode; `typos` is spelling-only; `rust-code-analysis` computes LOC per file but has no "max lines" gate; `clippy` has `too_many_lines` (a *function*-level lint in the `pedantic` group — VERIFIED from the clippy lint list) but no file-level size lint. **Decision:** implement the file-size gate as a **small project script** — `cargo xtask size-check` — reading `tokei --output json` plus a per-file line count, comparing against `qa/size-budget.toml`, and failing with a per-file table. This is ~80 lines of Rust with no new dependency, and it is auditable (the reviewer can read the whole gate). Writing a bespoke script is preferable to adopting an unmaintained tool for a policy we want to control.

**Complexity — `clippy::cognitive_complexity` exists but is explicitly *not* a cognitive-complexity metric, and clippy says so.**
* VERIFIED: the lint exists (`https://rust-lang.github.io/rust-clippy/master/index.html`, `id="cognitive_complexity"`).
* VERIFIED: its group is `restriction`, and its source (`https://raw.githubusercontent.com/rust-lang/rust-clippy/master/clippy_lints/src/cognitive_complexity.rs`) states: *"We used to think it measured how hard a method is to understand. … But that's not what this lint does. … This lint has been left in `restriction` so as to not mislead users into using this lint as a measurement tool."* The lint's own docs point to `excessive_nesting` and `too_many_lines` as "Lints to consider instead of this".
* **Therefore: do not adopt `clippy::cognitive_complexity` as a complexity gate.** Using it would mean shipping a gate whose own documentation disclaims the property it is being used for.
* **What we use instead:**
  1. **Blocking, lint-level proxies** in clippy: `clippy::excessive_nesting` (default-on `complexity` group) and `clippy::too_many_lines` (`pedantic` — promote to deny in our workspace `lints.toml`). These catch the failure modes clippy's own docs recommend.
  2. **Advisory, metric-level:** `rust-code-analysis-cli` 0.0.25 (MPL-2.0) computes `COGNITIVE`, `CC` (cyclomatic), `NOM`, `LOC`, `HALSTEAD` (VERIFIED from the project docs). It is tree-sitter based, runs on Linux/macOS/Windows, and is the tool actually used by Firefox's code-quality tooling. **Caveat:** the published crate is from 2023-01-13 while the repository was last pushed 2026-04-06 — pin the version and treat the numbers as advisory (report, don't block) until we have a baseline and a stable release.
  3. **Structural, our own:** the file-size script from above (QG-13) plus a rule that protocol handlers stay small.

**`cargo-lint` does not exist as a usable tool.** VERIFIED: `cargo-lint` 0.1.0 was published on crates.io on 2020-12-09, has 3,310 lifetime downloads, 179 in 90 days, one version ever, description "A lint check tool use by cargo", and its declared repository `https://github.com/baoyachi/cargo-lint` **returns HTTP 404** (checked 2026-11-14). It is a dead placeholder, not a maintained linting plugin. The real dependency-linting tool in this space is `cargo-deny` (0.20.2, actively maintained: repository last pushed 2026-11-13).

---

## 8. Honest-labels policy

**Goal:** a skipped, unrun, or inconclusive check can never be read as a pass — by a human skimming a CI page, by a tool parsing the summary, or by a downstream consumer of the results file.

### 8.1 The status vocabulary (exactly these strings)

| Status | Meaning | May be printed as a pass? |
|---|---|---|
| `PASS` | The check ran to completion and met its threshold. | Yes |
| `FAIL` | The check ran and did not meet its threshold. | No |
| `NOT RUN — <reason>` | The check was not attempted (tool missing, nightly unavailable, machine unavailable, time budget not allocated). | **No** |
| `SKIPPED (exit 77) — <reason>` | The check was attempted and deliberately skipped by the tool/run (e.g. `cargo test` filtered to nothing, nextest filter matched nothing, a test marked `#[ignore]`). | **No** |
| `BLOCKED — <reason>` | The check cannot run because a dependency check failed (e.g. build is broken, so tests cannot run). | **No** |
| `INCONCLUSIVE — <reason>` | The check ran but its result cannot be interpreted (e.g. the advisory DB fetch failed, the device under test was offline, the measurement had >10 % run-to-run variance). | **No** |
| `INFO — advisory only` | The check ran and produced information that is explicitly not a pass/fail criterion (e.g. `cargo outdated`, coverage %). | **No** |

### 8.2 Mandatory exact wording

* A gate that did not run is recorded as **`NOT RUN — <reason>`**. The reason is mandatory and must name the cause, not a placeholder:
  * `NOT RUN — nightly toolchain 2026-09-15 not installed on the runner (install step skipped)`
  * `NOT RUN — fuzz smoke budget not allocated in this run (scheduled weekly)`
  * `NOT RUN — macOS 15.6 runner unavailable; Windows runner only`
  * `NOT RUN — advisory database fetch failed (network timeout); NOT equivalent to "no advisories"`
* **`SKIP (exit 77) is not a pass`.** Any process exiting with the skip code (77) is reported as `SKIPPED (exit 77) — <reason>`. The rule is encoded in the aggregator, not in prose: the aggregator counts only `PASS` toward "ALL GATES PASSED", and prints `SKIPPED (exit 77)` in the non-pass section with the reason.
* **`INCONCLUSIVE` is never upgraded.** A flaky measurement (run-to-run variance > 10 % on the median) is `INCONCLUSIVE`, not `PASS` and not `FAIL`. Re-running until it passes is not permitted; the rule is "re-run once, and if it is still inconclusive, record it as inconclusive with both runs attached".
* **Fuzzing** uses the two verbatim forms in §4.3. `FUZZ RESULT: NO CRASH FOUND (not a pass)` and `FUZZ RESULT: CRASH FOUND (defect)`. The gate record for QG-19 carries the reason string `"smoke only: no crash in 60s x 5 targets; NOT a fuzzing verdict"` so that a reader of the JSON cannot mistake it for a clean bill of health.
* **Skipped tests inside a suite are visible.** nextest prints `SKIP` for filtered-out tests and prints the count of skipped tests; our summary reproduces that count (`14 tests run: 14 passed, 177 skipped` — the format nextest itself uses). A run in which the *majority* of tests were skipped is reported as `SKIPPED (exit 77)`-equivalent even if the exit code is 0. **ASSUMPTION** — nextest's exact skip-count semantics should be confirmed against the version we pin; VERIFIED that the summary line contains a "skipped" count (`https://nexte.st/docs/running/`).
* **An empty test selection is an error, not a pass.** nextest's `--no-tests` accepts `auto|pass|warn|fail` (VERIFIED, same page); we set `NEXTEST_NO_TESTS=fail` in CI so a filter that matches nothing fails the run.

### 8.3 Banned words and phrases

Banned in any gate report, PR description, release note, or README performance section:
"should pass", "should work", "expected to work", "looks good", "seems fine", "I think it's fine", "probably", "verified manually" (without naming what was verified and how), "tested" (without a device/OS/version and a date), "no issues found" (without stating what was searched and how).

Required instead: the **observed value**, the **expected value**, the **environment**, and the **date**.

### 8.4 The reporting rules

1. **One gate, one status, one reason string.** Reasons are free text but must be specific.
2. **Every artifact is content-addressed.** Artefacts (JUnit XML, criterion JSON, pcaps, DIDL dumps) are uploaded with a SHA-256 recorded in `qa/results/<run-id>.json`.
3. **The summary states what was not run.** Every run summary ends with an explicit "Not run in this run:" section listing the gates with non-`PASS` status. A green build with an unrun gate must show that gate in the summary.
4. **No cherry-picking.** A gate that is flaky is marked `INCONCLUSIVE` with both attempts attached; it is not silently re-run until green, and it is not removed from the matrix.
5. **Cross-client rows are per-device.** A pass on one client model is recorded against that model only; the summary must not aggregate "all clients pass" unless every row in the matrix is `pass`.
6. **Performance numbers are never gates by themselves.** A benchmark regression fails QG-21 only when the baseline is present, the environment matches the baseline's hardware profile, and the delta exceeds the threshold; otherwise the outcome is `INCONCLUSIVE — baseline environment mismatch`.

---

## 9. Open questions

1. **Accepting a pre-1.0 validator in the test path?** `xmlschema` is `=0.0.14`, described by its own README as a release candidate, with a companion `oxml` version that must match. Decide: adopt (pinned, dev-dependency only), or fall back to writing a narrow hand-rolled structural validator for DIDL-Lite plus `insta` goldens.
2. **Nightly Rust in the pipeline.** Fuzzing and Miri need nightly. Approve a *separate* pinned nightly toolchain used only by QG-19/QG-20, and decide which nightly to pin (a date, not "latest").
3. **C++ toolchain requirement on macOS for libFuzzer.** The book says a C++11 compiler is required; confirm Apple's clang satisfies it on the target macOS.
4. **`xctrace`/`powermetrics` privileges.** The fetched man pages do not state permission requirements; confirm whether the CI runner can attach the Time Profiler to our own process without `sudo`, and decide whether privileged measurements are permitted at all in CI.
5. **MSRV and semver baseline.** `cargo-semver-checks` needs a baseline tag; decide the MSRV (which also constrains which crate versions we may depend on, e.g. `xmlschema`'s MSRV).
6. **Dependency licence policy sign-off.** The `deny.toml` allow-list encodes a legal-ish judgement about MPL-2.0 and AGPL; it needs an explicit owner and sign-off before QG-08 blocks a release.
7. **Device matrix budget and ownership.** Which physical clients can we realistically obtain and who runs the tests? Every row we cannot obtain must be listed as planned, never as passing.
8. **Which DLNA profiles do we claim?** The matrix of "supported media profiles" determines the fixture set and the `protocolInfo` values we must emit. This is a product decision with a large testing cost.
9. **Windows support posture.** The project is macOS-first; decide whether Windows is in scope for any gate (this changes the platform column of QG-05/06 and the CI matrix).
10. **`cargo-outdated` / coverage / mutation gates are advisory.** Confirm that "advisory" really means the status is `INFO` and never `PASS` in the results file.
11. **Re-verify crates.io numbers at implementation time.** The counts in §1.2 are a 2026-11-14 snapshot; the *versions* must be re-pinned at implementation time (in particular `cargo-nextest`, `insta`, `proptest`, `xmlschema`).
12. **Unverified tool candidates.** `cargo-geiger` (unsafe-usage audit of the dependency tree) was mentioned but **not** checked against crates.io — **UNVERIFIED**; do not rely on it until it is.

---

## 10. Sources

All URLs below were fetched on **2026-11-14**. crates.io entries were retrieved through the JSON API (`https://crates.io/api/v1/crates/<name>` and `https://crates.io/api/v1/crates/<name>/versions`).

**crates.io (API)**
- https://crates.io/api/v1/crates/nextest
- https://crates.io/api/v1/crates/cargo-nextest
- https://crates.io/api/v1/crates/assert_cmd
- https://crates.io/api/v1/crates/predicates
- https://crates.io/api/v1/crates/tempfile
- https://crates.io/api/v1/crates/mockall
- https://crates.io/api/v1/crates/wiremock
- https://crates.io/api/v1/crates/httpmock
- https://crates.io/api/v1/crates/mockito
- https://crates.io/api/v1/crates/insta
- https://crates.io/api/v1/crates/proptest
- https://crates.io/api/v1/crates/quickcheck
- https://crates.io/api/v1/crates/cargo-fuzz
- https://crates.io/api/v1/crates/cargo-llvm-cov
- https://crates.io/api/v1/crates/criterion
- https://crates.io/api/v1/crates/cargo-deny
- https://crates.io/api/v1/crates/cargo-audit
- https://crates.io/api/v1/crates/cargo-outdated
- https://crates.io/api/v1/crates/cargo-semver-checks
- https://crates.io/api/v1/crates/typos
- https://crates.io/api/v1/crates/tokei
- https://crates.io/api/v1/crates/rust-code-analysis
- https://crates.io/api/v1/crates/rust-code-analysis-cli
- https://crates.io/api/v1/crates/cargo-lint
- https://crates.io/api/v1/crates/cargo-bloat
- https://crates.io/api/v1/crates/lychee
- https://crates.io/api/v1/crates/cargo-mutants
- https://crates.io/api/v1/crates/cargo-vet
- https://crates.io/api/v1/crates/cargo-hack
- https://crates.io/api/v1/crates/cargo-public-api
- https://crates.io/api/v1/crates/libxml
- https://crates.io/api/v1/crates/xmlschema
- https://crates.io/api/v1/crates/xsd
- https://crates.io/api/v1/crates/cargo-make
- https://crates.io/api/v1/crates/cargo-dist
- https://crates.io/api/v1/crates/cargo-msrv

**Tool documentation**
- https://nexte.st/ , https://nexte.st/docs/running/ , https://nexte.st/docs/installation/pre-built-binaries/
- https://rust-fuzz.github.io/book/ , https://rust-fuzz.github.io/book/cargo-fuzz/setup.html , https://rust-fuzz.github.io/book/cargo-fuzz/ci.html , https://rust-fuzz.github.io/book/print.html
- https://insta.rs/docs/ , https://insta.rs/docs/settings/ , https://insta.rs/docs/cli/ , https://insta.rs/docs/quickstart/
- https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html , https://raw.githubusercontent.com/EmbarkStudios/cargo-deny/main/README.md
- https://raw.githubusercontent.com/rustsec/cargo-audit/main/README.md , https://raw.githubusercontent.com/RustSec/advisory-db/main/README.md
- https://raw.githubusercontent.com/obi1kenobi/cargo-semver-checks/main/README.md
- https://raw.githubusercontent.com/kbknapp/cargo-outdated/master/README.md
- https://raw.githubusercontent.com/taiki-e/cargo-llvm-cov/main/README.md
- https://raw.githubusercontent.com/sharkdp/hyperfine/master/README.md
- https://raw.githubusercontent.com/rust-lang/miri/master/README.md
- https://raw.githubusercontent.com/crate-ci/typos/master/README.md
- https://raw.githubusercontent.com/XAMPPRocky/tokei/master/README.md
- https://raw.githubusercontent.com/lycheeverse/lychee/master/README.md
- https://raw.githubusercontent.com/RazrFalcon/cargo-bloat/master/README.md
- https://raw.githubusercontent.com/sebastienrousseau/xmlschema/main/README.md
- https://docs.rs/criterion/latest/criterion/ , https://bheisler.github.io/criterion.rs/book/user_guide/command_line_output.html , https://docs.rs/xsd/latest/xsd/ , https://docs.rs/xmlschema/latest/xmlschema/
- https://docs.rs/assert_cmd/latest/assert_cmd/ , https://docs.rs/predicates/latest/predicates/ , https://docs.rs/tempfile/latest/tempfile/ , https://docs.rs/mockall/latest/mockall/ , https://docs.rs/wiremock/latest/wiremock/ , https://docs.rs/httpmock/latest/httpmock/ , https://docs.rs/mockito/latest/mockito/
- https://proptest-rs.github.io/proptest/

**Clippy / complexity**
- https://rust-lang.github.io/rust-clippy/master/index.html
- https://raw.githubusercontent.com/rust-lang/rust-clippy/master/clippy_lints/src/cognitive_complexity.rs
- https://raw.githubusercontent.com/rust-lang/rust-clippy/master/clippy_lints/src/lib.rs
- https://mozilla.github.io/rust-code-analysis/print.html , https://raw.githubusercontent.com/mozilla/rust-code-analysis/master/README.md
- https://github.com/baoyachi/cargo-lint (HTTP 404 — dead repository)

**Platform / OS tooling**
- https://valgrind.org/ , https://valgrind.org/info/platforms.html
- https://man7.org/linux/man-pages/man1/time.1.html (GNU time, `-v`)
- https://man7.org/linux/man-pages/man1/perf.1.html
- https://keith.github.io/xcode-man-pages/time.1.html (BSD time on macOS, `-l`)
- https://keith.github.io/xcode-man-pages/leaks.1.html
- https://keith.github.io/xcode-man-pages/xctrace.1.html
- https://keith.github.io/xcode-man-pages/powermetrics.1.html

**UPnP / DLNA specifications and schemas**
- http://upnp.org/schemas/av/didl-lite.xsd (18,456 bytes)
- http://upnp.org/schemas/av/didl-lite-v3-20130331.xsd (18,456 bytes)
- http://upnp.org/schemas/av/upnp.xsd (43,622 bytes)
- http://upnp.org/schemas/av/av.xsd (31,960 bytes)
- http://dublincore.org/schemas/xmls/simpledc20021212.xsd (2,917 bytes)
- https://upnp.org/specs/av/UPnP-av-ContentDirectory-v4-Service-20101231.pdf (2,731,236 bytes)
- https://upnp.org/specs/arch/UPnP-arch-DeviceArchitecture-v1.0.pdf (520,380 bytes)
- https://github.com/4thline/cling (didl-lite-1.0.xsd mirror + EOL notice)

**Packet captures**
- https://wiki.wireshark.org/SampleCaptures
- https://wiki.wireshark.org/uploads/__moin_import__/attachments/SampleCaptures/Public_nic (59,184 bytes; SSDP **NOTIFY** announcements only — 132 `NOTIFY * HTTP/1.1` occurrences, 0 `M-SEARCH`)
- https://www.netresec.com/?page=PcapFiles

**Interoperability tooling**
- https://raw.githubusercontent.com/GNOME/gupnp-tools/master/README.md
- https://github.com/GNOME/gupnp , https://github.com/GNOME/gupnp-dlna
- https://github.com/pupnp , https://raw.githubusercontent.com/pupnp/pupnp/master/README.md
- https://github.com/plutinosoft/Platinum
- https://github.com/4thline/cling
- https://www.dlna.org/ (DLNA dissolved; test tools now via SpireSpark International)
- https://github.com/dlna
- https://developers.google.com/cast/docs/android_tv_receiver/debugging , https://github.com/googlecast/CastReceiver
