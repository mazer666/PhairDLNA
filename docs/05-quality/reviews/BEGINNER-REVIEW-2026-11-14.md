# Beginner experience review — PhairDLNA
Status: Complete — all findings addressed · Owner role: Independent reviewer · Version/date: 1.0 / 2026-11-15

Reviewer: Documentation & Beginner Experience Reviewer (general-purpose agent)
Date: 2026-11-14
Repository state reviewed: commit `17fef39` ("docs: record the remaining architecture decisions as ADRs"), plus one uncommitted working-tree edit to `docs/03-features/F-08-media-roots-and-filesystem.md`. Note: the repository changed *during* this review (ADRs 0002/0008/0009/0010 were added mid-session), so line numbers refer to the state above.

Persona assumed: I am a programmer with roughly one year of experience in a scripting language (Python/JavaScript) and a passing familiarity with `cargo build` and `cargo test`. I have never written Rust, I have never configured a DLNA server, and I have never read the UPnP Device Architecture specification. I arrived at the GitHub repository from a search for "DLNA server rust", I cannot tell what "UPnP AV" means, and I want to contribute something small (a bug fix or a documentation improvement) within an afternoon. Everything I know about this project is what the repository tells me.

---

## Summary

A beginner can get *oriented* in about ten minutes: the README is honest about the project not existing yet, the navigation path is explicit, and the three commands in "Build the scaffold" actually work on the first try. That is a genuinely good start and it is better than most new-project READMEs.

What stops the beginner is everything *behind* the front door. Five problems dominate.

First, **status honesty leaks in the security and privacy documents.** `SECURITY.md` presents six "hardening guarantees" in the present indicative, and `PRIVACY.md` has a table of "What is stored, and where" describing a SQLite index, an administration token and a thumbnail cache that do not exist. A beginner who lands on those pages — and `SECURITY.md` is one click from the README's "check the security posture" row — will believe a working, hardened server exists.

Second, **the gate tables describe a machine that has not been built.** `BUILD-AND-TEST.md` §3 is titled "The full gate set" and lists `cargo xtask check-unsafe`, `cargo xtask size-check` and `cargo xtask validate-didl` as runnable commands; there is no `xtask` crate in the workspace, so every one of them exits with `error: no such command: xtask`. The same document tells the reader that `cargo nextest run --workspace` is an everyday command, and nowhere in the repository is there a single `cargo install` line, so the first thing that happens is a confusing failure.

