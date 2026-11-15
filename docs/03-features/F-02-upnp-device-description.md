# F-02 — UPnP device and service description

Status: Draft
Owner role: Product/Requirements
Version/date: 0.1 / 2026-11-14
Requirements: INT-020, INT-021, INT-022, INT-023, INT-024, INT-025, INT-026,
Research sources: `docs/09-research/` - a citation such as `standards §8.3 C12` points at that file and section.
INT-027, INT-028, INT-029, INT-044, INT-045

## 1. Purpose and user story

The **device description** is the XML document in which a UPnP device says what it is and what it can do. A control point that has found the server over SSDP
(F-01) fetches that document from the `LOCATION` URL and reads three things from it: *which device type* this is (we are a `MediaServer:1`), *which services* it
offers, and *where to send control requests* (`controlURL`).

Each service has its own second document, the **SCPD** (Service Control Protocol Description), which lists the actions a client may call, the arguments of each
action and the state variables the service exposes. A client reads the SCPD before it calls anything, so an SCPD that lies about the service produces mysterious
failures.

**User story.** As a user, I want the server to describe itself completely and honestly, so that any television, phone or speaker app can find out what it can
do without guessing, and so that a failure points at a real cause instead of at a half-empty document.

## 2. Scope

**In scope** — the device description document: its URL, its XML shape, its identity elements, its service list and the rules for the URLs inside it; the SCPD
document of every advertised service (action list, argument list, state variables); the HTTP behaviour of both documents (status, `Content-Type`,
`Content-Length`); the SOAP dispatcher entry point and the fault codes for an unknown action or bad arguments.

**Explicitly out of scope** — the ContentDirectory actions themselves (F-03) and the ConnectionManager actions (F-04): this feature describes them, it does not
implement them. The DIDL-Lite metadata inside a browse answer (F-05). GENA event delivery: `eventSubURL` is advertised and non-empty, but the subscription
endpoint is a separate feature (F-04 §12, open question 3). `AVTransport:1` service instances and per-connection state (F-06). The `RenderingControl` service —
it is a **renderer-side** service and is not part of a media server's surface (standards §1.1, §0.3).

## 3. Functional requirements

**INT-020** — The server shall serve an XML device description at a stable URL path that contains a root device with `deviceType`
`urn:schemas-upnp-org:device:MediaServer:1`, a `UDN`, `friendlyName`, `manufacturer`, `modelName`, `modelNumber` and `modelURL`. [SPEC] `MediaServer:1` is the
device type of a DLNA media server (standards §1.1, §8.2 P9) and the element set follows the MediaServer description template (standards §3.1). [DESIGN] The
path is `/description.xml` on the media port.

**INT-021** — The device description shall list the `ContentDirectory:1` and `ConnectionManager:1` services, each with `serviceType`, `serviceId`, `SCPDURL`,
`controlURL` and a non-empty `eventSubURL`; `AVTransport:1` may be listed as an optional service. [SPEC] A `MediaServer:1` requires exactly a ContentDirectory
and a ConnectionManager service (standards §1.1). [SPEC] `controlURL` and `eventSubURL` are required and `URLBase` is forbidden in the Windows hosted-device
model (standards §3.1). [DESIGN] `eventSubURL` is always non-empty even when eventing is disabled, because a client that finds an empty value may treat the
service as incomplete.

**INT-022** — All URLs inside the device description shall be absolute HTTP URLs or root-relative paths that resolve correctly from the request host. [SPEC]
Both forms are used in practice; relative URLs resolve against the description URL (standards §3.2, §8.2 P7). [DESIGN] We emit absolute URLs built from the
interface-local address that served the description — the same host the `LOCATION` header used (INT-006), so the client never has to guess an address.

**INT-023** — The server shall serve a valid SCPD document for every advertised service, describing every action with its arguments and every state variable
with its data type and allowed value range. [SPEC] Control points parse the SCPD before they call anything (standards §8.2 P8).

**INT-024** — The XML served by the server shall be well-formed, shall declare its encoding, and shall not contain a DOCTYPE declaration. [DESIGN] The
declaration is `<?xml version="1.0" encoding="utf-8"?>`. A DOCTYPE is forbidden because a document type declaration is the entry point for entity-expansion
attacks (SEC-010).

