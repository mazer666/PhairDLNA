# Build and test — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: QA / Performance Engineer
Gates: `../05-quality/QUALITY-GATES.md` · Strategy: `../05-quality/TEST-STRATEGY.md`

---

## 1. The toolchain

| Tool | Why |
| --- | --- |
| A stable Rust toolchain, pinned in `rust-toolchain.toml` | Reproducible builds |
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

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo nextest run --workspace          # unit + integration
cargo test --workspace --doc           # doctests (nextest does not cover these)
```

---

## 3. The full gate set

| Gate | Command |
| --- | --- |
| QG-01 formatting | `cargo fmt --all -- --check` |
| QG-02 lints | `cargo clippy --workspace --all-targets -- -D warnings` |
| QG-03 no `unsafe` | `cargo xtask check-unsafe` |
| QG-04 build | `cargo build --workspace --all-targets` |
| QG-05 unit tests | `cargo nextest run --workspace` |
| QG-06 integration tests | `cargo nextest run --workspace --test '*'` |
| QG-07 doctests | `cargo test --workspace --doc` |
| QG-08 licences | `cargo deny check licenses` |
| QG-09 advisories | `cargo audit` |
| QG-10 bans and sources | `cargo deny check bans sources` |
| QG-11 link check | `scripts/check-doc-links.sh` |
| QG-12 spelling | `typos` |
| QG-13 file size | `scripts/check-file-size.sh` |
| QG-14 complexity | `cargo clippy --workspace --all-targets -- -D clippy::excessive_nesting -D clippy::too_many_lines` |
| QG-16 snapshots | `INSTA_UPDATE=no cargo insta test --check` |
| QG-17 DIDL-Lite conformance | `cargo xtask validate-didl` |
| QG-18 property tests | `cargo nextest run --workspace --test '*' proptest` |
| QG-21 benchmark baseline | `cargo bench --workspace -- --save-baseline` |

The scripts marked *planned* in the gate matrix do not exist yet; they arrive with
the implementation. A gate that does not exist is reported as `NOT RUN — <reason>`,
never as a pass.

---

## 4. Documentation gates

```bash
scripts/check-doc-links.sh          # every relative link resolves
scripts/check-file-size.sh          # file and function size gates
python3 scripts/generate_requirement_views.py --check   # the index is current
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

Six targets are planned: `fuzz_soap`, `fuzz_device_description`, `fuzz_http_request`,
`fuzz_range`, `fuzz_playlist`, `fuzz_media_header`. Each has a committed seed corpus.

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

The macOS universal binary is produced with `cargo build --target
aarch64-apple-darwin` plus `cargo build --target x86_64-apple-darwin` and
`lipo -create`.

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
