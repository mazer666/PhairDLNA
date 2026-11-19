# OD-15 security review — 2026-11-19

Status: **Independent design review complete; experiment probe not approved or run**  
Owner role: Security & Privacy Engineer  
Version/date: 1.0 / 2026-11-19  
Purpose: Review the DEC-079 direction and identify security prerequisites for the OD-15 service-access experiment.

## Scope

Read-only review of DEC-079, OD-15, the macOS service-access protocol, the platform
integration research and the `OPS-015`, `PLT-013` and `PLT-018` requirements. This review
does not select or approve a service model, authorize a PLT-013 change, approve probe code,
or certify macOS behavior that was not observed.

## Findings verified in project documents

- DEC-079 makes a per-user `LaunchAgent` the first OD-15 experiment candidate. It does not
  approve shipping that service or changing `PLT-013`.
- The experiment requires access attempts inside the actual server process, including a
  selected-folder read, unselected-sibling denial, denial after revocation, ordinary-user
  identity, Local Network behavior and login/logout lifecycle.
- `OPS-015` and `PLT-018` prohibit requiring root to serve the user's own folders. The
  current `PLT-013` still requires a system-context daemon and describes the UI as a
  per-user LaunchAgent, so a passing experiment and an owner-approved requirement revision
  are prerequisites to implementation.
- The protocol requires an experiment-only probe restricted to fixture labels and a fixed
  fixture root. It also requires independent review before the probe is used.

## Security analysis

**Inference requiring experiment:** a normal per-user LaunchAgent runs with the signed-in
user's filesystem permissions. That fact alone does not restrict it to the folder selected
in the UI. Unless a sandbox or another demonstrated access boundary limits the service,
the actual server process may also read an unselected sibling. Catalogue root containment
can prevent clients from browsing that sibling, but it does not prove operating-system or
process-level denial. Do not weaken the sibling-denial or revocation criteria to make the
candidate pass.

Apple documents that security-scoped access is activated by the process using the scoped URL;
the service's ability to use an authorization created by the UI must be demonstrated. Apple
also documents Local Network attribution to responsible code. Neither fact establishes the
behavior of PhairDLNA's separately launched service.

## Lead follow-up — separate from the independent review

The current checkout has no App Sandbox entitlements file or sandbox entitlement in the
packaged `PhairDLNA.app` (`codesign -d --entitlements :- target/bundle/PhairDLNA.app` emitted
no entitlements). Therefore the current bundle does not demonstrate selected-folder
isolation for a per-user service.

Apple's bookmark documentation says that a security-scoped app bookmark is usable by a
sandboxed caller with the same code-signing identity, and that the process accessing the
resource must activate and later relinquish the scope. This makes a sandboxed app plus a
same-identity bundled helper a candidate design to investigate; it does **not** prove that
PhairDLNA's `SMAppService` agent can receive the bookmark, access only the selected folder,
or satisfy Local Network privacy. The current ad-hoc signing setup and the unresolved
distribution identity also need to be considered before relying on a stable bookmark identity.

## Required work before probe implementation or execution

1. Design the probe as a compile-time experiment-only facility absent from standard and
   release builds. Accept only the `selected` and `sibling` labels and resolve them beneath
   a fixed, disposable fixture root; accept no arbitrary path, URL or shell command.
2. Determine whether a sandboxed app/helper with matching code-signing identity can meet
   sibling-denial and revocation. Specify how the service receives authorization and how
   the actual process activates and relinquishes it. Do not assume a UI-created bookmark
   transfers access.
3. Keep the endpoint inaccessible in ordinary builds and ensure requests cannot expose
   fixture contents, real paths, bookmark bytes or tokens.
4. Review the probe source and invocation independently before running it. Run each service
   candidate with its actual bundle, signature, launchd metadata, effective UID/GID and
   privacy identity. Treat mismatched setup as exploratory evidence.
5. If an ordinary-user service can read the unselected sibling, record the candidate as
   failed under the current protocol. Return the service design for a new owner decision;
   do not ship it based only on catalogue-level containment.

## Conclusion

DEC-079 is appropriately conditional. The per-user LaunchAgent remains a reasonable first
candidate to investigate, but it is **not ready for probe implementation or a pass/fail run**
until the access-isolation design is reviewed. OD-15 and the PLT-013 change remain open.

## Sources

- [Apple: `startAccessingSecurityScopedResource`](https://developer.apple.com/documentation/Foundation/NSURL/startAccessingSecurityScopedResource%28%29)
- [Apple: app-scoped bookmark signing-identity requirement](https://developer.apple.com/documentation/CoreFoundation/CFURLCreateBookmarkData%28_%3A_%3A_%3A_%3A_%3A_%3A%29)
- [Apple: App Sandbox](https://developer.apple.com/documentation/security/app-sandbox)
- [Apple: TN3179, Understanding Local Network Privacy](https://developer.apple.com/documentation/technotes/tn3179-understanding-local-network-privacy)
- `qa/macos-service-access-protocol.md`
- `docs/09-research/platform-integration.md` § 4–5