**INT-025** — The server shall answer an unknown SOAP action with an HTTP fault using the UPnP error codes `401 Invalid Action` or `402 Invalid Args` as
appropriate, and shall not crash. [DESIGN] An action name that is not in the SCPD of the addressed service is `401`; a known action with a missing or malformed
argument is `402`. The fault is a SOAP fault carrying the UPnP error code.

**INT-026** — The server shall respond to `GET` on the device description and on SCPD documents with `Content-Type: text/xml; charset="utf-8"` and a correct
`Content-Length`. [SPEC] (standards §8.2 P1, and the observed content type of shipped servers).

**INT-027** — The server shall not include a client-controllable value in the XML output without XML escaping. [SPEC] XML 1.0 requires `&`, `<` and `>` to be
escaped in element text and `&`, `<`, `>`, `"` and `'` in attribute values (standards §4.6). [DESIGN] The values that reach the description from a client or a
file system are the device name, the model strings and the icon URL; all of them are escaped by the one XML writer that F-05 also uses.

**INT-028** — The server shall advertise `ContentDirectory:1` in addition to any newer ContentDirectory version, so that legacy clients can be served. [SPEC]
Every shipped media server advertises `ContentDirectory:1` (standards §7.2), and a `MediaServer:2` implementation must answer the actions of `MediaServer:1`
(standards §1.1). [DESIGN] The baseline advertises `ContentDirectory:1` only; no newer version is advertised until its extra actions are implemented.

**INT-029** — The device description shall reference the `ConnectionManager` service even when no transcoding is available, because clients use it to learn the
transfer protocols. [SPEC] The service is mandatory on a media server regardless of the transfer protocols it supports (standards §1.1).

**INT-044** — The device description shall use the service identifiers `urn:upnp-org:serviceId:ContentDirectory` and `urn:upnp-org:serviceId:ConnectionManager`,
and shall declare `specVersion` 1.0. [SPEC] These are the spellings every shipped implementation uses (standards §3.1, §8.2 P5); the OCF MediaServer:1 template
renders the ConnectionManager identifier as `ConnectorManager`, an evident template typo, while the Annex and every implementation use `ConnectionManager`.
[OBSERVED] Every real device description declares `<major>1</major><minor>0</minor>` (standards §3.2).

**INT-045** — The device description shall contain exactly one `device` element and shall not contain a `URLBase` element. [SPEC] `URLBase` is forbidden in the
Windows hosted-device model (standards §3.1); the UDA element is optional and supplies a base for relative URLs (standards §3.2). [DESIGN] Because we emit
absolute URLs, no base is needed; the element is omitted entirely.

## 4. Technical requirements

- **Document shape.** The description follows the MediaServer template (standards §3.1): `<root xmlns="urn:schemas-upnp-org:device-1-0">` → `<specVersion>` →
  `<device>` → identity elements → `<iconList>` (optional) → `<serviceList>` → `<presentationURL>` (optional). [DESIGN] The `iconList` is omitted in the
  baseline: there are no icon assets, and an icon entry pointing at a missing file is worse than none.

- **Identity elements.** `deviceType`, `friendlyName`, `manufacturer`, `manufacturerURL`, `modelDescription`, `modelName`, `modelNumber`, `modelURL`,
  `serialNumber`, `UDN` — the `UDN` has the form `uuid:<UUID>` (standards §3.1). [DESIGN] `manufacturer` and `modelName` are fixed product strings,
  `friendlyName` is the user-configured device name, and `modelNumber` and `serialNumber` identify the build.

- **Service entries.** Each entry carries `serviceType`, `serviceId`, `SCPDURL`, `controlURL` and `eventSubURL` (INT-021). The service types are
  `urn:schemas-upnp-org:service:ContentDirectory:1` and `urn:schemas-upnp-org:service:ConnectionManager:1` (standards §1.1). [DESIGN] `AVTransport:1` is not
  listed in the baseline: it is optional and only meaningful when the server exposes per-connection instances (standards §1.1).

- **URLs.** Absolute URLs under the same origin as the description (INT-022): `/ContentDirectory.xml` and `/ConnectionManager.xml` for the SCPDs, `/ctl/CD` and
  `/ctl/CM` for the control endpoints, `/evt/CD` and `/evt/CM` for the event endpoints. [DESIGN] The two services share one control path with a distinguishing
  prefix, so one SOAP dispatcher serves both (ARCHITECTURE §4.3).