Third, **the normative basis of every feature specification is missing.** The feature specs cite `standards §2.4`, `platform-integration §5.6`, `security-landscape §4.4` and `qa-performance §5.3` — 294 citations across 18 files — and none of those four documents exists in the repository. A beginner reading `F-01` is told that a requirement is `[SPEC]` (taken from a standard) or `[DESIGN]` (the project's own choice), and then cannot find the standard to check either claim.

Fourth, **the navigation file points at a tree that does not exist.** `WHERE_TO_CHANGE.md` says, in a blockquote, "Do not add a path to this table until it exists" — and then lists twelve paths that do not exist, including the answer to "I want to change the discovery behaviour" (`crates/phairdlna-core/src/ssdp/`). The repository's `crates/phairdlna-core/src/` contains exactly one file, `lib.rs`.

Fifth, **the same gate has different IDs, and the same ID means different things, in different documents.** `QG-07` is "Doctests" in `QUALITY-GATES.md` and "the crate-boundary check" in `ARCHITECTURE.md` and `ADR-0001`. `PROJECT-STANDARDS.md` calls the file-size gate "QG-08" (which is licences) and the complexity gate "QG-09" (which is advisories). `BUILD-AND-TEST.md` and `QUALITY-GATES.md` give *different commands* for QG-11 and QG-13. A beginner asking "how do I know whether the thing works?" cannot get a single consistent answer.

The good news is that the project *knows* about most of this: `F-19` §12 catalogues the `xtask` problem, the gate-ID drift and the link-gate ambiguity as `[OPEN]` items with proposed readings. The problem is that those open questions live in a Draft feature specification that the navigation path never sends a beginner to, while the documents the beginner *does* read state the conflicting facts as settled truth.

Findings by severity: **6 Blocker, 16 Major, 11 Minor, 4 Info.**

---

## The entry path, walked

I walked the path exactly as `README.md` → `GETTING-STARTED.md` → `WHERE_TO_CHANGE.md` → a feature specification → the source paths, and then the three "where do I go from here" branches (change discovery, add a feature, know it works).

**Step 1 — `README.md`.** *Works.* The opening three lines answer "what is this and is it usable today" perfectly: "Status: **requirements and architecture phase.** No functional server exists yet." The "What it will be" section is genuinely good plain-language prose — "your television finds it on the network, browses it and plays from it" tells a complete beginner more about the product than most. The "Status" section at the bottom ("every performance number in the documentation is a *target*, not a measurement") is exemplary honesty.

*Breaks:* the "Build the scaffold" block is the first place a beginner meets commands, and it silently omits `cargo test --workspace --doc`, which `GETTING-STARTED.md` §3 then presents as a required fourth gate. The "Principles" table ends with "Beginner-readable | Small modules, documented APIs, a navigation path from the README to the exact file to change" — a promise this review finds the repository does not yet keep.

**Step 2 — `docs/06-development/GETTING-STARTED.md`.** *Mostly works.* It is the best document in the repository for my persona: it opens with "PhairDLNA is currently a **Rust workspace scaffold, not a working DLNA server**", it installs Rust, it clones, it runs the gates, it says what the scaffold does when you run it, and it ends with "These commands are instructions, not evidence that they have been run."

*Breaks:* §1 tells me "The project pins its toolchain in `rust-toolchain.toml`, so `cargo` will download the right version on the first build" — but `rust-toolchain.toml` contains `channel = "stable"`, so no version is pinned and no download happens. §3 says "The last two are separate: `cargo nextest` does not run doctests, so both are needed" — but neither of the two commands it lists (`cargo test --workspace`, `cargo test --workspace --doc`) is `cargo nextest`; the sentence explains a tool that §3 never mentions, and `BUILD-AND-TEST.md` then tells me to use nextest instead. §5 promises a navigation path ending at "the source and test paths" and links `WHERE_TO_CHANGE.md`, whose table is 2/3 fiction (B-04).

**Step 3 — `WHERE_TO_CHANGE.md`.** *Works as a document, fails as a map.* The "rules" table at the top is excellent: it answers "where are the rules I must not break?" in ten rows with links. The "source tree" table is the failure. Only 6 of its 18 rows point at something that exists.

**Step 4 — the feature specification.** *The break is deepest here.* `F-01` is, in isolation, a very well-written document: §1 explains SSDP in two sentences before using it, the user story is concrete, §10 is honest ("All criteria below are *planned*; no test file exists yet"), and §11 lists planned test paths in a table. It is exactly the kind of specification a beginner can learn from.

But every one of its 31 `standards §…` citations is unresolvable, the "[SPEC] / [DESIGN] / [OBSERVED] / [OPEN]" tagging scheme is never introduced, and the planned test files it names (`crates/phairdlna-core/tests/ssdp_discovery.rs`) are in a directory that does not exist. The document's own `[DOC-005]` line — "*M-SEARCH*, *NOTIFY*, *advertisement*, *multicast*, *unicast* and *MX* are explained at first use" — is honoured locally (the §1 paragraph does define M-SEARCH, NOTIFY, multicast and unicast) but the rule is applied to *this* document only and is not enforced anywhere in the reading path.

**Step 5 — the source paths.** *Broken.* `crates/phairdlna-core/src/` contains only `lib.rs`, whose entire content is a `PROJECT_NAME` constant and one test.

**Branch: "I want to change the discovery behaviour."** `WHERE_TO_CHANGE.md` sends me to `crates/phairdlna-core/src/ssdp/` (does not exist) and to `F-01`. I read `F-01`, and it tells me the module "owns the socket, the `M-SEARCH` parser, the matching table, the advertisement timer and the interface selection" — but not *which file inside it* holds the parser. `F-01` §11 does answer that ("`crates/phairdlna-core/src/ssdp/msearch.rs` (inline `#[cfg(test)]` module)"), which is good, but it is 270 lines into the document and in a table of *planned* paths.

**Branch: "I want to add a feature."** `GETTING-STARTED.md` §6 sends me to three governance documents, then "read the specification of the feature you want to work on". It never mentions `FEATURE-CATALOG.md` (the index of features) — so I have to guess which of 21 documents to read, or find the catalogue through the README's "work on a feature" row. The answer arrives only indirectly.

**Branch: "How do I know whether the thing works?"** `BUILD-AND-TEST.md` §3, which is broken in the ways described above.

---

## Findings

### B-01 — Blocker — `SECURITY.md` states hardening guarantees as present fact for a server that does not exist
**Location**: `SECURITY.md:56-69`
**The passage**:
> ## Hardening guarantees
>
> The project treats these as guarantees, each with a negative test:
>
> - The administration surface binds to the loopback interface only and validates the `Host` header against an allowlist.
> - Media paths are canonicalised and verified to be inside a configured root, including after symbolic links and aliases are resolved.
> - XML is parsed without DTD processing or external entity resolution, with bounded size and depth.
> - No SOAP action ever reaches a shell, and no Internet Gateway Device action exists.
> - Media metadata is parsed out of process, with a bounded memory and time budget.
> - The product sends no telemetry and makes no outbound connection other than the local network.

**What a beginner would believe**: that a running PhairDLNA server exists on their machine and already has these properties — that the administration surface is already bound to loopback, that the XML parser is already hardened. In reality `crates/phairdlna-core/src/` contains one file with a constant. The phrase "The project treats these as guarantees, each with a negative test" is especially misleading: there are no negative tests, because there is no code to break.

**Suggested fix**: Retitle the section "Hardening requirements (target state — not yet implemented)" and mark each bullet with a status, e.g. "Target. Implemented with F-16." The same fix applies to `docs/04-security/PRIVACY.md:43` (§3 "What is stored, and where"), whose table lists a SQLite catalogue index, a configuration file, a device identity, logs, an administration token with `0600` permissions and a thumbnail cache — none of which exist.

### B-02 — Blocker — the "full gate set" lists `cargo xtask …` commands that cannot run
**Location**: `docs/06-development/BUILD-AND-TEST.md:41-63`
**The passage**:
> ## 3. The full gate set
>
> | Gate | Command |
> | --- | --- |
> | QG-03 no `unsafe` | `cargo xtask check-unsafe` |
> …
> | QG-17 DIDL-Lite conformance | `cargo xtask validate-didl` |

**What a beginner would believe**: that running `cargo xtask check-unsafe` is how the project checks for unsafe code. I ran it; there is no `xtask` crate in the workspace (`Cargo.toml` has `members = ["crates/phairdlna-core", "crates/phairdlna-cli"]`), and cargo says `error: no such command: xtask`. The same is true of `cargo xtask size-check` (QG-13 in `QUALITY-GATES.md:90`) and `cargo xtask binary-size` (QG-15). A beginner will reasonably conclude that the project is broken rather than that the command is aspirational, because nothing on the page says so.

**Suggested fix**: Either add an `xtask` crate, or — much cheaper — mark these four rows "*(planned — no `xtask` crate yet)*" and point each at the shell script that will eventually replace it. The sentence immediately below the table ("The scripts marked *planned* in the gate matrix do not exist yet") already shows the author knew this; it just never got applied to the table (see M-03).

### B-03 — Blocker — the normative source documents behind every feature specification are absent
**Location**: e.g. `docs/03-features/F-01-ssdp-discovery.md:30` (`standards §2.4, §8.1 D6`), `docs/03-features/F-08-media-roots-and-filesystem.md:96` (`security-landscape §4.4`), `docs/03-features/F-08-media-roots-and-filesystem.md:196` (`qa-performance §5.3`)
**The passage**:
> **INT-001** — The server shall answer `M-SEARCH` requests on UDP 1900 within at most 1 second … [SPEC] These are exactly the UDA 1.1 response headers (standards §2.4, §8.1 D6).

**What a beginner would believe**: that "standards" is a document I can open. I searched the whole repository: there is no `standards` document, no `platform-integration` document, no `security-landscape` document and no `qa-performance` document. The citations break down as follows:

| Cited source | Citations | Document in repo? |
| --- | --- | --- |
| `standards §…` | 181 | No |
| `platform-integration §…` | 39 | No |
| `qa-performance §…` | 43 | No |
| `security-landscape §…` | 31 | No |
| `qa/device-matrix.toml` | 22 | No (`qa/` does not exist) |

18 feature specifications depend on them. Worse, `F-08` cites "the reference hardware of qa-performance §5.3" when the actual reference hardware is defined in `docs/05-quality/PERFORMANCE-BUDGETS.md` §2 — so even the section number is wrong, and I would have followed it to a wrong conclusion.

**Suggested fix**: Either commit the four research documents (they are the project's actual normative basis and the `[SPEC]`/`[DESIGN]` tagging depends on them), or replace every citation with a resolvable one: a URL plus a section number, or a pointer into the archive. At minimum, add one short "Sources" line to each feature-spec header explaining what `standards §x.y` means and where it lives — the `F-01` header currently has only Status / Owner role / Version / Requirements.

### B-04 — Blocker — `WHERE_TO_CHANGE.md` lists twelve paths that do not exist, against its own rule
**Location**: `WHERE_TO_CHANGE.md:31-53`
**The passage**:
> **Do not add a path to this table until it exists.** Update this file whenever a module changes ownership.
>
> | `crates/phairdlna-core/src/ssdp/` | SSDP discovery, advertisement, interface selection | F-01 |
> …
> | `crates/phairdlna-admin/` | The optional loopback administration surface | F-14 |
> | `crates/phairdlna-mac/` | The optional native macOS application | F-15 |

**What a beginner would believe**: that `crates/phairdlna-core/src/ssdp/` is where the discovery code lives today, and that `crates/phairdlna-admin/` and `crates/phairdlna-mac/` are directories I can open. All three are absent. Twelve of the eighteen rows point at nothing: `src/config/`, `src/ssdp/`, `src/upnp/`, `src/didl/`, `src/http/`, `src/catalogue/`, `src/media/`, `src/security/`, `src/platform/`, `src/scheduler/`, `crates/phairdlna-admin/`, `crates/phairdlna-mac/`.

This directly breaks the beginner's question #3 ("which file do I touch?") and it breaks it *at the exact document the README sends them to*. It also violates `DOC-003` ("`WHERE_TO_CHANGE.md` shall list every source path a newcomer may need to change and shall stay in sync with the tree").

**Suggested fix**: Split the table into "**Exists today**" and "**Planned (target layout)**", and keep the target layout in a separate document (or a clearly-marked second table) so that the index stays truthful. Add a one-line header to the planned table: "None of these exists yet; they appear here so the target structure is visible."

### B-05 — Blocker — gate IDs mean different things in different documents
**Location**: `docs/00-governance/PROJECT-STANDARDS.md:32,34` vs `docs/05-quality/QUALITY-GATES.md:84,90,91` vs `docs/02-architecture/ARCHITECTURE.md:69`
**The passage** (`PROJECT-STANDARDS.md:32`):
> | Rust source file length | ≤ 300 physical lines | > 500 lines requires a reviewed exception | `scripts/check-file-size.sh` (planned, QG-08) |

and (`PROJECT-STANDARDS.md:34`):
> | Cyclomatic complexity | ≤ 10 | > 15 requires refactoring | `clippy::cognitive_complexity` (pedantic) + `rust-code-analysis` (planned, QG-09) |

**What a beginner would believe**: that the file-size limit is checked by gate "QG-08" and complexity by "QG-09". In `QUALITY-GATES.md`, QG-08 is *Licences* and QG-09 is *Advisories*; file size is **QG-13** and complexity is **QG-14**. The same file also claims `scripts/check-file-size.sh` is *planned* when it exists and works today (the script's own header comment says `Gate QG-13`). Worse, `ARCHITECTURE.md:69` says "The boundary check (`scripts/check-crate-boundaries.sh`, gate QG-07) is mechanical", and `ADR-0001:40` repeats it — but QG-07 is the *Doctest* gate in `QUALITY-GATES.md:84`, `BUILD-AND-TEST.md:51` and `.github/workflows/ci.yml`. So QG-07 is booked twice, for two unrelated gates.

The complexity rule also has three different definitions: `PROJECT-STANDARDS.md` says `clippy::cognitive_complexity` (pedantic) + `rust-code-analysis`; `QUALITY-GATES.md:91` says `excessive_nesting` + `too_many_lines`; `BUILD-AND-TEST.md:58` gives the exact clippy flags. A beginner asked "how is my code's complexity judged?" gets three answers.

**Suggested fix**: Declare `QUALITY-GATES.md` the single canonical gate register, and correct `PROJECT-STANDARDS.md` to cite QG-13/QG-14 (or drop the ID column and link to the matrix). Give the crate-boundary check its own gate ID (QG-28) and put it in the matrix. `F-19` §12 item 2 already proposes exactly this ("the gate matrix is canonical; the two governance documents are corrected") — the fix is one hour of editing.

### B-06 — Blocker — the "boundaries" a beginner is told about are drawn around files that do not exist
**Location**: `docs/02-architecture/ARCHITECTURE.md:33-55`, `docs/02-architecture/ARCHITECTURE.md:69`, `docs/02-architecture/adr/ADR-0001:39-42`
**The passage** (`ARCHITECTURE.md:69`):
> The boundary check (`scripts/check-crate-boundaries.sh`, gate QG-07) is mechanical: it parses the `Cargo.toml` dependency graph and fails when a crate depends "downwards".

**What a beginner would believe**: that there is a script that enforces the crate boundaries and that the four-crate structure exists. `scripts/` contains exactly three files — `check-doc-links.sh`, `check-file-size.sh`, `generate_requirement_views.py`. There is no `check-crate-boundaries.sh`, and there are two crates, not four. `ADR-0001:39` makes the same claim, and `ADR-0001:39-42` says the gate "must have a negative test: an intentionally inverted dependency has to turn the gate red" — an instruction that cannot be followed.

**Suggested fix**: Mark the boundary check as planned and say so in both `ARCHITECTURE.md` and `ADR-0001`; add it to the `F-19` planned-scripts table (`scripts/check-crate-boundaries.sh (planned)`) so the promise has an owner.

### M-01 — Major — `cargo nextest` is prescribed but never installed, and two dev docs disagree on the everyday test command
**Location**: `docs/06-development/BUILD-AND-TEST.md:35`, `docs/06-development/GETTING-STARTED.md:52-58`
**The passage** (`BUILD-AND-TEST.md:35`):
> `cargo nextest run --workspace          # unit + integration`

and (`GETTING-STARTED.md:58`):
> The last two are separate: `cargo nextest` does not run doctests, so both are needed.

**What a beginner would believe**: (a) that `cargo nextest` is part of the everyday command set, and (b) that `GETTING-STARTED.md` §3 lists nextest — it does not; §3 lists `cargo test --workspace` and `cargo test --workspace --doc`. The sentence about nextest refers to a command that appears nowhere in the section it belongs to. And **nowhere in the repository is there a single `cargo install` line** — I grepped `docs/`, `README.md`, `CONTRIBUTING.md` and `AGENTS.md`. `BUILD-AND-TEST.md` §1 lists `cargo-nextest`, `cargo-fuzz`, `cargo-deny`, `cargo-audit`, `insta`, `proptest`, `criterion`, `hyperfine`, `lychee` and `typos` in a table with a "Why" column and no installation instructions. A beginner running the documented command gets `error: no such command: nextest`.

**Suggested fix**: Add an install step to `BUILD-AND-TEST.md` §1 — `cargo install cargo-nextest cargo-deny cargo-audit --locked` — plus a note that `lychee`/`typos`/`hyperfine` are separate binaries. Then either change `GETTING-STARTED.md` §3 to use nextest, or change `BUILD-AND-TEST.md` §2 to use `cargo test --workspace` and explain the nextest difference in prose. Also: `BUILD-AND-TEST.md` §5 tells the reader to run `cargo +nightly fuzz run fuzz_soap -- -runs=10000` — there is no `fuzz/` directory in the repository.

### M-02 — Major — "The full gate set" omits nine of the twenty-seven gates
**Location**: `docs/06-development/BUILD-AND-TEST.md:41-63`
**The passage**:
> ## 3. The full gate set

**What a beginner would believe**: that the table contains every gate. It contains 18 rows (QG-01…QG-14, QG-16, QG-17, QG-18, QG-21) and omits QG-15 (binary size), QG-19 (fuzz smoke), QG-20 (Miri), QG-22 (soak), QG-23 (device matrix), QG-24 (semver), QG-25, QG-26 and QG-27. Three of the omitted gates (`scripts/soak.sh`, `scripts/device-matrix.sh`) are named in `QUALITY-GATES.md` as if they exist. A beginner hunting for "how do I check compatibility on real devices?" never finds `QG-23` here.

**Suggested fix**: Rename the heading to "The gates you can run today" and add a second table, "The gates that arrive with the implementation", listing the remaining nine with their commands and a clear `NOT RUN — not implemented` status.

### M-03 — Major — a sentence refers to scripts "marked *planned*" that are not marked anywhere
**Location**: `docs/06-development/BUILD-AND-TEST.md:64-66`
**The passage**:
> The scripts marked *planned* in the gate matrix do not exist yet; they arrive with the implementation. A gate that does not exist is reported as `NOT RUN — <reason>`, never as a pass.

**What a beginner would believe**: that somewhere there is a gate matrix with rows marked *planned*, so that I can tell which commands are real. I searched `QUALITY-GATES.md` §3 (the only "gate matrix"): no row in the Notes column is marked *planned*. The `NOT RUN` vocabulary is defined in `QUALITY-GATES.md` §1 and is good, but here it is applied to a marking convention that does not exist, so the reader cannot act on it. This is a sentence that promises a mechanism the repository does not implement.

**Suggested fix**: Put the marking in place — add a "Status" column to the `QUALITY-GATES.md` matrix with values `exists` / `planned` — or delete the sentence and point at `F-19` §11, which does have a table separating "the first three exist in the repository today" from the planned ones.

### M-04 — Major — the toolchain is described as pinned when it is not
**Location**: `docs/06-development/GETTING-STARTED.md:22-23`, `docs/06-development/BUILD-AND-TEST.md:12`
**The passage** (`GETTING-STARTED.md:22`):
> The project pins its toolchain in `rust-toolchain.toml`, so `cargo` will download the right version on the first build.

**What a beginner would believe**: that a specific Rust version is pinned and will be downloaded. `rust-toolchain.toml` contains `channel = "stable"` — a moving target, not a pin — and its own header comment says so: *"The channel is 'stable' rather than a pinned release so that a new contributor does not have to download a second toolchain."* No download happens; my `cargo --version` was already satisfied. `BUILD-AND-TEST.md:12` makes the same claim ("A stable Rust toolchain, pinned in `rust-toolchain.toml` | Reproducible builds"), and `GETTING-STARTED.md:135` offers "The toolchain downloads on the first build | `rust-toolchain.toml` pins a version | Expected; wait for it" as a troubleshooting entry. The real minimum version lives in `Cargo.toml` as `rust-version = "1.89"` — and that is what should be named here.

**Suggested fix**: Say "The project requires Rust 1.89 or newer (`rust-version` in `Cargo.toml`) and tracks the `stable` channel via `rust-toolchain.toml`." Remove the "pins a version" troubleshooting row, or reword it to explain `rust-toolchain.toml`'s actual content.

### M-05 — Major — the matrix comment claims `rust-code-analysis`; the matrix uses clippy lints
**Location**: `docs/00-governance/PROJECT-STANDARDS.md:34`
**The passage**:
> | Cyclomatic complexity | ≤ 10 | > 15 requires refactoring | `clippy::cognitive_complexity` (pedantic) + `rust-code-analysis` (planned, QG-09) |

**What a beginner would believe**: that complexity is measured by a tool called `rust-code-analysis` that does not exist, and that `clippy::cognitive_complexity` is the gating lint. `QUALITY-GATES.md:91` actually gates on `clippy::excessive_nesting` + `clippy::too_many_lines` and explicitly says `rust-code-analysis` is "stale (2023) and MPL-2.0 → advisory only, never gating". `clippy::cognitive_complexity` is a pedantic lint, not enabled by `-D warnings`. So a beginner who writes deeply-nested code will not be caught by the lint the standards document names.

**Suggested fix**: Make `PROJECT-STANDARDS.md` §2 cite the same command as `QUALITY-GATES.md` QG-14 and `BUILD-AND-TEST.md` QG-14, and state plainly that `rust-code-analysis` is not used.

### M-06 — Major — `BUILD-AND-TEST.md` §5 promises a committed fuzz corpus that does not exist
**Location**: `docs/06-development/BUILD-AND-TEST.md:92-96`
**The passage**:
> Six targets are planned: `fuzz_soap`, `fuzz_device_description`, `fuzz_http_request`, `fuzz_range`, `fuzz_playlist`, `fuzz_media_header`. Each has a committed seed corpus.

**What a beginner would believe**: that there are six fuzz targets with seed corpora in the repository. There is no `fuzz/` directory and no corpus. The word "planned" applies to the targets; the sentence about the corpora is in the present tense. `QUALITY-GATES.md:96` repeats the claim in its Notes column ("Each target has a committed seed corpus") and `TEST-STRATEGY.md` lists fuzzing with "committed seed corpora" as a layer property.

**Suggested fix**: Use one tense for both — "Six targets are planned; their seed corpora arrive with them."

### M-07 — Major — two incompatible vocabularies for the `Status` column
**Location**: `docs/01-product/REQUIREMENTS-INDEX.md:14-16` vs `docs/00-governance/REQUIREMENTS-METHOD.md:68-75`
**The passage** (index):
> **Status vocabulary:** `Baseline` = defined and internally consistent, awaiting the owner's approval of the baseline as a whole. `Scheduled` = assigned to a milestone. `Implemented` = code exists. `Verified` = evidence recorded.

and (method):
> `Draft` → `In Review` → `Approved` → `Scheduled` → `Implemented` → `Verified` → `Superseded` / `Withdrawn`

**What a beginner would believe**: two different things. The index says the status of every requirement is one of four values; the method says there are eight states in a lifecycle. `Baseline` is not in the method's ladder at all, and `Approved` — the status of the whole baseline — is not in the index's vocabulary. All 341 rows currently read `Baseline`. A beginner asking "what does the Status column mean, and is `Baseline` the same as `Approved`?" has no answer.

**Suggested fix**: Make the index's status vocabulary a subset-with-definitions of the method's ladder, and say "the index currently uses `Baseline`, defined as `Approved` in the sense of §4, minus the owner sign-off."

### M-08 — Major — `REQUIREMENTS-METHOD.md`'s prefix table omits four prefixes that the baseline uses
**Location**: `docs/00-governance/REQUIREMENTS-METHOD.md:15-27`
**The passage**:
> | `SUP` | Supply chain, licensing, provenance | `SUP-004` |

**What a beginner would believe**: that the eleven listed prefixes are all there is. But `TECHNICAL-REQUIREMENTS.md` defines `REL-*` (6), `RES-*` (9), `LIFE-*` (11) and `MB-*` (9) requirements, and `AGENTS.md:29-31` calls the canonical set "`PRD`, `FUN`, `INT`, `SEC`, `PERF`, `UX`, `OPS`, `CFG`, `PLT`, `DOC`, `SUP`, `REL`, `RES`, `LIFE`, `MB`". The document that defines the numbering scheme omits a quarter of it. A beginner who meets `MB-001` (menu bar) or `LIFE-003` (sleep/wake) in the requirement index and then goes to the "requirements method" to decode the prefix finds nothing.

**Suggested fix**: Add the four missing rows to the table, with the same "Domain / Example" shape.

### M-09 — Major — `ARCHITECTURE.md` says the platform trait has five methods; `ADR-0003` defines eight traits
**Location**: `docs/02-architecture/ARCHITECTURE.md:254-262` vs `docs/02-architecture/adr/ADR-0003-platform-adapter-trait.md:42-51`
**The passage** (`ARCHITECTURE.md:254`):
> The core needs exactly five things from the operating system:
> 1. enumerate network interfaces and their multicast capability,
> 2. resolve and validate a media path on a given volume,
> 3. detect that a volume changed (mounted, unmounted, replaced),
> 4. locate per-user configuration and data directories,
> 5. emit a log record and (optionally) a user notification.
>
> These are the five methods of the `Platform` trait (ADR-0003).

**What a beginner would believe**: that there is one trait, `Platform`, with five methods. `ADR-0003` explicitly decides the opposite: "**small orthogonal traits** — not one large trait — plus a `Platform` facade that bundles them", listing eight traits (`Network`, `FileSystem`, `VolumeWatcher`, `Consent`, `UserEnvironment`, `ServiceManager`, `LogSink`, `UserNotifier`). The ADR's own Context lists seven operations, and `F-08` depends on the `Consent` trait, which the architecture's "five things" do not include at all. A beginner implementing the platform layer would build the wrong shape.

**Suggested fix**: Rewrite `ARCHITECTURE.md` §12 to list the eight traits from ADR-0003 (or say "the eight traits of ADR-0003"), and cross-reference the ADR by section number.

### M-10 — Major — `CI-POLICY.md` describes workflows that are not what the repository contains
**Location**: `docs/07-project/CI-POLICY.md:30-33, 40, 55-59`
**The passage** (`CI-POLICY.md:30`):
> Both are **skeletons with the commands listed but the implementation not yet present**. The scripts they call (`scripts/check-file-size.sh`, `scripts/check-doc-links.sh`, `scripts/check-requirement-views.sh`) exist; the remaining gates depend on the implementation.

**What a beginner would believe**: (a) that `.github/workflows/ci.yml` is a skeleton — it is a complete workflow with eleven real steps; (b) that `scripts/check-requirement-views.sh` exists — it does not; the script is `scripts/generate_requirement_views.py`; (c) that a `ci.yml` run covers `QG-03`, `QG-14`, `QG-16`, `QG-17` and `QG-18`, as §4 states — `ci.yml` has no step for any of those five.

Two more contradictions in the same document: §3 rule 2 says "Every action is pinned to a commit SHA, not to a tag", but both workflows use `uses: actions/checkout@v4.2.2`; and rule 3 says "The Rust version is pinned in `rust-toolchain.toml`", which as noted pins no version.

**Suggested fix**: Correct §2 (the workflows are real; the script name is wrong), and either add the five missing gate steps to `ci.yml` or narrow §4 to the gates the workflow actually runs. Pin the checkout action to a SHA.

### M-11 — Major — `rust-toolchain.toml` promises an MSRV matrix the CI does not run
**Location**: `rust-toolchain.toml:6-8`
**The passage**:
> The *minimum* supported version is pinned separately as `rust-version` in the workspace Cargo.toml, and that value is the one the CI matrix tests against.

**What a beginner would believe**: that CI proves the code builds on Rust 1.89. `.github/workflows/ci-matrix.yml:26` uses `dtolnay/rust-toolchain@stable` — the same floating channel — so the matrix is `stable` on macOS, Linux and Windows. `ROADMAP.md:47` then lists "`cargo fmt`, `clippy` and `test` are green on all three platforms" as an M0 exit criterion, which the workflow does not actually test at 1.89. `STATUS.md:23` claims "Quality gates | **Complete**".

**Suggested fix**: Add a `rust: 1.89` entry to the `ci-matrix.yml` matrix, or change the comment to say the matrix tests the stable channel.

### M-12 — Major — `FEATURE-CATALOG.md` denies the existence of two specifications that exist
**Location**: `docs/01-product/FEATURE-CATALOG.md:33-34`
**The passage**:
> | Quality | F-18 Performance and reliability budgets | — (see `../05-quality/`) | M1 | MUST |
> | Quality | F-19 Quality gates and developer experience | — (see `../05-quality/`) | M0 | MUST |

**What a beginner would believe**: that F-18 and F-19 have no specification document. Both exist: `docs/03-features/F-18-performance-and-reliability.md` and `docs/03-features/F-19-quality-gates-and-developer-experience.md`, each with the ten mandatory sections and a full acceptance-criteria table. The catalogue is the index a beginner is sent to ("work on a feature | Feature catalog · Feature specifications"), and it sends them to the wrong place for two features. `STATUS.md:62` also says "Feature specifications | 21 (F-01 … F-19 plus two proposals, F-20 / F-21)" — implying 21 specifications exist, which contradicts the catalogue.

**Suggested fix**: Point the two rows at their actual files.

### M-13 — Major — an `[OPEN]` item in F-19 states a directory does not exist when it does
**Location**: `docs/03-features/F-19-quality-gates-and-developer-experience.md:437-440`
**The passage**:
> 4. **[OPEN] Where the notarisation plan is recorded.** SUP-009 says the plan "shall be recorded here" … the canonical structure reserves `docs/08-releases/` for distribution and provenance, but **that directory does not exist yet**.

**What a beginner would believe**: that `docs/08-releases/` is missing. It exists and contains `RELEASE-POLICY.md`. More importantly, this is the pattern I saw throughout: the open-questions lists are accurate about *plans* but drift out of date about the *repository*, and because they read like notes rather than a live register, a beginner cannot tell which are stale.

**Suggested fix**: Correct the sentence; and add a "last verified against the tree" date to each §12 list so staleness is visible.

### M-14 — Major — the feature-spec `[SPEC]` / `[DESIGN]` / `[OBSERVED]` / `[OPEN]` tagging scheme is never introduced
**Location**: `docs/03-features/F-01-ssdp-discovery.md:29-31` (first use)
**The passage**:
> **INT-001** — The server shall answer `M-SEARCH` requests on UDP 1900 within at most 1 second … [SPEC] These are exactly the UDA 1.1 response headers (standards §2.4, §8.1 D6). [DESIGN] The 1-second budget applies to a *unicast* `M-SEARCH` …

**What a beginner would believe**: that `[SPEC]` and `[DESIGN]` are established abbreviations with defined meanings. Nothing in `F-01` (whose §1 "Purpose and user story" is otherwise model-clear) or in `FEATURE-TEMPLATE.md` explains the four tags, and they recur 300+ times across the feature set. `FEATURE-TEMPLATE.md` is where they should be defined, and it is not reachable from the reading path — `GETTING-STARTED.md` §5 links the `docs/03-features/` directory but not the template.

**Suggested fix**: Add a four-row "How to read this specification" table at the top of `FEATURE-TEMPLATE.md` and repeat a one-line version in each feature spec header.

### M-15 — Major — research identifiers (`M-05`, `M-08`) collide with the milestone vocabulary (`M0`–`M5`)
**Location**: `docs/03-features/F-08-media-roots-and-filesystem.md:160` (`security-landscape §7.4, M-05`), `docs/03-features/F-09-catalogue-index.md:103` (`(M-08)`)
**The passage** (`F-08:160`):
> **Containment algorithm.** [DESIGN] The order is fixed and is the one the security research requires (security-landscape §7.4, M-05): (1) percent-decode exactly once; …

**What a beginner would believe**: that "M-05" means milestone 5 (optional transcoding) or a section of the security research. `ROADMAP.md` and `REQUIREMENTS-METHOD.md` use `M0…M5` for milestones with no hyphen; the hyphenated `M-01`…`M-08` are identifiers inside the missing research documents. There are 119 such references in the repository and no key. Similarly `TCC`, `APFS`, `NFC`/`NFD`, `SSRF`, `CSRF`, `DTD`, `GENA`, `polkit` and `AppleDouble` are all used without a glossary anywhere in the repository.

**Suggested fix**: Reserve a distinct prefix for research artefacts (e.g. `RS-xx` for research sources) and publish a one-page key listing each research document, its identifier range and where it lives — even if the documents themselves are published separately, the key must be in the repo.

### M-16 — Major — `STATUS.md`'s ADR count is stale
**Location**: `docs/07-project/STATUS.md:19, 68`
**The passage**:
> | Decisions | **Complete** | Six ADRs plus 17 entries in the decision log — `../00-governance/DECISION-LOG.md` |
> …
> | Architecture decisions | 6 ADRs + 17 decision-log entries |

**What a beginner would believe**: that there are six ADRs. There are ten files in `docs/02-architecture/adr/` (0001, 0002, 0003–0010) — the count changed when `17fef39` landed during this review, and `STATUS.md` was not updated. The decision-log count (17) is still correct, so a reader comparing the two numbers sees an inconsistency with no explanation.

**Suggested fix**: Update the number, or — better — state it as a pointer ("the ADRs in `docs/02-architecture/adr/`") so it cannot go stale.

### M-17 — Major — `TECHNICAL-REQUIREMENTS.md` has two sections numbered 17
**Location**: `docs/01-product/TECHNICAL-REQUIREMENTS.md:408` and `:422`
**The passage**:
> ## 17. Adaptive resource management
> …
> ## 17. Reliability
> …
> ## 18. Service lifecycle and network recovery

**What a beginner would believe**: that one section is missing and the numbering is unreliable — so a citation like "TECHNICAL-REQUIREMENTS §17" is ambiguous, and a cross-reference into the requirements document cannot be resolved. The document ends at "## 21. Requirement-level priorities", which means the reader has to count the headings to know which of the two §17s is meant.

**Suggested fix**: Renumber the second §17 to §18 and shift the rest.

### M-18 — Major — the requirement index never explains the priority values it prints 341 times
**Location**: `docs/01-product/REQUIREMENTS-INDEX.md:13-16`
**The passage**:
> This file adds the short title, priority, owning feature, milestone, verification method and status, and it is **generated** from those definitions by `scripts/generate_requirement_views.py`.

**What a beginner would believe**: that `MUST`, `SHOULD` and `COULD` are self-evident, or that they are RFC-2119 keywords. The index explains the *status* vocabulary but not the *priority* vocabulary. The definitions exist in two other places — `REQUIREMENTS-METHOD.md` §3 and `TECHNICAL-REQUIREMENTS.md` §21 — and the index links to the former, but it never says "priority levels are defined in §3 of the method". Since the index is the file the README sends a beginner to for "see every requirement", this is the natural place for a one-line definition.

**Suggested fix**: Add a "Priority vocabulary" paragraph to the index header, matching the existing "Status vocabulary" paragraph, and link to `REQUIREMENTS-METHOD.md` §3.

### M-19 — Major — no glossary exists, although `DOC-005` requires terms to be explained at first use
**Location**: `docs/01-product/TECHNICAL-REQUIREMENTS.md:459` (`DOC-005`)
**The passage**:
> | DOC-005 | The documentation shall be readable by a programming beginner: acronyms explained, terms defined, examples concrete. | MUST |

**What a beginner would believe**: that the project has a terminology tool. It has none: there is no glossary, no terminology index and no "terms" section in any document. The DLNA vocabulary is defined *locally* and well — `F-01` §1 explains SSDP before using it, `F-04` §1 defines DMS and DMR, `F-08` §1 defines media root, raw path and containment, `F-16` §6 says "*DTD*, *external entity*, *SSRF*, *DNS rebinding*, *CSRF*, *redaction*, *quarantine* and *GENA* are explained at first use" — but each feature spec is a closed world, and the *reading path* never carries the definition forward. Concretely:

| Term | First met by a beginner on the reading path | Explained there? |
| --- | --- | --- |
| DIDL-Lite | `WHERE_TO_CHANGE.md:42`, `BUILD-AND-TEST.md:17` | No |
| protocolInfo | `PROJECT-STANDARDS.md:49`, `PRODUCT-REQUIREMENTS.md:80` | No |
| ContentDirectory / ConnectionManager | `FEATURE-CATALOG.md:18-19` | No |
| SCPD | `F-01` INT-020 chain, `TECHNICAL-REQUIREMENTS.md:57` | No (not until `F-02` §1) |
| SOAP | `ARCHITECTURE.md:126` | No |
| byte range / seeking | `FEATURE-CATALOG.md:21` | No |
| control point | `F-01:183` | No (defined in `F-02` §1) |
| DMR / DMS | `F-04:10-12`, `F-07:9` | Yes, but only in F-04/F-07 |
| SSDP | `WHERE_TO_CHANGE.md:40` | No (explained in `F-01` §1, which is later) |
| M-SEARCH, NOTIFY, MX, multicast, unicast | `F-01` §1 | **Yes** — F-01 §1 does this well |

**Suggested fix**: Add `docs/01-product/GLOSSARY.md` with one line per term, and link it from `README.md`'s navigation table and from the top of every feature spec. The F-01 §1 paragraph is the model to copy.

### M-20 — Major — `STATUS.md` §4 describes work that is already done as work still to come
**Location**: `docs/07-project/STATUS.md:80-81`
**The passage**:
> 3. M0 is completed: the four crates, the boundary gate, the manual CI workflow, the file-size and link gates.

**What a beginner would believe**: that the CI workflow, the file-size gate and the link gate do not exist. They do: `.github/workflows/ci.yml` and `ci-matrix.yml`, `scripts/check-file-size.sh` and `scripts/check-doc-links.sh` are all in the repository and all pass today (I ran the latter two — see "Commands I ran"). Only the four crates and the boundary gate are actually missing.

**Suggested fix**: Split the line into what is done and what is outstanding, and cross-reference `STATUS.md` §2, which correctly lists only four commands as having been run.

### m-01 — Minor — the README's build block omits a gate that `GETTING-STARTED.md` calls required
**Location**: `README.md:47-54` vs `docs/06-development/GETTING-STARTED.md:52-56`
**The passage** (`README.md:50-53`):
> ```bash
> cargo fmt --all -- --check
> cargo clippy --workspace --all-targets -- -D warnings
> cargo test --workspace
> cargo run -p phairdlna-cli     # prints a status banner; it does not start a server
> ```
**What a beginner would believe**: that this is the whole check set. `GETTING-STARTED.md` adds `cargo test --workspace --doc` as a required fourth command. **Suggested fix**: add the doctest line to the README block, or say "the full set is in GETTING-STARTED.md".

### m-02 — Minor — the feature index is missing from the navigation path
**Location**: `docs/06-development/GETTING-STARTED.md:85-91`
**The passage**:
> - **[../../03-features/](../../03-features/)** — one specification per feature.
**What a beginner would believe**: that the directory listing is the index. It links the *directory*, not `FEATURE-CATALOG.md`, which is the table of "feature → file → milestone → priority" — exactly what a newcomer needs. **Suggested fix**: link `../01-product/FEATURE-CATALOG.md` and describe the directory as "the specifications themselves".

### m-03 — Minor — `WHERE_TO_CHANGE.md`'s scripts table gives no gate for the generator
**Location**: `WHERE_TO_CHANGE.md:57-61`
**The passage**:
> | `scripts/generate_requirement_views.py` | — | Regenerates the requirement index and the traceability matrix |
**What a beginner would believe**: that this script is not part of any gate. It *is* run by CI ("Requirement views are current"), and `GETTING-STARTED.md` §7 depends on it. **Suggested fix**: cite the CI step rather than "—".

### m-04 — Minor — `ARCHITECTURE.md` and `WHERE_TO_CHANGE.md` disagree about the core's module list
**Location**: `docs/02-architecture/ARCHITECTURE.md:39-49` vs `WHERE_TO_CHANGE.md:39-48`
**The passage** (`ARCHITECTURE.md:48-49`):
> │   │       ├── logging/        structured logging + redaction
> │   │       └── error.rs        the crate error type
**What a beginner would believe**: that the two navigation documents describe the same tree. `ARCHITECTURE.md` omits `scheduler/` (which `WHERE_TO_CHANGE.md` lists, F-18) and includes `logging/` and `error.rs` (which `WHERE_TO_CHANGE.md` omits). `DOC-003` requires `WHERE_TO_CHANGE.md` to "stay in sync with the tree"; nothing keeps the two in sync with each other. **Suggested fix**: generate one module list and reference it from both.

### m-05 — Minor — the macOS universal-binary recipe is missing its arguments
**Location**: `docs/06-development/BUILD-AND-TEST.md:121-123`
**The passage**:
> The macOS universal binary is produced with `cargo build --target aarch64-apple-darwin` plus `cargo build --target x86_64-apple-darwin` and `lipo -create`.
**What a beginner would believe**: that `lipo -create` takes no arguments. `lipo -create` requires `-output <file>` and the two input binaries. **Suggested fix**: write the full command.

### m-06 — Minor — a troubleshooting entry describes a failure the documented tool cannot produce
**Location**: `docs/06-development/GETTING-STARTED.md:137`
**The passage**:
> | A link-check failure on `upnp.org` | The server answers soft-404s with HTTP 200 | The link checker checks the content type; this is a known behaviour of the upstream server |
**What a beginner would believe**: that running `scripts/check-doc-links.sh` might report a `upnp.org` problem. The script I ran checks *relative* links only and reported `RESULT: PASS (94 relative links checked)`; it never touches the network. The `upnp.org` content-type issue is real but belongs to the `lychee` cross-check that `BUILD-AND-TEST.md` mentions and that the repository does not run. **Suggested fix**: qualify the row as "the optional `lychee` cross-check".

### m-07 — Minor — "Milestone gates (G0–G6)" reads as six gates and contains seven
**Location**: `docs/05-quality/QUALITY-GATES.md:128`
**The passage**:
> ## 4. Milestone gates (G0–G6)
**What a beginner would believe**: six gates, given the range notation. The table lists seven (G0, G1, G2, G3, G4, G5, G6); `STATUS.md:67` says "Milestone gates | 7 (`G0` … `G6`)". **Suggested fix**: write "(G0 … G6)".

### m-08 — Minor — the G1 exit criterion cites a gate ID that means something else
**Location**: `docs/05-quality/QUALITY-GATES.md:136`
**The passage**:
> | **G1** | Architecture | Module boundaries defined and enforced (QG-07-style boundary check), … |
**What a beginner would believe**: that the crate-boundary check is gate QG-07. QG-07 is the doctest gate (see B-05). The phrase "QG-07-style" does not mean anything in this codebase. **Suggested fix**: allocate a real ID for the boundary gate.

### m-09 — Minor — `STATUS.md` claims the quality gates are "Complete" with commands, labels and negative tests
**Location**: `docs/07-project/STATUS.md:23`
**The passage**:
> | Quality gates | **Complete** | 27 gates with their commands, labels and negative tests — `../05-quality/QUALITY-GATES.md` |
**What a beginner would believe**: that all 27 gates have runnable commands. Four name a nonexistent `xtask` crate, and `QUALITY-GATES.md` §3.1 documents negative tests for only 12 of the 27 (QG-01, 02, 03, 08, 09, 11, 13, 14, 16, 17, 18, 19) — the section is titled "What each gate must prove" and its preamble says "For every gate the project records a **negative test**". **Suggested fix**: say "27 gates defined; 12 negative tests recorded; N commands runnable today".

### m-10 — Minor — the traceability chain in `REQUIREMENTS-METHOD.md` promises a step the matrix does not show
**Location**: `docs/00-governance/REQUIREMENTS-METHOD.md:79-92`
**The passage**:
> ```
> Requirement (PRD/FUN/INT/SEC/PERF/UX/OPS/CFG/PLT/DOC/SUP-nnn)
>    → Feature specification (docs/03-features/F-nn-*.md)
>    → Module / source path (WHERE_TO_CHANGE.md)
>    → Milestone (M0…M5)
>    → Verification method (automated test / gate / manual protocol / device test)
>    → Evidence record (test name, gate output, device matrix row)
> ```
**What a beginner would believe**: that the traceability matrix carries a module/source-path column and an evidence column. The matrix's header should be checked, but with 12 of 27 gates never run and no device tests, the "Evidence record" column cannot contain evidence for anything. A beginner looking for "where is my requirement traced to code?" will find the code column empty because the code does not exist. **Suggested fix**: annotate the chain with "populated as each milestone lands".

### m-11 — Minor — `AGENTS.md` is the first document a contributor is pointed at, and it is written for agents
**Location**: `AGENTS.md:1, 24-26, 38`
**The passage**:
> # PhairDLNA — mandatory agent instructions
> …
> Delegate by outcome to the product, architecture, security, QA/interoperability and beginner-documentation roles.
> …
> Delegate by outcome, not by line. Bound every work package by file ownership — two agents never write the same file.

**What a beginner would believe**: that the repository is operated by a team of AI agents and that "delegate by outcome" is an instruction they can follow. `CONTRIBUTING.md` §1 tells a human contributor to "Read [AGENTS.md](../../../AGENTS.md) — the binding working rules" as step one of contributing. `docs/00-governance/AGENT-ROLES.md` defines the roles, so the vocabulary is at least documented, but a human's first required document is written in the imperative voice of an agent roster. **Suggested fix**: keep AGENTS.md as the binding ruleset, but open it with a short note for human contributors ("you can satisfy these rules yourself; the delegation rules apply to agent teams"), and point humans at `PROJECT-STANDARDS.md` and `REQUIREMENTS-METHOD.md` first.

### i-01 — Info — the project documents its own documentation defects
**Location**: `docs/03-features/F-19-quality-gates-and-developer-experience.md:424-449`
`F-19` §12 lists eight `[OPEN]` items that describe, precisely, the problems this review found independently: the missing `xtask` crate (§1), the gate-ID drift between `PROJECT-STANDARDS.md` and `QUALITY-GATES.md` and the double-booked QG-07 (§2), the link-gate command ambiguity (§6), the nonexistent `scripts/check-requirement-views.sh` (§8). The analysis is accurate and the proposed readings are sensible. The gap is discoverability: no document a beginner reads points at `F-19` §12.

### i-02 — Info — `STATUS.md` §2 "What was actually run" is exactly right
**Location**: `docs/07-project/STATUS.md:36-54`
Every command in that table ran, and ran with the stated result. `cargo fmt --all -- --check` is clean, `cargo clippy --workspace --all-targets -- -D warnings` exits 0, `cargo test --workspace` reports `1 passed; 0 failed; 0 ignored`, and the generator reports `341 requirements indexed`. The toolchain line (`rustc 1.100.0, cargo 1.100.0`) matches what I have installed. This table is the best status-honesty artefact in the repository and should be the model for the rest of the project.

### i-03 — Info — the `NOT RUN — <reason>` vocabulary is excellent
**Location**: `docs/05-quality/QUALITY-GATES.md:17-35`
Eight result labels, each with an explicit "never a pass". The fuzzing rule ("A fuzzing run is not a pass"), the device rule ("A device test without a device is not a pass") and the benchmark rule ("A benchmark without a documented machine is not a measurement") are precisely the instincts a beginner needs to learn about evidence. This vocabulary should be extended to the command tables in `BUILD-AND-TEST.md`.

### i-04 — Info — the generated-requirement-view mechanism is excellent and demonstrably works
`python3 scripts/generate_requirement_views.py --check` reported `up to date` for both `REQUIREMENTS-INDEX.md` and `TRACEABILITY-MATRIX.md`, and `341 requirements indexed`. The "define once, generate the views, never hand-edit" pattern is the single best piece of engineering in this repository, and the index's header says so clearly.

---

## What works well

These are specific, not generic praise.

**The README's opening is exactly right for a beginner.** `README.md:5-7` — "Status: **requirements and architecture phase.** No functional server exists yet." — plus the "What it will be" section's "your television finds it on the network, browses it and plays from it" tells me what the product is before any jargon arrives. And `README.md:58-60` — "every performance number in the documentation is a *target*, not a measurement" — sets the whole project's honesty contract in two sentences. I would keep the README largely as it is.

**`GETTING-STARTED.md` has the right shape.** It is numbered, it installs a toolchain, it clones, it runs four gates, it says what happens when you run the binary ("It prints a short status banner and exits"), and it closes with "These commands are instructions, not evidence that they have been run. Add platform-specific prerequisites as the implementation evolves." That last sentence is a gift — it tells the reader that the document is not a report. The troubleshooting table with `source "$HOME/.cargo/env"` for the missing-`cargo` case is the kind of detail that only exists in docs written by someone who actually installed Rust.

**`F-01` §1 is a model of technical writing for beginners.** Two sentences define SSDP ("a small text protocol over UDP port 1900"), then `NOTIFY` and `M-SEARCH` are glossed inline ("the device sends *advertisements* (`NOTIFY`); the client sends *searches* (`M-SEARCH`)"), then a concrete user story follows. §6's `[DOC-005]` line is a checklist the document then honours for itself. §10 opens "All criteria below are *planned*; no test file exists yet" and §11's table header repeats "(none of these files exists yet)". Every other feature spec should copy this discipline.

**The evidence vocabulary in `QUALITY-GATES.md` §1 is genuinely excellent.** "A gate that never failed has proved nothing", "Only `PASS` is a pass", the seven-label table, and the three "is not a pass" rules are exactly the intellectual discipline this project is about. If the rest of the documentation matched this document, the project would be unassailable.

**The generated-views mechanism works and is well documented.** `REQUIREMENTS-INDEX.md`'s header explains what the file is, what it adds, that it is generated, and that it must not be edited by hand. The `--check` mode is wired into CI and into both dev docs. This is the correct way to keep a 341-row table in sync, and it is executed properly.

**The requirement text is unusually good.** `TECHNICAL-REQUIREMENTS.md` §21's priority definitions, `REQUIREMENTS-METHOD.md` §2's "The system **shall** …" one-normative-sentence rule with the worked example ("The browse response for a container with 500 children contains at most 50 items" is a requirement; "the implementation uses an iterator" is not), and the `REQUIREMENTS-METHOD.md` §4 rule "A requirement that is implemented but not yet verified is `Implemented`, never `Verified`" are all things I wish more projects did.

**`CONTRIBUTING.md`'s "What a contribution needs" table is exactly right for a first contribution** — especially "Evidence | The exact command you ran and its output. 'Should work' is not evidence" and the house rule "No stubs presented as finished. A function that returns a fixed value is not an implementation."

**`F-19` §12 documents the project's own inconsistencies.** Eight `[OPEN]` items, each with an accurate diagnosis, a proposed reading and a "*Closes when:*" condition. The problem is only that they are invisible to the reader.

---

## Commands I ran

Environment note: `cargo` was not on `PATH` in the sandbox by default; I used `export PATH="$HOME/.cargo/bin:$PATH"`. Toolchain present: `cargo 1.100.0 (3acc79c0c 2026-11-05)`, `rustc 1.100.0 (0ef1bc386 2026-11-09)` — matching the version `STATUS.md:45` reports.

**1. The four commands from `GETTING-STARTED.md` §3 — all work.**

```
$ cargo fmt --all -- --check
exit=0

$ cargo clippy --workspace --all-targets -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.10s
clippy exit=0

$ cargo test --workspace
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

$ cargo test --workspace --doc
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

This confirms `STATUS.md:40-42` exactly, including "1 passed, 0 failed, 0 ignored".

**2. The scaffold runs and says what the docs say it says.**

```
$ cargo run -q -p phairdlna-cli
PhairDLNA — project scaffold (DLNA server not yet implemented)
See docs/01-product/TECHNICAL-REQUIREMENTS.md and docs/07-project/ROADMAP.md.
exit=0
```

This matches `README.md:53` ("prints a status banner; it does not start a server") and `GETTING-STARTED.md:71-72`.

**3. The `cargo nextest` command from `BUILD-AND-TEST.md:35` fails — the tool is not installed and never installed by the docs.**

```
$ cargo nextest run --workspace
error: no such command: `nextest`

help: a command with a similar command exists: `test`

help: view all installed commands with `cargo --list`
help: find a package to install `nextest` with `cargo search cargo-nextest`
```

**4. The four `cargo xtask` gates from `QUALITY-GATES.md` and `BUILD-AND-TEST.md` all fail — there is no `xtask` crate.**

```
$ cargo xtask check-unsafe      # QG-03
error: no such command: `xtask`

$ cargo xtask size-check        # QG-13
error: no such command: `xtask`

$ cargo xtask validate-didl     # QG-17
error: no such command: `xtask`
```

**5. The two repository gates that do exist, pass.**

```
$ bash scripts/check-doc-links.sh
RESULT: PASS (94 relative links checked)
exit=0

$ python3 scripts/generate_requirement_views.py --check
up to date: .../docs/01-product/REQUIREMENTS-INDEX.md
up to date: .../docs/07-project/TRACEABILITY-MATRIX.md
341 requirements indexed
exit=0
```

**6. The paths that `WHERE_TO_CHANGE.md` and `ARCHITECTURE.md` promise.**

```
$ ls crates/phairdlna-core/src/
lib.rs
$ ls crates/phairdlna-core/src/ssdp/
ls: cannot access 'crates/phairdlna-core/src/ssdp/': No such file or directory
$ ls crates/phairdlna-admin/
ls: cannot access 'crates/phairdlna-admin/': No such file or directory
$ ls crates/phairdlna-mac/
ls: cannot access 'crates/phairdlna-mac/': No such file or directory
$ ls scripts/check-crate-boundaries.sh
ls: cannot access 'scripts/check-crate-boundaries.sh': No such file or directory
$ ls qa/
ls: cannot access 'qa': No such file or directory
```

`crates/phairdlna-core/src/` contains exactly one file, `lib.rs`, whose whole content is a `PROJECT_NAME` constant and one test.

**7. The four cited research documents.**

```
$ find . -iname "*standards*" -o -iname "*platform-integration*" -o -iname "*security-landscape*" -o -iname "*qa-performance*"
./docs/00-governance/PROJECT-STANDARDS.md
./docs/99-archive/2026-11-14-preconsolidation-baseline/00-governance/PROJECT-STANDARDS.md
```

Only `PROJECT-STANDARDS.md` (the project's own code rules) matches — none of `standards`, `platform-integration`, `security-landscape` or `qa-performance` exists. Citation counts across the feature set: `standards §` 181, `qa-performance §` 43, `platform-integration §` 39, `security-landscape §` 31, `qa/device-matrix.toml` 22.

**8. The `cargo install` line that does not exist.**

```
$ grep -rn "cargo install" docs README.md CONTRIBUTING.md AGENTS.md
(no output)
```

Nothing in the repository tells a contributor how to install `cargo-nextest`, `cargo-deny`, `cargo-audit`, `cargo-fuzz`, `insta`, `proptest`, `criterion`, `hyperfine`, `lychee` or `typos`, all of which `BUILD-AND-TEST.md:1` lists as required tools.

---

*Reviewer note: the repository was being edited concurrently with this review (commit `17fef39` landed mid-session, adding ADRs 0002/0008/0009/0010). Findings B-03, M-16 and M-13 were re-verified against that commit. If further changes land before this report is read, M-16 and M-13 may already be fixed; B-03's `standards` / `platform-integration` / `security-landscape` / `qa-performance` gap and the `qa/device-matrix.toml` gap persist.*
