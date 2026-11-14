# F-LIFECYCLE — Sleep, wake, network change and recovery

Status: Draft. No behavior verified.

## User outcome
PhairDLNA recovers predictably when a Mac sleeps, switches Wi-Fi/Ethernet, reconnects to VPN, changes IP, or loses NAS access. The user sees clear state and does not have to restart the app manually.

## Requirements
LIFE-001 Define distinct server states: Starting, Ready, Degraded, Suspended, Recovering, Stopping, Stopped, Failed. Every transition has reason, timestamp and user-facing explanation.
LIFE-002 Handle sleep/wake events without spinning, resource leaks or misleading 'ready' status. On wake, refresh active interfaces, rebind listeners as required and re-advertise SSDP only on permitted interfaces.
LIFE-003 Handle Wi-Fi/Ethernet, DHCP address, subnet, VPN and multicast route changes. Never silently widen network exposure beyond configured policy.
LIFE-004 When configured interface disappears, enter Degraded/Recovering; maintain explicit fallback policy (default: no automatic exposure on another interface).
LIFE-005 Detect NAS/removable-volume loss and restoration; preserve configured library roots, retain index safely with unavailable markers and bounded reconciliation. Never silently delete media records as if user removed them.
LIFE-006 Active streams interrupted by actual sleep/network loss must terminate or recover according to protocol reality; never promise transparent continuity if client must reconnect. Explain when resuming is client-controlled.
LIFE-007 Retry discovery/rebinding with bounded exponential backoff and jitter; avoid multicast storms and tight loops.
LIFE-008 Recover from service crash with atomic state, consistent catalog or safe repair. Do not require GUI to remain open for headless operation.
LIFE-009 Local admin and UI reflect current effective interface, IP, listener state and last recovery event; technical details expandable.
LIFE-010 Startup/login service installation and permissions are explicit and reversible. No hidden privileged helper or automatic firewall bypass.
LIFE-011 Provide local, redacted diagnostic timeline for network changes and recovery with user-reviewed export.

## Acceptance scenarios
A. Sleep 1 min and 30 min then wake: server eventually returns to Ready or accurately reports Degraded, with bounded CPU and no leaked listeners.
B. Switch Wi-Fi to Ethernet, change DHCP IP, disable/re-enable network: verify SSDP advertisements and HTTP URLs match active permitted interface.
C. Enable/disable VPN and change default route: no unauthorized multicast advertising or admin exposure.
D. Unmount/remount NAS or removable drive mid-scan and mid-stream: no source modification, config loss, index corruption or runaway retry.
E. SIGKILL during index update: recover or offer safe rebuild; media checksums unchanged.
F. Repeat 100 lifecycle cycles in automation/simulation plus real macOS integration trials; measure recovery time and failures, thresholds to be calibrated.
G. UI quits/reopens while headless server active: service behavior follows documented user preference, not accidental GUI process lifetime.

## Security/UX
Never disable firewall, expose WAN, modify router ports or follow untrusted network discovery automatically. Status distinguishes server failure, network unavailable, and client-specific incompatibility.

## Open decisions
macOS service mechanism, login-start permissions, sleep-prevention policy (default no prevention), interface fallback, reconnect SLO and exact OS support.