- **SCPD content.** Each SCPD lists the actions of its service with every argument and its direction (`in` / `out`), and the state variables with their data
  type and, where the type has a fixed value set, the allowed values (INT-023). [DESIGN] The ContentDirectory SCPD lists the four required actions plus `Search`
  (F-03); the ConnectionManager SCPD lists the actions of F-04. [DESIGN] The action list of a service is generated from the same table the dispatcher uses, so
  the two cannot drift apart.

- **Generation.** Both documents are generated in memory from the configuration and the device identity; nothing is read from disk per request. [DESIGN] The
  XML writer is the single writer of the project (ARCHITECTURE §4.4): it escapes element text and attribute values in one place (INT-027) and never emits a
  DOCTYPE (INT-024).

- **SOAP dispatcher.** The dispatcher selects the service from the request path and then the action from the `SOAPAction` header. An unknown service or action
  yields the UPnP fault `401`; a malformed argument yields `402` (INT-025). The dispatcher never panics (ARCHITECTURE §4.3).

- **Module boundary.** The device model, the service list, the SCPD generation and the SOAP dispatcher live in `phairdlna-core/src/upnp/` (ARCHITECTURE §4.3).
  The HTTP layer routes `/description.xml`, the SCPD paths and the control paths to it.

## 5. Performance requirements

All figures below are **targets** with a measurement method; none is a measured value.

- [DESIGN] Target: the device description and each SCPD are generated once per configuration change and served from memory, so a `GET` performs no XML
  generation and no catalogue access. Method: code review plus an automated test that asserts no catalogue handle is opened.

- [DESIGN] Target: a `GET` of the device description is served in under 5 ms median on the reference hardware documented for the performance feature, measured
  with `criterion` over ≥ 30 iterations, median and p95 reported (PERF-011).

- [DESIGN] Target: the served description and SCPDs stay small (kilobytes, not megabytes) so that a client on a slow link fetches them quickly. Method: a size
  assertion in the protocol conformance test.

## 6. Usability and accessibility

- The feature has no graphical surface. Its user-visible effect is that the server appears in a television's list with the name the user chose.

- [UX-010] The `friendlyName` the user sets in the configuration is the name the television shows; the onboarding text shall say so instead of naming the XML
  element.

- [UX-004] When a client rejects the description, the log record shall name the element or service that was missing and the URL the client fetched, with
  addresses redacted (OPS-007, SEC-094).

- [UX-001] Everything here is operable without a GUI. The diagnostic command prints the description for inspection (OPS-005).

- [DOC-005] *Device description*, *SCPD*, *service*, *control URL* and *state variable* are explained at first use.

## 7. Security and privacy

- The description is served on the local network only; it contains no personal data beyond the device name the user chose (SEC-093).

- No filesystem path, no user name and no directory listing appears in either document (PLT-008).

- The DOCTYPE declaration is forbidden, and the XML parser that reads a *request* must not process a DTD (INT-024, SEC-010). [DESIGN] The documents we *serve*
  are produced by our own writer, never by echoing client input.

- Client-controllable values (the device name, the model strings) are XML-escaped (INT-027), so a name containing `<` cannot inject an element.

- The SOAP request body is bounded in size and nesting depth before it is parsed (SEC-010, SEC-011, SEC-060).

- An unknown action is answered with a fault that reveals only the action name — no stack trace, no path (SEC-013).

## 8. Interoperability

- Normative: UPnP AV `MediaServer:1` device template and required service table (standards §1.1, §3.1); UPnP Device Architecture device description and SCPD
  requirements (standards §8.2 P1–P9).

- [OBSERVED] MiniDLNA, ReadyMedia and rclone use root-relative paths (`/ctl`, `/ContentDirectory.xml`) and emit no `URLBase` (standards §3.2). Our absolute
  URLs are equally valid and avoid a resolution ambiguity on multi-homed hosts.

- [OBSERVED] rclone's live description carries an *empty* `eventSubURL` (standards §3.2); INT-021 requires a non-empty one, which is the stricter of the two
  and is what DLNA certification tooling expects (standards §3.2).

