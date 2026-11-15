# Build and test — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: QA / Performance Engineer
Gates: `../05-quality/QUALITY-GATES.md` · Strategy: `../05-quality/TEST-STRATEGY.md`

---

## 1. The toolchain

## 1.1 Installing the tools

```bash
cargo install cargo-nextest cargo-deny cargo-audit --locked
```

`lychee`, `typos` and `hyperfine` are separate binaries and are installed with
`cargo install <name> --locked` as well. `cargo-fuzz` additionally needs a nightly
toolchain: `rustup toolchain install nightly`.

**None of these is needed to build the scaffold.** `cargo build`, `cargo test` and
`cargo clippy` need only the Rust toolchain.

| Tool | Why |
| --- | --- |
| A stable Rust toolchain, at least 1.89 (the `rust-version` in `Cargo.toml`) | The `stable` channel is selected by `rust-toolchain.toml` |
| `cargo-nextest` | Fast, parallel test runs — **does not run doctests** |
| `cargo-fuzz` | Fuzzing; needs a pinned nightly toolchain |
| `cargo-deny` | Licence, advisory, ban and source checks |
| `cargo-audit` | RustSec advisories |
| `insta` | Snapshot tests (DIDL-Lite output) |
| `proptest` | Property tests |
| `criterion` | Benchmarks |
| `hyperfine` | End-to-end command timing |
| `lychee` | Link checking (offline mode) |
| `typos` | Spelling |

Two things that **do not exist** and must not be planned around: `cargo-lint` (a dead
crates.io stub) and an official XSD for a UPnP device description (every
`upnp.org/schemas/device/*` path answers with HTML).

---

## 2. The everyday commands

