# Decision log

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Lead / Architect

Every decision has an ID, a status, the date it was taken and who may change it.
Statuses:

- **Owner direction** — decided by the project owner; agents may not change it.
- **Accepted** — an architecture/process decision taken by the Lead within the mandate
  given by the owner; reversible by the Lead, recorded here.
- **Proposed** — a recommendation waiting for owner approval.
- **Superseded** — replaced by a later decision; the original text stays in this table.

Architectural decisions with options and trade-offs live as ADRs in
`../02-architecture/adr/` and are referenced from this table.

---

## Product and licensing

| ID | Decision | Status | Date | Authority |
| --- | --- | --- | --- | --- |
| DEC-001 | Public, AGPL-3.0-or-later, ad-free, telemetry-free, privacy-first, no vendor account | Owner direction | 2026-11-14 | Owner |
| DEC-002 | Rust-first portable core; macOS first, Linux and Windows architecturally included from the start | Owner direction | 2026-11-14 | Owner |
| DEC-003 | Documentation is a product deliverable and must be readable by a programming beginner | Owner direction | 2026-11-14 | Owner |
| DEC-004 | GitHub Actions are manual-only; no unapproved releases or tags | Owner direction (confirmed) | 2026-11-14 | Owner |
| DEC-005 | Full, unmodified AGPL-3.0 text ships in `LICENSE`; the project is "AGPL-3.0-or-later" | Accepted | 2026-11-14 | Lead |
| DEC-006 | Offline/LAN-first: no internet dependency for normal operation, no automatic router port mapping, no cloud services | Owner direction | 2026-11-14 | Owner |
| DEC-007 | Media files are opened read-only by default; the server never modifies originals without explicit user consent | Accepted | 2026-11-14 | Lead |
| DEC-008 | The headless server must be fully functional without any GUI; the macOS menu bar app is an optional front end for the same core | Accepted | 2026-11-14 | Lead |

## Requirements and quality

| ID | Decision | Status | Date | Authority |
| --- | --- | --- | --- | --- |
| DEC-009 | Requirements before implementation: the owner approves a requirements baseline before feature code is written | Owner direction | 2026-11-14 | Owner |
| DEC-010 | Requirement IDs are stable, prefixed by domain, and traceable to a feature, a milestone and a verification method | Accepted | 2026-11-14 | Lead (see `REQUIREMENTS-METHOD.md`) |
| DEC-011 | A gate result is only reported as passed when it was actually observed; `NOT RUN` and skip exit codes are never reported as success | Accepted | 2026-11-14 | Lead |
| DEC-012 | No performance, compatibility or security claim without recorded evidence | Owner direction | 2026-11-14 | Owner |

## Architecture

