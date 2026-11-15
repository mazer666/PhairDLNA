# Glossary — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Documentation reviewer

A newcomer should be able to read the documentation without meeting a term for the
first time in a table. This is that list. A term explained in a feature
specification is also explained here; the specification is the detailed version.

---

## The protocol

| Term | Meaning |
| --- | --- |
| **DLNA** | Digital Living Network Alliance — a certification programme for devices that share media over a home network. Its practical meaning for us: a set of rules that real televisions actually follow, on top of UPnP |
| **UPnP** | Universal Plug and Play — the 2002 protocol family for discovering and controlling devices on a local network. UPnP AV is its audio/video profile |
| **UPnP AV** | The audio/video extension of UPnP that defines the MediaServer, MediaRenderer and ConnectionManager devices |
| **SSDP** | Simple Service Discovery Protocol — the small text protocol over UDP port 1900 that lets a client find a server. The server sends *advertisements* (`NOTIFY`); the client sends *searches* (`M-SEARCH`) |
| **M-SEARCH** | The SSDP search a client sends to ask "who is out there?" The server answers with a unicast reply |
| **NOTIFY** | The SSDP advertisement the server sends to announce itself, and the goodbye message it sends on shutdown (`ssdp:byebye`) |
| **ST / USN / LOCATION** | The three SSDP headers that matter: what is being searched for, the unique identity of the answer, and the URL where the device description lives |
| **UDN** | Universal Device Name — the server's identity, a `uuid:` string. It must be stable across restarts |
| **Device description** | The XML document that describes the server: its type, its name, its identity and the services it offers. The `LOCATION` header points at it |
| **SCPD** | Service Contract Description — the XML document that describes one service: its actions, their arguments and its state variables. A client reads it to know what it may call |
| **SOAP** | The XML envelope format that carries the control requests. A request names an *action* in the `SOAPAction` header and passes arguments in the body |
| **ContentDirectory** | The service a client browses: it answers `Browse` (give me the children of this container), `Search`, and the capability questions |
| **Browse** | The action that returns the children of a container, as a DIDL-Lite document plus counts |
| **Container / item** | A folder and a file, in the catalogue's vocabulary. A container has children; an item has a resource |
| **Object ID** | The stable, opaque identifier of a container or an item. It is what a client asks for later |
| **ConnectionManager** | The service that tells a client which formats the server can send, and what is currently being transferred |
| **protocolInfo** | The four-field string `http-get:*:video/mp4:*` that says how a resource is transported, over what network, in what format, and with what extra flags |
| **DIDL-Lite** | The XML format in which a browse result is delivered: a list of items and containers with their metadata |
| **DML / `upnp:class`** | The classification of an object — for example `object.item.videoItem.movie` or `object.container.storageFolder` |
| **`res`** | The element inside a DIDL-Lite item that points at the media itself, with the `protocolInfo`, the size and the duration |
| **DMR** | Digital Media Renderer — the device that plays the media (the television). A **DMS** (Digital Media Server) is the device that serves it |
| **Control point** | The client application that browses and controls. A television's DLNA app and VLC are control points |
| **GENA** | General Event Notification Architecture — the UPnP mechanism for pushing change notifications to a subscribed client |
| **IGD** | Internet Gateway Device — the UPnP profile for routers. **PhairDLNA does not implement it**, because it is an attack surface with no benefit here |
| **CallStranger** | A 2020 disclosure that UPnP event subscriptions could be abused to make a server fetch arbitrary URLs. The reason eventing is off by default here |
| **Byte range** | A part of a file, requested with a `Range: bytes=…` header so that a client can seek without downloading the whole file |
| **Content-Range** | The header that says which part of the file a `206 Partial Content` response contains |
| **sec: / dlna:** | Two different XML namespaces. `dlna:` is the DLNA metadata namespace; `sec:` is a **Samsung** extension. They are not interchangeable |

## The product

| Term | Meaning |
| --- | --- |
| **Media root** | A folder the user has chosen to share. The server reads it and nothing else |
| **Catalogue** | The server's picture of the library: the objects, their metadata and their relations. It is a **cache**; the filesystem is the truth |
| **Index** | The SQLite database that holds the catalogue |
| **Scan** | Walking a media root and comparing what is there with what the index knows |
| **Incremental scan** | A scan that re-reads only the files that changed, using size and modification time |
| **Direct play** | Sending the file as it is. The default, and the only thing the server does unless the user asks for more |
| **Transcoding** | Converting a file into a format the client can play. Optional, off by default, and never applied silently |
| **Device profile** | A declarative record of a client's quirks and the workaround for each. Data, not code, so a user can switch one off |
| **Quirk** | A documented deviation of a client from the specification |
| **Quarantine** | Setting a file or a subtree aside because it could not be read safely — for example because the operating system refused access |
| **Presentation mode** | The *simple* or *advanced* view of the user interface. Both show the same settings; they differ in how much detail is visible |
| **Source offer** | The route by which the running server serves the corresponding source of its own build, as the AGPL requires |
| **Hardening** | A protective measure with a test that proves it works — not a claim |
| **Direct play / transmux** | Transmuxing repackages a stream without re-encoding it; PhairDLNA does not do it |

