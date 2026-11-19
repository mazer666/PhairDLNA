# macOS service identity and folder-access protocol

Status: **Draft — isolation design and probe prerequisite not approved; experiment not run; OD-15 remains open**  
Owner roles: Platform Specialist and Security Engineer  
Version/date: 0.3 / 2026-11-19  
Requirements: `OPS-015`, `PLT-013`, `PLT-017`, `PLT-018`, `MB-001`  
References: `../docs/07-project/OPEN-DECISIONS.md` (OD-14/15), `../docs/03-features/F-22-platform-portability.md` (AC-14), `../docs/09-research/platform-integration.md` § 5.4

## 1. Purpose and limits

Determine whether a macOS service model can read only user-selected media folders,
without elevated privileges, while behaving correctly with Local Network privacy.
This protocol records evidence for OD-15; it does not select a service model or
change the approved requirements.

Run the candidates separately on a disposable macOS 15 or later installation:

| Candidate | Service context | Decision constraint |
| --- | --- | --- |
| A | System-domain `LaunchDaemon` configured to run as the ordinary test user | Any effective root identity fails `OPS-015` and `PLT-018`. |
| B | Per-user `LaunchAgent` | A change to `PLT-013` requires owner approval; verify Local Network behavior and login/logout availability. |

**Security design warning:** a normal per-user agent has the signed-in user's filesystem
permissions. That does not by itself restrict reads to the folder selected in the UI; it may
also read an unselected sibling. The candidate must meet the sibling-denial and revocation
criteria as written. Before implementing the probe, determine whether a sandbox or another
reviewed boundary can achieve them and how the service process receives and revokes access.
Do not treat catalogue root containment as operating-system denial and do not relax the
acceptance criteria to make the candidate pass. The independent review records this as an
inference that the experiment must resolve: `../docs/05-quality/reviews/OD15-SECURITY-REVIEW-2026-11-19.md`.

Do not treat folder selection, a path in configuration, or a same-user shell probe as
proof of access by the server process. The current server has no demonstrated way to
probe an unselected sibling from inside its own process. Before implementing the probe,
Security and Platform must review and approve its isolation design; before running this
protocol, they must independently review the experiment-only probe with these constraints:

- It runs the filesystem `open` attempt inside the already-running server process;
  a helper process with the same UID is not sufficient.
- It accepts only two fixture labels, `selected` and `sibling`, and resolves them
  relative to a disposable fixture root established when the experiment build starts.
  It accepts no arbitrary path, URL or shell command.
- It reports only whether the fixture can be read and whether a prearranged synthetic
  canary hash matches; it never returns file contents or real paths.
- It is compiled behind an explicit experiment-only build option, is absent from
  release builds, and is removed after the experiment. Its source and invocation must
  receive independent security review before use.
- It must run in the candidate service package with the candidate's bundle identifier,
  signing identity, launchd metadata, effective UID/GID configuration and relevant
  entitlements. Record and compare those values. If any relevant value differs, label
  the result `EXPLORATORY` and do not use it alone to decide OD-15.

Until this prerequisite exists and is reviewed, record the negative-access and
revocation cases as `NOT RUN — service-process probe unavailable`. Do not add a
general filesystem probe to a release build.

## 2. Isolated test setup

Use a disposable Mac and account. Do not use Full Disk Access, change broad folder
permissions, add ACLs, or test against personal media. Stop other PhairDLNA instances.

