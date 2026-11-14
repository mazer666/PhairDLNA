# Getting started — for newcomers

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Documentation Reviewer

PhairDLNA is currently a **Rust workspace scaffold, not a working DLNA server**. The
requirements are written; the implementation has not started. This page helps you
build the project, run the gates and find your way around the code.

If you only want to *use* the product, it does not exist yet — see
`../07-project/ROADMAP.md` for the plan and `../07-project/STATUS.md` for the state.

---

## 1. Install a Rust toolchain

You need a stable Rust toolchain. The easiest way is [rustup](https://rustup.rs/):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

The project pins its toolchain in `rust-toolchain.toml`, so `cargo` will download the
right version on the first build.

Check it:

```bash
cargo --version
rustc --version
```

You do **not** need a C compiler for the scaffold; the crates that need one arrive
with the implementation.

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

The last two are separate: `cargo nextest` does not run doctests, so both are needed.

More commands, including the licence, advisory, link and size gates, are in
[BUILD-AND-TEST.md](BUILD-AND-TEST.md).

---

## 4. Run the scaffold

```bash
cargo run -p phairdlna-cli
```

It prints a short status banner and exits. It deliberately does **not** start a
server: the network listener arrives with the implementation of F-01 and later.

---

## 5. Find your way around

The navigation path for a newcomer is:

```
README.md  →  docs/06-development/GETTING-STARTED.md  →  WHERE_TO_CHANGE.md
           →  the feature specification  →  the source and test paths
```

- **[WHERE_TO_CHANGE.md](../../WHERE_TO_CHANGE.md)** — the index of the code: which
  file owns which responsibility.
- **[../03-features/](../03-features/)** — one specification per feature.
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
| The toolchain downloads on the first build | `rust-toolchain.toml` pins a version | Expected; wait for it |
| `cargo fmt` reports a diff | The file is not formatted | `cargo fmt --all` |
| A link-check failure on `upnp.org` | The server answers soft-404s with HTTP 200 | The link checker checks the content type; this is a known behaviour of the upstream server |
| `scripts/check-file-size.sh` fails | A file exceeds the size gate | Split the file, or record an exception in `docs/00-governance/EXCEPTIONS.md` |

---

## 9. Where to ask

- A question about the product → an issue on GitHub.
- A suspected vulnerability → **not** a public issue. See [../../SECURITY.md](../../SECURITY.md).
- A question about the requirements → a discussion on the relevant specification.

These commands are instructions, not evidence that they have been run. Add
platform-specific prerequisites as the implementation evolves.
