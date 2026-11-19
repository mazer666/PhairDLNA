# Project status — PhairDLNA

Status: **Baseline approved (2026-11-15) · M1.11 implemented · M2.1 implemented · M2.2 implemented · M2.3 implemented · M2.4 implemented (partly) · M2.5 implemented (partly) · M2.6 implemented (partly) · M2.9 implemented** · Compiled: 2026-11-18
Owner role: Lead / Project Manager

---

> **Current-head addendum (2026-11-17, commit `81a76d1e` before documentation fixes).**
> The narrative below chiefly records the M2.9 gate run and earlier M2.6 work. More
> recently, code was committed for interactive **CLI setup** (`1bec4cb7`),
> CLI diagnostics and exports (`1bec4cb7`), **Simple/Advanced configuration
> explanations** (`ab4d3645`), an embedded local **settings-inspection web UI**
> (`be02dff8`) and a **diagnostics-export action in the macOS menu** (`81a76d1e`).
> Therefore any unqualified statement below that “first-run onboarding”,
> “plain-language diagnostics”, or “UI diagnostics export” does not exist
> refers to an **earlier implementation stage** and is superseded for the
> named features. **Full native GUI onboarding, physical-device usability and
> accessibility acceptance remain open.** A full quality-gate run for the
> latest feature commits is **not documented**. The recorded 1,038 passing
> tests and three successful manual CI runs refer to **earlier revisions**.
> No real-device tests, performance benchmarks or seven-day soak tests have
> been recorded. The project remains not release-ready.

> **App-first UI decisions (2026-11-18, DEC-075 and DEC-076).** The embedded browser interface was removed;
> the native macOS application is the primary graphical setup and administration surface, and
> the loopback administration service is its authenticated local API. The app is a regular Dock
> application with a main window and a menu-bar status item. It currently has a
> menu bar status menu, folder picker, first-run folder prompt, preferences window and diagnostics
> export. The native overview opens automatically after the optional first-run prompt; its full setup flow, lifecycle onboarding and optional discovery check are still open.

## 1. Where the project stands

The **requirements baseline is approved** (2026-11-15), and the implementation is running:
the milestones M1.1 to M1.11 are coded, and M1.9 has been through an independent review
whose findings are addressed. The server now **serves over the network** — but **no device
test has been performed**, so no milestone exit criterion is met yet. The repository
contains a Rust workspace, a complete documentation baseline and the gates that will hold
the implementation honest.

The first native overview window is now wired to the live app snapshot and shows
server state, library count, shared folders, current scan/playback and TV
connection name/address with live discovery availability in the selected app language. This is an implementation
slice. Before its first status response it now says that the status is being checked instead of incorrectly claiming the server is stopped. AC-19/20/22 remain open: first-run guidance, state-specific controls,
discovery verification, degraded-state mapping, automated acceptance coverage
and real-Mac usability review remain open. The activity area only shows current
work, not a historical event log.

The milestone that was closed last is **M1.11 — the live daemon**: `phairdlna-cli serve`
reads the configuration, loads the device identity, scans the library, opens the HTTP
server and the discovery socket, and answers a client until it receives `SIGINT` or
`SIGTERM`. The server serves the device description, the two service descriptions, the
SOAP control endpoints, the media with its byte ranges and the artwork route; the
discovery socket advertises the server, answers the searches of a client and withdraws its
advertisements when the daemon stops. The order of the stop is the one of `OPS-008`:
the advertisements go first, the discovery stops, the HTTP server drains its transfers,
and the process exits with the code 0.

**What is proven and how.** The daemon is tested as a client sees it, over a real socket:
`crates/phairdlna-cli/tests/serve.rs` reads the description, browses the catalogue with
a SOAP request, streams a file and asks for a byte range; `crates/phairdlna-cli/tests/discovery.rs`
speaks the real SSDP protocol and observes the advertisement, the answer of a search and
the withdrawal; `crates/phairdlna-cli/tests/serve_command.rs` starts the binary, reads the
description and ends it with a `SIGTERM`. The rules themselves stay in the core, and the
core is unchanged by M1.11 — it takes no new dependency.

