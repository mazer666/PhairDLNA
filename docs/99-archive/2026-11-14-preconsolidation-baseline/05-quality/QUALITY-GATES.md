# Quality gates — draft

G0 requirements: traceable IDs, scope, acceptance criteria, open decisions, owner review.
G1 architecture: module boundaries, ADRs, threat model, dependency/license review.
G2 implementation: rustfmt, reviewed Clippy policy, build/test, code-size checks, error handling and docs.
G3 interoperability: UPnP/DLNA protocol fixtures, range/seek, real-client matrix, regressions.
G4 robustness: fuzz, malformed input, traversal, resource bounds, crash recovery, soak test.
G5 distribution: platform artifacts, reproducible build evidence, SBOM/license review, checksums, installation/recovery docs.
G6 release: no critical/high unresolved security findings, documented exceptions, explicit owner approval.

Do not mark a gate passed without actual tool output or reproducible evidence. Targets are provisional until hardware/test fixtures are specified.
