# Acceptance and evidence matrix — draft

| Requirement | Scenario | Evidence required | Gate |
|---|---|---|---|
| UX-001 | Novice first run | recorded tasks, n>=10, success/time | UX |
| UX-002 | All server states | UI screenshots and state test | UX |
| DLNA-001 | LAN discovery | 30 trials, p95 timing and packet trace | Interop |
| DLNA-003 | Browse pagination/escaping | protocol fixtures and negative tests | Interop |
| STR-002 | HTTP byte ranges | automated vectors incl. 206/416 | Protocol |
| LIB-004 | Crash/index corruption | SIGKILL and recovery logs | Reliability |
| LIB-005 | Traversal/symlink | negative security tests | Security |
| LIB-006 | 100k item scan | reproducible dataset and memory profile | Performance |
| SEC-001 | No public default exposure | port/interface and IGD checks | Security |
| MAC-006 | Accessibility | keyboard/VoiceOver audit | UX |
| REL-001 | No source modification | media checksums before/after faults | Reliability |

For every item: record test ID, hardware, OS, client model/firmware, expected and observed outcome, artifacts and commit. Status must be Not run / Pass / Fail / Blocked; never mark Pass based on design intent.