**The first step of M2 is closed: M2.1 — multiple roots, external volumes and network
shares.** The catalogue keeps the roots apart: every root has its own key in the
identifiers, its own place in the tree and its own boundary, so a link from one root
into another is not followed and a root that lies inside another root is published
once. The catalogue now **remembers the volume of every root** between two reads, and
it says what changed: a drive that was unplugged takes its files with it and is
reported, a drive that comes back under the same identity keeps its identifiers, and a
drive that comes back under **another** identity is given new identifiers while the
identifier of the catalogue grows, so that a client browses again (`PLT-011`). The
daemon watches the roots — a cheap question that reads no file of the library — and
reads the catalogue again when the answer is "yes"; a file that is added *inside* a
root is noticed by the next full scan, and the schedule of the scans is the work of
`FUN-024` (M2.4). The evidence is
`crates/phairdlna-core/tests/media_roots_multiple.rs`,
`crates/phairdlna-core/tests/volume_events.rs` and
`crates/phairdlna-core/tests/volume_platform_events.rs`; the decisions are D-15 to
D-20 of `../03-features/F-08-media-roots-and-filesystem.md`.

**The second step of M2 is closed: M2.2 — metadata extraction, artwork and thumbnails.**
The library now shows its pictures. A **pure-Rust** parser reads the cover out of an ID3v2
tag (`APIC`, versions 2.2, 2.3 and 2.4) and out of an MP4 container (the `covr` atom), and
the image that lies **next to** a file — `cover.jpg`, `folder.jpg`, `album.jpg` or the name of
the file itself with an image extension — wins over the cover in the tag. A photo of the
library is its own picture: the server produces a small JPEG of it, never larger than it, and
keeps it in a **disposable cache** that may be deleted at any time, that never grows beyond
its bound and that removes its oldest pictures when it does. The picture is served at
`/art/<id>`, and the DIDL-Lite document names it in `upnp:albumArtURI` — an absolute address
of this server, with no path of the disk and no `dlna:profileID`. A file that is broken
gives no picture and no error: the library stays browsable and the daemon keeps serving. The
evidence is `crates/phairdlna-core/tests/artwork.rs` and `crates/phairdlna-cli/tests/artwork.rs`.

**The third step of M2 is closed: M2.3 — playlists, subtitles and organisation.** The library now
understands the lists the owner wrote by hand. A **playlist** — M3U, M3U8 or PLS — is read within
bounds (a mebibyte, ten thousand entries, a line of four kibibytes), and every entry is resolved
against the folder of the playlist itself. An entry that names a **network address** is refused
outright: the server makes no request for it. An entry that walks **out of the root** is refused
and reported. An entry that names a file that is not there stays in the playlist as a child that
carries no resource, so that the user learns where the track went instead of wondering. The
playlist is a **container** of the class `object.container.playlistContainer`, and its children keep
the **order** of the playlist — a list that is sorted is a different list. A broken playlist never
takes a scan down: it produces a diagnostic and the rest of the library is indexed.

A **subtitle** that lies next to a film is paired with it while the root is read, not while a
client browses. The pairing is an **additional `res`** of the media object, with its own
`protocolInfo` that names the type of the file (`text/vtt`, `application/x-subrip`, `text/x-ssa`,
`text/x-microdvd`), and the file is served from the media endpoint with that type and the
character set it is written in. The element is additive: a client that does not understand it still
plays the film. When several sidecars match one base name, the one with the highest priority
extension wins — WebVTT first, because it is the format a browser-based client reads without a
conversion.

The **organisation** is decided as well. The catalogue computes an **identity key** for every
file — the path, per component, with Unicode NFC and the case fold of the volume, scoped by the
identity of the volume — and it takes the **uniqueness** of a file from the stable file identity
the platform gives, not from the path. A file that two roots reach is therefore **one** catalogue
entry: the first root wins, the second sighting resolves to the entry that is already there, and
the identifier of the object does not change. The evidence is
`crates/phairdlna-core/tests/playlists.rs`, `crates/phairdlna-core/tests/subtitles.rs` and
`crates/phairdlna-cli/tests/subtitles.rs`; the decisions are DEC-048 to DEC-057 of
`../00-governance/DECISION-LOG.md`.