| ID | Decision | Status | Date | Authority |
| --- | --- | --- | --- | --- |
| DEC-013 | Documentation is consolidated into the canonical `00`–`99` structure; the legacy `docs/requirements`, `docs/architecture`, `docs/security` and `docs/project` trees are migrated with an archive snapshot | Accepted | 2026-11-14 | Lead |
| DEC-014 | The Rust core is platform-neutral; everything OS-specific lives behind a narrow platform-adapter trait | Accepted | 2026-11-14 | Lead (ADR-0003) |
| DEC-015 | SSDP and the UPnP control layer are implemented in-project; no third-party UPnP/SSDP stack is taken as a dependency | Accepted | 2026-11-14 | Lead (ADR-0005) |
| DEC-016 | The SQLite index is the single source of truth for the catalogue; the index is rebuildable and never authoritative for file contents | Accepted | 2026-11-14 | Lead (ADR-0006) |
| DEC-017 | Transcoding is optional, isolated in its own process boundary, and off by default | Accepted | 2026-11-14 | Lead (ADR-0009) |
| DEC-018 | Media metadata and thumbnails are parsed in a sandboxed child process, never in the server process | Accepted | 2026-11-14 | Lead (ADR-0008) |
| DEC-019 | The runtime, HTTP server and XML parser are Tokio, Hyper/Axum, quick-xml and rusqlite, with the HTTP framing policy and the XML policy owned by the project | Accepted | 2026-11-14 | Lead (ADR-0002) |
| DEC-020 | The AGPL network-source offer is a product feature (an unauthenticated route on the running server), and the server never updates itself over the network | Accepted | 2026-11-14 | Lead (ADR-0010) |
| DEC-021 | `docs/05-quality/QUALITY-GATES.md` is the single register of the gate identifiers; every other document refers to a gate by the ID defined there | Accepted | 2026-11-15 | Lead |
| DEC-022 | The research briefs the specifications cite are committed under `docs/09-research/` as evidence, not as norm; a citation such as `standards §8.3 C12` resolves there | Accepted | 2026-11-15 | Lead |
| DEC-023 | The documentation carries one identifier scheme only. A draft that introduced its own IDs is archived or explicitly marked as non-normative | Accepted | 2026-11-15 | Lead |
| DEC-024 | The competitive bar is the macOS media player **Elmedia**: PhairDLNA must be the better macOS media *server* and must not be the worse macOS media *tool*. The bar is expressed as eleven testable criteria, not as a slogan | Owner direction | 2026-11-15 | Owner |
| DEC-025 | A competitor feature is adopted only after an evaluation record: source, user problem, our independent better outcome, architecture fit, security and privacy analysis, resource budget and acceptance tests. The record decides CORE / OPTIONAL / LATER / REJECT | Accepted | 2026-11-15 | Lead |
| DEC-026 | The Apple media libraries stay out of the baseline. They are a deliberate, per-library opt-in (OD-13), never a default | Accepted | 2026-11-15 | Lead |
| DEC-027 | The object identifiers of the catalogue are the blake3 hash of the volume identity, the root key and the relative identity, cut to 128 bits and rendered as hexadecimal | Owner direction | 2026-11-15 | Owner |
| DEC-028 | The portable filesystem adapter of the core reports the constant volume identity `local`, so that the object identifiers survive a remount and a move of the library; the real volume identity arrives with the platform adapter of F-13 | Accepted | 2026-11-15 | Lead (ADR-0007, `PLT-005`) |
| DEC-029 | The root key — the position of the root in the configuration — is part of the object identifier, so that two roots of one configuration keep their files apart; the name of a root is a label and is not part of it | Accepted | 2026-11-15 | Lead (FUN-022, ADR-0007) |
| DEC-030 | A folder of the library is a container in the DIDL-Lite document and not an item; the catalogue model carries `MediaKind::Folder` and the number of children | Accepted | 2026-11-15 | Lead (INT-042) |
| DEC-031 | The requirements baseline of the v1.0 documents is approved, so feature implementation may proceed | Owner direction | 2026-11-15 | Owner |
| DEC-032 | The minimum supported macOS version is macOS 15 (Sequoia) and later, with the launchd daemon model throughout (OD-05) | Owner direction | 2026-11-15 | Owner |
| DEC-033 | The asynchronous runtime (`tokio`), the HTTP server (`axum`) and the sockets live in `phairdlna-cli`; the core keeps the rules and stays portable and free of a runtime | Accepted | 2026-11-15 | Lead (M1.11, ADR-0002) |
| DEC-034 | The SSDP socket lives in the command-line crate; the core keeps the protocol — the parser, the matching table and the bytes of the advertisement and of the answer. The deviation from F-01 § 4 is recorded in that document | Accepted | 2026-11-15 | Lead (M1.11, F-01 § 12 item 7) |
| DEC-035 | The daemon serves the paths the device description advertises (`/ContentDirectory.xml`, `/ConnectionManager.xml`, `/ctl/CD`, `/ctl/CM`) and serves the conventional paths as well, because a client may hard-code them | Accepted | 2026-11-15 | Lead (M1.11, F-13 § 12 item 9) |
| DEC-036 | The `serve` command takes a `--port` switch, and the daemon accepts another port for the discovery socket and another path for the identity file, so that a test writes nothing into the configuration directory of the user and competes with no running server | Accepted | 2026-11-15 | Lead (M1.11, F-13 § 12 items 12 and 13) |
| DEC-037 | `FUN-008`, `FUN-009` and `FUN-010` — the catalogue that survives a rescan, the index on disk and its recovery from corruption — move to M2. The catalogue of M1 is clean, in memory and rebuilt at start-up; the index on disk is the work of M2 | Owner direction | 2026-11-16 | Owner (F-09, ADR-0006) |
| DEC-038 | The milestones M1.10 and M1.11 are reviewed together, because M1.11 was implemented before M1.10 and the security baseline must be proved against the code that is already written | Accepted | 2026-11-16 | Lead (M1.10, M1.11) |
| DEC-039 | The catalogue remembers the volume of every root between two reads, and the daemon reads the catalogue again when a drive comes or goes; a volume that comes back under another identity is given new identifiers, a volume that comes back under the same identity keeps them | Accepted | 2026-11-16 | Lead (M2.1, `PLT-011`, F-08 D-15 … D-20) |
| DEC-040 | The fingerprint of a path — what the platform observes at the path right now — is a fact of its own and not the identity of the volume; the portable adapter answers the device of the path, and the real volume identity still arrives with the platform adapter of F-13 | Accepted | 2026-11-16 | Lead (M2.1, `PLT-005`, `PLT-011`, F-08 D-16) |
| DEC-041 | Two roots may not share a name and may not share a path, and a root that lies inside another root is published once, under its own identifier | Accepted | 2026-11-16 | Lead (M2.1, `CFG-002`, `FUN-017`, F-08 D-18, D-20) |
| DEC-042 | The picture pipeline of M2.2 parses **in the server process**, in pure Rust and with bounded reads, until the sandbox child of ADR-0008 exists (M-06). The deviation is written down in ADR-0011 and ends with the child | Accepted | 2026-11-16 | Lead (M2.2, `SEC-066`, `SEC-067`, ADR-0011, F-10 D-01) |
| DEC-043 | A sidecar image next to a media file wins over the cover embedded in its tag, and an embedded cover that is larger than the bound of the pipeline is discarded so that the sidecar is used instead | Accepted | 2026-11-16 | Lead (M2.2, `FUN-013`, F-10 D-02, D-03, F-10 § 12 Q2, Q3) |
| DEC-044 | The picture of a media file is looked for when a client asks for it and not while the catalogue is scanned; the answer is remembered under a key that carries the size and the modification time of the file | Accepted | 2026-11-16 | Lead (M2.2, `PERF-004`, `FUN-014`, F-10 D-04) |
| DEC-045 | The picture of an object is served at `/art/<id>`, where the identifier is the identifier of the catalogue object; the address in the DIDL-Lite document is built from the base address of the server and carries no `dlna:profileID` | Accepted | 2026-11-16 | Lead (M2.2, `INT-067`, `PLT-008`, F-10 D-05) |
| DEC-046 | The pictures of a library are looked for only when `library.extract_artwork` is on, and the disposable cache of the produced pictures lives in `library.cache_directory` or in the data directory of the user | Accepted | 2026-11-16 | Lead (M2.2, `CFG-008`, `FUN-018`, `PERF-004`, F-10 D-07) |
| DEC-047 | A folder of the library carries no picture of its own in the DIDL-Lite document; the album cover of a folder reaches the client through the items of that folder. The document model of the container is F-05's business | Accepted | 2026-11-16 | Lead (M2.2, `INT-067`, F-10 D-08) |
| DEC-048 | A playlist is a **container** of the catalogue, of the class `object.container.playlistContainer`; its children are the media objects it names, in the order the playlist writes them, and they are not sorted | Accepted | 2026-11-16 | Lead (M2.3, `FUN-015`, F-11 D-01) |
| DEC-049 | The entries of a playlist are resolved against the folder of the playlist and must stay **inside the root that holds the playlist**; a network address is refused outright and no request is made for it, and an entry that walks out of the root is refused and reported | Accepted | 2026-11-16 | Lead (M2.3, `FUN-015`, `SEC-068`, `SEC-091`, F-11 D-02) |
| DEC-050 | A playlist entry that names a file that is not there stays in the playlist as a child that carries no resource, so that the user learns where the track went; a **refused** entry is not offered at all and is reported in the diagnostics | Accepted | 2026-11-16 | Lead (M2.3, `UX-004`, `FUN-015`, F-11 D-03) |
| DEC-051 | The containment of a reference lives in `security::containment` and not in the catalogue alone, so that the catalogue, the media layer and the artwork finder judge a path the same way | Accepted | 2026-11-16 | Lead (M2.3, `SEC-001`, `SEC-003`, `INT-098`, F-11 D-04) |
| DEC-052 | A subtitle is offered as an **additional** `res` of the media object, with its own `protocolInfo` that names the MIME type of the file; the capability is **on by default**, because the element is additive and a client that does not understand it still plays the film | Accepted | 2026-11-16 | Lead (M2.3, `FUN-016`, `FUN-026`, `INT-105`, F-11 § 12 Q3) |
| DEC-053 | When several sidecars match one base name, the one with the highest priority extension wins — WebVTT, then SubRip, then ASS, then MicroDVD — and the name that is answered is the name on the disk | Accepted | 2026-11-16 | Lead (M2.3, `FUN-016`, F-11 § 12 Q4) |
| DEC-054 | The **identity key** is scoped by the volume and built from the path of the file, per component, with Unicode NFC and the case fold of the volume; the **uniqueness** of a file comes from the stable file identity the platform gives, not from the path | Accepted | 2026-11-16 | Lead (M2.3, `FUN-020`, F-11 § 12 Q5) |
| DEC-055 | A file that two roots reach is one catalogue entry: the first root wins, the second sighting resolves to the entry that is already there, and the identifier of the object does not change | Accepted | 2026-11-16 | Lead (M2.3, `FUN-022`, F-11 § 12 Q6) |
| DEC-056 | The answer of a subtitle names the character set it is written in; a file that is written in Latin-1 is delivered as it is and is **not** transcoded | Accepted | 2026-11-16 | Lead (M2.3, `FUN-027`, F-11 D-05) |
| DEC-057 | A file whose name shares its identity with another file of the same folder — two names that differ only in the Unicode normalisation form — keeps the identity the platform gives it, so that the two files have two object identifiers and neither of them is lost | Accepted | 2026-11-16 | Lead (M2.3, `FUN-020`, `PLT-005`, F-11 D-06) |
| DEC-058 | The **daemon crate** depends on the administration crate, not the other way round: the surface runs in the process of the daemon, so the crate that owns the process starts the surface. The dependency sits behind the cargo feature `admin`, so that a build without it is a build without the surface | Accepted | 2026-11-16 | Lead (M2.4, M2.5, ADR-0012, `OPS-025`) |
| DEC-059 | The administration surface asks for the **bearer token on every request**, not only on the requests that change something. A request that only reads is harmless without the token, but a surface that asks for one thing is easier to reason about than a surface with two rules | Accepted | 2026-11-16 | Lead (M2.4, `OPS-021`, `SEC-071`) |
| DEC-060 | The "cross-site request forgery" token is **derived from the token of the installation** with a keyed hash, and is handed out in the header of every answer. The session is the token itself, as the open question 4 of F-14 proposes; no cookie and no session store exist | Accepted | 2026-11-16 | Lead (M2.4, `OPS-023`, F-14 § 12 Q4) |
| DEC-061 | The surface answers only to a `Host` header that names the **loopback interface** — `localhost`, `127.0.0.1` and `[::1]`. A configured name of the machine is not accepted: a name is what an attacker can point somewhere else | Accepted | 2026-11-16 | Lead (M2.4, `OPS-022`, `SEC-071`, `SEC-TH-23`) |
| DEC-062 | The token file is written with the permissions `600` inside a folder with the permissions `700`, and a file that others may read **stops the surface** instead of being repaired silently. The user is told which file is the problem and what the permissions should be | Accepted | 2026-11-16 | Lead (M2.4, `OPS-021`, `CFG-014`) |
| DEC-063 | The source revision is embedded into the binary **at build time** by the build script of the core, so that the running server and the source it offers cannot disagree. A build without a repository behind it reports the revision `unknown` and says so | Accepted | 2026-11-16 | Lead (M2.5, `OPS-017`, ADR-0010) |
| DEC-064 | The status and the diagnostics report the memory of the process **only where the platform can be asked**: the number of bytes is `null` on a platform that does not tell. A figure that is made up is worse than a figure that is missing | Accepted | 2026-11-16 | Lead (M2.5, `PERF-012`) |
| DEC-065 | The menu bar application is a **client of the administration surface** and holds no state of the server of its own. It reads the state on every tick and shows the last answer; a surface that is not there is a stopped server and not a cached value | Accepted | 2026-11-16 | Lead (M2.6, `MB-001`, `MB-005`) |
| DEC-066 | The application speaks to the administration surface with a **client of its own** — a small HTTP/1.1 client and a small reader of JSON — and not with a network library of the ecosystem. The crate therefore depends on the core and on the bindings of the system and on nothing else, and the test can answer the client with a recorded answer | Accepted | 2026-11-16 | Lead (M2.6, `MB-005`, `SEC-090`, `SEC-091`) |
| DEC-067 | The requests to the surface run on a **thread of their own** and the main thread of the application never waits for the network. The main thread draws and takes the answers; the thread asks and answers. The channel of the commands holds one command, so that the application never has more than one request in flight | Accepted | 2026-11-16 | Lead (M2.6, `MB-002`) |
| DEC-068 | The **token of the "cross-site request forgery" defence** is read out of the header that the surface hands with every answer and is sent back with the request that changes something. The application does not compute the value itself and needs no hashing library of its own | Accepted | 2026-11-16 | Lead (M2.6, `OPS-023`, `SEC-072`) |
| DEC-069 | The **only file of the workspace that uses `unsafe`** is the bridge to the Objective-C runtime of the menu bar application, and the gate QG-03 knows that file by the record that carries its safety argument (ADR-0013). The exemption is narrow: the marker names the record, and the gate fails when the record does not exist | Accepted | 2026-11-16 | Lead (M2.6, ADR-0013, `PROJECT-STANDARDS` § 2.6) |
| DEC-070 | The **mode of the presentation** is a preference of the application and lives in a file of its own, beside the configuration of the server and not inside it. Switching the mode changes no setting of the server, restarts nothing and interrupts no transfer | Accepted | 2026-11-16 | Lead (M2.6, `UX-015`, `UX-016`, `MB-004`) |
| DEC-071 | The **texts of the user interface** live in one resource file per language — `en-US` and `en-GB` — and in no other place. The list of the languages is the list that F-15 § 12 item 5 proposes; the owner has not decided it yet, and the format carries more languages when the owner does | Accepted | 2026-11-16 | Lead (M2.6, `UX-020`, F-15 § 12 Q5) |
| DEC-072 | The **catalogue index on disk** is a SQLite database of the user, at `library.database_path` or in the data directory of the user. It holds the objects, the containers, the membership of the containers, the roots and the identifier of the catalogue; the whole catalogue is written in **one transaction**, so a browse during a rescan sees the catalogue of before or the catalogue of now. The index is a **cache**: it may be deleted at any time and the library is then read again | Accepted | 2026-11-17 | Lead (M2.9, `FUN-008`, `FUN-009`, ADR-0006, F-09 D-12, D-13, D-16) |
| DEC-073 | A file of the index that is **damaged** — bytes that are not a database, a database that fails the integrity check, or a database of another schema — is moved aside under the name `.corrupt.<moment>`, a new index takes its place, the library is read again, and the daemon says so in the log **and** in the status of the server. The damaged file is kept: the user may want to look at it | Accepted | 2026-11-17 | Lead (M2.9, `FUN-010`, F-09 D-15) |
| DEC-074 | The index is written with **SQLite compiled into the binary** (`rusqlite` with the feature `bundled`, without `loadable_extension`), so that the server does not depend on the SQLite of the system and cannot load an extension into its own database. The price is a **C compiler at the time of the build**, which is in tension with `PLT-009` and with the description of the workspace. The dependency review of § 1 rule 6 makes this an **owner decision**; the decision was taken for SQLite in ADR-0006 and is recorded here so that the owner may overrule it | Owner direction (recorded) | 2026-11-17 | Owner (M2.9, `PLT-009`, `SEC-095`, ADR-0006) |
| DEC-075 | The native macOS application is the primary user-facing administration and setup surface. The loopback administration service remains an authenticated local API for the application and automation; it serves no browser UI. The command line remains available for headless operation and advanced use | Owner direction | 2026-11-18 | Owner (M2.6, OD-08, F-14, F-15) |
| DEC-076 | PhairDLNA for macOS is a regular application with a main window, Dock presence and a menu-bar status item. The menu bar is a quick access point; the main window is the primary setup and everyday administration surface. The bundle must not set `LSUIElement` | Owner direction | 2026-11-18 | Owner (M2.6, DEC-075, F-15, Apple AppKit activation policy) |
| DEC-077 | The macOS app shall provide an app-owned text-size preference from 100% through 200%, saved with app preferences and applied across the main overview and menus. Layouts shall reflow without clipping, overlap or hidden actions at supported sizes. This replaces the unsupported Dynamic Type interpretation while preserving the larger-text accessibility commitment | Owner direction | 2026-11-18 | Owner (M2.6, OD-17, `UX-008`, `UX-017`, `MB-003`, F-15) |
| DEC-078 | Preserve ADR-0002 and DEC-019: integrate `quick-xml` at the bounded XML parser boundary and `tower-http` for tracing and request-handler timeouts, retaining project-owned framing/entity policy and the independent per-read/per-write socket deadlines | Owner direction | 2026-11-19 | Owner (OD-18, M-02, `SEC-010`, `SEC-016`) |
| DEC-079 | Use app-managed `SMAppService` registration for the bundled macOS service while retaining CLI lifecycle support; evaluate a per-user `LaunchAgent` as the preferred OD-15 candidate, but do not amend PLT-013 or ship that model until the selected-folder, sibling-denial, revocation and Local Network experiment passes independent security review; bootstrap the first configuration locally in the app, then use authenticated, validated, atomic API updates followed by a controlled service restart | Owner direction | 2026-11-19 | Owner (OD-14, OD-15, OD-16, `OPS-015`, `PLT-013`, `PLT-018`, `UX-002`, ADR-0004) |

