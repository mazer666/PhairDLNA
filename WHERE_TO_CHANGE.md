# WHERE_TO_CHANGE — beginner navigation

**Start here:** [README.md](README.md) → [docs/06-development/GETTING-STARTED.md](docs/06-development/GETTING-STARTED.md) → this file → the feature specification → the source and test paths.

## The rules

| Rule | Where it is written down |
| --- | --- |
| Code size, complexity, error handling, `unsafe` | [docs/00-governance/PROJECT-STANDARDS.md](docs/00-governance/PROJECT-STANDARDS.md) |
| Document structure, naming, migration | [docs/00-governance/DOCUMENTATION-POLICY.md](docs/00-governance/DOCUMENTATION-POLICY.md) |
| Requirement IDs, priorities, traceability | [docs/00-governance/REQUIREMENTS-METHOD.md](docs/00-governance/REQUIREMENTS-METHOD.md) |
| Agent responsibilities | [docs/00-governance/AGENT-ROLES.md](docs/00-governance/AGENT-ROLES.md) |
| Approved exceptions | [docs/00-governance/EXCEPTIONS.md](docs/00-governance/EXCEPTIONS.md) |
| Decisions and ADRs | [docs/00-governance/DECISION-LOG.md](docs/00-governance/DECISION-LOG.md) · [docs/02-architecture/adr/](docs/02-architecture/adr/) |
| What the product must do | [docs/01-product/TECHNICAL-REQUIREMENTS.md](docs/01-product/TECHNICAL-REQUIREMENTS.md) |
| Requirement index (generated) | [docs/01-product/REQUIREMENTS-INDEX.md](docs/01-product/REQUIREMENTS-INDEX.md) |
| User journeys | [docs/01-product/USER-JOURNEYS.md](docs/01-product/USER-JOURNEYS.md) |
| Feature index and specifications | [docs/01-product/FEATURE-CATALOG.md](docs/01-product/FEATURE-CATALOG.md) · [docs/03-features/](docs/03-features/) |
| Architecture and module boundaries | [docs/02-architecture/ARCHITECTURE.md](docs/02-architecture/ARCHITECTURE.md) |
| Security and privacy | [docs/04-security/SECURITY-MODEL.md](docs/04-security/SECURITY-MODEL.md) · [docs/04-security/PRIVACY.md](docs/04-security/PRIVACY.md) |
| Dependencies and licences | [docs/04-security/DEPENDENCY-LICENSE-REVIEW.md](docs/04-security/DEPENDENCY-LICENSE-REVIEW.md) |
| Quality gates and tests | [docs/05-quality/QUALITY-GATES.md](docs/05-quality/QUALITY-GATES.md) · [docs/05-quality/TEST-STRATEGY.md](docs/05-quality/TEST-STRATEGY.md) |
| Interoperability and device evidence | [docs/05-quality/INTEROPERABILITY-MATRIX.md](docs/05-quality/INTEROPERABILITY-MATRIX.md) |
| Performance budgets | [docs/05-quality/PERFORMANCE-BUDGETS.md](docs/05-quality/PERFORMANCE-BUDGETS.md) |
| The research the requirements cite | [docs/09-research/](docs/09-research/) |
| Build and test commands | [docs/06-development/BUILD-AND-TEST.md](docs/06-development/BUILD-AND-TEST.md) |
| Roadmap, traceability, risks, decisions | [docs/07-project/](docs/07-project/) |
| Release policy | [docs/08-releases/RELEASE-POLICY.md](docs/08-releases/RELEASE-POLICY.md) |

## The source tree

### Exists today

