# Release blockers — current evidence checklist

Status: **NOT RELEASE-READY** · Reconciled 2026-11-17 · Original approved baseline retained in Git history.

This document distinguishes completed groundwork from remaining release acceptance.
A checked task means the named evidence exists; it does **not** imply that all
release gates are cleared. See [project status](STATUS.md) and
[quality gates](../05-quality/QUALITY-GATES.md).

## Groundwork established

- [x] Full official GNU AGPLv3 text is present as `LICENSE`.
- [x] Functional headless Rust DLNA daemon, SQLite catalogue and socket-based
      integration tests exist (M1.11 / M2.9).
- [x] Dependency/license policy tooling (`cargo deny`) and advisory tooling
      (`cargo audit`) have a documented successful local M2.9 run.
- [x] Requirements, architecture, security threat model and test plans exist.
- [x] Earlier manually triggered CI runs completed successfully (not on latest HEAD).
- [x] Native macOS menu-bar frontend, optional loopback admin API, CLI setup,
      diagnostics and a lightweight settings-inspection web UI are implemented
      at code level; full UX acceptance remains open.
- [x] Native Apple Silicon support verified on host (`PLT-004`) without Rosetta translation.
- [x] First client/device interoperability test protocol automated against live daemon fixture
      (VLC media player, description, CDS SCPD, Browse root, byte ranges verified).

## Open release blockers

- [ ] Re-run and archive the **complete** quality/security gate set against the
      **latest HEAD**, including all 2026-11-17 UX/diagnostics commits.
- [ ] Complete the source-distribution offer and review notices/provenance,
      third-party license obligations and final dependency set.
- [ ] Verify protocol conformance against additional external clients and physical Smart TVs;
      expand the physical-device interoperability matrix beyond the initial VLC/host test.
- [ ] Complete independent security review, fuzzing/negative tests and resolve
      outstanding parser isolation and network/resource risks.
- [ ] Measure performance limits on documented reference hardware and run the
      long-duration soak/recovery tests.
- [ ] Complete native macOS onboarding, accessibility/VoiceOver, service integration
      and usability acceptance on actual Mac hardware.
- [ ] Validate the cross-project PhairDLNA ↔ PhairPlay/PhairPlayRust integration
      with real endpoints; specifications alone do not establish interoperability.
- [ ] Verify macOS and other supported platform builds, install/upgrade/uninstall,
      recovery paths and final packaging; record version/commit/checksums.
- [ ] Obtain explicit owner approval for release and publication.

No item in the open list is closed by the existence of implementation code alone.
No public release or tag without explicit authorization.