- [OBSERVED] The ConnectionManager `serviceId` is spelled `ConnectionManager` in every shipped implementation, although the OCF template shows
  `ConnectorManager` (standards §3.1). We follow the implementations (INT-044).

- [SPEC, secondary] The service identifier list of the Annex A table and the `MediaServer:2` backward-compatibility rule both treat a media server as a
  ContentDirectory plus a ConnectionManager (standards §1.1, §1.3).

- [OPEN] The normative wording of the DLNA guideline table a DMS must satisfy (guidelines 7.3.55–7.3.80) was only available second hand (standards §9 Q3); it
  constrains browsing, not the description.

## 9. Error and recovery behaviour

| Situation | Response | Basis |
| --- | --- | --- |
| `GET` of an unknown path under the media port | `404 Not Found` | INT-094 |
| `GET` of the description or an SCPD | `200 OK`, `text/xml; charset="utf-8"`, correct `Content-Length` | INT-026 |
| `POST` to a control path with an unknown service or action | SOAP fault `401 Invalid Action` | INT-025 |
| `POST` to a control path with a missing or malformed argument | SOAP fault `402 Invalid Args` | INT-025 |
| `POST` with a body that is not well-formed XML | SOAP fault `402 Invalid Args`, no panic | INT-025, SEC-010 |
| `POST` with a body larger than the accepted bound | the request is rejected before parsing | SEC-011 |
| The description cannot be generated (configuration incomplete) | the server refuses to start with a message naming the missing field | CFG-002, UX-004 |

No failure of this feature may take down the HTTP task: a malformed description request produces an HTTP error, never a panic (UX-005).

## 10. Acceptance criteria and traceability

All criteria below are *planned*; no test file exists yet.

**AC-01** — *Given* a running server, *when* the client fetches the advertised `LOCATION`, *then* the response is `200 OK` with `Content-Type: text/xml;
charset="utf-8"` and a `Content-Length` equal to the body length. Requirements: INT-026, INT-020 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/device_description.rs`

**AC-02** — *Given* the fetched description, *when* it is parsed, *then* it is well-formed, declares `encoding="utf-8"`, contains no DOCTYPE declaration and has
exactly one `device` element. Requirements: INT-024, INT-045 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/device_description.rs`

**AC-03** — *Given* the fetched description, *when* it is parsed, *then* it declares `specVersion` 1.0 and contains no `URLBase` element. Requirements: INT-044,
INT-045 · Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/device_description.rs`

**AC-04** — *Given* the fetched description, *when* the root device is inspected, *then* `deviceType` is `urn:schemas-upnp-org:device:MediaServer:1` and the
elements `UDN`, `friendlyName`, `manufacturer`, `modelName`, `modelNumber` and `modelURL` are present and non-empty, and `UDN` has the form `uuid:<UUID>`.
Requirements: INT-020 · Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/device_description.rs`

**AC-05** — *Given* the fetched description, *when* the service list is inspected, *then* exactly the `ContentDirectory:1` and `ConnectionManager:1` services
are listed, each with `serviceType`, `serviceId`, `SCPDURL`, `controlURL` and a non-empty `eventSubURL`, and the `serviceId` values are
`urn:upnp-org:serviceId:ContentDirectory` and `urn:upnp-org:serviceId:ConnectionManager`. Requirements: INT-021, INT-044 · Verification: protocol conformance
test · Planned location: `crates/phairdlna-core/tests/device_description.rs`

**AC-06** — *Given* the service list, *when* the service types are inspected, *then* `urn:schemas-upnp-org:service:RenderingControl:1` is absent and
`urn:schemas-upnp-org:service:AVTransport:1` is either absent or listed as an optional service. Requirements: INT-021 · Verification: protocol conformance test
· Planned location: `crates/phairdlna-core/tests/device_description.rs`

**AC-07** — *Given* the description and every `SCPDURL` it contains, *when* each URL is fetched, *then* it resolves to a document with `Content-Type: text/xml;
charset="utf-8"` and a correct `Content-Length`. Requirements: INT-022, INT-026 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/device_description.rs`

**AC-08** — *Given* each SCPD, *when* it is parsed, *then* every action of the service is listed with its arguments and their direction, and every state
variable is listed with its data type and, where the type has a fixed value set, its allowed values. Requirements: INT-023 · Verification: protocol conformance
test · Planned location: `crates/phairdlna-core/tests/scpd.rs`

**AC-09** — *Given* the device name is configured to a string containing `<`, `>` and `&`, *when* the description is fetched, *then* those characters appear
escaped in the XML and no extra element was created. Requirements: INT-027 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/device_description.rs`

