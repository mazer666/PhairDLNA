# PhairDLNA

**Privacy-first, cross-platform DLNA/UPnP AV media server built in Rust.**

Status: **requirements and architecture phase.** No functional server exists yet. The
requirements baseline is complete and awaits owner approval — see
[the status report](docs/07-project/STATUS.md).

## What it will be

A local media server for your own files: your television finds it on the network,
browses it and plays from it. Free, advertising-free, tracking-free, no account, no
cloud, no internet access in normal operation.

- macOS first (Apple Silicon and Intel), Linux and Windows designed in from the start.
- Direct play first; transcoding optional, isolated and off by default.
- Secure local-network defaults; never exposed to the internet by default.
- Compatibility proven against real devices, with failures published too.
- Documentation, security and reproducible verification treated as product features.

## Principles

| Principle | What it means here |
| --- | --- |
| Public and free forever | AGPL-3.0-or-later, no monetisation, no accounts |
| Privacy first | No telemetry, no analytics, no crash reporting, no internet dependency |
| Standards first | A conforming UPnP AV media server, not a look-alike |
| Evidence over claims | Every performance, compatibility and security statement has a recorded measurement or a named test |
| Beginner-readable | Small modules, documented APIs, a navigation path from the README to the exact file to change |

## The project documents

| I want to … | Read |
| --- | --- |
| understand the product | [Vision](docs/01-product/PRODUCT-VISION.md) · [Product requirements](docs/01-product/PRODUCT-REQUIREMENTS.md) · [Technical requirements](docs/01-product/TECHNICAL-REQUIREMENTS.md) |
| see every requirement | [Requirements index](docs/01-product/REQUIREMENTS-INDEX.md) (341 requirements) |
| understand the architecture | [Architecture](docs/02-architecture/ARCHITECTURE.md) · [Decisions](docs/00-governance/DECISION-LOG.md) · [ADRs](docs/02-architecture/adr/) |
| work on a feature | [Feature catalog](docs/01-product/FEATURE-CATALOG.md) · [Feature specifications](docs/03-features/) |
| check the security posture | [Security model](docs/04-security/SECURITY-MODEL.md) · [Privacy model](docs/04-security/PRIVACY.md) |
| check the quality gates | [Quality gates](docs/05-quality/QUALITY-GATES.md) · [Test strategy](docs/05-quality/TEST-STRATEGY.md) · [Interoperability matrix](docs/05-quality/INTEROPERABILITY-MATRIX.md) |
| know the plan | [Roadmap](docs/07-project/ROADMAP.md) · [Status](docs/07-project/STATUS.md) · [Open decisions](docs/07-project/OPEN-DECISIONS.md) |
| check a claim against its source | [Research inputs](docs/09-research/) — the briefs every specification cites |
| build and test | [Getting started](docs/06-development/GETTING-STARTED.md) · [Build and test](docs/06-development/BUILD-AND-TEST.md) |
| contribute | [AGENTS.md](AGENTS.md) · [WHERE_TO_CHANGE.md](WHERE_TO_CHANGE.md) · [CONTRIBUTING.md](CONTRIBUTING.md) |

## Build the scaffold

```bash
git clone https://github.com/mazer666/PhairDLNA.git
cd PhairDLNA
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p phairdlna-cli     # prints a status banner; it does not start a server
```

## Status

**No performance or compatibility claim is verified yet.** The device matrix is empty
and every performance number in the documentation is a *target*, not a measurement.
That is deliberate: the requirements phase records what will be measured and how.

## Licence

[AGPL-3.0-or-later](LICENSE) — the complete licence text is in `LICENSE`.
