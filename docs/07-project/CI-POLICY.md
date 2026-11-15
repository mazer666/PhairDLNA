# Continuous integration policy — PhairDLNA

Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Lead / QA

---

## 1. The rule

GitHub Actions are **manual only**. Every workflow uses `workflow_dispatch` and
nothing else. There is no automatic push trigger, no automatic pull-request trigger
and no automatic schedule.

**Why:** an automated pipeline that runs on every push turns every unfinished
experiment into a red run, and a red run that nobody looks at is worse than no
pipeline. The project prefers a small number of *trusted* manual runs whose output
is read and recorded.

**Changing this rule requires an owner decision** and is recorded in
`../00-governance/DECISION-LOG.md`.

---

## 2. What exists

| Workflow | Trigger | Purpose |
| --- | --- | --- |
| `ci.yml` | `workflow_dispatch` | The full gate set on one platform |
| `ci-matrix.yml` | `workflow_dispatch` | The build and test set on macOS, Linux and Windows |

Both workflows are **real and complete**: `ci.yml` runs eleven gate steps and
`ci-matrix.yml` builds and tests on macOS, Linux and Windows plus the minimum
supported Rust version. What they cover today is limited by what exists: the gates
that depend on the implementation cannot run, and the workflows that call them say
so explicitly rather than pretending.

The scripts the workflows call are `scripts/check-file-size.sh` (QG-13),
`scripts/check-doc-links.sh` (QG-11), `scripts/check-no-unsafe.sh` (QG-03) and
`scripts/generate_requirement_views.py`.

---

## 3. Rules for a workflow

1. **`workflow_dispatch` only.** No `push`, no `pull_request`, no `schedule`.
2. **Pinned actions.** Every action is pinned to an exact release tag, and moves to
   a commit SHA as the project grows. *(In progress: the actions currently use
   release tags, not digests.)*
3. **Pinned toolchain.** The `stable` channel is selected in `rust-toolchain.toml`,
   and the build matrix additionally tests the minimum supported version from
   `Cargo.toml` (`rust-version`).
4. **No secrets.** The gates need none. A workflow that needs a secret needs an
   owner decision and an explanation.
5. **Failures are informative.** A failing step prints the command, the exit code
   and the relevant output.
6. **The output is the evidence.** The result of a manual run is what may be cited
   as evidence; nothing else is.

---

## 4. The gate set a run must execute

The full list is in `../05-quality/QUALITY-GATES.md`. A `ci.yml` run covers:

`QG-01` formatting · `QG-02` lints · `QG-03` no unsafe · `QG-04` build ·
`QG-05` unit tests · `QG-07` doctests · `QG-08` licences · `QG-09` advisories ·
`QG-10` bans and sources · `QG-11` link check · `QG-13` file size.

The remaining gates — `QG-06` integration tests, `QG-12` spelling, `QG-14`
complexity, `QG-15` binary size, `QG-16` snapshots, `QG-17` DIDL-Lite conformance,
`QG-18` property tests, `QG-19` fuzz smoke, `QG-20` Miri, `QG-21` benchmarks,
`QG-22` soak, `QG-23` device matrix, `QG-24` semver, `QG-25` outdated,
`QG-26` coverage, `QG-27` mutation testing and `QG-28` crate boundaries — arrive
with the code they guard. **A gate that is not in the workflow is not run, and a
gate that is not run is never reported as a pass.** The canonical list with the
state of each is in `../05-quality/QUALITY-GATES.md`.

---

## 5. Reporting a CI result

| Label | Meaning |
| --- | --- |
| `PASS` | The workflow ran and every step succeeded |
| `FAIL` | A step failed; the output names the failing gate |
| `NOT RUN — <reason>` | The workflow was not triggered, or a step was skipped |
| `BLOCKED — <reason>` | Waiting for something external |

**A workflow that was never triggered is not a green workflow.** Before any claim
that "CI is green", the run list and the runner list must both be checked.

---

## 6. Before a workflow may become automatic

1. Its gates have been run manually at least **three** times, green.
2. The **negative test** of every gate in it has been demonstrated at least once.
3. Its failure output has been read by a human and found useful.
4. The owner has approved the change.

Until then, manual only — without exception.