## Reviews and evidence

| Review | Date | Result |
| --- | --- | --- |
| Independent beginner-experience review | 2026-11-14 | 6 blocker, 16 major, 11 minor, 4 info findings — **all addressed**; the report is kept at `../05-quality/reviews/BEGINNER-REVIEW-2026-11-14.md` |
| Independent consistency review (round 2) | 2026-11-15 | 2 blocker, 6 major, 15 minor, 4 info findings — **all addressed**; the report is kept at `../05-quality/reviews/CONSISTENCY-REVIEW-2026-11-15.md` |
| Independent review of M1.9 (the catalogue over one root) | 2026-11-15 | 3 blocker, 7 major, 11 minor, 3 info findings — the blockers and the majors are addressed by the milestone: the update identifier is monotonic and content-sensitive, the hard links are one object, the link policy is per root, the catalogue holds the target of a link, the diagnostics name no path, the root container has the parent `-1`, a browse answers for an item, and the tables of F-08 and F-09 name every criterion with its result |

## Open decisions requiring owner approval

These are collected in `../07-project/OPEN-DECISIONS.md` with options and consequences.
Summary: **OD-01** macOS distribution/signing model · **OD-02** transcoding dependency
policy (FFmpeg) · **OD-03** macOS UI technology (pure Rust vs. native Swift helper) ·
**OD-04** admin authentication model · **OD-06** policy on shipping a prebuilt binary vs.
source-only distribution. **OD-05** (the minimum supported macOS version) was decided on
2026-11-15: macOS 15 and later (DEC-032).
