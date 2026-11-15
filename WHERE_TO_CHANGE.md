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
| `crates/phairdlna-core/src/platform/` | The eight capability traits, the path model and the default implementation | F-08, F-13, F-15 |
| `crates/phairdlna-core/src/ssdp/` | The discovery protocol: parsing a search, matching the targets, building the bytes | F-01 |
| `crates/phairdlna-core/src/identity.rs` | The persistent device identity | F-01, F-02 |
| `crates/phairdlna-core/src/config/` | The typed, validated, versioned configuration | F-12 |
| `crates/phairdlna-cli/src/lib.rs` | The commands, as pure functions over their arguments | F-13 |
| `crates/phairdlna-cli/src/main.rs` | The argument parsing and the exit code | F-13 |
| `crates/phairdlna-admin/src/lib.rs` | The optional administration surface — the token path so far | F-14 |
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
| `crates/phairdlna-core/src/ssdp/` | SSDP discovery, advertisement, interface selection | F-01 |
| `crates/phairdlna-core/src/upnp/` | Device model, SCPD, SOAP dispatch | F-02, F-03, F-04 |
| `crates/phairdlna-core/src/didl/` | DIDL-Lite generation and escaping | F-05 |
| `crates/phairdlna-core/src/http/` | Media delivery, ranges, MIME | F-06 |
| `crates/phairdlna-core/src/catalogue/` | Library model, index, scanning, identifiers | F-08, F-09 |
| `crates/phairdlna-core/src/media/` | Media inspection, metadata, artwork | F-10, F-11 |
| `crates/phairdlna-core/src/security/` | Path containment, redaction, limits | F-16 |
| `crates/phairdlna-core/src/platform/` | Platform traits and per-OS implementations | F-08, F-13, F-15 |
| `crates/phairdlna-core/src/scheduler/` | Priority classes, bounded pools, backpressure | F-18 |
| `crates/phairdlna-core/src/lifecycle/` | Server states, sleep/wake and network recovery | F-13 |
| `crates/phairdlna-core/src/logging/` | Structured logging and redaction | F-13, F-16 |
| `crates/phairdlna-core/src/error.rs` | The crate error type | — |
| `crates/phairdlna-admin/` | The optional loopback administration surface | F-14 |
| `crates/phairdlna-mac/` | The optional native macOS application | F-15 |

### The scripts

| Script | Gate | State | What it does |
| --- | --- | --- | --- |
| `scripts/check-file-size.sh` | QG-13 | **exists and runs** | Enforces the file-length budget |
| `scripts/check-doc-links.sh` | QG-11 | **exists and runs** | Resolves every relative documentation link |
| `scripts/check-no-unsafe.sh` | QG-03 | **exists and runs** | Finds `unsafe` outside of comments |
| `scripts/generate_requirement_views.py` | CI step "Requirement views are current" | **exists and runs** | Generates the requirement index and the traceability matrix |
| `scripts/check-crate-boundaries.sh` | QG-28 | **exists and runs** | Fails when a crate depends "downwards" |
| `scripts/soak.sh` | QG-22 | planned | Runs the seven-day soak and samples the signals |
| `scripts/device-matrix.sh` | QG-23 | planned | Records a device test in the interoperability matrix |