**AC-10** — *Given* a SOAP request to a control path with an action name that the SCPD does not list, *when* it is processed, *then* the response is a SOAP
fault with the UPnP error code `401 Invalid Action` and the process keeps running. Requirements: INT-025 · Verification: protocol conformance test · Planned
location: `crates/phairdlna-core/tests/soap_negative.rs`

**AC-11** — *Given* a SOAP request for a known action with a missing argument, *when* it is processed, *then* the response is a SOAP fault with the UPnP error
code `402 Invalid Args`. Requirements: INT-025 · Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/soap_negative.rs`

**AC-12** — *Given* the description, *when* the `serviceType` of the ContentDirectory entry is inspected, *then* it is
`urn:schemas-upnp-org:service:ContentDirectory:1`. Requirements: INT-028 · Verification: protocol conformance test · Planned location:
`crates/phairdlna-core/tests/device_description.rs`

**AC-13** — *Given* a server with transcoding disabled, *when* the description is fetched, *then* the ConnectionManager service is still listed with a reachable
`controlURL`. Requirements: INT-029 · Verification: protocol conformance test · Planned location: `crates/phairdlna-core/tests/device_description.rs`

## 11. Test strategy and dependencies

**Planned test locations** (none of these files exists yet):

| Purpose | Planned path |
| --- | --- |
| Device description structure and header set | `crates/phairdlna-core/tests/device_description.rs` |
| SCPD structure per service | `crates/phairdlna-core/tests/scpd.rs` |
| SOAP fault codes and malformed bodies | `crates/phairdlna-core/tests/soap_negative.rs` |
| XML writer escaping rules | `crates/phairdlna-core/src/upnp/xml_writer.rs` (inline `#[cfg(test)]` module) |
| Description against a second control point | `qa/device-matrix.toml` |

**Method.** The description is generated from a pure function over the configuration, so it can be compared structurally without a socket; an HTTP fixture
serves it and the SCPDs. The SOAP negatives are driven by a hand-written SOAP client, the same tool F-03 and F-04 use.

**Dependencies.** F-01 (the `LOCATION` URL that points at the description and the identity strings of INT-012), F-03 and F-04 (the action lists their SCPDs
contain), and the configuration module (CFG-001, CFG-006).

## 12. Open questions and decisions

1. **[OPEN] The `AVTransport:1` question.** INT-021 allows `AVTransport:1` to be listed as an optional service; the baseline has no requirement ID for an
   AVTransport implementation on the server (F-06 owns the transfer-side behaviour). This document does not list it. *Closes when:* the owner states whether the
   baseline advertises it, or F-06 takes it over. 2. **[OPEN] Icon list and presentation URL.** The MediaServer template shows both (standards §3.1), but the
   baseline has no asset pipeline and no requirement for them. They are omitted. *Proposed requirement, not in the baseline:* `INT-105 — The server shall
   advertise at least one PNG icon in the device description.` (INT-105 is the next free identifier in the INT block; INT-046 is taken by the ContentDirectory
   error rule in F-03.) Rationale: some client applications show a generic icon otherwise. *Closes when:* the owner accepts the omission or adds the
   requirement. 3. **[OPEN] Absolute versus root-relative URLs.** INT-022 permits both; the shipped servers use root-relative paths (standards §3.2). We emit
   absolute URLs built from the interface-local address, because the client may have reached us through a different interface than the one it will use for
   control requests. *Closes when:* a device round shows that absolute URLs cause a problem. 4. **[OPEN] The `ConnectorManager` typo in the OCF template.** The
   MediaServer:1 template renders the ConnectionManager `serviceId` as `ConnectorManager`, while the Annex and every implementation use `ConnectionManager`
   (standards §3.1). INT-044 settles the spelling for us; the note exists so that a future reader does not "correct" it back. *Closes when:* the owner confirms
   the decision is final. 5. **[OPEN] The `manufacturer` and `modelName` values.** They are product strings that the project has not fixed yet; INT-012 requires
   them to be identical in the SSDP answer and the description. *Closes when:* the product strings are chosen.
