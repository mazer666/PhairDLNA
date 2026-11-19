# Release blockers — current evidence checklist

Status: **NOT RELEASE-READY** · Reconciled 2026-11-18 · Original approved baseline retained in Git history.

**Existing publication:** the owner-published prerelease `v0.1.0-alpha.1` points to
`66e94c9`; current `main` is `9481e49` and contains the later app-first migration. The
alpha's release notes describe the earlier Web Admin workflow. It is a historical
artifact, not release evidence for current HEAD; all current acceptance blockers below
remain open.

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
- [x] Native macOS app with main window, Dock presence and menu-bar status item,
      optional authenticated loopback API, CLI setup and diagnostics exist at code
      level. The browser UI was removed by DEC-075; full native app flows and
      real-Mac UX acceptance remain open (DEC-076).
- [x] Native Apple Silicon support verified on host (`PLT-004`) without Rosetta translation.
- [x] Universal binary packaging script (`PLT-014`) created, combining arm64 and x86_64 into macOS application bundle.
- [x] First client/device interoperability test protocol automated against live daemon fixture
      (VLC media player, description, CDS SCPD, Browse root, byte ranges verified).

## Open release blockers

- [ ] Re-run and archive the **complete** quality/security gate set against the
      **latest HEAD**, including the app-first UX and service-integration research of 2026-11-18.
- [ ] Complete the source-distribution offer and review notices/provenance,
      third-party license obligations and final dependency set.
- [ ] Verify protocol conformance against additional external clients and physical Smart TVs;
      expand the physical-device interoperability matrix beyond the initial VLC/host test.
- [ ] Complete independent security review, fuzzing/negative tests and resolve
      outstanding parser isolation and network/resource risks.
- [ ] Prove that the macOS server can read user-selected protected folders without
      reading unselected folders, and that revocation removes access (OD-15, research Q3).
- [ ] Measure performance limits on documented reference hardware and run the
      long-duration soak/recovery tests.
- [ ] Complete native macOS onboarding, accessibility/VoiceOver, service integration
      and usability acceptance on actual Mac hardware.
- [ ] Validate the cross-project PhairDLNA ↔ PhairPlay/PhairPlayRust integration
      with real endpoints; specifications alone do not establish interoperability.
- [ ] Verify macOS and other supported platform builds, install/upgrade/uninstall,
      recovery paths and final packaging; record version/commit/checksums.
- [ ] Obtain explicit owner approval for any next release and publication; approval of
      the existing alpha does not authorize later tags or artefacts.

No item in the open list is closed by the existence of implementation code alone.
No public release or tag without explicit authorization.
