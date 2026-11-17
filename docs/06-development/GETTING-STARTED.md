# Getting started — for newcomers

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Documentation Reviewer

PhairDLNA is an **actively implemented Rust DLNA MediaServer**, not merely a
workspace scaffold. The headless daemon already handles real network requests
in automated socket-level integration tests. The optional local administration
surface and a macOS menu-bar front end also exist.

**It is not release-ready**: no real TV/device interoperability test, long-running
soak test or complete macOS accessibility/usability acceptance is documented.
See [ROADMAP](../07-project/ROADMAP.md) and [STATUS](../07-project/STATUS.md)
for the evidence and current limitations. This guide is for developers; there
is no independently validated end-user installer yet.

---

## 1. Install a Rust toolchain

You need a stable Rust toolchain. The easiest way is [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

The project **requires Rust 1.89 or newer**. That minimum is recorded as
`rust-version` in `Cargo.toml`, and it is what the build matrix tests.

`rust-toolchain.toml` selects the **stable channel** and its components
(`clippy`, `rustfmt`). It deliberately does not pin an exact release, so that a new
contributor does not have to download a second toolchain. If your installed stable
channel is older than 1.89, `rustup update stable` fixes it.

Check it:

```bash
cargo --version
rustc --version
```

The current workspace uses bundled SQLite through `rusqlite`. Building this
dependency requires a C compiler/toolchain appropriate for the target platform.
This is a **build-time** requirement; the bundled SQLite runtime does not require
users to install a separate SQLite library.

---

## 2. Get the code

```bash
git clone https://github.com/mazer666/PhairDLNA.git
cd PhairDLNA
```

---

## 3. Build and run the gates

These are the commands the project checks itself with. Run them from the repository
root:

```bash
cargo fmt --all -- --check                          # formatting
cargo clippy --workspace --all-targets -- -D warnings   # lints
cargo test --workspace                              # tests
cargo test --workspace --doc                       # documentation tests
```

Doctests are a separate step: `cargo nextest` does not run them. `cargo test --workspace`
runs both, so it is the simpler command; the CI workflow uses `cargo nextest` for
speed and runs the doctests separately.

More commands, including the licence, advisory, link and size gates, are in
[BUILD-AND-TEST.md](BUILD-AND-TEST.md).

---

## 4. Run the current CLI and daemon

PhairDLNA is split into two components:
1. **The headless server (`phairdlna`)**: Runs as a daemon (foreground or `launchd`), serving SSDP, UPnP, and HTTP media streams.
2. **The menu-bar frontend (`PhairDLNA.app`)**: Connects to the local server at `http://127.0.0.1:8210` to display status, stream counts, and quick links to the Web UI.

### Step 1: Configure & Start the Server

```bash
cargo run -p phairdlna-cli -- --help
cargo run -p phairdlna-cli -- setup
cargo run -p phairdlna-cli -- config validate
cargo run -p phairdlna-cli -- serve
```

`setup` asks for the server name, media folders and login-start preference,
then writes validated configuration. `serve` starts the real foreground DLNA
server, including HTTP and SSDP listeners, provided the configuration and
network permissions allow it.

To run the server as a persistent macOS background service without keeping a terminal open:
```bash
cargo run -p phairdlna-cli -- service install
cargo run -p phairdlna-cli -- service start
```

### Step 2: Launch the Menu Bar Companion

When running the prebuilt bundle or compiling from source:
```bash
open target/universal-apple-darwin/release/PhairDLNA.app
```
The menu-bar icon will reflect "Server running" as long as the daemon process is active.

The CLI also supports `diagnostics`, `config explain`, `service stop/status/uninstall`
and catalogue commands. Run `--help` for the exact current command syntax.

---

## 5. Find your way around

The navigation path for a newcomer is:

```
README.md  →  docs/06-development/GETTING-STARTED.md  →  WHERE_TO_CHANGE.md
           →  the feature specification  →  the source and test paths
```

- **[WHERE_TO_CHANGE.md](../../WHERE_TO_CHANGE.md)** — the index of the code: which
  file owns which responsibility.
- **[../01-product/FEATURE-CATALOG.md](../01-product/FEATURE-CATALOG.md)** — the index
  of features: which file owns what, in which milestone.
- **[../03-features/](../03-features/)** — the specifications themselves.
- **[../01-product/REQUIREMENTS-INDEX.md](../01-product/REQUIREMENTS-INDEX.md)** — every
  requirement with its ID, priority, feature and verification method.
- **[../02-architecture/ARCHITECTURE.md](../02-architecture/ARCHITECTURE.md)** — how the
  pieces fit together.

---

## 6. Before you write code

Read these three documents. They are short, and they are binding:

1. [../00-governance/PROJECT-STANDARDS.md](../00-governance/PROJECT-STANDARDS.md) — the
   code rules (file length, function length, complexity, error handling, `unsafe`).
2. [../00-governance/DOCUMENTATION-POLICY.md](../00-governance/DOCUMENTATION-POLICY.md) —
   how documents are structured and named.
3. [../00-governance/REQUIREMENTS-METHOD.md](../00-governance/REQUIREMENTS-METHOD.md) —
   how a requirement is written, numbered and traced.

Then read the specification of the feature you want to work on. It ends with
acceptance criteria and a test strategy.

---

## 7. Write your change

Every change updates, as applicable:

- the feature specification (if the behaviour changes),
- the implementation,
- the tests — including negative and boundary cases,
- `WHERE_TO_CHANGE.md` (if a path is added or renamed),
- the requirement index and the traceability matrix, if a requirement changed.

The requirement index and the traceability matrix are **generated**:

```bash
python3 scripts/generate_requirement_views.py
python3 scripts/generate_requirement_views.py --check   # fails if out of date
```

---

## 8. Troubleshooting

| Symptom | Likely cause | What to do |
| --- | --- | --- |
| `cargo` is not found | rustup is installed but not on `PATH` | Open a new terminal, or `source "$HOME/.cargo/env"` |
| `cargo` refuses to build because of `rust-version` | Your Rust is older than 1.89 | `rustup update stable` |
| `cargo nextest` is not found | The test runner is not installed by default | `cargo install cargo-nextest --locked`, or use `cargo test --workspace` |
| `cargo fmt` reports a diff | The file is not formatted | `cargo fmt --all` |
| The optional `lychee` cross-check reports `upnp.org` | That server answers soft-404s with HTTP 200 and an HTML body | `lychee` is not part of the repository gates today; when it is enabled, it must check the content type, not only the status code |
| `scripts/check-file-size.sh` fails | A file exceeds the size gate | Split the file, or record an exception in `docs/00-governance/EXCEPTIONS.md` |

---

## 9. Where to ask

- A question about the product → an issue on GitHub.
- A suspected vulnerability → **not** a public issue. See [../../SECURITY.md](../../SECURITY.md).
- A question about the requirements → a discussion on the relevant specification.

These commands are instructions, not evidence that they have been run. Add
platform-specific prerequisites as the implementation evolves.
