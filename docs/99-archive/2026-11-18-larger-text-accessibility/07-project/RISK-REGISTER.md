# Risk register — PhairDLNA

Status: **Approved baseline v1.0 (2026-11-14); current register v1.3 (2026-11-18)** · Owner role: Lead / Project Manager
Review: at every milestone review, and whenever a risk changes state.
Revision history: v1.1 adds R-14 for the unverified macOS service-to-folder access boundary; v1.2 adds R-15 for app-side configuration writes and activation drift; v1.3 adds R-16 for larger-text accessibility and layout clipping. The approved v1.0 baseline is preserved in `../99-archive/2026-11-18-risk-register-baseline/`.

Scoring: **Likelihood** and **Impact** are each Low / Medium / High. The rating is the
product of the two. A High risk must have a named mitigation and an owner before the
milestone it affects can start.

---

## R-01 — The first real device test fails in a way nobody anticipated

| | |
| --- | --- |
| Likelihood | High |
| Impact | High |
| Rating | **High** |
| Mitigation | The protocol work is based on primary specifications and the conformance checklist is automated before any device is touched. The device matrix records failures as first-class results. The first device round is scheduled inside M1, not after it |
| Owner | Interoperability Specialist |
| Detection | A device test that finds a defect |
| Residual | Some client behaviour is simply not documented anywhere. The mitigation is the matrix, not a prediction |

## R-02 — The macOS firewall blocks the server and the user does not know why

| | |
| --- | --- |
| Likelihood | High |
| Impact | Medium |
| Rating | **High** |
| Mitigation | The firewall behaviour for an unsigned or ad-hoc binary is the largest macOS-specific device-test risk. The first-run output names the interface and the address; the status command reports the advertised address; the documentation explains the "Open Anyway" flow and the firewall prompt. A signed build (OD-01) removes the cause rather than the symptom |
| Owner | Lead |
| Detection | The first macOS device test |
| Residual | Until OD-01 is settled, some users will hit a firewall prompt. Acceptable and documented |

## R-03 — The macOS local-network permission blocks SSDP

| | |
| --- | --- |
| Likelihood | Medium |
| Impact | High |
| Rating | **High** |
| Mitigation | PLT-013 currently requires a system-context launchd daemon, but its effective identity and behavior must be established with OD-15 while satisfying OPS-015/PLT-018. Verify Local Network behavior for the selected process model on macOS 15+ and make the app explain any required permission step |
| Owner | Architect |
| Detection | The first macOS device test on macOS 15 or later |
| Residual | The applicable exemption depends on the actual process/service configuration; no real-Mac verification is recorded |

## R-04 — The Unicode and case-sensitivity model is wrong on an exotic volume

| | |
| --- | --- |
| Likelihood | Medium |
| Impact | Medium |
| Rating | **Medium** |
| Mitigation | Identity is normalised per root and scoped by the volume; uniqueness comes from the file identity, not the path (ADR-0007). The behaviour is detected per root rather than assumed. A volume that changes its behaviour between mounts is a documented edge case with a defined recovery |
| Owner | Architect |
| Detection | A device test on an exFAT or HFS+ volume |
| Residual | A file system with unusual semantics can still surprise us; the mitigation is that the index is rebuildable |

## R-05 — The scope of the baseline is too large for one milestone

| | |
| --- | --- |
| Likelihood | Medium |
| Impact | Medium |
| Rating | **Medium** |
| Mitigation | M1 is deliberately narrow: one root, direct play, no artwork, no application. Every feature is separately specified and separately acceptable, so the plan can be cut without breaking the product |
| Owner | Lead |
| Detection | Milestone review |
| Residual | A cut M1 is a smaller product; the roadmap says which capabilities are lost |

## R-06 — A dependency turns out to be unmaintained or unsafe after adoption

| | |
| --- | --- |
| Likelihood | Medium |
| Impact | High |
| Rating | **High** |
| Mitigation | `cargo audit` and `cargo deny` are release gates; `Cargo.lock` is committed and diff-reviewed; every dependency is recorded with a named reviewer; the dependency set is deliberately small and the risky ones (UPnP, XML, HTTP) are ours |
| Owner | Security Engineer |
| Detection | The advisory gate |
| Residual | A vulnerability with no fixed release would require replacing the dependency; the dependency review records alternatives for exactly that case |

## R-07 — The AGPL network-source obligation is implemented incorrectly

| | |
| --- | --- |
| Likelihood | Low |
| Impact | High |
| Rating | **Medium** |
| Mitigation | The source offer is a specified product feature with its own requirement IDs (`OPS-016`, `OPS-017`, `SEC-070`) and an acceptance criterion. It must serve the **exact** running build including the dependency tree and the build recipe, without authentication |
| Owner | Security Engineer |
| Detection | The acceptance criterion of F-14 and F-16 |
| Residual | Legal advice would be the final word; the implementation follows the licence text verbatim |

## R-08 — The product is perceived as "another DLNA server"

| | |
| --- | --- |
| Likelihood | Medium |
| Impact | Medium |
| Rating | **Medium** |
| Mitigation | The competitive research identified two direct Rust competitors (VuIO, Spritz). The differentiation is not "Rust" or "lightweight" but **explainable compatibility**, **guided troubleshooting**, **resilient everyday operation** and **a native macOS experience** — see `../01-product/DIFFERENTIATION-STRATEGY.md` |
| Owner | Product Owner |
| Detection | User feedback and the usability study |
| Residual | Differentiators only count when they are visible and fast; the Compatibility Inspector and the Playback Doctor are the concrete proposals |

