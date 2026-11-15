# Privacy model — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Security & Privacy Engineer
Requirements: `PRD-002`, `PRD-003`, `PRD-004`, `PRD-013`, `SEC-090` … `SEC-096`
Companion: `SECURITY-MODEL.md`

---

## 1. The promise

PhairDLNA is **privacy-first**. Concretely, that means:

> The server talks to the devices on your local network and to nothing else. It has no
> account, no telemetry, no analytics, no advertising, no crash reporting and no cloud
> dependency — and it never will.

This document states what the product does with data, so that the promise is auditable
rather than aspirational.

---

## 2. The data flow

```
        your media ──▶ the server ──▶ your television
                         │
                         ├──▶ the catalogue index   (local file, in your data directory)
                         ├──▶ the configuration     (local file, in your data directory)
                         └──▶ logs                  (local file or standard error)
```

There is **no third arrow pointing out of the box.** The only network traffic the
server generates without being asked is:

- SSDP advertisements and discovery replies on the local network,
- the media transfers it serves,
- the reply to the client that asked it something.

Everything else is opt-in and documented.

---

## 3. What is stored, and where

**Target state.** No server exists yet; the table below is the inventory the
implementation must satisfy, not a description of running software. The last column
names the feature that delivers it.

| Data | Where | Lifetime | Delivered by |
| --- | --- | --- | --- |
| Catalogue index | SQLite file in the user's data directory | Until deleted | F-09 | Contains file paths and media metadata. Not encrypted — it is a local file with user-only permissions |
| Configuration | Configuration file chosen by the user | Until deleted | F-12 | May contain folder paths and access-control lists |
| Device identity (UDN) | In the configuration and the index | Until the identity is reset | F-02 | A random UUID, not derived from hardware |
| Logs | Standard error by default, or a file the user names | Until deleted by the user | F-13 | Redactable |
| Administration token | A file with owner-only permissions (`0600`) in a `0700` directory | Regenerated on request | F-14 | Never in the configuration file, never in a log line, never in a command-line argument |
| Thumbnails and generated artwork | Disposable cache directory | Evicted automatically when the size limit is reached | F-10 | Regenerable at any time |
| The corresponding source of the running build | Served over HTTP, not stored | — | F-14, F-16 | Required by the AGPL network-use clause |

Nothing else is written. There is no analytics database, no usage profile, no crash
dump uploaded anywhere, no "first run" identifier sent to a vendor.

---

## 4. What leaves the machine

| Connection | When | Destination |
| --- | --- | --- |
| SSDP multicast | While the server runs | `239.255.255.250:1900` on the local network |
| SSDP unicast reply | When a client searches | The address of the searching client |
| HTTP media transfer | When a client streams | The address of the streaming client |
| HTTP SOAP and description | When a client browses | The address of the browsing client |
| **Nothing else** | — | — |

Specifically, the server does **not**:

- resolve or contact any domain name,
- send a crash report, an update check, a "phone home" heartbeat or a usage ping,
- embed a third-party analytics or advertising identifier,
- read anything from the internet (including artwork or metadata scraping) unless the
  user explicitly enables a feature that does so,
- attempt to configure the router or speak the UPnP Internet Gateway Device protocol.

**How this will be verified:** the release checklist includes an outbound-connection
audit — the server is started with all network egress blocked except the local
network, and the test suite asserts that a normal browse-and-play session produces
**zero** outbound connections. That audit has not been performed, because there is no
server to audit. Until it has been, this section states a requirement and a planned
verification, not an observation.

---

## 5. Data minimisation

The catalogue is the most sensitive artefact, because it maps what the user owns and
what they watch. The design follows three rules:

1. **Only what a client needs.** The index stores the fields a browse response
   actually uses: identity, path, size, modification time, media type, and the
   metadata that is displayed. It does not store a hash of the file content, a
   "play count", a "last played" timestamp, or anything about *when* the user watches
   something.
2. **No behavioural profile.** There is no history, no recommendation engine, no
   "recently played", no "because you watched X". The catalogue describes the files,
   not the person.
3. **Redaction on request.** Logs and diagnostics can redact file paths, client
   addresses and device identifiers, replacing them with stable pseudonyms, so a user
   can share a log with the project without exposing their library layout.

---

## 6. Network exposure

| Question | Answer |
| --- | --- |
| Does the server bind a non-loopback port? | Yes — the media port, on the local network, so that televisions can reach it. This is the product's purpose. |
| Does it listen on the internet? | No. It binds only the addresses of the local interfaces, and the installation instructions require the host's firewall to allow it only on the private network. |
| Does it open a hole in the router? | Never. No port forwarding, no UPnP IGD, no tunnel, no relay. |
| Does it discover the internet address? | No. It does not know or ask. |
| Is the administration surface reachable from the network? | No. It binds to the loopback interface only, and refuses to start otherwise without an explicit override. |
| Can a web page in the user's browser reach it? | Not by accident: the `Host` header is checked against an allowlist, state-changing requests need a CSRF token and a custom header, and no CORS headers are sent. |

---

## 7. AGPL and the source offer

The AGPL's network-use clause (§13) obliges a modified version of the program to
offer its corresponding source to the users who interact with it over a network. A
LAN is a network, so the obligation applies.

PhairDLNA therefore ships the offer **as a product feature**: the running server
serves the corresponding source of the exact build — including the dependency tree
and the build recipe — over HTTP, without authentication, at no charge
(`OPS-016`, `OPS-017`, `SEC-070`). This is a requirement of the baseline, not a
description of existing behaviour; the route arrives with F-14.

This is not an obligation to be minimised; it is the licence's central promise, and
implementing it as a first-class route is the honest way to comply.

---

## 8. Third-party components

Every dependency is recorded with its licence, purpose and maintenance status in
`DEPENDENCY-LICENSE-REVIEW.md`. Two rules protect the privacy promise:

1. **No dependency that phones home.** A dependency that opens a network connection on
   its own is prohibited. (This is why no telemetry crate, no update checker and no
   crash reporter may ever be added.)
2. **No advertising or tracking identifier**, ever — not in a dependency, not in a
   build script, not in the macOS application.

---

## 9. The macOS application

The optional native front end adds no data flow of its own:

- It stores its settings in the user's own preferences directory.
- It talks to the local server over the loopback interface.
- It requests no macOS permission beyond what the server itself needs
  (the local-network permission, and access to the folders the user chose).
- It requests **Full Disk Access** only if the user explicitly asks for access to
  protected locations, and never works around the user refusing it.

---

## 10. Checklist before a release

- [ ] Outbound-connection audit executed; zero unexpected connections observed.
- [ ] No telemetry, analytics or crash-reporting code anywhere in the tree or the
      dependency graph.
- [ ] Every stored artefact is listed in §3 with its location and lifetime.
- [ ] Redaction verified: a log with redaction enabled contains no absolute path.
- [ ] The source offer route works and serves the exact running revision.
- [ ] The administration surface is loopback-only in the shipped configuration.
- [ ] No router configuration, no IGD, no tunnel.
- [ ] The privacy section of the documentation matches the observed behaviour.

---

## 11. Reporting a privacy problem

If you believe the product collects, transmits or exposes data it should not, report
it privately as described in `../../SECURITY.md`. Privacy reports are treated as
security reports and are never filed as public issues.