**The fourth step of M2 is closed in part: M2.4 — the configuration that means something.**
Two settings of the configuration were written down long ago and were never obeyed, and that is
the worst kind of setting: the user believes in it and it does not exist. The **access-control
list** (`security.allowed_clients`) is now read at the start, checked against the **address of
the socket** — and not against a header, which a client that is not allowed may write as it
likes — and a client that is not on the list is refused with `403` before a handler sees the
request. An entry of the list is an address (`192.168.1.20`), a network (`192.168.1.0/24`) or the
word `any`; an entry that cannot be read stops the server before a socket is opened, because a
list that is silently ignored is a list that the user believes in. The **administration surface**
has its own settings now: the port it listens on, the addresses it binds to, the file that holds
its token, and the word that lets it leave the machine — a configuration that would put the
surface on an address of the network is **refused** unless the user says so in the configuration
(`CFG-005`, DEC-061). The **export** and the **import** of the configuration exist as commands:
`phairdlna config export` writes the configuration as the server understands it, with the schema
version, and `phairdlna config import <file>` reads a file and reports the configuration it
would become — it writes nothing, and that is the part that is still open. The evidence is
`crates/phairdlna-cli/tests/serve.rs`, `crates/phairdlna-cli/tests/cli.rs` and
`crates/phairdlna-core/src/security/access.rs`.

**The fifth step of M2 is closed in part: M2.5 — the local administration surface.** The server
now has a door for its owner, and the door is narrow on purpose. The surface lives in the crate
`phairdlna-admin`, listens on the **loopback interface only** (`127.0.0.1` and `::1`), on a
port of its own, and is started only when the configuration asks for it; a build without the
cargo feature `admin` has no surface at all. Every request must carry the **bearer token** of
the installation — a random secret of 256 bits in a file with the permissions `600` inside a
folder with the permissions `700`; a file that others may read stops the surface instead of being
repaired silently. The surface answers only to a `Host` header that names the loopback, so that
the "DNS rebinding" attack finds nothing; it answers only to a request that carries the custom
header `X-Phair-Admin`, which a form in a browser cannot send; it carries **no header that would
let another origin read the answer**, and it answers no question about permissions. A request
that changes something carries the token of the **"cross-site request forgery"** defence, which
is bound to the session — the session *is* the token of the installation (DEC-060).

The surface answers three questions. **`/api/status`** says how long the server has been running,
how many streams are running, how many objects the catalogue holds, how much memory the process
occupies (where the platform can say), what the roots of the library look like and which
identifier the server tells its clients that it has changed. **`/api/catalogue/rescan`** reads
the library again. **`/api/diagnostics`** shows the records of the log and the warnings of the
server — with the paths of the user replaced by their pseudonyms, and the surface redacts again
what the daemon behind it hands over, so that a leak needs two mistakes and not one. The
transfers that run are registered, so that the diagnostics name the client and what it is
playing. The **source revision** is embedded into the binary when it is built, so that the
running server and the source it will offer cannot disagree. The evidence is
`crates/phairdlna-admin/tests/admin_surface.rs` and `crates/phairdlna-cli/tests/admin.rs`; the
decisions are DEC-058 to DEC-064 and **ADR-0012**.

**The sixth step of M2 is closed in part: M2.6 — the macOS application.** The regular app has a main overview and menu-bar status item. The crate
`phairdlna-mac` holds a real application now, and the application is nothing but a front end: it
holds **no state of the server of its own**, asks the administration surface of M2.5 what the
server does, and shows the answer as a line of text in the menu bar (DEC-065). The client of the
surface is **written in this project** — a small HTTP/1.1 client, a small reader of JSON with
bounds on its size and its depth, and a transport that is a trait, so that the tests answer the
client with a recorded answer and need no socket (DEC-066). The crate depends on the core of
the project and on the bindings of the system and on **nothing else**: no network library, no
serialization framework, and no connection to anything but the loopback interface — a test walks
the sources and the manifest of the crate and proves it (`tests/no_egress.rs`). The requests run
on a **thread of their own**, so that the main thread of the application never waits for the
network and the menu never stutters (DEC-067); the channel of the commands holds one command,
so that the application never has more than one request in flight. The token of the
"cross-site request forgery" defence is read out of the header that the surface hands with every
answer and sent back with the request that changes something (DEC-068).

