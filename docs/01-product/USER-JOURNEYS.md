# User journeys — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Product Owner
Traceability: each journey maps to the requirements it exercises and the feature
specifications that deliver them.

A *journey* is the path a person takes, including the moments where they are
confused, blocked or disappointed. It is written so that an implementation can be
tested against it: each step says what the person does, what the product does, and
what would make them give up.

---

## J1 — First run: "I want to watch my films on the TV"

**Persona:** first-time user, technical but not an expert. Just installed the server.

| Step | The person | The product | Requirement |
| --- | --- | --- | --- |
| J1.1 | Installs the server | The installation completes without a compiler, without an account and without a network connection to a vendor | `PRD-003` |
| J1.2 | Starts it | The server starts, binds the media port on the local network and logs what it did | `OPS-001` |
| J1.3 | Is asked for a folder | The prompt says "choose a folder with your media" and explains that the folder is only ever read | `UX-002`, `PRD-007` |
| J1.4 | Selects the folder | The server accepts it, creates the index and starts scanning | `FUN-001`, `FUN-009` |
| J1.5 | Waits | The output shows progress ("scanning, 1 240 files") and does not look frozen | `UX-013` |
| J1.6 | Reads the summary | The output names the device, the address it is reachable at, and how to check it from the television | `UX-003` |
| J1.7 | Opens the DLNA app on the TV | The server appears in the device list within a few seconds | `INT-001` |
| J1.8 | Browses | The folder structure appears with readable titles | `INT-030`, `INT-066` |
| J1.9 | Plays a film | The film starts without a long buffering pause | `FUN-040`, `INT-081` |
| J1.10 | Seeks to the middle | The film continues from the requested position | `INT-081` |

**Failure paths that must be handled gracefully:**

- The selected folder is inside "Desktop" or "Documents" and macOS shows a privacy
  prompt → the server explains which folder is protected and how to grant access,
  and keeps serving the other roots (`PLT-017`, `UX-004`).
- The folder is on an external drive that is not attached → the server starts
  normally, reports the missing volume in its status and recovers when the drive
  appears (`FUN-007`, `PLT-011`).
- The television does not show up in the device list → the status output names the
  interface it is advertising on and how to check discovery from another device
  (`OPS-004`, `UX-003`).

**Definition of "the journey works":** a person who has never used a DLNA server
completes J1 without reading documentation beyond the first-run output, and without
filing a question.

---

## J2 — Daily use: "the TV asks for something that is not there"

**Persona:** household member using the television.

| Step | The person | The product |
| --- | --- | --- |
| J2.1 | Browses to a folder | Only files that exist are listed |
| J2.2 | Selects a file that was deleted yesterday | The entry is gone (the index followed the file system) |
| J2.3 | Tries to play a file on a disconnected drive | The entry is visible, the playback fails with a message naming the missing volume |
| J2.4 | Sees an item whose title is empty | The title falls back to the file name, never blank |

**Requirements:** `FUN-006`, `FUN-007`, `INT-066`, `UX-004`.

---

## J3 — Growing the library: "I added a new folder"

**Persona:** collector with several sources.

| Step | The person | The product | Requirement |
| --- | --- | --- | --- |
| J3.1 | Adds a second root | The catalogue becomes the union of all roots | `FUN-002` |
| J3.2 | Points a root at a NAS share | It is indexed like a local folder | `FUN-017` |
| J3.3 | Has two roots on the same volume | No duplicate entries appear | `FUN-022` |
| J3.4 | Has a folder with a symbolic link into another folder | The behaviour follows the configured link policy and is documented | `SEC-006`, `FUN-003` |
| J3.5 | Waits for the initial scan of 100 000 files | The scan is incremental afterwards; a later rescan is fast | `PERF-003`, `PERF-004` |

---

## J4 — Troubleshooting: "it did not play"

**Persona:** the person in the household who is asked to fix it.

| Step | The person | The product | Requirement |
| --- | --- | --- | --- |
| J4.1 | Opens the status | The connected client and the current transfer are shown | `UX-012`, `OPS-002` |
| J4.2 | Reads the log | The entry says what failed, why and what to do | `UX-004`, `OPS-006` |
| J4.3 | Copies the diagnostics | The report is anonymised on request | `UX-011`, `SEC-094` |
| J4.4 | Hides the paths | Redaction replaces paths with stable pseudonyms | `SEC-094` |
| J4.5 | Asks "which format does my TV want?" | The server prints the protocolInfo it advertises and the formats it found | `OPS-005` |

---

## J5 — Configuration: "I want it to start with my computer"

**Persona:** tinkerer.

| Step | The person | The product | Requirement |
| --- | --- | --- | --- |
| J5.1 | Edits the configuration | The file is commented, versioned and validated | `CFG-001`, `CFG-003` |
| J5.2 | Makes a typo | The server refuses to start and names the field, the file and the accepted values | `CFG-002`, `CFG-013` |
| J5.3 | Validates without starting | A dedicated command checks the file | `CFG-011` |
| J5.4 | Moves to another machine | Export and import preserve the schema version | `CFG-012` |
| J5.5 | Wants to limit who can stream | An access-control list per address | `CFG-009` |

---

## J6 — Long-term: "it has been running for a week"

**Persona:** the person who installed it and forgot about it.

| Step | The person | The product |
| --- | --- | --- |
| J6.1 | Restarts the machine | The server comes back automatically (service integration) |
| J6.2 | Checks after a week | Memory, descriptors and thread count are flat |
| J6.3 | Updates the software | The update comes from the package manager, not from the server itself |
| J6.4 | Checks the licence promise | The server itself offers the source of the running build |

**Requirements:** `OPS-009`, `PLT-013`, `PERF-001`, `INT-059`, `OPS-016`.

---

## J7 — The newcomer contributor: "I want to change something"

**Persona:** a programming beginner who wants to contribute.

| Step | The person | The product |
| --- | --- | --- |
| J7.1 | Reads the README | It links to the getting-started guide, which links to `WHERE_TO_CHANGE.md` |
| J7.2 | Finds the file to change | `WHERE_TO_CHANGE.md` names the exact path and its owner |
| J7.3 | Reads the feature specification | It explains the behaviour and the acceptance criteria |
| J7.4 | Runs the gates | The documented commands work as written |
| J7.5 | Adds a feature | The template tells them which ten sections to write |
| J7.6 | Submits | The pull-request template asks for tests, evidence, security impact and documentation |

**Requirements:** `DOC-001` … `DOC-012`, `UX-006`.

---

## Journey-to-requirement coverage

| Journey | Primary requirements | Delivering features |
| --- | --- | --- |
| J1 First run | `PRD-003`, `PRD-007`, `UX-002`, `UX-003`, `INT-001`, `INT-030`, `INT-081` | F-01, F-03, F-06, F-08, F-12, F-13, F-15 |
| J2 Daily use | `FUN-006`, `FUN-007`, `INT-066`, `UX-004` | F-08, F-09, F-14 |
| J3 Growing the library | `FUN-002`, `FUN-017`, `FUN-022`, `SEC-006`, `PERF-003` | F-08, F-09, F-11 |
| J4 Troubleshooting | `UX-011`, `UX-012`, `OPS-005`, `OPS-006`, `SEC-094` | F-13, F-14 |
| J5 Configuration | `CFG-001` … `CFG-013` | F-12 |
| J6 Long-term | `OPS-009`, `PLT-013`, `INT-059`, `OPS-016` | F-13, F-15, F-16 |
| J7 Contributing | `DOC-001` … `DOC-012`, `UX-006` | F-19 |
