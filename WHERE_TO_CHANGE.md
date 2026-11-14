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
| Build and test commands | [docs/06-development/BUILD-AND-TEST.md](docs/06-development/BUILD-AND-TEST.md) |
| Roadmap, traceability, risks, decisions | [docs/07-project/](docs/07-project/) |
| Release policy | [docs/08-releases/RELEASE-POLICY.md](docs/08-releases/RELEASE-POLICY.md) |

## The source tree

> **Do not add a path to this table until it exists.** Update this file whenever a
> module changes ownership.

| Path | Responsibility | Feature |
| --- | --- | --- |
| `Cargo.toml` | Workspace definition, shared package metadata | — |
| `rust-toolchain.toml` | Pinned toolchain versions | — |
| `crates/phairdlna-core/` | The portable domain core — no OS-specific dependencies | F-01 … F-12 |
| `crates/phairdlna-core/src/config/` | Typed, validated, versioned configuration | F-12 |
| `crates/phairdlna-core/src/ssdp/` | SSDP discovery, advertisement, interface selection | F-01 |
| `crates/phairdlna-core/src/upnp/` | Device model, SCPD, SOAP dispatch | F-02, F-03, F-04 |
| `crates/phairdlna-core/src/didl/` | DIDL-Lite generation and escaping | F-05 |
| `crates/phairdlna-core/src/http/` | Media delivery, ranges, MIME | F-06 |
| `crates/phairdlna-core/src/catalogue/` | Library model, index, scanning, identifiers | F-08, F-09 |
| `crates/phairdlna-core/src/media/` | Media inspection, metadata, artwork | F-10, F-11 |
| `crates/phairdlna-core/src/security/` | Path containment, redaction, limits | F-16 |
| `crates/phairdlna-core/src/platform/` | Platform traits and per-OS implementations | F-08, F-13, F-15 |
| `crates/phairdlna-core/src/scheduler/` | Priority classes, bounded pools, backpressure | F-18 |
| `crates/phairdlna-cli/` | The daemon and the command line | F-13 |
| `crates/phairdlna-admin/` | The optional loopback administration surface | F-14 |
| `crates/phairdlna-mac/` | The optional native macOS application | F-15 |
| `scripts/` | Repository gates and the requirement-view generator | F-19 |
| `.github/workflows/` | Manual-only continuous integration | F-19 |

## The scripts

| Script | Gate | What it does |
| --- | --- | --- |
| `scripts/check-file-size.sh` | QG-13 | Enforces the file-length budget |
| `scripts/check-doc-links.sh` | QG-11 | Resolves every relative documentation link |
| `scripts/generate_requirement_views.py` | — | Regenerates the requirement index and the traceability matrix |
