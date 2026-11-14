# Interoperability matrix — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: DLNA / UPnP Interoperability Specialist
Requirements: `INT-100` … `INT-104`, `INT-001` … `INT-099`, `UX-012`
Companion: `QUALITY-GATES.md` (QG-23), `../04-security/SECURITY-MODEL.md`

---

## 1. What this matrix is

This is the project's **evidence of compatibility**. Every row is a test that was
actually performed on a device, with the device identified precisely enough that
another person could repeat it.

Three rules:

1. **A row without a device is not a row.** An untested device appears in §5 as
   *planned*, never as *working*.
2. **Failures are published.** A device that refused to play a file is more valuable
   in this table than a device that worked — it is the reason the next version exists.
3. **No certification claim.** The DLNA certification programme is proprietary and the
   organisation behind it no longer operates the same way. This project states
   *measured compatibility*, never *certified*.

---

## 2. Protocol conformance checklist

These checks are automated and apply to every client. They are the floor, not the
ceiling.

| ID | Check | Requirement | Method |
| --- | --- | --- | --- |
| PC-01 | `M-SEARCH` for `upnp:rootdevice` is answered within 1 s with the required headers | `INT-001` | Automated fixture |
| PC-02 | `M-SEARCH` without `MX` is ignored | `INT-008` | Automated fixture |
| PC-03 | The reply is unicast to the requester | `INT-003` | Automated fixture |
| PC-04 | `USN` has the required forms for root device, device type and both services | `INT-005` | Automated fixture |
| PC-05 | `LOCATION` is reachable on the requesting interface | `INT-006` | Manual, two-interface host |
| PC-06 | The device description is valid XML with the required elements | `INT-020`, `INT-044` | Automated fixture |
| PC-07 | Every advertised service has a valid SCPD | `INT-023` | Automated fixture |
| PC-08 | `Browse` returns `Result`, `NumberReturned`, `TotalMatches`, `UpdateID` | `INT-031` | Automated fixture |
| PC-09 | `BrowseMetadata` returns `NumberReturned = TotalMatches = 1` | `INT-033` | Automated fixture |
| PC-10 | An unknown `ObjectID` returns error 701 | `INT-039` | Automated fixture |
| PC-11 | `GetSearchCapabilities` and `GetSortCapabilities` return only implemented capabilities | `INT-035`, `INT-049` | Automated fixture |
| PC-12 | No non-standard ContentDirectory action is advertised | `INT-048` | Automated fixture |
| PC-13 | `GetProtocolInfo` lists only MIME types the server can serve | `INT-050`, `INT-051` | Automated fixture |
| PC-14 | The DIDL-Lite document validates against the vendored XSD | `INT-060` … `INT-079` | Automated (`cargo xtask validate-didl`) |
| PC-15 | The `Result` element carries the DIDL-Lite document escaped a second time | `INT-073` | Automated fixture |
| PC-16 | `GET` without a range returns 200 with `Content-Length` | `INT-080` | Automated fixture |
| PC-17 | A single byte range returns 206 with a correct `Content-Range` | `INT-081` | Automated fixture |
| PC-18 | An unsatisfiable range returns 416 with `Content-Range: bytes */size` | `INT-082` | Automated fixture |
| PC-19 | A malformed range returns 416 and never a partial body | `INT-083` | Automated fixture |
| PC-20 | `HEAD` returns the same headers as `GET` and no body | `INT-086` | Automated fixture |
| PC-21 | Media responses carry `Accept-Ranges: bytes` and never chunked encoding | `INT-084`, `INT-088` | Automated fixture |
| PC-22 | The `Content-Type` equals the MIME type in `res@protocolInfo` | `INT-064`, `INT-087` | Automated fixture |
| PC-23 | A traversal attempt in the media URL is rejected | `INT-098`, `SEC-001` | Negative test |
| PC-24 | An unknown SOAP action returns a UPnP fault, not a crash | `INT-025` | Negative test |
| PC-25 | A malformed SOAP body is rejected without a panic | `SEC-010` | Negative test |
| PC-26 | A DIDL-Lite document with a missing required attribute fails XSD validation | `INT-060`, `INT-066` | Negative test |

The SSDP and SOAP **fixture corpus** must be captured by the project itself: the only
public SSDP packet capture found contains no `M-SEARCH` requests at all. The corpus
therefore starts empty and grows with every real client observed.

---

## 3. Control points used for automated testing