The **texts of the user interface live in one resource file per language** — `en-US` and `en-GB`
— and in no other place: the lines of the menu, the labels that the screen reader reads, the
messages of a failure and the help of the command line (DEC-071). A test reads the sources of
the crate and refuses every text that a user would read. The **two views** — the short one and
the detailed one — read the same state, and the mode is a preference of the *application* that
lives in a file of its own, beside the configuration of the server and not inside it: switching
the view changes no setting of the server, restarts nothing and interrupts no transfer (DEC-070,
`UX-015`, `UX-016`). The state of the server is **text and never a colour alone**: the item
carries the label and the value that the screen reader reads, and the picture of the system
follows the appearance of the menu bar (`UX-009`, `UX-018`, `MB-006`).

The **only file of the workspace that uses `unsafe`** is the bridge to the Objective-C runtime of
the menu bar application, because AppKit hands the action of a menu item to a target object as a
message and a class of our own cannot be written without it. The record **ADR-0013** carries the
safety argument of every block, and the gate **QG-03** knows that one file by that record: a
file with `unsafe` and without the marker still fails the gate, and a marker that names a record
that does not exist fails the gate as well (DEC-069). The evidence is
`crates/phairdlna-mac/tests/ipc_client.rs`, `crates/phairdlna-mac/tests/menu.rs`,
`crates/phairdlna-mac/tests/application.rs`, `crates/phairdlna-mac/tests/worker.rs`,
`crates/phairdlna-mac/tests/presentation_modes.rs`, `crates/phairdlna-mac/tests/bundle_metadata.rs`,
`crates/phairdlna-mac/tests/no_egress.rs` and `crates/phairdlna-mac/tests/strings.rs`; the
decisions are DEC-065 to DEC-071.

**Parts of M2.6 are still open, and they are named.** A **first-run folder prompt** exists, and
the overview now has an add-folder button and a read-only explanation. The full assistant
(`UX-002`, `UX-003`, `MB-007`) is not built: the first-run folder prompt itself does not show the
server name/address or offer the optional discovery check. The main overview receives those values
from the authenticated status API. The **verification step** (`UX-001`)
is not built: the application does not yet tell the user that a client has found the server.
Start and stop menu actions are wired to `launchctl`, but the helper looks for a per-user
`LaunchAgents` plist; the approved service model is a system-context launchd daemon
(`DEC-032`, `PLT-013`). That mismatch and the install/control/recovery path are not resolved or
verified, so `UX-019` is not complete. Scan progress and cancellation (`UX-013`) are also not
built: the app shows a scanning state but no progress or cancel action. The **usability protocol** (`AC-01` … `AC-07`, `AC-15`, `AC-16`, `AC-21`) is now written in
`qa/macos-usability-protocol.md`; it has not been run. The **screen-reader audit** (`UX-017`) still needs a Mac with the app installed, so **no device test of the application has been
performed**. The **universal binary** (`PLT-004`) is a step of the release and not of this
milestone. The server's CLI remains its headless entry point. GUI control must follow the
approved service installation design; the existing `launchctl` calls are a partial implementation,
not verified evidence that the approved daemon can be safely started and stopped. Folder additions
currently write the configuration file directly; the first-run wizard hides save failures, and the
preferences Save button is unwired. F-14 has no write route and the F-12 baseline rejects runtime
reload. The wizard still requests a rescan after returning `Configured`, but that rescan only
refreshes the media library built from the server's startup configuration; it does not load newly
saved roots. A failed write can therefore appear configured, and a successful write still does not
make the new roots effective in the running server. Treat these as prototype gaps gated by OD-16,
not as completed app settings.

**The catalogue of the library now lives on disk as well: M2.9 — the index.** The
catalogue is still built in memory and answered from there, but a copy of it is written
into a **SQLite database** that belongs to the user alone: the file is created with the
permissions 600, and the directory that holds it with 700. The path of the file is the
setting `library.database_path`, and the data directory of the user holds it when the
setting is missing.

