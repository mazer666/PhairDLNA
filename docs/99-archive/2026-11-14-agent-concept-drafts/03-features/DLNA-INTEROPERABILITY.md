# F-DLNA — discovery, browse and interoperability

Status: Draft. Owner approval pending.

## Functional
DLNA-001 Advertise valid UPnP MediaServer device/services via SSDP on configured interfaces; respond to valid M-SEARCH.
DLNA-002 Provide well-formed device/service descriptions and ContentDirectory/ConnectionManager behavior required by supported clients.
DLNA-003 Browse containers/items with stable IDs, pagination, sorting where advertised, and escaped DIDL-Lite.
DLNA-004 Expose protocolInfo and resource URLs consistent with actual HTTP response.
DLNA-005 Log and diagnose client-specific compatibility issues without personal-data leakage.
DLNA-006 Graceful shutdown announces departure where possible; restart restores discoverability.

## Acceptance
- Reference client discovers server within proposed 10 seconds on documented single-subnet LAN; repeat 30 trials and record p95.
- Protocol fixture suite validates SSDP headers, SOAP actions, XML escaping, object IDs, pagination, malformed requests and unsupported actions.
- Test at least 3 independently implemented DLNA clients before first stable; maintain matrix with exact models, firmware, media and outcome.
- No claim of DLNA certification without actual certification.
- Discovery failures must be distinguishable from playback failures in diagnostics.

## Security
No router port mapping; restrict interface binding; bounded parsing and requests; test malformed multicast/SOAP payloads.
