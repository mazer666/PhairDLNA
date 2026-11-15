# Independent code review — M1.9, the catalogue over one root

Status: **Addressed** · Date: 2026-11-15 · Reviewer: an independent agent that did not
write the code · Milestone: M1.9

Scope of the review: the files of the milestone M1.9 — the catalogue of the core, the
scanner of a root, the object identifiers, the DIDL-Lite mapping of folders, the two
catalogue commands of the command line, the tests, and the documentation that was
changed with the milestone.

Method: the binding rules (`AGENTS.md`, `docs/00-governance/PROJECT-STANDARDS.md`), the
feature specifications F-08 and F-09 and ADR-0007 were read first; the whole change was
read; the gates were run (`cargo test --workspace`, `cargo fmt --check`, `cargo clippy
--all-targets -D warnings`, the size, boundary, link, licence and advisory checks); and
a hostile-tree harness was built in `/tmp` against the real crate — symbolic links that
point out of the root, cycles, a tree that is deeper than the limit, a link that is
repointed after the scan, hard links, names that are not valid UTF-8, exclusions, a
changed root order, and a SOAP `Browse` against the real catalogue.

## What held up

The containment of a symbolic link, the termination of a cycle, the cut of the depth
limit, the claim that the scanner never writes (every file was compared before and
after the scan), and the claim that a browse reads nothing from the disk.

## The findings and what was done

| # | Severity | Finding | What was done |
| --- | --- | --- | --- |
| B1 | blocker | `system_update_id` was a hash: it decreased when the catalogue grew and did not change when the content of a file changed, so it broke `INT-037`. The recorded evidence proved the contract with a fixture and not with the product | The identifier of the whole catalogue is a **counter** that is raised whenever the catalogue of that library changes, and the identifier of a container now covers the size and the modification time of its children. The deviation — the counter starts again after a restart, because it is not written to the disk — is stated in F-09 § 12.1 (D-05). New tests against the real catalogue: `the_update_id_never_goes_backwards`, `the_update_id_changes_when_a_file_changes` |
| B2 | blocker | F-09 § 12.1 claimed the criterion for hard links was implemented; it was not. Two names of one file were two objects | The scanner asks the platform for the identity of a file and keeps one object per identity (`crates/phairdlna-core/src/catalogue/scanner/walk.rs`). New test: `a_hard_link_is_not_a_second_object` |
| B3 | blocker | F-09 D-04 stated a guarantee the code did not keep | Rewritten with the behaviour of B1 |
| M1 | major | The tables of F-08 and F-09 named acceptance criteria that do not exist in that form, and the criteria that were not exercised were missing | Both tables list **every** criterion of § 10 with its result: *implemented*, *partly implemented* with what is missing, or **NOT RUN** with the reason |
| M2 | major | The symbolic-link policy was global; `CFG-007` asks for a per-root setting | `MediaRoot` carries `symlink_policy`, the setting of the server is the default, and the scanner uses the policy of the root. New test: `a_root_may_have_a_policy_of_its_own` |
| M3 | major | A followed link was stored by its link path, so a link that was repointed after the scan could serve a file outside the root | The catalogue holds the **target** of a link. New test: `a_link_that_is_repointed_after_the_scan_cannot_escape_the_root` |
| M4 | major | The diagnostics carried absolute paths, against F-08 § 7 | The diagnostics name the path below the root. New test: `the_diagnostics_name_no_absolute_path` |
| M5 | major | The root container had the parent identifier `""` while the fixture and the retired placeholder used `-1` | The root container has the parent `-1`. New test: `the_root_container_has_the_parent_id_minus_one` |
| M6 | major | A browse of the metadata of an item answered `701 No Such Object` | The catalogue answers for an item (`ContentSource::item_of`), and the browse answers with the item. New tests: `the_metadata_of_an_item_can_be_asked_for` (the protocol) and `the_catalogue_browse_shows_an_item` (the command line) |
| M7 | major | The size gate measured only the tracked files, and two new files were above the 300-line target | The gate measures the untracked files as well (`git ls-files -co`), and every file of the milestone is below 300 lines |
| M8 | minor | The generated views still named the retired placeholder catalogue | The generator names the real catalogue, and the views were regenerated |
| M9 | minor | The exclusions were matched against the raw path, while the identity is case-folded | The exclusions use the identity of the path. New test: `an_excluded_folder_is_excluded_whatever_its_case` |
| M10 | minor | `platform::identity_of` folds the case and is the variant a caller reaches for first | Its documentation says plainly that it is the case-insensitive variant and that the catalogue uses `identity_of_with` |
| M11 | minor | `Catalogue::path_of` had no consumer | It is documented as the seam of the delivery layer (F-16) and is used by the tests of the link policy |
| M12 | minor | The licence review named `arrayref`, which is not in the lock file, and did not name `cpufeatures`, which is | The review names the crates that are really new and assesses `cpufeatures` |
| M13 | minor | ADR-0007 still described the hash input without the root key and called the hash "non-cryptographic" | The ADR carries a dated amendment that names the two decisions of the owner (`DEC-027`, `DEC-029`) |
| M14 | minor | The test of `IndexAsLink` asserted the same expectation as the test of `Follow`, and the scanner treated both alike | The two policies differ: `Follow` makes the link transparent, `IndexAsLink` indexes the link as its own entry |
| M15 | minor | A root that is a file was reported as present | A root that is not a folder is reported as absent. New test: `a_root_that_is_not_a_folder_is_reported_as_absent` |
| M16 | minor | The depth limit said "unless the configuration says otherwise", and there is no such key | The documentation says that the limit is a constant of the scanner |
| M17 | minor | The numbers of `STATUS.md` were one test short, a milestone was called "reviewed" before the review, and a step was marked done although its gate is planned | The numbers, the wording and the step are corrected |
| M18 | minor | "Opaque" could be read as "secret" | F-08 D-02 says that "opaque" means *carries no part of the path*, and that a guess can be confirmed because the hash is not a secret |

## What the review did not verify

The `UF_HIDDEN` flag of macOS (the portable adapter applies the convention of the dot
and the sidecar `._name`), the case-insensitive path of Windows, and the behaviour of a
name that is not valid UTF-8 (macOS refuses such names; the Linux behaviour needs a
fixture). These are named in the tables of F-08 § 12.1 as **NOT RUN** with the reason.