The index has **two connections**: one that writes and one that reads, and the journal
of the database is the write-ahead log. The whole catalogue is written in **one
transaction**, so a browse during a rescan sees the catalogue of before or the catalogue
of now, and never a mixture of the two (`FUN-008`). The server **takes the catalogue of
the index at its start** and reads the library again in the background, so a restart does
not cost a full scan and a client never meets an empty server because the index is empty
(`FUN-009`).

A file that is **damaged** — bytes that are not a database, a database that fails the
integrity check, or a database of another schema — is moved aside under the name
`.corrupt.<moment>`, a new index takes its place, the library is read again, and the
daemon says so **in the log and in the status of the server** (`FUN-010`). The index is a
cache: deleting it costs a re-scan and nothing else, and the media of the user is never
written to.

The evidence is `crates/phairdlna-core/tests/catalogue_rebuild.rs` and
`crates/phairdlna-cli/tests/catalogue_index.rs`; the decisions are D-01, D-05 and
D-12 to D-16 of `../03-features/F-09-catalogue-index.md`.

**One consequence of the index is on the record.** SQLite is compiled into the server
(the binding `rusqlite` with the feature `bundled`, as ADR-0006 and the dependency review
decide), so that the server does not depend on the SQLite of the system and cannot load
an extension into its own database. The price is a **C compiler at the time of the
build**, which sits in tension with `PLT-009` ("no C toolchain beyond what the platform
already provides") and with the claim of the workspace that the core is free of a C
compiler. The decision itself is the owner's and was taken in the dependency review; the
consequence is recorded here and in `../04-security/DEPENDENCY-LICENSE-REVIEW.md`, and it
is the one thing of this step that the owner may want to overrule.

**At the M2.9 checkpoint, two parts of M2.4 and M2.5 were still open.** The **source offer**
(`OPS-016`, `SEC-070`) does not exist: the archive, the route and the build step that produces
the artefact are not written, and the open question 2 of F-14 asks the owner how the archive is
to be produced. The **plain-language first-run and diagnostics CLI** were then open, but the later commit `1bec4cb7` adds `setup` and `diagnostics` commands. Their current full behavior and acceptance must be reviewed against the updated tests. The
**trusted-client list** (`CFG-010`) waits for the device profiles of F-07. The **address of the
client** is not known to the routes of the media server yet, so the diagnostics name the client
and the title but not the address.

**One deviation is on the record.** The picture pipeline of M2.2 parses **in the server
process**, not in the sandboxed child that ADR-0008 prescribes, because that child does not
exist yet — it is the work of M-06. Every read and every allocation of the pipeline is
bounded, and the parsers are written so that they move into the child without a change of
their interface. The deviation is written down in **ADR-0011** and in DEC-042, and it ends
with the child.

**What is still missing.** The IPv6 discovery, the interface selection of the sending
socket (the daemon has one address), the rate limiter of the answers, the video posters
(the baseline ships no video decoder), the sandbox child of the picture pipeline (M-06,
ADR-0011), the transfer limits, the incremental scan of the index and the change events of the
filesystem. Each of them is named in the requirement index with the state that is true
today.

The milestone before it, **M1.9 — the catalogue over one root**, is unchanged: a scanner
that reads a configured root, opaque blake3 identifiers that survive a rescan and a move
of the library, and a ContentDirectory browse that shows real folders and files. The
placeholder catalogue is retired; `phairdlna-cli catalogue scan` prints the tree with the
identifiers a client would see, and `phairdlna-cli catalogue browse <id>` prints the
document a client would receive.

**The configuration takes a list of roots**, and M2.1 is the milestone that makes the
list mean something: several roots are browsable at the same time, they keep their own
identifiers and their own boundaries, and a root that comes and goes with its drive is
handled while the server runs.

**The requirement group of M1.9 that was open is now decided.** The roadmap assigned
`FUN-008`, `FUN-009` and `FUN-010` — the SQLite index that survives a rescan, that can
be rebuilt and that recovers from corruption — to M1.9. The scope the owner approved for
M1.9 was the scanner, the opaque identifiers and the browse over one root; the index on
disk was not part of it, so the catalogue of M1 lives in memory. The owner decided on
2026-11-16 (**DEC-037**) that the three requirements **move to M2**, where the rest of the
index work lives (step M2.9 of the roadmap). The in-memory catalogue of M1 is the
accepted baseline, and the requirement index carries the new milestone.

| Area | State | Evidence |
| --- | --- | --- |
| Requirements | **Complete** | 346 requirements with IDs, priorities, features, milestones and verification methods — `../01-product/REQUIREMENTS-INDEX.md` |
| Traceability | **Complete** | Generated matrix requirement → feature → milestone → module → verification → evidence — `TRACEABILITY-MATRIX.md` |
| Architecture | **Complete** | Module boundaries, data flows, runtime model — `../02-architecture/ARCHITECTURE.md` |
| Decisions | **Complete** | The ADRs in `../02-architecture/adr/` plus 76 entries in the decision log — `../00-governance/DECISION-LOG.md` |
| Security model | **Complete** | Eight mitigations, 28 threats, each with its verification — `../04-security/SECURITY-MODEL.md` |
| Privacy model | **Complete** | Data inventory, network posture, source offer — `../04-security/PRIVACY.md` |
| Dependency review | **Complete** | Licence data for every candidate crate, retrieved 2026-11-14 — `../04-security/DEPENDENCY-LICENSE-REVIEW.md` |
| Quality gates | **Defined** | 28 gates with their commands, the label vocabulary and 15 recorded negative tests; 13 gates can run today — `../05-quality/QUALITY-GATES.md` |
| Test strategy | **Complete** | Test layers, property tests, negative catalogue, fuzz targets — `../05-quality/TEST-STRATEGY.md` |
| Interoperability | **Framework complete, evidence empty** | 26 conformance checks defined; **no device test has been performed** — `../05-quality/INTEROPERABILITY-MATRIX.md` |
| Performance | **Budgets defined, not measured** | Budgets and the measurement protocol — `../05-quality/PERFORMANCE-BUDGETS.md` |
| Roadmap | **Complete** | Six milestones with exit criteria — `ROADMAP.md` |
| Risk register | **Complete** | 16 risks with mitigations and owners — `RISK-REGISTER.md` |
| Open decisions | **15 awaiting the owner** — OD-05 (minimum macOS version) and OD-08 (app-first interface) are decided; OD-15 gates service lifecycle, OD-16 gates app configuration writes, and OD-17 gates larger-text behavior | `OPEN-DECISIONS.md` |
| Glossary | **Complete** | The protocol, product, platform, security and project vocabulary — `../01-product/GLOSSARY.md` |
| Research inputs | **Complete** | The briefs the specifications cite — `../09-research/` |
| Implementation | **M0 done, M1.1–M1.11 implemented, M2.1, M2.2, M2.3 and M2.9 implemented, M2.4, M2.5 and M2.6 in part** | The four crates exist with the boundary enforced; configuration, platform traits, discovery, media serving, catalogue, CLI, diagnostics and native menu app are implemented in part or in full; the browser UI was removed (DEC-075); the initial native overview shows live state, item count and folders; full app flows, lifecycle, accessibility review and device discovery remain open; **no device test has been performed**, so no milestone exit criterion is met yet |
| Device evidence | **None** | Every device row reads `NOT TESTED` |
| Release | **Not releasable** | `RELEASE-BLOCKERS.md` |

---

## 2. What was actually run, and what it produced

| Command | Result |
| --- | --- |
| `./scripts/local-ci.sh` | **all eleven gates green** (2026-11-17, the run of M2.9): format, clippy, tests, no unsafe, file size, crate boundaries, doc links, licences, bans and sources, advisories, requirement views |
| `cargo fmt --all -- --check` | clean (it failed before this phase — the CLI entry point was reformatted) |
| `cargo clippy --workspace --all-targets -- -D warnings` | exit 0 |
| `cargo test --workspace` | **1038 passed, 0 failed, 0 ignored** in the recorded M2.9 run of 2026-11-17; **not rerun here against the newest UI commits** |
| `scripts/check-file-size.sh` | `RESULT: PASS` |
| `scripts/check-crate-boundaries.sh` | `RESULT: PASS` (3 sibling dependencies across 4 crate manifests) |
| `cargo deny check licenses` | `licenses ok` (exit 0) |
| `cargo deny check bans sources` | `bans ok, sources ok` (exit 0) |
| `cargo audit` | exit 0 — 1509 advisories loaded, 124 dependencies scanned, no finding |
| `scripts/check-doc-links.sh` | `RESULT: PASS` (relative links checked) |
| `scripts/check-no-unsafe.sh` | `RESULT: PASS` (no unsafe code in the tracked Rust files) |
| `python3 scripts/generate_requirement_views.py --check` | up to date; 346 requirements indexed |
| GitHub Actions, manual runs | **success on earlier commit SHAs, not current HEAD** — `43698100636`, `43698591233`, `43699136828`, all 13 gate steps green each time: https://github.com/mazer666/PhairDLNA/actions/runs/43699136828 |
| `python3 scripts/generate_requirement_views.py` | 346 requirements indexed; index and traceability matrix regenerated |

Toolchain used for the M2.1 run (and for M1.11 before it): rustc 1.99.1, cargo 1.99.1,
clippy 0.1.99, rustfmt 1.8.0-stable (Homebrew). `rust-toolchain.toml` pins the channel `stable`, so the toolchain
is within the support window (`rust-version` is 1.89).

Everything else in this repository is **documented and unmeasured**. In particular:

- no performance number has been recorded,
- no device test has been performed,
- no fuzzing run has been performed,
- no soak test has been performed,
- no claim of DLNA certification exists and none will be made.

---

## 3. The baseline in numbers

| Number | Value |
| --- | --- |
| Requirements | 346 — 284 MUST, 57 SHOULD, 5 COULD |
| Feature specifications | 22 (F-01 … F-19, F-22, plus two proposals, F-20 / F-21) |
| Acceptance criteria | 330 across the 20 baseline specifications (F-20 and F-21 are proposals and define none) |
| Glossary entries | 77 terms across five vocabularies |
| Independent reviews | 3 — a beginner-experience review, a consistency review and the review of M1.9, all in `../05-quality/reviews/` |
| Protocol conformance checks | 26 (`PC-01` … `PC-26`) |
| Threats modelled | 28 (`SEC-TH-01` … `SEC-TH-28`) |
| Quality gates | 28 (`QG-01` … `QG-28`), of which 13 can run today |
| Milestone gates | 7 (`G0` … `G6`) |
| Architecture decisions | The ADRs in `../02-architecture/adr/` + 76 decision-log entries |
| Open owner decisions | 15 listed in `OPEN-DECISIONS.md`; OD-05 and OD-08 are decided, OD-15 gates macOS service lifecycle, OD-16 gates app configuration writes, and OD-17 gates larger-text accessibility |
| Risks | 16 (`R-01` … `R-16`) |
| Archived baselines | 3 snapshots, each with a SHA-256 manifest |
| Research briefs committed | 9 documents under `../09-research/` |
| Competitors mapped | 11 products with per-feature proposals — `../01-product/COMPETITOR-FEATURE-ATLAS.md` |
| Competitive criteria | 11 testable criteria against the macOS field — `../01-product/COMPETITIVE-PARITY-ELMEDIA.md` |

---

## 4. What we should do next — the proposal

### 4.1 M0 is closed, with two items still open

M0 was the foundation: the approved baseline, the architecture, the four crates with the
boundary enforced, the licence and advisory gates on the real dependency set, and the
manual CI workflow. All of it is done. Two items of it are not:

| Item | What | Done when |
| --- | --- | --- |
| M0.5 | The manual CI workflow has been run on macOS and on Windows, not only on Linux | three run URLs exist in the run list, one per platform, and every step passed |
| M0.6 | The Intel macOS target of the build matrix builds | the matrix shows a green x86_64-apple-darwin build |

### 4.2 M1, in ten small steps

Each step ends in something observable, and each is small enough to review in one
sitting. The order is not negotiable: each step needs the one before it.

| # | Step | The observable that says it is done | State |
| --- | --- | --- | --- |
| 1 | Configuration and the platform traits | `phairdlna-cli config validate` accepts a generated configuration and rejects a broken one with a named field | done (M1.1, M1.2) |
| 2 | SSDP discovery | `phairdlna-cli ssdp headers` prints the exact bytes that will be sent, and a recorded `M-SEARCH` from a real client is answered | done (M1.3) |
| 3 | The device description and the SCPD | `curl http://localhost:PORT/description.xml` returns a document that a real client accepts | done (M1.4) |
| 4 | The SOAP dispatcher and ContentDirectory | A recorded `Browse` returns a DIDL-Lite document that validates against the vendored XSD (gate QG-17) | done (M1.5, M1.6); the XSD validation of QG-17 is still **planned** |
| 5 | ConnectionManager | `GetProtocolInfo` names only MIME types the server can actually serve | done (M1.6) |
| 6 | DIDL-Lite generation | The escaping property test passes, and the XSD validation passes | done (M1.7) |
| 7 | HTTP media delivery | A byte range returns 206 with a correct `Content-Range`; an unsatisfiable range returns 416 | done (M1.8) |
| 8 | The catalogue over one root | A browse shows one folder of real files with stable ids | **done (M1.9)** |
| 9 | The live daemon | `phairdlna-cli serve` answers a client over HTTP and over SSDP, and stops gracefully on a signal | **done (M1.11)** |
| 10 | The security baseline | Every threat row in F-16 has its negative test, and they pass | next |

**Step 10 is not optional and not last.** The threat rows are cheap to test while the
code is small, and expensive to retrofit. It is also the step that M1.11 was implemented
ahead of; it is closed next, against the code that is already written.

### 4.3 The first real device, and when

**No device test happens before steps 1 to 8 are green.** The conformance checklist
(`PC-01` … `PC-26`) is automated and runs first; a device is the *second* test, not
the first. The first device round is scheduled inside M1, and its result — including
failures — goes into the interoperability matrix the same day.

**Which device first:** a Samsung or LG television, because they are the most
frequently reported source of DLNA incompatibilities. If only one is available, use
VLC on a laptop, because it logs what it receives.

### 4.4 What remains outside the current M2.6 slice

- The app-first native experience is in progress. Transcoding, broad media-library
  integration and Linux/Windows packaging remain later scope; they do not replace the
  current M2.6 onboarding, safe settings and accessibility work.
- No plugin system, ever, without an ADR.
- No automatic CI. Manual only, until the gates have been run three times green and
  their negative tests demonstrated.

### 4.5 What the owner has to answer, and what happens if they do not

Fifteen open decisions are listed in `OPEN-DECISIONS.md`, each with a recommendation
**and** the default that applies if there is no answer. **OD-05** (the minimum macOS
version) was decided on 2026-11-15: macOS 15 (Sequoia) and later. OD-15 keeps the exact
service identity and selected-folder access model open; OD-16 gates app settings writes
and activation; OD-17 proposes an app-level larger-text control because macOS does not
support Dynamic Type. The revised accessibility requirement is awaiting owner approval.

### 4.6 The honest risk of this plan

The plan is a plan, not a forecast. The largest uncertainty is still the device
matrix: the first device round may reveal that a requirement is wrong. That is the
point of the matrix, and it is cheaper to learn now than after M2.

---

## 5. Honest limitations of this baseline

- **The interoperability matrix is empty.** The mechanism is defined; the evidence is
  not there yet. The first device round will change several requirements, and that is
  the intended purpose of the matrix.
- **The performance budgets are targets.** They were chosen to be plausible and
  measurable; they have not been checked against a real machine.
- **The macOS firewall and local-network behaviour are documented by Apple and not yet
  verified on a device.** They are the two largest macOS-specific risks (R-02, R-03).
- **The competitive landscape rests on public documentation, not on comparative
  testing.** No competitor was benchmarked.
- **Some protocol details could not be verified against a primary specification** and
  are recorded as open questions in the feature specifications — in particular the
  `writeStatus` attribute, the exact `dc:date` format constraints, and the DLNA
  guideline table that a media server must satisfy.