| Path | Responsibility | Feature |
| --- | --- | --- |
| `Cargo.toml` | Workspace definition, shared package metadata | — |
| `rust-toolchain.toml` | Pinned toolchain and components | — |
| `deny.toml` | Licence, advisory, ban and source policy | F-19 |
| `crates/phairdlna-core/src/lib.rs` | The portable domain core — the crate root and the error type | — |
| `crates/phairdlna-core/src/error.rs` | The core error type | F-12 |
| `crates/phairdlna-core/src/xml/` | The one XML writer of the project — the escaping, the length rules and the writer | F-02, F-05 |
| `crates/phairdlna-core/src/platform/` | The capability traits, the path model, the portable filesystem adapter, the memory of the process (`memory.rs`) and the default implementation | F-08, F-13, F-14, F-15 |
| `crates/phairdlna-core/src/ssdp/` | The discovery protocol: parsing a search, matching the targets, building the bytes (the **socket** is in `crates/phairdlna-cli/src/serve/ssdp.rs`) | F-01 |
| `crates/phairdlna-core/src/identity.rs` | The persistent device identity | F-01, F-02 |
| `crates/phairdlna-core/src/catalogue/` | The catalogue — the model of the items and containers, the scanner of a root, the opaque identifiers, the identity key of a file, the playlists (`playlist.rs`), the subtitle sidecars (`subtitle.rs`), the memory of the volumes of the roots, the assembly of the tree, the sniffer of a file and the registry of the formats the server serves | F-05, F-08, F-09, F-11 |
| `crates/phairdlna-core/src/media/` | The pictures of the library — the discovery of the artwork of a media file (`artwork/`), the pictures that are embedded in it (`id3/`, `mp4/`), the images that lie next to it (`sidecar.rs`), the thumbnail of a photo (`thumbnail.rs`) and the disposable cache of the pictures that were produced (`cache.rs`) | F-10 |
| `crates/phairdlna-core/src/didl/` | The DIDL-Lite document — the object model, the `protocolInfo`, the value rules, the policy of the document and the writer | F-05, F-11 |
| `crates/phairdlna-core/src/http/` | The delivery of media over HTTP — the `Range` header, the headers of a media answer, the containment of a media URL and the evaluator of a request | F-06, F-11 |
| `crates/phairdlna-core/src/upnp/` | The device description, the service descriptions (SCPD), the SOAP dispatcher and the ContentDirectory and ConnectionManager actions | F-02, F-03, F-04 |
| `crates/phairdlna-core/src/config/` | The typed, validated, versioned configuration | F-12 |
| `crates/phairdlna-cli/src/lib.rs` | The commands, as pure functions over their arguments | F-13 |
| `crates/phairdlna-cli/src/catalogue.rs` | The catalogue commands — the tree of the library and the document a client receives | F-08, F-09, F-13 |
| `crates/phairdlna-cli/src/serve/` | The daemon — the start of the server, the HTTP routes, the discovery socket, the library and the watch of the drives, the media of the library, the body of a media answer, the cache of the pictures, the log, the signals and the transfers that run (`transfers.rs`) | F-01, F-02, F-03, F-04, F-06, F-08, F-10, F-13 |
| `crates/phairdlna-cli/src/serve/admin.rs` | The wiring of the administration surface — the view of the daemon that the surface asks | F-14 |
| `crates/phairdlna-cli/src/main.rs` | The argument parsing and the exit code | F-13 |
| `crates/phairdlna-core/tests/media_roots.rs` | The acceptance criteria of F-08 for one root | F-08 |
| `crates/phairdlna-core/tests/media_roots_multiple.rs` | The acceptance criteria of F-08 for more than one root | F-08 |
| `crates/phairdlna-core/tests/support/` | The fixtures of the tests of the core — the scripted platform of the volumes, the checker of a document and the catalogue of the protocol tests | F-08 |
| `crates/phairdlna-core/tests/playlists.rs` | The acceptance criteria of F-11 for the playlists — the formats, the order, the containment and the broken playlist | F-11 |
| `crates/phairdlna-core/tests/subtitles.rs` | The acceptance criteria of F-11 for the subtitle sidecars — the pairing, the additional resource and the reachability of the file | F-11 |
| `crates/phairdlna-core/tests/catalogue_identity.rs` | The acceptance criteria of F-11 for the organisation — the identity key, the de-duplication of two roots and the exclusion of a folder | F-11 |
| `crates/phairdlna-core/tests/volume_events.rs` | The acceptance criteria of F-08 for the volumes of a root — a drive that goes, comes back and comes back under another identity | F-08 |
| `crates/phairdlna-core/tests/volume_platform_events.rs` | The acceptance criteria of F-08 for the events the platform reports — a volume that was unmounted and one that was mounted again | F-08 |
| `crates/phairdlna-cli/tests/serve.rs` | The daemon that answers a client — the description, the service descriptions, the SOAP control, the media and the artwork | F-02, F-03, F-04, F-06 |
| `crates/phairdlna-cli/tests/subtitles.rs` | The daemon that serves a subtitle — the document that names it and the answer that carries it | F-11 |
| `crates/phairdlna-core/tests/artwork.rs` | The acceptance criteria of F-10 — the embedded cover, the sidecar, the thumbnail and the files that are broken | F-10 |
| `crates/phairdlna-cli/tests/artwork.rs` | The artwork of the daemon — the route that answers a request for a picture | F-10 |
| `crates/phairdlna-cli/tests/discovery.rs` | The discovery of the daemon — the advertisements, the answer of a search and the withdrawal | F-01 |
| `crates/phairdlna-cli/tests/serve_command.rs` | The `serve` command of the command line and the graceful shutdown | F-13 |
| `crates/phairdlna-cli/tests/support/` | The fixtures of the tests of the daemon — the daemon in a thread and the small HTTP client | F-13 |
| `crates/phairdlna-admin/src/lib.rs` | The optional administration surface — the crate root and the public API | F-14 |
| `crates/phairdlna-admin/src/token.rs` | The token of the surface — the file, the permissions and the comparison | F-14 |
| `crates/phairdlna-admin/src/guard.rs` | The rules every request of the surface must pass — host, framing, origin, token, custom header | F-14 |
| `crates/phairdlna-admin/src/routes.rs` | The three routes of the surface and the JSON they answer with | F-14 |
| `crates/phairdlna-admin/src/backend.rs` | What the surface asks the server, and the answers it understands | F-14 |
| `crates/phairdlna-admin/src/json.rs` | The JSON writer of the surface | F-14 |
| `crates/phairdlna-admin/src/server.rs` | The server of the surface — the listeners and the shutdown | F-14 |
| `crates/phairdlna-admin/tests/admin_surface.rs` | The acceptance criteria of F-14 — the answers and the refusals | F-14 |
| `crates/phairdlna-cli/tests/admin.rs` | The administration surface of a running daemon | F-14 |
| `crates/phairdlna-mac/src/main.rs` | The optional macOS application — a banner so far | F-15 |
| `scripts/` | Repository gates and the requirement-view generator | F-19 |
| `.github/workflows/` | Manual-only continuous integration | F-19 |
| `docs/` | The documentation tree | F-19 |

