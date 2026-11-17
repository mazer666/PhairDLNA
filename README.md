# PhairDLNA

**Privacy-first, cross-platform DLNA/UPnP AV media server built in Rust.**

Status: **active implementation (not release-ready)**. The requirements baseline was approved on
2026-11-15. A functioning headless DLNA server, SQLite media catalogue, optional local
administration service and a macOS menu-bar application have been implemented and
covered by automated tests. **Real-device interoperability, performance budgets,
accessibility and long-term reliability remain unverified.** See
[the status report](docs/07-project/STATUS.md).

## What PhairDLNA aims to be

A local media server for your own files: your television finds it on the network,
browses it and plays from it. Free, advertising-free, tracking-free, no account, no
cloud, no internet access in normal operation.

- macOS first (Apple Silicon and Intel), Linux and Windows designed in from the start.
- Direct play first; transcoding optional, isolated and off by default.
- Secure local-network defaults; never exposed to the internet by default.
- Real-device compatibility must be tested and reported, including failures; no such tests are recorded yet.
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
| see every requirement | [Requirements index](docs/01-product/REQUIREMENTS-INDEX.md) (346 requirements in the last documented generated baseline) |
| look up a term | [Glossary](docs/01-product/GLOSSARY.md) |
| understand the architecture | [Architecture](docs/02-architecture/ARCHITECTURE.md) · [Decisions](docs/00-governance/DECISION-LOG.md) · [ADRs](docs/02-architecture/adr/) |
| work on a feature | [Feature catalog](docs/01-product/FEATURE-CATALOG.md) · [Feature specifications](docs/03-features/) |
| check the security posture | [Security model](docs/04-security/SECURITY-MODEL.md) · [Privacy model](docs/04-security/PRIVACY.md) |
| check the quality gates | [Quality gates](docs/05-quality/QUALITY-GATES.md) · [Test strategy](docs/05-quality/TEST-STRATEGY.md) · [Interoperability matrix](docs/05-quality/INTEROPERABILITY-MATRIX.md) |
| know the plan | [Roadmap](docs/07-project/ROADMAP.md) · [Status](docs/07-project/STATUS.md) · [Open decisions](docs/07-project/OPEN-DECISIONS.md) |
| check a claim against its source | [Research inputs](docs/09-research/) — the briefs every specification cites |
| build and test | [Getting started](docs/06-development/GETTING-STARTED.md) · [Build and test](docs/06-development/BUILD-AND-TEST.md) |
| contribute | [AGENTS.md](AGENTS.md) · [WHERE_TO_CHANGE.md](WHERE_TO_CHANGE.md) · [CONTRIBUTING.md](CONTRIBUTING.md) |

## Build and use the current implementation

```bash
git clone https://github.com/mazer666/PhairDLNA.git
cd PhairDLNA
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test --workspace --doc   # doctests are a separate step
cargo run -p phairdlna-cli -- --help
cargo run -p phairdlna-cli -- setup          # interactive CLI first-run setup
cargo run -p phairdlna-cli -- config validate
cargo run -p phairdlna-cli -- serve          # run DLNA daemon (requires usable configuration)
```

The complete gate set is in
[docs/06-development/BUILD-AND-TEST.md](docs/06-development/BUILD-AND-TEST.md).

## Verified status and limitations

- The daemon serves SSDP/UPnP descriptions, SOAP browsing and HTTP media with byte ranges.
  Socket-level integration tests exist, but real TVs and third-party receivers have not been tested.
- Multiple media roots, persistent SQLite catalogue, artwork, playlists and sidecar subtitles
  have implemented paths and automated test coverage.
- The local administration surface and macOS menu-bar application exist. The newer commits
  of 2026-11-17 add CLI onboarding, diagnostics export, two-tier settings explanations and
  a lightweight settings-inspection web UI. This is **not** proof of complete graphical
  onboarding or of a completed usability/accessibility study.
- The documented 2026-11-17 M2.9 local gate run reported **1,038 tests passed**. These
  results predate later 2026-11-17 UI/diagnostics commits; do not treat them as a
  verified full regression run of current HEAD.
- Three GitHub CI runs were successful on earlier commits, not on the latest HEAD.
- No measured performance comparison, seven-day soak or physical-device compatibility
  evidence is recorded. See the [interoperability matrix](docs/05-quality/INTEROPERABILITY-MATRIX.md).
- The product is **not release-ready**; see [release blockers](docs/07-project/RELEASE-BLOCKERS.md).


## Licence

[AGPL-3.0-or-later](LICENSE) — the complete licence text is in `LICENSE`.