These need nothing but the Rust toolchain:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace                 # unit + integration
cargo test --workspace --doc           # doctests
```

`cargo nextest` is faster and is what the CI workflow uses, but it is **not installed
by default** — install it once with:

```bash
cargo install cargo-nextest --locked
```

Until it is installed, use `cargo test --workspace`; the two run the same tests.

---

## 3. The gates you can run today

| Gate | Command |
| --- | --- |
| QG-01 formatting | `cargo fmt --all -- --check` |
| QG-02 lints | `cargo clippy --workspace --all-targets -- -D warnings` |
| QG-03 no `unsafe` | `scripts/check-no-unsafe.sh` |
| QG-04 build | `cargo build --workspace --all-targets` |
| QG-05 unit tests | `cargo nextest run --workspace` (or `cargo test --workspace`) |
| QG-06 integration tests | `cargo nextest run --workspace --test '*'` (planned — no integration tests exist yet) |
| QG-07 doctests | `cargo test --workspace --doc` |
| QG-08 licences | `cargo deny check licenses` |
| QG-09 advisories | `cargo audit` |
| QG-10 bans and sources | `cargo deny check bans sources` |
| QG-11 link check | `scripts/check-doc-links.sh` |
| QG-12 spelling | `typos` |
| QG-13 file size | `scripts/check-file-size.sh` |
| QG-14 complexity | `cargo clippy --workspace --all-targets -- -D clippy::excessive_nesting -D clippy::too_many_lines` |
| QG-16 snapshots | `INSTA_UPDATE=no cargo insta test --check` *(planned — no snapshots exist yet)* |
| QG-17 DIDL-Lite conformance | *planned* — an `xtask` command against the vendored XSD |
| QG-18 property tests | `cargo nextest run --workspace --test '*' proptest` *(planned — no property tests exist yet)* |
| QG-21 benchmark baseline | `cargo bench --workspace -- --save-baseline` *(planned — no benchmarks exist yet)* |

**State of each gate.** The canonical register is
[`../05-quality/QUALITY-GATES.md`](../05-quality/QUALITY-GATES.md), and every row in
it carries a **State** column: `exists` or `planned`.

The gates that exist and run today:

```bash
scripts/check-file-size.sh      # QG-13
scripts/check-doc-links.sh      # QG-11
scripts/check-no-unsafe.sh      # QG-03
python3 scripts/generate_requirement_views.py --check
```

### 3.1 The gates that arrive with the implementation

| Gate | What it will check | Command |
| --- | --- | --- |
| QG-06 | Integration tests | `cargo nextest run --workspace --test '*'` |
| QG-12 | Spelling | `typos` |
| QG-14 | Complexity and nesting | `cargo clippy --workspace --all-targets -- -D clippy::excessive_nesting -D clippy::too_many_lines` |
| QG-15 | Binary size | an `xtask` command |
| QG-16 | Snapshot review | `INSTA_UPDATE=no cargo insta test --check` |
| QG-17 | DIDL-Lite conformance | an `xtask` command against the vendored XSD |
| QG-18 | Property tests | `cargo nextest run --workspace --test '*' proptest` |
| QG-19 | Fuzz smoke | `cargo fuzz run <target> -- -runs=N` |
| QG-20 | Miri | `cargo +nightly miri test` |
| QG-21 | Benchmark baseline | `cargo bench --workspace -- --save-baseline` |
| QG-22 | Soak test | `scripts/soak.sh` |
| QG-23 | Device matrix | `scripts/device-matrix.sh` |
| QG-24 | Public API compatibility | `cargo semver-checks` |
| QG-25 | Outdated dependencies | `cargo outdated` (informational) |
| QG-26 | Coverage | `cargo llvm-cov` (informational) |
| QG-27 | Mutation testing | `cargo mutants` (informational) |
| QG-28 | Crate boundaries | `scripts/check-crate-boundaries.sh` |

**A gate that does not exist is reported as `NOT RUN — <reason>`, never as a pass.**

---

## 4. Documentation gates

```bash
scripts/check-doc-links.sh          # QG-11: every relative link resolves
scripts/check-file-size.sh          # QG-13: the file-length budget
scripts/check-no-unsafe.sh          # QG-03: no unsafe code
python3 scripts/generate_requirement_views.py --check   # the generated views are current
```

The requirement index and the traceability matrix are **generated**. If you change a
requirement definition, regenerate them:

```bash
python3 scripts/generate_requirement_views.py
```

---

## 5. Fuzzing

```bash
cargo +nightly fuzz run fuzz_soap -- -runs=10000
```

Six targets are **planned**: `fuzz_soap`, `fuzz_device_description`,
`fuzz_http_request`, `fuzz_range`, `fuzz_playlist`, `fuzz_media_header`. Their seed
corpora arrive with them — there is no `fuzz/` directory in the repository yet, so
this command cannot run today.

A fuzzing run is reported as `NO CRASH FOUND (not a pass)` — never as a pass.

---

## 6. Benchmarks and measurements

```bash
cargo bench --workspace
hyperfine --warmup 3 --runs 30 'cargo run -p phairdlna-cli -- --version'
```

Every published measurement needs the hardware profile, the method, the iteration
count and the statistic. The protocol is in `../05-quality/PERFORMANCE-BUDGETS.md`.

---

## 7. Cross-platform builds

```bash
cargo build --workspace --target x86_64-apple-darwin
cargo build --workspace --target aarch64-apple-darwin
cargo build --workspace --target x86_64-unknown-linux-gnu
cargo build --workspace --target x86_64-pc-windows-msvc
```

The macOS universal binary is produced with:

```bash
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin
lipo -create \
  target/aarch64-apple-darwin/release/phairdlna-cli \
  target/x86_64-apple-darwin/release/phairdlna-cli \
  -output target/universal/phairdlna-cli
```

---

## 8. What to do when a gate fails

1. **Read the output.** The gate prints the file, the line and the rule.
2. **Fix the cause**, not the gate. A gate that is edited to pass is a broken gate.
3. If the rule is genuinely wrong, change the rule through an ADR and record the
   exception in `../00-governance/EXCEPTIONS.md` with an approver and a review date.
4. If the gate is wrong (a false positive), fix the gate — and add the case it
   mis-handled as a test.

Never: `#[allow]`, `#[ignore]`, a commented-out test, or a raised threshold without
a record.