## R-09 — A device-specific workaround breaks another client

| | |
| --- | --- |
| Likelihood | Medium |
| Impact | Medium |
| Rating | **Medium** |
| Mitigation | A workaround may never violate a normative requirement for other clients (`INT-101`), must be switchable off without recompiling (`INT-104`), and must be recorded with its evidence in the interoperability matrix |
| Owner | Interoperability Specialist |
| Detection | The regression suite of the device matrix |
| Residual | A subtle interaction may only appear with a client combination that is not tested |

## R-10 — Nobody can build the project because the toolchain is unclear

| | |
| --- | --- |
| Likelihood | Low |
| Impact | Medium |
| Rating | **Medium** |
| Mitigation | `WHERE_TO_CHANGE.md`, the getting-started guide and a build-and-test document exist before the first feature; the CI workflow is manual but reproducible; the file-size and complexity gates are automated rather than cultural |
| Owner | Documentation Reviewer |
| Detection | A newcomer failing to build |
| Residual | The first newcomer is the best detector of this risk; the M0 exit criterion includes a clean-room build |

## R-11 — The seven-day soak is skipped or done too late

| | |
| --- | --- |
| Likelihood | Medium |
| Impact | High |
| Rating | **High** |
| Mitigation | The soak is an explicit release gate (`QG-22`, `REL-004`) with a defined signal set. Starting it early is cheap; discovering a leak after a release is not |
| Owner | QA Engineer |
| Detection | The trend report of the soak |
| Residual | A leak that only appears after 8 days would still be missed; the soak duration is a stated limitation |

## R-12 — Privacy claims are made without evidence

| | |
| --- | --- |
| Likelihood | Medium |
| Impact | High |
| Rating | **High** |
| Mitigation | The privacy model states exactly what is stored and what leaves the machine; the release checklist contains an **outbound-connection audit** that must observe zero unexpected connections during a normal session; no telemetry claim is made without that observation |
| Owner | Security Engineer |
| Detection | The outbound-connection audit |
| Residual | A dependency that opens a connection only under rare conditions could be missed; the dependency review and the "no dependency phones home" rule cover that |

## R-13 — The AGPL forces a business model change later

| | |
| --- | --- |
| Likelihood | Low |
| Impact | Medium |
| Rating | **Low** |
| Mitigation | The product is permanently free, advertising-free and account-free by owner decision (`DEC-001`). No commercial dependency exists that could be lost. The licence is the licence |
| Owner | Owner |
| Detection | — |
| Residual | None within the current mandate |

## R-14 — Selected-folder access across the macOS service process boundary is unverified

| | |
| --- | --- |
| Likelihood | Medium (unverified) |
| Impact | High |
| Rating | **High** |
| Mitigation | Keep service lifecycle implementation gated by OD-15. On a real Mac, verify access to a selected protected folder, denial for an unselected sibling, and effective revocation; have Security and Platform review the result before choosing the process model |
| Owner | Security Engineer / Platform Specialist |
| Detection | The OD-15 real-Mac experiment and independent security review |
| Residual | Apple documents scoped bookmarks for App Sandbox processes and per-process access, but not transfer of the unsandboxed UI's consent to a separate service process. A per-user service changes the approved availability and privacy model |

## R-15 — Native app configuration writes bypass the API and may not reach the running server

| | |
| --- | --- |
| Likelihood | High (current implementation observed) |
| Impact | High |
| Rating | **High** |
| Mitigation | Gate the folder/settings flow on OD-16. If option A or B is selected, specify and review the authenticated API write path, validation, persistence and activation behavior; if option C is selected, remove or disable app writes and revise F-14/F-15 scope. Do not claim a setting is active before the server confirms it |
| Owner | Architect / Security Engineer |
| Detection | OD-16 option review; for A/B, API failure/recovery criteria for invalid, concurrent and interrupted updates; for C, verification that app settings are read-only and make no file writes |
| Residual | The wizard currently reports completion after a failed save, and the preferences Save control is unwired. The final API and activation model depends on OD-14/OD-15; ADR-0004 currently rejects runtime reload |

## R-16 — The native app clips content at larger text sizes

| | |
| --- | --- |
| Likelihood | High (fixed font sizes, fixed frames and line caps observed) |
| Impact | High (accessibility and ordinary-user comprehension) |
| Rating | **High** |
| Mitigation | Resolve OD-17. If an app-level size control is selected, require reflow, no hidden actions or content, and manual checks at 150% and 200% across the supported locales; do not claim Dynamic Type on macOS |
| Owner | Product Owner / Accessibility reviewer |
| Detection | Mac usability and accessibility protocol at multiple text sizes and locales |
| Residual | No rendered-app verification has been recorded; the current view can truncate text |

---

## Risk review questions

At every milestone review, these are asked and answered in writing:

1. Which risk materialised since the last review, and what did it cost?
2. Which risk became more likely, and why?
3. Which mitigation was never implemented?
4. Is there a risk we are carrying that is not on this list?