Create two sibling folders inside one macOS-protected location (for example, the
test account's Documents folder):

- `PhairDLNA-Selected`, containing one known small media fixture and a unique canary;
- `PhairDLNA-Unselected`, containing a different fixture and canary.

Record hashes of both canaries. Select only the first folder through the actual native
folder picker. Keep the second folder absent from every server configuration and
bookmark. Start exactly one candidate service and confirm that test output comes from
that running server process. Never expose canary contents or real paths in evidence.

Before recording a candidate result, capture its service label, executable, bundle
identifier, signing identity, relevant entitlements, launchd metadata, effective
UID/GID, login-session state, Local Network permission state, macOS version/build, and
whether Full Disk Access is enabled. Also record the selected-folder authorization
mechanism: which process receives the picker selection, which process creates or stores
any authorization artifact (such as a security-scoped bookmark), where that artifact is
stored if one exists, and how the running service resolves or activates it. Record only redacted
metadata or a one-way fixture-specific hash; never include raw bookmark bytes, tokens or
real paths. If access relies on ambient process or TCC permissions instead of a
transferred artifact, record that explicitly and identify which process owns the grant.
A missing observation is `NOT RUN`, not an inferred pass.

## 3. Procedure — repeat for each candidate

1. **Identity and least privilege.** Start the service. Confirm its effective identity
   is the ordinary test user and not root. Record the process identity from the actual
   running service, not from the installer or UI process.
2. **No selection.** Before granting folder access, ask the actual service process to
   read the selected canary by invoking the reviewed experiment-only probe. Record the
   denial. If the platform has not yet prompted, record which process receives the
   prompt; do not infer consent from the UI picker.
3. **Selected-folder access.** Select only `PhairDLNA-Selected` through the UI and
   complete only the platform consent flow presented for this candidate. Trace the
   resulting authorization mechanism or artifact to the running service process; a
   UI-side bookmark or a permission prompt alone is not proof that the service received
   access. Use the server's real catalogue/media path to confirm that the service can
   read the selected fixture. Record the observed result, process and authorization
   mechanism responsible for it.
4. **Unselected sibling denial.** Ask the same service process to read the
   `PhairDLNA-Unselected` canary through the reviewed probe. It must be denied. A
   failure caused only by the catalogue omitting that folder is insufficient: the
   observation must distinguish service-process filesystem denial from
   application-level root containment.
5. **Revocation.** Revoke the platform grant using the candidate's actual consent
   mechanism and invalidate or deactivate any associated authorization artifact, then
   restart the service if that mechanism requires it. Record which process performs the
   revocation and how the service learns the grant is no longer valid. Repeat the
   selected-canary read in the service process through the reviewed probe. Access must
   now be denied. Removing a root from configuration alone does not count as permission
   revocation.
6. **Local Network behavior.** Begin with a clean permission state in the disposable
   account. Trigger SSDP send and receive from the running service. If macOS presents a
   prompt, record which process it names; deny access and verify the observed discovery
   result from a second device on the same LAN. Then grant access through System
   Settings, repeat, and record the result. If no prompt or control exists for the
   candidate, record that fact and the observed network behavior without assuming an
   exemption. Do not count a loopback status response as network discovery evidence.
7. **Lifecycle.** For the agent candidate, record behavior across logout/login. For the
   daemon candidate, record boot/login availability and whether the identity remains
   non-root. Confirm stopping or unregistering the candidate removes its process and
   does not leave a hidden helper behind.
8. **Cleanup.** Stop and unregister the tested candidate, remove its temporary data and
   consent grants, and verify no test service remains installed.

## 4. Decision evidence

A candidate passes the folder boundary only if all of these are observed in the actual
server process: ordinary-user identity; denial before consent; read access to the one
selected folder after consent; denial for the unselected sibling; and denial after
revocation. The selected read must be attributable to the recorded authorization
mechanism in that server process; ambient access that bypasses the selected-folder flow
is insufficient. The candidate must also meet the approved Local Network and service
availability requirements. A root identity, Full Disk Access workaround, broad
permission change, unselected-folder read, or post-revocation read is a failure.

If the negative observation cannot be isolated from the server's root-containment
logic, the result is inconclusive. Keep OD-15 open until an independent security
review accepts the probe and the evidence. If neither candidate satisfies all
requirements, return the conflict to the owner; do not weaken a requirement in code.

| Candidate | Identity | Authorization mechanism/flow | Selected read | Sibling denied | Revoked denied | Local Network | Lifecycle | Result |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| A — system daemon | `NOT RUN` | `NOT RUN` | `NOT RUN` | `NOT RUN` | `NOT RUN` | `NOT RUN` | `NOT RUN` | `NOT RUN — no real-Mac run recorded` |
| B — user agent | `NOT RUN` | `NOT RUN` | `NOT RUN` | `NOT RUN` | `NOT RUN` | `NOT RUN` | `NOT RUN` | `NOT RUN — no real-Mac run recorded` |

For each run, attach only redacted logs, service metadata, permission-state captures,
canary hashes and the independent review. Never attach tokens, private media, or
personal filesystem paths.

## Revision history

| Version/date | Change |
| --- | --- |
| 0.3 / 2026-11-19 | Records the independent review finding that an ordinary-user LaunchAgent may read unselected siblings; requires an isolation design review before probe implementation. |
| 0.2 / 2026-11-19 | Adds the actual-server-process probe constraints, process identity and authorization-flow evidence, and independent review requirements. |