## The platform

| Term | Meaning |
| --- | --- |
| **TCC** | Transparency, Consent and Control — the macOS system that protects Desktop, Documents, Downloads, network and removable volumes and asks the user for permission |
| **Full Disk Access** | The macOS permission that lifts the TCC protection for an application. The user grants it; the server never works around a refusal |
| **launchd** | The macOS service manager. A **launchd agent** runs per user; a **launchd daemon** runs system-wide |
| **Local network privacy** | The macOS 15+ permission that governs UDP multicast traffic. A daemon is exempt, an agent is not — which is why the server runs as a daemon |
| **APFS** | Apple's default file system: case-insensitive by default, and **normalisation-insensitive** — two spellings of the same name are the same file |
| **NFC / NFD** | The two Unicode normalisation forms. macOS preserves whichever form a file was written in, and the Finder displays the decomposed one |
| **AppleDouble (`._*`)** | The sidecar files macOS creates on file systems that cannot store resource forks. The server ignores them |
| **Notifier / status item** | The small icon in the macOS menu bar that represents the running server |
| **Gatekeeper** | The macOS mechanism that decides whether a downloaded application may run. Unsigned software prompts the user once |
| **Notarisation** | Apple's service that checks a signed application before Gatekeeper lets it run without a prompt |
| **polkit** | The Linux system for asking a user for authority to perform a privileged action |
| **systemd** | The Linux service manager. A **user unit** runs per user; a **system unit** runs system-wide |
| **longPathAware** | The Windows manifest flag, together with the `LongPathsEnabled` policy, that lifts the 260-character path limit |
| **Universal binary** | One macOS executable containing both the Apple Silicon and the Intel build |

## Security vocabulary

| Term | Meaning |
| --- | --- |
| **SSRF** | Server-Side Request Forgery — making a server issue a request to an address the attacker chose |
| **DNS rebinding** | Pointing a hostname at a local address so that a web page in the user's browser reaches a service on the LAN. The reason the administration surface validates the `Host` header |
| **CSRF** | Cross-Site Request Forgery — making the user's browser perform a state-changing request it did not intend |
| **DTD / external entity** | The parts of XML that let a document refer to other content. A parser that resolves them can be tricked into reading local files |
| **XXE** | XML External Entity attack — the abuse of that mechanism |
| **Entity expansion** | A "billion laughs" style attack in which nested entity definitions multiply into gigabytes |
| **Framing / request smuggling** | Two interpretations of where a request ends. A server that guesses differently from a proxy can be desynchronised |
| **TOCTOU** | Time of check to time of use — checking a path and then using it, while something changes it in between |
| **Path containment** | The rule that every path the server opens must resolve inside a configured root |
| **Redaction** | Replacing sensitive values — paths, addresses, identifiers — with stable pseudonyms so that a log can be shared |
| **Outbound-connection audit** | Running the server with every destination blocked except the local network, and checking that a normal session produces no outbound connection at all |

## The project's own vocabulary

| Term | Meaning |
| --- | --- |
| **Requirement ID** | A stable identifier such as `INT-030`. The prefix names the domain; the number is permanent and is never reused |
| **Feature specification** | The document that elaborates a group of requirements with acceptance criteria. `F-nn` |
| **Milestone** | A phase of delivery, `M0` … `M5`, each with exit criteria |
| **Milestone gate** | The check that a phase is finished, `G0` … `G6` |
| **Quality gate** | A mechanical check, `QG-nn`, with a command and a threshold |
| **Acceptance criterion** | An objectively testable statement in a feature specification, `AC-nn` |
| **Threat row** | One entry of the threat table, `SEC-TH-nn` |
| **Mitigation** | One of the eight normative security measures, `M-01` … `M-08` |
| **Conformance check** | One of the protocol checks, `PC-nn` |
| **Evidence record** | The observed output that proves a claim. A link, a command's output, a device row |
| **NOT RUN** | The result label for a check that could not be performed. **Never a pass** |

---

## Where a term is defined in detail

| Question | Document |
| --- | --- |
| What does the product do? | [../01-product/PRODUCT-VISION.md](../01-product/PRODUCT-VISION.md) |
| What must it do? | [../01-product/TECHNICAL-REQUIREMENTS.md](../01-product/TECHNICAL-REQUIREMENTS.md) |
| How is it built? | [../02-architecture/ARCHITECTURE.md](../02-architecture/ARCHITECTURE.md) |
| What can go wrong? | [../04-security/SECURITY-MODEL.md](../04-security/SECURITY-MODEL.md) |
| Where did a rule come from? | [../09-research/](09-research/) |