| Tool | Role | Licence note |
| --- | --- | --- |
| `gupnp-tools` (`gssdp`, `gupnp-*`) | Independent control point: sends real `M-SEARCH` requests and browses our catalogue | **GPL-2.0** — used as a *test tool only*, never linked into or shipped with PhairDLNA; recorded here so the licence boundary is explicit |
| `libupnp` / Platinum | Second opinion on SOAP handling | GPL/LGPL — test tool only, same rule |
| `curl` with hand-written SOAP bodies | Raw protocol probing | — |

Because these tools are copyleft, they stay **outside** the repository and outside the
release artefacts. PhairDLNA itself remains AGPL-3.0-or-later and links no GPL code.

---

## 4. Device matrix

**Status legend:** `WORKS` · `WORKS (limitations)` · `FAILS` · `PARTIAL` ·
`NOT TESTED` · `NOT AVAILABLE`

Every row must carry: device model, firmware version, client application and version,
the media format used, the observed outcome, the date and the person who ran it.

### 4.1 Priority client families

The first real-device round targets the families with the largest installed base and
the most reported incompatibilities. Ordered by expected value:

| # | Family | Why first | Formats to test |
| --- | --- | --- | --- |
| 1 | Samsung Smart TV (Tizen and older) | Most reported DLNA quirks in the field | MP4/H.264, MKV, MP3, JPEG, subtitle sidecar |
| 2 | LG / webOS TV | Common, strict about DIDL details | MP4/H.264, MP3, JPEG |
| 3 | Sony Bravia | Known to be strict about `upnp:class` and dates | MP4/H.264, MKV |
| 4 | VLC (desktop and mobile) | Reference client, easy to automate | all supported formats |
| 5 | Kodi | Rich client, exposes protocol details in its log | all supported formats |
| 6 | Windows "Media Player"/Windows Media Center | Historic reference client, strict about DLNA fields | MP4, MP3, JPEG |
| 7 | Android TV / Google TV | Growing, uses a common UPnP stack | MP4/H.264, MP3 |
| 8 | Apple TV (with a DLNA client app) | macOS-first audience | MP4/H.264, MP3 |
| 9 | Xbox, PlayStation | Console renderers, own quirks | MP4/H.264 |
| 10 | Philips / Panasonic / Hisense TVs | Breadth of the installed base | MP4/H.264, MP3, JPEG |

### 4.2 Rows

*The table is empty on purpose: no device test has been performed yet.*

| Device | Firmware | Client | Format | Outcome | Notes | Date | By |
| --- | --- | --- | --- | --- | --- | --- | --- |
| — | — | — | — | `NOT TESTED` | No device test has been performed yet | — | — |

---

## 5. Known client quirks

A quirk is a documented deviation of a client from the specification that the server
must work around. Every entry needs: the client family, the symptom, the workaround,
and the **evidence** that justified it (issue link, capture or test).

**No quirk is recorded yet**, because no device test has been performed. When the
first device round happens, the table becomes the project's most valuable asset — it is
what separates a compatible server from a merely correct one.

| ID | Client family | Symptom | Workaround | Evidence | Since |
| --- | --- | --- | --- | --- | --- |
| — | — | — | — | — | — |

Rules for this table:

- A workaround must never violate a normative requirement for *other* clients
  (`INT-101`).
- Every workaround must be switchable off by the user without touching source code
  (`INT-104`).
- A quirk that is only a guess does not enter this table. It goes into the feature
  specification as an open question.

---

## 6. Reporting protocol for a device test

1. Record the device model **exactly** as the device reports it, plus the firmware
   version.
2. Record the client application and version — the same device with two different
   DLNA apps can behave differently.
3. Use a **controlled media set**: one file per format, known-good, small enough to
   repeat quickly, plus at least one file with a subtitle sidecar.
4. Perform at least four operations: discovery, browse one level deep, start
   playback, seek to the middle.
5. Record the outcome per operation, not just overall.
6. Capture the server's log for the session — a failure without the server-side view
   is half an observation.
7. Store any packet capture that helped; it becomes a regression fixture.

---

## 7. How the matrix grows

- Every device test appends a row (§4) and, when it reveals a deviation, a quirk
  entry (§5).
- Every quirk that needs a workaround produces a **device profile** — a declarative
  entry in the compatibility profile store, keyed by the client's `User-Agent` or
  address, with the workaround and the evidence.
- Device profiles are data, not code, so that a user can disable one without
  recompiling and so that a new quirk does not require a release.