> **A path moves into this table the day it exists.** Until then it belongs in the
> planned table below.

### Planned layout

**None of these paths exists yet.** They are listed so that the target structure is
visible and so that a contributor can see where their change will land.

| Planned path | Responsibility | Feature |
| --- | --- | --- |
| `crates/phairdlna-core/src/ssdp/interfaces.rs` | The selection of the sending interface and the rate limiter of the answers | F-01 |
| `crates/phairdlna-core/src/catalogue/index.rs` | The index of the catalogue on disk, the rebuild and the event-driven scan | F-09 |
| `crates/phairdlna-core/src/scheduler/` | Priority classes, bounded pools, backpressure | F-18 |
| `crates/phairdlna-core/src/platform/` | Platform traits and per-OS implementations | F-08, F-13, F-15 |
| `crates/phairdlna-core/src/lifecycle/` | Server states, sleep/wake and network recovery | F-13 |
| `crates/phairdlna-core/src/logging/` | Structured logging and redaction | F-13, F-16 |
| `crates/phairdlna-core/src/error.rs` | The crate error type | — |
| `crates/phairdlna-mac/` | The optional native macOS application | F-15 |

### The scripts

| Script | Gate | State | What it does |
| --- | --- | --- | --- |
| `scripts/check-file-size.sh` | QG-13 | **exists and runs** | Enforces the file-length budget |
| `scripts/check-doc-links.sh` | QG-11 | **exists and runs** | Resolves every relative documentation link |
| `scripts/check-no-unsafe.sh` | QG-03 | **exists and runs** | Finds `unsafe` outside of comments |
| `scripts/generate_requirement_views.py` | CI step "Requirement views are current" | **exists and runs** | Generates the requirement index and the traceability matrix |
| `scripts/check-crate-boundaries.sh` | QG-28 | **exists and runs** | Fails when a crate depends "downwards" (ADR-0001, amended by ADR-0012) |
| `scripts/soak.sh` | QG-22 | planned | Runs the seven-day soak and samples the signals |
| `scripts/device-matrix.sh` | QG-23 | planned | Records a device test in the interoperability matrix |
