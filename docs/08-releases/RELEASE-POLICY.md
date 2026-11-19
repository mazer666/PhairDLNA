# Release policy — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Lead / Release Manager
Blockers: `../07-project/RELEASE-BLOCKERS.md` · Gates: `../05-quality/QUALITY-GATES.md`

---

## 1. The rule

> **No public release and no Git tag without explicit owner approval.**

This applies to every channel: a GitHub release, a Homebrew tap entry, a Debian or
Red Hat package, a Windows installer, a container image, a nightly build artefact.

A *pre-release* artefact built from a commit and uploaded for internal testing is not
a release, but it must be named unambiguously (`phairdlna-<version>-<commit>-preview`)
and must never carry a version tag.

---

## 2. Release channels

| Channel | Audience | Stability | Approval |
| --- | --- | --- | --- |
| **Preview** | Contributors and testers | May break at any time | Lead |
| **Alpha** | Enthusiasts who file issues | Interfaces may still change | Owner |
| **Beta** | Early users | Features frozen; bugs expected | Owner |
| **Stable** | Everyone | The compatibility promise applies | Owner |

**Observed release state (2026-11-18):** GitHub contains the owner-published public
prerelease [v0.1.0-alpha.1](https://github.com/mazer666/PhairDLNA/releases/tag/v0.1.0-alpha.1),
tagged at `66e94c9`. It predates current `main` (`9481e49`) and its notes describe the
earlier Web Admin workflow. This historical alpha does not establish that current HEAD
is release-ready or waive the checklist for any future release. The rule above remains
in force: every new release and tag needs explicit owner approval, and the release
checklist applies.

---

## 3. What a release contains

| Artefact | Requirement |
| --- | --- |
| The complete corresponding source, including the dependency tree and the build recipe | `SUP-005`, `SEC-070`, `OPS-016` |
| The full AGPL-3.0 licence text | `SUP-001` |
| A software bill of materials | `SUP-005` |
| A SHA-256 checksum per artefact | `SUP-005` |
| A changelog listing what changed, what was fixed and what was **broken** | — |
| The evidence bundle of the milestone | `../07-project/STATUS.md` |
| Installation, upgrade, recovery and uninstall instructions | `RELEASE-BLOCKERS.md` |
| Native Linux distribution packages (e.g. `.deb`, `.rpm`, or unconfined tarball; sandboxed packaging formats like Snap/Flatpak are forbidden as primary channel to ensure unrestricted media root access) | `PLT-016`, `F-22` |

---

## 4. The release checklist

1. Every `MUST` requirement of the milestone is **verified**, not merely implemented.
2. Every applicable gate is green with observed output.
3. The negative test of every applicable gate has been demonstrated.
4. The seven-day soak is complete, with flat memory, descriptor and thread trends.
5. The device matrix contains the observed results the milestone promised.
6. The independent security review has no open critical or high finding.
7. Every documented exception is current, with an approver and a review date.
8. The licence, advisory and ban gates are green, and `Cargo.lock` is committed.
9. The artefacts are built reproducibly and their checksums are published.
10. The owner has approved the release **in writing**.

---

## 5. Versioning

Semantic versioning, `MAJOR.MINOR.PATCH`.

- **MAJOR** — a change that breaks a documented interface or the compatibility promise.
- **MINOR** — a new feature within the existing promise.
- **PATCH** — a fix that changes no interface.

The version appears in the binary, in the `SERVER` header, in the device description
and in the source offer, and it must match the Git tag exactly (`OPS-017`).

---

## 6. Disclosure and security fixes

A security fix is released like any other patch. A vulnerability is disclosed
privately first, following `../../SECURITY.md`, and the public release happens after a
fix exists — never before.

The project makes **no certification claim** and no compatibility claim that is not
backed by a row in the interoperability matrix.

---

## 7. Rollback

A release is not withdrawn. A broken release is followed by a patch release that
reverts the defect, and the defect is described in the changelog. The index format is
stable, so a client that cannot handle a newer feature still works — that is a
requirement, not a hope.
