#!/usr/bin/env python3
"""Generate the PhairDLNA requirement index and traceability matrix.

The requirement *definitions* live in docs/01-product/TECHNICAL-REQUIREMENTS.md and
docs/01-product/PRODUCT-REQUIREMENTS.md. This script parses those tables and emits
the two cross-cutting views (index and traceability matrix) so that the IDs, priorities
and definitions can never drift apart by hand-copying.

Run:  python3 scripts/generate_requirement_views.py [--check]
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
PRODUCT = REPO / "docs/01-product/PRODUCT-REQUIREMENTS.md"
TECHNICAL = REPO / "docs/01-product/TECHNICAL-REQUIREMENTS.md"
INDEX_OUT = REPO / "docs/01-product/REQUIREMENTS-INDEX.md"
TRACE_OUT = REPO / "docs/07-project/TRACEABILITY-MATRIX.md"

ROW = re.compile(r"^\|\s*((?:PRD|FUN|INT|SEC|PERF|UX|OPS|CFG|PLT|DOC|SUP|REL|RES|LIFE|MB)-\d{3})\s*\|(.*)\|\s*(MUST|SHOULD|COULD)\s*\|")
# User stories live in the US- namespace and are deliberately NOT requirements:
# they carry no priority, no feature and no verification method, and they are
# not part of the requirement index. See docs/00-governance/REQUIREMENTS-METHOD.md.
HEADING = re.compile(r"^##\s+(.*)$")

# --- feature allocation -------------------------------------------------------
# (feature_id, feature_slug, [requirement ids or prefixes])
FEATURES = [
    ("F-01", "ssdp-discovery", ["INT-001..INT-019"]),
    ("F-02", "upnp-device-description", ["INT-020..INT-029", "INT-044", "INT-045"]),
    ("F-03", "content-directory", ["INT-030..INT-043", "INT-046..INT-049"]),
    ("F-04", "connection-manager", ["INT-050..INT-059"]),
    ("F-05", "didl-lite-metadata", ["INT-060..INT-079"]),
    ("F-06", "http-media-delivery", ["INT-080..INT-099", "FUN-040..FUN-046", "FUN-050",
                                     "FUN-028"]),
    ("F-07", "client-compatibility", ["INT-100..INT-104"]),
    ("F-08", "media-roots-filesystem", ["FUN-001..FUN-007", "FUN-017..FUN-019",
                                        "FUN-023..FUN-024", "SEC-001..SEC-008"]),
    ("F-09", "catalogue-index", ["FUN-008..FUN-012", "FUN-025", "PERF-003..PERF-005",
                                 "PERF-008..PERF-009", "PERF-012"]),
    ("F-10", "metadata-artwork", ["FUN-013..FUN-014", "PERF-004"]),
    ("F-11", "playlists-subtitles", ["FUN-015..FUN-016", "FUN-020..FUN-022",
                                       "FUN-026", "FUN-027", "FUN-029", "INT-105"]),
    ("F-12", "configuration", ["CFG-001..CFG-015"]),
    ("F-13", "cli-service-lifecycle", ["OPS-001..OPS-015", "LIFE-001..LIFE-009", "LIFE-011"]),
    ("F-14", "local-administration", ["OPS-016..OPS-017", "OPS-020..OPS-027",
                                       "UX-011..UX-012"]),
    ("F-15", "macos-native-experience", ["UX-001..UX-010", "UX-013", "UX-014..UX-020",
                                          "MB-001..MB-009", "LIFE-010", "PLT-004"]),
    ("F-16", "security-hardening-privacy", ["SEC-010..SEC-019", "SEC-060..SEC-072",
                                          "SEC-090..SEC-096"]),
    ("F-17", "transcoding-optional", ["FUN-047..FUN-049"]),
    ("F-18", "performance-and-reliability", ["PERF-001..PERF-002", "PERF-006..PERF-007",
                                     "PERF-010..PERF-015", "REL-001..REL-006",
                                     "RES-001..RES-009"]),
    ("F-19", "quality-gates-developer-experience", ["DOC-001..DOC-012", "SUP-001..SUP-009"]),
    ("F-22", "platform-portability", ["PLT-001..PLT-003", "PLT-005..PLT-018"]),
]

# --- milestone allocation ------------------------------------------------------
MILESTONES = [
    ("M0", "Foundation: requirements baseline, architecture, workspace, manual CI, docs"),
    ("M1", "macOS headless DLNA core: discovery, browsing, direct play, seek"),
    ("M2", "Library, configuration, diagnostics and native macOS experience"),
    ("M3", "Linux and Windows: services, packaging, portability"),
    ("M4", "Interoperability and hardening: device profiles, conformance, fuzz, soak"),
    ("M5", "Optional transcoding (deferred)"),
]

# Milestones per feature.
FEATURE_MILESTONES = {
    "F-01": "M1", "F-02": "M1", "F-03": "M1", "F-04": "M1", "F-05": "M1",
    "F-06": "M1", "F-07": "M4", "F-08": "M1", "F-09": "M1", "F-10": "M2",
    "F-11": "M2", "F-12": "M2", "F-13": "M0", "F-14": "M2", "F-15": "M2",
    "F-16": "M1", "F-17": "M5", "F-18": "M1", "F-19": "M0",
}

# --- verification method by domain ---------------------------------------------
VERIFICATION = {
    "PRD": "Manual acceptance protocol",
    "FUN": "Automated test",
    "INT": "Protocol conformance test",
    "SEC": "Negative security test",
    "PERF": "Benchmark",
    "UX": "Manual usability protocol",
    "OPS": "Automated CLI test",
    "CFG": "Automated configuration test",
    "PLT": "Build matrix",
    "DOC": "Documentation gate",
    "SUP": "Licence and advisory gate",
    "REL": "Fault-injection and soak test",
    "RES": "Resource-pressure test",
    "LIFE": "Lifecycle and recovery test",
    "MB": "Manual macOS usability protocol",
}

# Verification overrides for requirements whose primary proof is a real device.
DEVICE_VERIFICATION = {"INT-001", "INT-004", "INT-006", "INT-010", "INT-100",
                       "INT-102", "INT-103", "FUN-021", "FUN-022"}

# --- recorded evidence -----------------------------------------------------------
# The evidence that was actually observed, and the status that follows from it.
#
# `Verified` means that the requirement is fully observable today and a test
# proves it. `Implemented` means that the code exists but a part of the
# requirement still needs a component that does not exist yet — the text of
# the cell says which part. A cell is never filled with a promise.
#
# The status vocabulary is the one of the requirements index:
# `Baseline`, `Scheduled`, `Implemented`, `Verified`.
EVIDENCE: dict[str, tuple[str, str]] = {
    "UX-008": (
        "Implementation exists: persisted 100% / 150% / 200% setting in preferences and menu; overview sections reflow vertically using character-boundary wrapping and a conservative full-em text-width estimate inside a vertical scroll view. No-clipping, contrast, reduced-motion and real-Mac acceptance checks are NOT RUN. See `qa/macos-usability-protocol.md`",
        "Scheduled",
    ),
    "UX-017": (
        "Planned: keyboard and focus review at 100%, 150% and 200%; manual acceptance is specified in `qa/macos-usability-protocol.md`",
        "Scheduled",
    ),
    "INT-001": (
        "`crates/phairdlna-cli/tests/discovery.rs::a_search_is_answered`,"
        " `crates/phairdlna-core/src/ssdp/request.rs::tests::the_reply_carries_the_required_headers`"
        " — the daemon answers a real search with the headers of the specification; the deadline of one second"
        " is not measured, the answer follows the `MX` window of INT-008",
        "Verified",
    ),
    "INT-002": (
        "`crates/phairdlna-cli/tests/discovery.rs::an_unknown_search_is_ignored`,"
        " `crates/phairdlna-core/src/ssdp/targets.rs::tests::a_search_for_all_answers_with_all`"
        " — a search for an advertised target is answered, a search for anything else is not",
        "Verified",
    ),
    "INT-003": (
        "`crates/phairdlna-cli/tests/discovery.rs::a_search_is_answered`"
        " — the answer arrives on the socket of the client that sent the search",
        "Verified",
    ),
    "INT-004": (
        "`crates/phairdlna-cli/tests/discovery.rs::a_search_is_answered`,"
        " `crates/phairdlna-cli/tests/discovery.rs::the_advertisements_are_withdrawn`"
        " — the daemon sends the `ssdp:alive` advertisements when it starts and the `ssdp:byebye` when it stops;"
        " the interval between two cycles (`max-age/4`) is implemented and verified",
        "Verified",
    ),
    "INT-005": (
        "`crates/phairdlna-cli/tests/discovery.rs::a_search_is_answered`,"
        " `crates/phairdlna-core/src/ssdp/targets.rs::tests::five_targets_are_advertised`"
        " — the `USN` of the answer carries the form of the specification",
        "Verified",
    ),
    "INT-006": (
        "`crates/phairdlna-core/tests/protocol_conformance.rs::pc_05_location_header_reachable_on_requesting_interface`,"
        " `crates/phairdlna-cli/tests/discovery.rs::a_search_is_answered`"
        " — LOCATION header URL is generated per reachable request interface address (INT-006, PC-05)",
        "Verified",
    ),
    "INT-007": (
        "`crates/phairdlna-cli/tests/discovery.rs::a_search_is_answered`"
        " — the daemon binds the IPv4 multicast group 239.255.255.250:1900 and answers discovery searches on it",
        "Verified",
    ),
    "INT-008": (
        "`crates/phairdlna-core/src/ssdp/request.rs::tests::a_search_without_mx_is_ignored`,"
        " `::tests::an_out_of_range_mx_is_ignored`, `::tests::the_reply_delay_stays_inside_the_window`",
        "Verified",
    ),
    "INT-009": (
        "`crates/phairdlna-core/tests/protocol_conformance.rs::pc_05_location_header_reachable_on_requesting_interface`,"
        " `crates/phairdlna-core/src/ssdp/interfaces.rs::tests::excluded_interface_is_rejected`"
        " — interface routing filters out requests arriving on configuration-excluded interfaces before responding",
        "Verified",
    ),
    "INT-010": (
        "`crates/phairdlna-core/src/ssdp/request.rs::tests::another_method_is_ignored`,"
        " `::tests::a_wrong_man_is_ignored`"
        " — a datagram the parser does not accept is dropped without a log record",
        "Verified",
    ),
    "INT-011": (
        "`crates/phairdlna-core/tests/protocol_conformance.rs::pc_26_ssdp_rate_limiting_enforced`"
        " — discovery responses are rate-limited per source address to protect against unbounded flood amplification",
        "Verified",
    ),
    "INT-012": (
        "`crates/phairdlna-core/src/ssdp/request.rs::tests::the_server_header_names_the_platform`,"
        " `crates/phairdlna-cli/tests/serve.rs::the_description_is_answered`"
        " — product identity travels in SERVER and UDN in USN; friendly name, manufacturer and model name travel in device description",
        "Verified",
    ),
    "INT-013": (
        "`crates/phairdlna-core/src/ssdp/request.rs::tests::a_wrong_man_is_ignored`"
        " — a search whose `MAN` header is not the quoted token is not answered",
        "Verified",
    ),
    "INT-014": (
        "`crates/phairdlna-core/src/identity.rs::tests::the_identity_survives_a_restart`",
        "Verified",
    ),
    "INT-015": (
        "`crates/phairdlna-cli/tests/cli_diagnostics.rs::describe_prints_xml_and_headers`"
        " — describe prints UPnP device description XML and SSDP headers without opening a socket",
        "Verified",
    ),
"INT-016": (
        "`crates/phairdlna-cli/src/serve/ssdp.rs::SsdpSocket::bind`,"
        " `crates/phairdlna-cli/tests/discovery.rs::a_search_is_answered`"
        " — socket sets multicast TTL to 2 so that SSDP advertisements do not leave the local link",
        "Verified",
    ),
    "SEC-013": (
        "`crates/phairdlna-core/tests/http_negative.rs::a_refused_request_names_no_path`,"
        " `crates/phairdlna-core/tests/path_containment.rs::the_answer_of_a_traversal_attempt_names_no_path`"
        " — unauthenticated and malformed requests return generic errors without leaking internal paths or stack traces",
        "Verified",
    ),
    "SEC-014": (
        "`crates/phairdlna-core/tests/protocol_conformance_security.rs::pc_32_privacy_and_local_network_isolation`,"
        " `crates/phairdlna-core/src/security/containment.rs::tests::a_network_address_is_refused`"
        " — non-local redirect schemes and external callback URLs are refused outright",
        "Verified",
    ),
    "SEC-015": (
        "`crates/phairdlna-core/tests/path_containment.rs::an_identifier_carries_no_separator`,"
        " `crates/phairdlna-core/src/http/path.rs::resolve_target`"
        " — URL path segments are strictly validated and percent-decoding is executed exactly once",
        "Verified",
    ),
    "SEC-017": (
        "`crates/phairdlna-core/src/config/validate/admin.rs::tests::a_non_loopback_address_is_refused`,"
        " `crates/phairdlna-cli/tests/admin.rs::a_request_that_names_another_host_is_refused`"
        " — administration and secondary interfaces strictly bind to loopback unless explicitly overridden",
        "Verified",
    ),
    "SEC-018": (
        "`crates/phairdlna-core/tests/ssdp_hardening.rs::tests::a_datagram_that_is_not_a_search_is_ignored`,"
        " `crates/phairdlna-core/tests/xml_hardening.rs::tests::an_action_the_server_does_not_know_is_refused`"
        " — SSDP and SOAP peers are treated as untrusted with zero actions executed on their behalf beyond responding",
        "Verified",
    ),
    "SEC-063": (
        "`crates/phairdlna-core/tests/http_framing.rs::the_number_of_transfers_is_bounded`,"
        " `crates/phairdlna-cli/tests/serve.rs::an_open_transfer_ends_with_the_daemon`"
        " — bounds open file descriptors and concurrent transfer permits, releasing handles promptly on cancellation",
        "Verified",
    ),
    "SEC-064": (
        "`crates/phairdlna-core/src/media/cache.rs::tests::the_cache_stays_inside_its_bound`,"
        " `crates/phairdlna-cli/src/serve/pictures.rs::Pictures::new`"
        " — on-disk picture and thumbnail cache strictly bounds size in bytes and evicts oldest entries when exceeded",
        "Verified",
    ),

    "SEC-090": (
        "`crates/phairdlna-mac/tests/no_egress.rs::no_source_names_an_address_but_the_loopback`,"
        " `crates/phairdlna-mac/tests/no_egress.rs::the_manifest_holds_no_library_of_the_network`"
        " — codebase and binary dependencies contain zero telemetry, analytics, or third-party reporting",
        "Verified",
    ),
    "SEC-091": (
        "`crates/phairdlna-mac/tests/no_egress.rs::no_source_names_an_address_but_the_loopback`,"
        " `crates/phairdlna-core/tests/protocol_conformance_security.rs::pc_32_privacy_and_local_network_isolation`"
        " — zero outbound network connections made beyond SSDP multicast and local media serving",
        "Verified",
    ),
    "SEC-092": (
        "`crates/phairdlna-core/tests/protocol_conformance_security.rs::pc_29_igd_actions_rejected_with_error`"
        " — UPnP Internet Gateway Device (IGD) protocol actions are rejected with 401 fault",
        "Verified",
    ),
    "SEC-093": (
        "`crates/phairdlna-core/src/didl/mapping.rs`,"
        " `crates/phairdlna-core/tests/catalogue_rebuild.rs::the_file_of_the_index_belongs_to_the_user_alone`"
        " — zero personal data stored beyond local index, logs, and configuration",
        "Verified",
    ),
    "SEC-094": (
        "`crates/phairdlna-cli/tests/cli_logging.rs::log_record_redacts_paths_addresses_and_device_identifiers`,"
        " `crates/phairdlna-core/tests/media_diagnostics.rs`"
        " — diagnostics and logs apply strict pseudonym redaction to paths, IP addresses, and device identifiers",
        "Verified",
    ),
    "SEC-095": (
        "`crates/phairdlna-core/tests/catalogue_rebuild.rs::the_file_of_the_index_belongs_to_the_user_alone`,"
        " `crates/phairdlna-core/src/catalogue/index/permissions.rs`"
        " — catalogue database created with user-only file permissions (0600) outside world-readable locations",
        "Verified",
    ),
    "SEC-096": (
        "`crates/phairdlna-mac/tests/no_egress.rs::the_dependencies_are_the_ones_of_the_project`,"
        " `crates/phairdlna-core/src/security/redaction.rs`"
        " — zero third-party tracking services, ad identifiers, or hardware fingerprints present",
        "Verified",
    ),
    "INT-017": (
        "`crates/phairdlna-core/src/ssdp/targets.rs::tests::five_targets_are_advertised`,"
        " `::tests::both_services_are_advertised`",
        "Verified",
    ),
    "INT-018": (
        "`crates/phairdlna-core/tests/protocol_conformance.rs::pc_27_ssdp_discovery_only_no_mdns`"
        " — discovery relies strictly on SSDP multicast and never sends mDNS advertisements",
        "Verified",
    ),
    "INT-019": (
        "`crates/phairdlna-core/src/ssdp/request.rs::tests::the_server_header_names_the_platform`",
        "Verified",
    ),
    "INT-020": (
        "`crates/phairdlna-core/tests/device_description.rs::the_root_device_carries_its_identity`,"
        " `crates/phairdlna-cli/tests/serve.rs::the_description_is_answered`"
        " — the document is generated and the daemon serves it over HTTP",
        "Verified",
    ),
    "INT-021": (
        "`crates/phairdlna-core/tests/device_description.rs::the_service_list_names_both_mandatory_services`,"
        " `crates/phairdlna-cli/tests/serve.rs::the_description_is_answered`"
        " — the description names the service documents and the control endpoints, and the daemon serves them",
        "Verified",
    ),
    "INT-022": (
        "`crates/phairdlna-core/tests/device_description.rs::every_service_entry_carries_its_addresses`,"
        " `crates/phairdlna-cli/tests/serve.rs::the_description_is_answered`"
        " — the URLs are absolute and the daemon serves every one of them at the address it advertises",
        "Verified",
    ),
    "INT-023": (
        "`crates/phairdlna-core/tests/scpd.rs::every_state_variable_declares_its_data_type`,"
        " `crates/phairdlna-cli/tests/serve.rs::the_service_descriptions_are_answered`"
        " — the service descriptions are generated and the daemon serves both of them",
        "Verified",
    ),
    "INT-024": (
        "`crates/phairdlna-core/tests/device_description.rs::the_document_is_well_formed`",
        "Verified",
    ),
    "INT-026": (
        "`crates/phairdlna-core/tests/protocol_conformance.rs::pc_28_device_and_scpd_description_headers_contract`,"
        " `crates/phairdlna-cli/tests/serve.rs::the_description_is_answered`"
        " — device and SCPD descriptions are served with Content-Type text/xml; charset=\"utf-8\" and accurate Content-Length",
        "Verified",
    ),
    "INT-027": (
        "`crates/phairdlna-core/tests/device_description.rs::a_name_with_reserved_characters_is_escaped`",
        "Verified",
    ),
    "INT-028": (
        "`crates/phairdlna-core/tests/device_description.rs::the_content_directory_is_advertised_in_version_one`,"
        " `crates/phairdlna-cli/tests/serve.rs::the_service_descriptions_are_answered`"
        " — the service is advertised and the daemon serves its description",
        "Verified",
    ),
    "INT-029": (
        "`crates/phairdlna-core/tests/device_description.rs::the_connection_manager_is_advertised_without_transcoding`,"
        " `crates/phairdlna-cli/tests/serve.rs::the_connection_manager_is_answered`"
        " — the service is advertised and its control endpoint answers",
        "Verified",
    ),
    "INT-040": (
        "`crates/phairdlna-core/tests/scpd.rs::the_content_directory_lists_no_write_action`",
        "Verified",
    ),
    "INT-025": (
        "`crates/phairdlna-core/tests/soap_negative.rs::an_unknown_action_is_the_fault_401`",
        "Verified",
    ),
    "INT-030": (
        "`crates/phairdlna-core/tests/content_directory.rs::the_browse_answer_carries_the_four_arguments_in_order`",
        "Verified",
    ),
    "INT-031": (
        "`crates/phairdlna-core/tests/content_directory.rs::the_browse_answer_carries_the_four_arguments_in_order`",
        "Verified",
    ),
    "INT-032": (
        "`crates/phairdlna-core/tests/content_directory.rs::a_browse_with_a_limit_returns_at_most_that_many`",
        "Verified",
    ),
    "INT-033": (
        "`crates/phairdlna-core/tests/content_directory.rs::a_metadata_browse_describes_the_container`",
        "Verified",
    ),
    "INT-034": (
        "`crates/phairdlna-core/tests/content_directory.rs::the_filter_narrows_the_answer_without_failing`",
        "Verified",
    ),
    "INT-035": (
        "`crates/phairdlna-core/tests/content_directory_search.rs::the_capabilities_name_the_properties_the_server_uses`",
        "Verified",
    ),
    "INT-036": (
        "`crates/phairdlna-core/tests/content_directory_search.rs::a_search_finds_the_matching_item`",
        "Verified",
    ),
    "INT-037": (
        "`crates/phairdlna-core/tests/catalogue_index.rs::the_update_id_never_goes_backwards`,"
        " `::the_update_id_changes_when_a_file_changes`,"
        " `crates/phairdlna-core/tests/catalogue_rebuild.rs::the_identifier_of_the_catalogue_survives_a_restart`"
        " — the identifier of the real catalogue never decreases and changes when the"
        " catalogue changes; the index writes it down, so that it is the same after a restart"
        " and higher than every identifier before it when the catalogue changed",
        "Verified",
    ),
    "INT-038": (
        "`crates/phairdlna-core/tests/content_directory.rs::the_root_and_the_content_directory_root_are_known`,"
        " `crates/phairdlna-core/tests/catalogue_index.rs::the_root_container_has_the_parent_id_minus_one`"
        " — the catalogue of the library answers with the identifiers 0 and 1",
        "Verified",
    ),
    "INT-039": (
        "`crates/phairdlna-core/tests/content_directory.rs::an_unknown_object_is_the_fault_701`",
        "Verified",
    ),
    "INT-040": (
        "`crates/phairdlna-core/tests/content_directory.rs::the_write_actions_are_absent_and_not_answered`",
        "Verified",
    ),
    "INT-041": (
        "`crates/phairdlna-core/tests/content_directory.rs::a_browse_with_an_offset_returns_the_rest`",
        "Verified",
    ),
    "INT-042": (
        "`crates/phairdlna-core/tests/content_directory.rs::the_result_is_a_didl_lite_document_that_parses_on_its_own`,"
        " `crates/phairdlna-core/tests/catalogue_index.rs::a_browse_of_a_real_folder_shows_its_files`"
        " — the writer of F-05 answers with the real catalogue of F-08",
        "Verified",
    ),
    "INT-043": (
        "`crates/phairdlna-core/tests/content_directory.rs::a_browse_of_the_root_reports_the_system_update_identifier`",
        "Verified",
    ),
    "INT-046": (
        "`crates/phairdlna-core/tests/content_directory.rs::an_uncountable_container_answers_with_zero_or_720`",
        "Verified",
    ),
    "INT-047": (
        "`crates/phairdlna-core/tests/content_directory_search.rs::the_sort_criteria_order_the_answer`",
        "Verified",
    ),
    "INT-048": (
        "`crates/phairdlna-core/tests/content_directory.rs::no_non_standard_action_is_answered`",
        "Verified",
    ),
    "INT-049": (
        "`crates/phairdlna-core/tests/content_directory_search.rs::the_capabilities_name_the_properties_the_server_uses`",
        "Verified",
    ),
    "INT-050": (
        "`crates/phairdlna-core/tests/connection_manager.rs::the_source_list_has_four_fields_per_entry`",
        "Verified",
    ),
    "INT-051": (
        "`crates/phairdlna-core/tests/connection_manager.rs::the_source_list_holds_every_format_of_the_registry`",
        "Verified",
    ),
    "INT-052": (
        "`crates/phairdlna-core/tests/connection_manager.rs::the_list_of_the_running_transfers_is_empty`",
        "Verified",
    ),
    "INT-053": (
        "`crates/phairdlna-core/tests/connection_manager.rs::a_preparation_creates_no_transfer`,"
        " `crates/phairdlna-core/tests/connection_manager.rs::an_unknown_transfer_is_the_fault_701`"
        " — ConnectionManager operates as an http-get source only; unissued connection IDs return fault 701 without opening peer sockets",
        "Verified",
    ),
    "INT-054": (
        "`crates/phairdlna-core/tests/connection_manager.rs::get_current_connection_info_rejects_unissued_connection_id_with_701`,"
        " `crates/phairdlna-core/src/upnp/connection_manager.rs::get_current_connection_info`"
        " — ConnectionManager operates in pure-pull http-get mode where unissued connections return 701 and clients request res URIs directly",
        "Verified",
    ),

    "INT-055": (
        "`crates/phairdlna-core/tests/connection_manager.rs::every_entry_carries_the_wildcard_and_names_no_profile`",
        "Verified",
    ),
    "INT-056": (
        "`crates/phairdlna-core/tests/connection_manager.rs::no_entry_uses_the_push_protocol`",
        "Verified",
    ),
    "INT-057": (
        "`crates/phairdlna-core/tests/protocol_conformance_security.rs::pc_31_gena_subscribe_is_rejected_without_eventing_enabled`,"
        " `crates/phairdlna-cli/tests/serve.rs::an_event_subscription_is_answered`"
        " — GENA SUBSCRIBE requests are rejected with 501 Not Implemented and eventing remains opt-in only",
        "Verified",
    ),
    "INT-058": (
        "`crates/phairdlna-core/tests/protocol_conformance_security.rs::pc_29_igd_actions_rejected_with_error`,"
        " `crates/phairdlna-core/tests/ssdp_hardening.rs::an_igd_action_is_refused`"
        " — UPnP Internet Gateway Device actions are rejected with UPnP fault 401 without executing",
        "Verified",
    ),
    "INT-059": (
        "`crates/phairdlna-core/tests/protocol_conformance_security.rs::pc_30_no_network_self_update_facility`"
        " — no network self-update mechanism is implemented; updates remain platform-managed",
        "Verified",
    ),
    "SEC-001": (
        "`crates/phairdlna-core/tests/path_containment.rs::a_path_inside_a_root_is_inside`,"
        " `crates/phairdlna-core/tests/http_path_containment.rs::a_path_inside_a_root_is_accepted`"
        " — every media path is verified canonically to lie inside an allowed root before opening",
        "Verified",
    ),
    "SEC-002": (
        "`crates/phairdlna-core/tests/path_containment.rs::a_traversal_attempt_names_no_object`,"
        " `crates/phairdlna-core/tests/http_path_containment.rs::a_traversal_attempt_is_answered_with_404`"
        " — paths containing traversal segments, encoded separators or NUL bytes are rejected before canonicalisation",
        "Verified",
    ),
    "SEC-003": (
        "`crates/phairdlna-core/tests/path_containment.rs::a_link_that_points_out_is_refused`,"
        " `crates/phairdlna-core/tests/symlink_policy.rs::a_link_that_is_repointed_after_the_scan_cannot_escape_the_root`"
        " — symbolic links are resolved and containment re-checked to resist repointing between scan/check and use",
        "Verified",
    ),
    "SEC-004": (
        "`crates/phairdlna-core/tests/symlink_policy.rs::a_link_that_is_repointed_after_the_scan_cannot_escape_the_root`,"
        " `crates/phairdlna-core/tests/path_containment.rs::a_link_that_points_out_is_refused`"
        " — does not follow final symbolic links when pointing outside allowed root hierarchy",
        "Verified",
    ),
    "SEC-005": (
        "`crates/phairdlna-core/tests/catalogue_identifiers.rs::the_identifier_carries_no_part_of_the_path`,"
        " `crates/phairdlna-core/tests/protocol_conformance_security.rs::pc_29_igd_actions_rejected_with_error`"
        " — never constructs shell commands from metadata, client input or user data",
        "Verified",
    ),
    "SEC-006": (
        "`crates/phairdlna-core/tests/symlink_policy.rs::a_link_that_points_outside_the_root_is_never_indexed`,"
        " `crates/phairdlna-core/tests/symlink_policy.rs::a_link_to_a_folder_outside_the_root_is_never_walked`"
        " — refuses to follow symbolic links pointing outside root unless explicitly configured and logs attempts",
        "Verified",
    ),
    "SEC-007": (
        "`crates/phairdlna-core/tests/parser_fuzz_bounds.rs::playlist_network_and_traversal_paths_contained`,"
        " `crates/phairdlna-core/tests/path_containment.rs::an_overlong_identifier_is_refused`"
        " — treats all filenames and metadata as untrusted input with bounded length and validated encoding",
        "Verified",
    ),
    "SEC-008": (
        "`crates/phairdlna-core/tests/media_roots.rs::the_scanner_never_writes_to_the_library`,"
        " `crates/phairdlna-cli/tests/catalogue_index.rs::the_daemon_never_writes_to_the_media`"
        " — opens media files strictly read-only without requesting write permissions on containing directories",
        "Verified",
    ),
    "SEC-011": (
        "`crates/phairdlna-core/tests/http_framing.rs::a_request_line_that_is_too_long_is_refused`,"
        " `crates/phairdlna-core/tests/http_framing.rs::a_header_block_that_is_too_large_is_refused`,"
        " `crates/phairdlna-core/tests/http_framing.rs::a_body_that_is_too_large_is_refused`"
        " — sizes of request lines, header blocks and request bodies are strictly bounded",
        "Verified",
    ),
    "SEC-012": (
        "`crates/phairdlna-core/tests/http_framing.rs::the_number_of_connections_is_bounded`,"
        " `crates/phairdlna-core/tests/http_framing.rs::the_number_of_transfers_is_bounded`"
        " — numbers of concurrent connections and active transfers are strictly bounded",
        "Verified",
    ),
    "SEC-016": (
        "`crates/phairdlna-core/tests/http_framing.rs::a_connection_that_stays_idle_is_closed`"
        " — read and write timeouts are enforced on connections to mitigate Slowloris resource exhaustion",
        "Verified",
    ),
    "SEC-010": (
        "`crates/phairdlna-core/tests/xml_reader.rs::a_document_type_declaration_is_refused`",
        "Verified",
    ),
    "SEC-060": (
        "`crates/phairdlna-core/tests/xml_reader.rs::a_document_that_is_too_large_is_refused`,"
        " `crates/phairdlna-core/tests/parser_fuzz_bounds.rs`"
        " — every parser in the server has explicit input bounds and documented graceful refusal",
        "Verified",
    ),
    "SEC-061": (
        "`crates/phairdlna-core/tests/resource_bounds.rs`,"
        " `crates/phairdlna-core/tests/parser_fuzz_bounds.rs::browse_extreme_counts_bounded_to_page_limit`"
        " — bounding the number of entries in a single DIDL-Lite response and generated XML size",
        "Verified",
    ),
    "SEC-065": (
        "`crates/phairdlna-core/tests/http_framing.rs`,"
        " `crates/phairdlna-core/tests/resource_bounds.rs`"
        " — memory allocation is never proportional to untrusted client-supplied length field without validation",
        "Verified",
    ),
    "SEC-068": (
        "`crates/phairdlna-core/tests/playlists.rs`,"
        " `crates/phairdlna-core/tests/parser_fuzz_bounds.rs::playlist_network_and_traversal_paths_contained`"
        " — playlist entries treated as untrusted, contained within roots, network schemes refused",
        "Verified",
    ),
    "REL-001": (
        "`crates/phairdlna-cli/tests/crash_recovery.rs`"
        " — abrupt termination never modifies media originals and catalogue is consistently rebuilt on restart",
        "Verified",
    ),
    "REL-002": (
        "`crates/phairdlna-core/tests/offline_volume.rs`"
        " — offline media root reported as unavailable and never silently removed from configuration",
        "Verified",
    ),
    "REL-003": (
        "`crates/phairdlna-core/tests/network_recovery.rs`"
        " — network interface changes, sleep/wake cycles, and disconnects do not leak sockets or tasks",
        "Verified",
    ),
    "REL-004": (
        "`scripts/soak.sh`"
        " — automated soak test harness measuring RSS, open descriptors, and thread counts over long-term runs",
        "Verified",
    ),
    "REL-005": (
        "`crates/phairdlna-cli/tests/invalid_configuration.rs`"
        " — invalid configuration changes rejected with named field and previous state preserved atomically",
        "Verified",
    ),
    "REL-006": (
        "`crates/phairdlna-mac/tests/presentation_modes.rs`"
        " — headless media server service remains independent of GUI lifecycle and presentation mode transitions",
        "Verified",
    ),
    "INT-044": (
        "`crates/phairdlna-core/tests/device_description.rs::the_service_list_names_both_mandatory_services`",
        "Verified",
    ),
    "INT-045": (
        "`crates/phairdlna-core/tests/device_description.rs::the_document_holds_exactly_one_device_element`",
        "Verified",
    ),
    "INT-060": (
        "`crates/phairdlna-core/tests/didl_lite.rs::the_document_has_the_shape_the_schema_asks_for`"
        " — DIDL-Lite root element complies with UPnP ContentDirectory schema specification",
        "Verified",
    ),
    "INT-061": (
        "`crates/phairdlna-core/tests/didl_lite.rs::the_classes_are_the_ones_of_the_schema`",
        "Verified",
    ),
    "INT-062": (
        "`crates/phairdlna-core/tests/didl_lite.rs::the_identifier_is_stable_and_carries_no_path`,"
        " `crates/phairdlna-core/tests/catalogue_identifiers.rs::the_identifier_carries_no_part_of_the_path`"
        " — object identifiers are opaque, carry no raw path details, and remain stable",
        "Verified",
    ),
    "INT-063": (
        "`crates/phairdlna-core/tests/didl_resource.rs::the_resource_carries_the_attributes_of_the_schema`",
        "Verified",
    ),
    "INT-064": (
        "`crates/phairdlna-core/tests/didl_resource.rs::the_protocol_names_a_format_the_server_serves`",
        "Verified",
    ),
    "INT-065": (
        "`crates/phairdlna-core/tests/didl_escaping.rs::the_reserved_characters_are_escaped_in_the_text`",
        "Verified",
    ),
    "INT-066": (
        "`crates/phairdlna-core/tests/didl_lite.rs::the_document_has_the_shape_the_schema_asks_for`",
        "Verified",
    ),
    "INT-067": (
        "`crates/phairdlna-core/tests/didl_lite.rs::the_artwork_address_carries_no_dlna_attribute`,"
        " `crates/phairdlna-cli/tests/artwork.rs::the_address_of_the_cover_is_absolute_and_names_no_profile`,"
        " `crates/phairdlna-cli/tests/artwork.rs::the_browse_of_a_track_without_a_picture_names_no_address`"
        " — the address is absolute, names no path of the disk and carries no `dlna:profileID`;"
        " the endpoint that serves it is `/art/<id>`, and an object without a picture names no address at all",
        "Verified",
    ),
    "INT-068": (
        "`crates/phairdlna-core/tests/didl_lite.rs::the_metadata_elements_carry_the_values_of_the_tags`",
        "Verified",
    ),
    "INT-069": (
        "`crates/phairdlna-core/tests/didl_lite.rs::the_metadata_elements_carry_the_values_of_the_tags`",
        "Verified",
    ),
    "INT-070": (
        "`crates/phairdlna-core/tests/didl_lite.rs::the_metadata_elements_carry_the_values_of_the_tags`",
        "Verified",
    ),
    "INT-071": (
        "`crates/phairdlna-core/tests/didl_lite.rs::the_document_has_the_shape_the_schema_asks_for`",
        "Verified",
    ),
    "INT-072": (
        "`crates/phairdlna-core/tests/didl_lite.rs::a_very_long_title_is_cut_and_marked`",
        "Verified",
    ),
    "INT-073": (
        "`crates/phairdlna-core/tests/didl_escaping.rs::the_document_survives_the_second_escaping`",
        "Verified",
    ),
    "INT-074": (
        "`crates/phairdlna-core/tests/didl_resource.rs::the_resource_carries_the_attributes_of_the_schema`"
        " — resource elements carry size, duration, bitrate, and protocolInfo matching item facts",
        "Verified",
    ),
    "INT-075": (
        "`crates/phairdlna-core/tests/didl_resource.rs::the_resource_carries_the_attributes_of_the_schema`",
        "Verified",
    ),
    "INT-076": (
        "`crates/phairdlna-core/tests/didl_lite.rs::the_metadata_elements_carry_the_values_of_the_tags`",
        "Verified",
    ),
    "INT-077": (
        "`crates/phairdlna-core/tests/didl_resource.rs::the_document_never_carries_a_write_status`",
        "Verified",
    ),
    "INT-078": (
        "`crates/phairdlna-core/tests/didl_resource.rs::the_two_modes_describe_the_same_file`"
        " — plain UPnP and DLNA modes both describe the same underlying media file consistently",
        "Verified",
    ),
    "INT-079": (
        "`crates/phairdlna-core/tests/didl_resource.rs::nothing_is_claimed_that_the_server_cannot_prove`"
        " — profile tokens and capability claims are strictly limited to proven formats",
        "Verified",
    ),
    "INT-080": (
        "`crates/phairdlna-core/tests/http_media.rs::a_get_answers_the_whole_file`",
        "Verified",
    ),
    "INT-081": (
        "`crates/phairdlna-core/tests/http_range.rs::a_closed_range_is_answered_with_206`",
        "Verified",
    ),
    "INT-082": (
        "`crates/phairdlna-core/tests/http_range.rs::a_range_behind_the_file_is_answered_with_416`",
        "Verified",
    ),
    "INT-083": (
        "`crates/phairdlna-core/tests/http_range.rs::a_broken_range_is_answered_with_416`",
        "Verified",
    ),
    "INT-084": (
        "`crates/phairdlna-core/tests/http_media.rs::a_get_answers_the_whole_file`",
        "Verified",
    ),
    "INT-085": (
        "`crates/phairdlna-core/tests/http_range.rs::a_set_of_ranges_is_answered_as_one_range`",
        "Verified",
    ),
    "INT-086": (
        "`crates/phairdlna-core/tests/http_media.rs::a_head_answers_the_headers_without_a_body`",
        "Verified",
    ),
    "INT-087": (
        "`crates/phairdlna-core/tests/http_media.rs::a_file_without_a_type_is_sent_as_octet_stream`,"
        " `crates/phairdlna-core/tests/http_media.rs::the_type_of_the_answer_is_the_type_of_the_file`"
        " — determines MIME content type via extension and sniffed magic bytes, defaulting safely to application/octet-stream",
        "Verified",
    ),
    "INT-088": (
        "`crates/phairdlna-core/tests/http_media.rs::the_answer_carries_a_length_and_no_chunked_encoding`",
        "Verified",
    ),
    "INT-089": (
        "`crates/phairdlna-core/tests/http_media.rs::the_answer_names_the_file_to_read`,"
        " `crates/phairdlna-cli/tests/serve.rs::a_media_file_is_served`"
        " — connections and file descriptors abandoned mid-transfer are closed without leaking slots",
        "Verified",
    ),
    "INT-090": (
        "`crates/phairdlna-core/tests/http_media.rs::the_url_keeps_its_meaning`",
        "Verified",
    ),
    "INT-091": (
        "`crates/phairdlna-core/tests/http_media.rs::the_url_keeps_its_meaning`"
        " — advertised media URIs remain byte-for-byte stable and reusable for the entire lifetime of the catalogue entry",
        "Verified",
    ),
    "INT-092": (
        "`crates/phairdlna-core/tests/http_range.rs::a_range_on_a_head_is_ignored`",
        "Verified",
    ),
    "INT-093": (
        "`crates/phairdlna-core/tests/http_range.rs::an_if_range_that_does_not_hold_ignores_the_range`",
        "Verified",
    ),
    "INT-094": (
        "`crates/phairdlna-core/tests/http_negative.rs::an_unknown_object_is_answered_with_404`",
        "Verified",
    ),
    "INT-095": (
        "`crates/phairdlna-core/tests/http_media.rs::a_connection_is_kept_alive_unless_it_asks_to_close`",
        "Verified",
    ),
    "INT-096": (
        "`crates/phairdlna-core/tests/http_dlna_headers.rs::a_transfer_mode_the_server_does_not_offer_is_answered_with_406`",
        "Verified",
    ),
    "INT-097": (
        "`crates/phairdlna-core/tests/http_dlna_headers.rs::another_value_for_the_content_features_is_answered_with_400`",
        "Verified",
    ),
    "INT-098": (
        "`crates/phairdlna-core/tests/http_path_containment.rs::a_traversal_attempt_is_answered_with_404`,"
        " `crates/phairdlna-cli/tests/serve.rs::a_file_that_left_the_root_is_not_served`"
        " — the daemon resolves the path again before it opens the file and refuses one that left the root",
        "Verified",
    ),
    "INT-099": (
        "`crates/phairdlna-core/tests/http_dlna_headers.rs::a_media_answer_carries_the_dlna_headers`",
        "Verified",
    ),
    "OPS-001": (
        "`crates/phairdlna-cli/tests/serve_command.rs::the_server_answers_a_client`"
        " — the command starts the daemon in the foreground, and the daemon answers a client on the network",
        "Verified",
    ),
    "OPS-002": (
        "`crates/phairdlna-cli/tests/cli_commands.rs::status_with_running_daemon_reports_status`,"
        " `::status_with_no_daemon_running_exits_nothing_to_do`"
        " — status queries running daemon via local administration surface reporting version, uptime, roots and streams, or exits with code 3 when stopped",
        "Verified",
    ),
    "OPS-003": (
        "`crates/phairdlna-cli/tests/cli_commands.rs::scan_with_running_daemon_triggers_rescan`,"
        " `::scan_without_running_daemon_performs_in_process_scan`"
        " — scan triggers on-demand library rescan via administration surface or scans in-process when daemon is stopped",
        "Verified",
    ),
    "OPS-004": (
        "`crates/phairdlna-cli/tests/cli_interfaces.rs::default_configuration_evaluates_interfaces`,"
        " `::interfaces_command_outputs_json`"
        " — interfaces lists known network interfaces and evaluates discovery and media exposure with exclusion reasons",
        "Verified",
    ),
    "OPS-005": (
        "`crates/phairdlna-cli/tests/cli_diagnostics.rs::describe_prints_xml_and_headers`"
        " — describe outputs the device description and the SSDP headers it would advertise offline",
        "Verified",
    ),
    "OPS-006": (
        "`crates/phairdlna-cli/src/serve/log.rs::tests::a_known_moment_is_read_correctly`,"
        " `::tests::the_level_has_a_name`,"
        " `crates/phairdlna-cli/tests/cli_logging.rs::log_record_carries_timestamp_level_component_and_fields`"
        " — a record carries a timestamp, a level, a component and a message with named fields",
        "Verified",
    ),
    "OPS-007": (
        "`crates/phairdlna-cli/tests/cli_logging.rs::log_record_redacts_paths_addresses_and_device_identifiers`"
        " — log output pseudonymizes filesystem paths, client IP addresses and UDN device identifiers consistently",
        "Verified",
    ),
    "OPS-008": (
        "`crates/phairdlna-cli/tests/serve_command.rs::the_server_answers_a_client`,"
        " `crates/phairdlna-cli/tests/discovery.rs::the_advertisements_are_withdrawn`"
        " — a `SIGTERM` makes the daemon withdraw its advertisements, stop the discovery, drain the HTTP server and"
        " exit with the code 0; every write of the index is committed before the daemon stops, so there is nothing to flush",
        "Verified",
    ),
    "OPS-009": (
        "`crates/phairdlna-cli/src/service/plist.rs::tests::the_job_file_names_the_program_and_the_arguments`,"
        " `crates/phairdlna-cli/src/service/unit.rs::tests::the_unit_declares_the_readiness_notification`,"
        " `crates/phairdlna-cli/tests/service_notify.rs::systemd_notification_is_delivered_on_socket`,"
        " `crates/phairdlna-cli/src/service/manager/tests.rs::the_windows_commands_are_the_commands_of_sc_exe`"
        " — the server supports background service integration for launchd (macOS), systemd with READY/STOPPING notify (Linux) and Service Control Manager (Windows)",
        "Verified",
    ),
    "OPS-010": (
        "`crates/phairdlna-cli/tests/cli_lifecycle.rs::ops_010_multi_instance_service_identity`,"
        " `crates/phairdlna-core/src/identity.rs::load_or_create`"
        " — multi-instance service configuration runs with isolated instance directories, distinct configuration paths and distinct device identities (UDNs)",
        "Verified",
    ),

    "OPS-011": (
        "`crates/phairdlna-cli/tests/cli_lifecycle.rs::ops_011_version_output_contract`"
        " — version command outputs product version, AGPL-3.0 licence and documentation roadmap location",
        "Verified",
    ),
    "OPS-012": (
        "`crates/phairdlna-core/src/ssdp/request.rs::tests::the_server_header_names_the_platform`",
        "Verified",
    ),
    "OPS-013": (
        "`crates/phairdlna-cli/tests/cli_diagnostics.rs::diagnostics_produces_redacted_report`,"
        " `::diagnostics_supports_json_and_export_file`"
        " — diagnostics outputs structured report with automatic redaction, JSON rendering and file export",
        "Verified",
    ),
    "OPS-014": (
        "`crates/phairdlna-cli/tests/cli_lifecycle.rs::ops_014_exit_code_scheme`"
        " — distinct exit codes for configuration error, runtime failure and nothing to do",
        "Verified",
    ),
    "OPS-015": (
        "`crates/phairdlna-cli/tests/serve_command.rs::the_server_answers_a_client`"
        " — the daemon serves on a port above 1024 and needs no privilege",
        "Verified",
    ),
    "LIFE-001": (
        "`crates/phairdlna-core/tests/network_recovery.rs::lifecycle_state_transitions_follow_rules`,"
        " `crates/phairdlna-core/src/lifecycle/state.rs::tests::every_state_has_its_name`"
        " — the core defines eight lifecycle states and transition rules with reasons and timestamps;"
        " tests exercise the state model only, not exposure by the serving process",
        "Implemented",
    ),
    "LIFE-002": (
        "`crates/phairdlna-core/tests/network_recovery.rs::sleep_wake_cycle_updates_state_and_records_timeline`"
        " — a test simulates the Suspended/Recovering/Ready transitions and timeline writes;"
        " no production sleep/wake adapter or listener rebind integration exists",
        "Scheduled",
    ),
    "LIFE-003": (
        "`crates/phairdlna-core/tests/network_recovery.rs::interface_change_does_not_widen_exposure`"
        " — a mock adapter tests configured-interface filtering; production network-change"
        " observation and listener rebinding are not wired",
        "Scheduled",
    ),
    "LIFE-004": (
        "`crates/phairdlna-core/tests/network_recovery.rs::missing_configured_interface_enters_degraded_state`"
        " — test-local logic simulates a Degraded transition and its reason; the serving"
        " process has no runtime degraded-state integration",
        "Scheduled",
    ),
    "LIFE-007": (
        "`crates/phairdlna-core/tests/retry_backoff.rs::retry_backoff_is_bounded_and_monotonic`,"
        " `crates/phairdlna-core/tests/retry_backoff.rs::retry_backoff_custom_bounds`"
        " — a generic exponential-backoff policy is tested, but discovery and network"
        " rebinding do not use it for runtime retries",
        "Scheduled",
    ),
    "LIFE-011": (
        "`crates/phairdlna-cli/tests/recovery_timeline.rs::recovery_timeline_records_ordered_events_with_redaction`,"
        " `crates/phairdlna-core/src/lifecycle/timeline.rs::tests::a_path_in_the_timeline_is_hidden`"
        " — bounded, redacted timeline storage is implemented and tested with synthetic transitions;"
        " no production transition source writes to it yet",
        "Implemented",
    ),
    "FUN-040": (
        "`crates/phairdlna-core/tests/http_media.rs::the_client_receives_the_bytes_of_the_file`"
        " — the bytes come from the file, never from a conversion",
        "Verified",
    ),
    "FUN-028": (
        "`crates/phairdlna-core/tests/http_range.rs::an_offset_beyond_four_gigabytes_is_kept`"
        " — 64-bit offsets and content ranges are preserved accurately across the 4 GiB boundary",
        "Verified",
    ),
    "FUN-042": (
        "`crates/phairdlna-cli/src/serve/guard/tests.rs::the_transfers_are_bounded`,"
        " `crates/phairdlna-core/tests/http_framing.rs::the_number_of_transfers_is_bounded`"
        " — supports multiple simultaneous direct-play streams bounded by configured limit",
        "Verified",
    ),
    "FUN-043": (
        "`crates/phairdlna-cli/src/serve/guard/tests.rs::the_transfers_are_bounded`,"
        " `crates/phairdlna-core/tests/http_framing.rs::the_number_of_transfers_is_bounded`"
        " — bounds concurrent transfers and refuses excess requests rather than exhausting host resources",
        "Verified",
    ),
    "FUN-046": (
        "`crates/phairdlna-cli/tests/serve.rs::an_open_transfer_ends_with_the_daemon`,"
        " `crates/phairdlna-core/tests/http_media.rs::a_connection_is_kept_alive_unless_it_asks_to_close`"
        " — transfers and associated permits are cleaned up safely when connections terminate",
        "Verified",
    ),
    "FUN-050": (
        "`crates/phairdlna-core/tests/http_framing.rs::a_body_that_is_too_large_is_refused`,"
        " `crates/phairdlna-core/tests/resource_bounds.rs::a_document_that_grows_past_its_bound_is_not_written`"
        " — buffer sizes and chunk windows remain strictly bounded to prevent memory exhaustion",
        "Verified",
    ),
    "FUN-041": (
        "`crates/phairdlna-core/tests/http_media.rs::the_type_of_the_answer_is_the_type_of_the_file`"
        " — reports container and codec matching client expectations via standardized MIME and protocolInfo",
        "Verified",
    ),
    "FUN-017": (
        "`crates/phairdlna-core/tests/volume_events.rs::a_drive_that_is_unplugged_takes_its_files_with_it`,"
        " `::a_drive_that_comes_back_keeps_its_identifiers`,"
        " `crates/phairdlna-core/tests/volume_platform_events.rs::a_volume_that_was_unmounted_takes_its_files_with_it`,"
        " `crates/phairdlna-cli/src/serve/watch.rs::tests::the_watch_reads_the_catalogue_again`"
        " — a drive that is unplugged takes its files with it, a drive that comes back is published again,"
        " and the running daemon notices both; the physical drive itself remains a manual device check",
        "Implemented",
    ),
    "FUN-018": (
        "`crates/phairdlna-core/tests/media_roots.rs::the_scanner_never_writes_to_the_library`"
        " — the walk opens directories and files to read them and writes nothing; the cache directory"
        " is not configured yet, so the exception of the requirement does not apply",
        "Implemented",
    ),
    "FUN-019": (
        "`crates/phairdlna-core/tests/media_roots_multiple.rs::the_path_of_a_file_is_the_path_of_the_disk`"
        " — the path of the catalogue is the path of the disk, byte for byte; the identity is a separate,"
        " derived value",
        "Verified",
    ),
    "FUN-015": (
        "`crates/phairdlna-core/tests/playlists.rs::a_playlist_is_a_container_with_its_tracks`,"
        " `::the_children_keep_the_order_of_the_playlist`,"
        " `::the_three_formats_are_read`,"
        " `::an_entry_outside_the_root_is_refused`,"
        " `::a_network_address_is_refused`,"
        " `::a_missing_entry_stays_in_the_playlist`,"
        " `::a_broken_playlist_does_not_take_the_root_down`,"
        " `::a_file_named_twice_is_one_object`,"
        " `crates/phairdlna-core/src/catalogue/playlist.rs::tests`"
        " — the M3U, M3U8 and PLS formats are parsed within their bounds, the entries are resolved against"
        " the folder of the playlist, an entry that walks out of the root and a network address are refused,"
        " and the playlist is a container whose children keep the order of the playlist",
        "Verified",
    ),
    "FUN-016": (
        "`crates/phairdlna-core/tests/subtitles.rs::a_sidecar_is_paired_with_its_film`,"
        " `::a_film_carries_the_subtitle_as_a_second_resource`,"
        " `::a_policy_without_the_capability_carries_one_resource`,"
        " `::another_file_is_not_a_subtitle`,"
        " `::the_subtitle_is_reachable_by_its_identifier`,"
        " `::the_sidecar_with_the_highest_priority_wins`,"
        " `::a_browse_of_a_subtitle_reads_nothing_from_the_disk`,"
        " `crates/phairdlna-cli/tests/subtitles.rs::a_client_finds_the_subtitle_and_downloads_it`"
        " — the sidecar is paired with its film while the root is read, it is offered as an additional res"
        " with its own type, and a client finds it in the document and downloads it",
        "Verified",
    ),
    "FUN-020": (
        "`crates/phairdlna-core/tests/catalogue_identity.rs::the_identity_key_of_a_file_is_the_same_for_every_root`,"
        " `::two_volumes_keep_two_identity_keys`,"
        " `::two_normalisation_forms_are_two_files_on_a_volume_that_keeps_the_bytes`,"
        " `::two_case_variants_are_one_file_on_a_case_insensitive_volume`,"
        " `crates/phairdlna-core/tests/catalogue_identifiers.rs::a_hard_link_is_not_a_second_object`,"
        " `crates/phairdlna-core/src/catalogue/identity.rs::tests`"
        " — the identity key is scoped by the volume and built from the path of the file, and the"
        " uniqueness of a file comes from the stable identity the platform gives; the two tests that"
        " need a volume that keeps the bytes read the volume they run on and assert what is true there",
        "Verified",
    ),
    "FUN-021": (
        "`crates/phairdlna-core/tests/catalogue_identity.rs::an_excluded_file_is_not_served`,"
        " `crates/phairdlna-core/tests/media_roots.rs::an_excluded_folder_is_not_indexed`"
        " — an excluded folder is not indexed and the identifier of a file it held names nothing, so a"
        " client that still holds the identifier gets nothing back; the exclusion takes effect on the"
        " next scan",
        "Implemented",
    ),
    "FUN-022": (
        "`crates/phairdlna-core/tests/catalogue_identity.rs::two_roots_that_reach_one_file_list_it_once`,"
        " `::a_root_inside_another_root_lists_the_file_once`"
        " — a file that two roots reach is one catalogue entry, and the first root wins; the device round"
        " that shows it on a television is still missing",
        "Implemented",
    ),
    "FUN-001": (
        "`crates/phairdlna-core/tests/media_roots.rs::a_root_is_scanned_into_a_tree`,"
        " `crates/phairdlna-core/tests/media_roots_multiple.rs::two_roots_are_browsable_and_keep_apart`"
        " — supports configuring one or more media roots with custom display name and absolute filesystem path",
        "Verified",
    ),
    "FUN-002": (
        "`crates/phairdlna-core/tests/media_roots_multiple.rs::the_same_name_in_two_roots_is_two_files`,"
        " `crates/phairdlna-core/tests/media_roots_multiple.rs::a_root_inside_another_root_is_published_once`"
        " — serves the union of all configured media roots as a unified catalogue",
        "Verified",
    ),
    "FUN-003": (
        "`crates/phairdlna-core/tests/symlink_policy.rs::a_link_is_not_followed_by_default`,"
        " `crates/phairdlna-core/tests/symlink_policy.rs::a_link_that_points_outside_the_root_is_never_indexed`"
        " — does not follow symbolic links pointing outside root unless explicitly enabled in configuration",
        "Verified",
    ),
    "FUN-004": (
        "`crates/phairdlna-core/tests/media_roots.rs::a_hidden_file_is_not_indexed`"
        " — ignores hidden files and macOS resource-fork sidecar files (._*) during catalogue building",
        "Verified",
    ),
    "FUN-005": (
        "`crates/phairdlna-core/tests/media_roots.rs::a_file_of_an_unknown_kind_stays_out_of_the_tree`,"
        " `crates/phairdlna-core/tests/media_roots.rs::a_file_without_a_name_is_placed_by_its_content`"
        " — indexes audio, video, image and playlist files by extension and sniffed content type",
        "Verified",
    ),
    "FUN-006": (
        "`crates/phairdlna-core/tests/catalogue_incremental.rs::incremental_scan_does_not_read_content_of_unchanged_files`"
        " — incremental scanner detects added, modified and removed files without full re-read of unchanged items",
        "Verified",
    ),
    "FUN-007": (
        "`crates/phairdlna-core/tests/media_roots.rs::a_missing_root_is_reported_and_the_library_stays_browsable`,"
        " `crates/phairdlna-core/tests/offline_volume.rs::offline_media_root_reported_unavailable_and_preserved`"
        " — unmounted or missing roots are reported as unavailable in diagnostics while remaining catalogue stays browsable",
        "Verified",
    ),
    "FUN-008": (
        "`crates/phairdlna-core/tests/catalogue_rebuild.rs::a_browse_during_a_rescan_sees_the_previous_catalogue`"
        " — the reader of the index has a connection of its own and the writer writes the whole"
        " catalogue in one transaction, so a browse during a rescan sees the catalogue of before"
        " or the catalogue of now, and never a mixture of the two",
        "Verified",
    ),
    "FUN-009": (
        "`crates/phairdlna-core/tests/catalogue_rebuild.rs::the_index_is_written_and_read_back`,"
        " `::the_catalogue_survives_a_restart_of_the_server`,"
        " `::the_file_of_the_index_belongs_to_the_user_alone`,"
        " `::a_deleted_index_is_built_again`,"
        " `::the_media_of_the_library_is_never_written_to`,"
        " `::a_playlist_survives_the_round_trip`,"
        " `::a_subtitle_survives_the_round_trip`,"
        " `crates/phairdlna-cli/tests/catalogue_index.rs::the_daemon_writes_the_catalogue_into_its_index`,"
        " `::the_daemon_serves_the_library_after_a_restart`"
        " — the catalogue is written into a SQLite database of the user and read back after the"
        " server stopped; a playlist and a subtitle survive the round trip with their order and"
        " their names; the file belongs to the user alone, the index is a cache that may be"
        " deleted at any time, and the media of the library is never touched",
        "Verified",
    ),
    "FUN-010": (
        "`crates/phairdlna-core/tests/catalogue_recovery.rs::a_damaged_index_is_moved_aside_and_rebuilt`,"
        " `::an_index_of_another_schema_is_moved_aside`,"
        " `crates/phairdlna-cli/tests/catalogue_index.rs::a_damaged_index_does_not_stop_the_daemon`"
        " — a file that is damaged or that carries another schema is moved aside under the name"
        " `.corrupt.<moment>`, a new index takes its place, the library is read again, and the"
        " daemon says so in the log and in the status of the server",
        "Verified",
    ),
    "FUN-011": (
        "`crates/phairdlna-core/tests/catalogue_identifiers.rs::the_identifier_of_a_file_is_the_same_after_a_second_scan`,"
        " `crates/phairdlna-core/tests/catalogue_identifiers.rs::the_identifier_of_a_file_does_not_change_when_the_library_moves`,"
        " `crates/phairdlna-core/tests/catalogue_identifiers.rs::the_identifier_carries_no_part_of_the_path`"
        " — assigns deterministic, stable and path-opaque object identifiers preserved across rescans and library moves",
        "Verified",
    ),
    "FUN-026": (
        "`crates/phairdlna-core/tests/subtitles.rs::a_film_carries_the_subtitle_as_a_second_resource`,"
        " `::a_smil_sidecar_is_offered_with_its_own_type`,"
        " `crates/phairdlna-cli/tests/subtitles.rs::a_client_finds_the_subtitle_and_downloads_it`,"
        " `crates/phairdlna-core/src/catalogue/subtitle.rs::tests`"
        " — the resource of a subtitle names the type of the file, and the answer of the media endpoint"
        " carries the same type; the formats are SRT, WebVTT, ASS, Sub and SMIL",
        "Verified",
    ),
    "FUN-027": (
        "`crates/phairdlna-core/tests/subtitles.rs::a_latin_1_subtitle_states_its_character_set`,"
        " `::a_utf_8_subtitle_states_utf_8`,"
        " `crates/phairdlna-cli/tests/subtitles.rs::a_latin_1_subtitle_states_its_character_set`"
        " — the answer of a subtitle names the character set it is written in, and the file is delivered"
        " in that character set; a Latin-1 file is not transcoded to UTF-8 on the way out, so the"
        " transcoding of the design of F-11 § 4 is not implemented (F-11 D-05)",
        "Implemented",
    ),
    "FUN-029": (
        "`crates/phairdlna-core/tests/subtitles.rs::the_sidecar_with_the_highest_priority_wins`"
        " — the additional subtitle of a media item is exposed as its own resource; a media item with"
        " several additional audio or subtitle tracks is not supported, because the baseline pairs one"
        " sidecar per media file (DEC-053)",
        "Implemented",
    ),
    "INT-105": (
        "`crates/phairdlna-core/tests/subtitles.rs::a_film_carries_the_subtitle_as_a_second_resource`"
        " — the additional resource of an item has a protocolInfo that names its own MIME type",
        "Verified",
    ),
    "FUN-013": (
        "`crates/phairdlna-core/tests/artwork.rs::an_embedded_cover_is_found`,"
        " `::an_embedded_cover_of_a_m4a_is_found`,"
        " `::a_sidecar_wins_over_the_embedded_cover`,"
        " `::a_sidecar_may_carry_the_name_of_the_file`,"
        " `::a_sidecar_outside_the_root_is_not_used`,"
        " `::a_track_without_a_picture_has_none`,"
        " `::a_file_outside_the_root_has_no_picture`"
        " — the cover of an ID3v2 tag and the cover of an MP4 container are read, the image next to the"
        " file wins over the cover in the tag, a sidecar that leads out of the root is refused, and a file"
        " without a picture names no address; the parsing happens in the server process until the sandbox"
        " child of ADR-0008 exists (M-06, ADR-0011)",
        "Implemented",
    ),
    "FUN-014": (
        "`crates/phairdlna-core/src/media/thumbnail.rs::tests::the_produced_picture_is_a_jpeg`,"
        " `crates/phairdlna-core/src/media/cache.rs::tests::the_cache_stays_inside_its_bound`,"
        " `::a_deleted_directory_is_recreated`,"
        " `crates/phairdlna-cli/tests/artwork.rs::the_thumbnail_of_a_photo_is_answered`,"
        " `::the_second_request_is_answered_from_the_cache`"
        " — a photo is its own picture, the thumbnail is produced on demand and is kept in a disposable"
        " cache that stays inside its bound; the video poster, the second half of the requirement, is not"
        " produced, because the baseline ships no video decoder (F-10 § 12 Q1)",
        "Implemented",
    ),
    "FUN-025": (
        "`crates/phairdlna-core/src/platform/fs_watcher.rs`,"
        " `crates/phairdlna-cli/src/serve/watch.rs::tests::the_watch_picks_up_newly_created_media_files`"
        " — the server monitors media roots with native OS change events (FSEvents/inotify), debounces event bursts"
        " and automatically updates the catalogue within the 30-second window",
        "Verified",
    ),
    "SEC-067": (
        "`crates/phairdlna-core/src/media/thumbnail.rs::tests::a_picture_that_lies_about_its_size_is_refused_before_it_is_decoded`,"
        " `crates/phairdlna-core/src/media/thumbnail.rs::tests::a_picture_with_too_many_pixels_is_refused`"
        " — image dimension headers are pre-validated and images exceeding pixel thresholds are rejected prior to full decoding",
        "Verified",
    ),
    "CFG-008": (
        "`crates/phairdlna-core/src/media/cache.rs::tests::the_cache_stays_inside_its_bound`,"
        " `crates/phairdlna-core/tests/artwork.rs::an_embedded_cover_that_is_too_large_is_refused`"
        " — the bound of the cache of the pictures is honoured and the oldest pictures are removed when it"
        " is exceeded; the limit of the concurrent streams, the limit of the extraction time and the number"
        " of the index workers are the work of M2.4",
        "Implemented",
    ),
    "PLT-011": (
        "`crates/phairdlna-core/tests/volume_platform_events.rs::a_volume_that_was_unmounted_takes_its_files_with_it`,"
        " `::a_volume_that_was_mounted_again_is_published_again`,"
        " `crates/phairdlna-core/tests/volume_events.rs::a_drive_that_comes_back_under_another_identity_is_given_new_identifiers`"
        " — a mount, an unmount and a remount are first-class events, and the entries of a volume that"
        " comes back under another identity are given new identifiers; the core honours the events of the"
        " platform through `Catalogue::apply_volume_event`, and the daemon derives the same outcome from"
        " the observations of the roots, because the portable adapter has no native events — the native"
        " watcher (DiskArbitration) and the check on the build matrix are the work of the platform adapter"
        " (F-13)",
        "Implemented",
    ),
    "PLT-013": (
        "`crates/phairdlna-cli/src/service/plist.rs::tests::the_job_file_names_the_program_and_the_arguments`,"
        " `crates/phairdlna-cli/src/service/plist.rs::tests::the_job_file_restarts_the_server`,"
        " — these checks cover only the server job. No approved per-user UI LaunchAgent is packaged, and the requirement conflicts with the regular Dock app in DEC-076; OD-15 must resolve the process model",
        "Not implemented",
    ),
    "CFG-005": (
        "`crates/phairdlna-core/src/config/validate/admin.rs::tests::a_non_loopback_address_is_refused`,"
        " `crates/phairdlna-core/src/config/validate/admin.rs::tests::the_user_may_override_the_refusal`"
        " — a configuration that would put the administration surface on an address of the network is refused"
        " and the refusal names the setting; the user may override the refusal, and then the address is"
        " accepted",
        "Verified",
    ),
    "CFG-007": (
        "`crates/phairdlna-core/tests/symlink_policy.rs::a_link_is_not_followed_by_default`,"
        " `crates/phairdlna-core/tests/symlink_policy.rs::a_root_may_have_a_policy_of_its_own`"
        " — the policy of a root decides whether a symbolic link is followed, and the default follows none",
        "Verified",
    ),
    "CFG-009": (
        "`crates/phairdlna-cli/tests/serve.rs::a_client_that_is_not_on_the_list_is_refused`,"
        " `crates/phairdlna-cli/tests/serve.rs::a_client_that_is_on_the_list_is_answered`,"
        " `crates/phairdlna-core/src/security/access.rs::tests::a_network_admits_its_addresses`"
        " — the list of the clients is read from the configuration and checked against the address of the"
        " socket; a client that is not on the list is refused with 403 before a handler sees the request",
        "Verified",
    ),
    "CFG-012": (
        "`crates/phairdlna-cli/tests/cli.rs::the_effective_configuration_is_exported`,"
        " `crates/phairdlna-cli/tests/cli.rs::a_configuration_file_is_imported`,"
        " `crates/phairdlna-cli/tests/cli.rs::an_imported_configuration_is_applied_to_target`"
        " — the export writes the configuration as the server understands it, with the schema version; the"
        " import reads a file and reports what it would become or atomically writes it to target with --apply",
        "Verified",
    ),
    "CFG-013": (
        "`crates/phairdlna-core/tests/config.rs::an_unknown_key_is_refused`"
        " — a key that the server does not know is an error and not a silent no-op",
        "Verified",
    ),
    "CFG-014": (
        "`crates/phairdlna-admin/tests/admin_surface.rs::the_token_file_is_private`,"
        " `crates/phairdlna-cli/tests/admin.rs::the_token_file_belongs_to_its_owner_alone`"
        " — the file that holds the token of the administration surface has the permissions 600 inside a"
        " folder with the permissions 700, and a file that others may read stops the surface",
        "Verified",
    ),
    "OPS-017": (
        "`crates/phairdlna-core/build.rs`, `crates/phairdlna-cli/src/serve/admin.rs`"
        " — the revision is embedded into the binary when it is built and the status output names it; the"
        " source offer does not exist yet, so the two cannot be compared",
        "Implemented",
    ),
    "OPS-020": (
        "`crates/phairdlna-admin/src/server.rs`,"
        " `crates/phairdlna-core/src/config/validate/admin.rs::tests::a_non_loopback_address_is_refused`"
        " — the surface listens on 127.0.0.1 and ::1 and on no other address, and a configuration that asks"
        " for another address is refused unless the user says so",
        "Verified",
    ),
    "OPS-021": (
        "`crates/phairdlna-admin/tests/admin_surface.rs::a_request_without_a_token_is_refused`,"
        " `crates/phairdlna-admin/tests/admin_surface.rs::a_request_with_a_wrong_token_is_refused`,"
        " `crates/phairdlna-admin/src/token.rs::tests::the_token_file_is_written_and_read_back`"
        " — every request carries the bearer token, a request without it or with a wrong one is refused with"
        " 401, and the token is read from a file that belongs to the owner alone; the token is asked for on"
        " every request and not only on the requests that change something",
        "Verified",
    ),
    "OPS-022": (
        "`crates/phairdlna-admin/tests/admin_surface.rs::a_request_that_names_another_host_is_refused`,"
        " `crates/phairdlna-admin/src/guard.rs::tests::a_name_the_attacker_controls_is_refused`"
        " — the surface answers only to a Host header that names the loopback interface; a name the attacker"
        " controls is refused with 421 before a handler sees the request",
        "Verified",
    ),
    "OPS-023": (
        "`crates/phairdlna-admin/tests/admin_surface.rs::a_change_without_the_csrf_token_is_refused`,"
        " `crates/phairdlna-admin/tests/admin_surface.rs::a_change_with_a_wrong_csrf_token_is_refused`,"
        " `crates/phairdlna-admin/src/token.rs::tests::the_csrf_token_belongs_to_the_token`"
        " — a request that changes something carries the token of the cross-site request forgery defence, and"
        " the value is bound to the session, which is the token of the installation",
        "Verified",
    ),
    "OPS-024": (
        "`crates/phairdlna-cli/tests/admin.rs::the_media_server_does_not_know_the_routes_of_the_surface`,"
        " `crates/phairdlna-admin/tests/admin_surface.rs::an_unknown_path_is_refused`"
        " — the surface has its own listener and authenticated API routes; the media server knows none of them,"
        " and a path the surface does not know is a generic 404",
        "Verified",
    ),
    "OPS-025": (
        "`crates/phairdlna-cli/Cargo.toml` (the feature admin),"
        " `scripts/local-ci.sh` (the build without the default features)"
        " — the surface sits behind the cargo feature admin and behind security.admin_enabled; a build"
        " without the feature has no administration socket, no token and no administration code",
        "Verified",
    ),
    "OPS-026": (
        "`crates/phairdlna-cli/src/serve/setup.rs::explain_the_admin_surface`"
        " — the daemon names the address of the surface and the file of its token when it starts; the"
        " explanation in plain language and the sentence for the disabled case are the work of the first-run"
        " output of the command line",
        "Implemented",
    ),
    "OPS-027": (
        "`crates/phairdlna-admin/src/server.rs` (the limits of the surface),"
        " `crates/phairdlna-admin/src/guard.rs`"
        " — the surface reads the same limit table as the media server and bounds the head of the request"
        " and the time of the answer; the body of the surface is empty, so its own bound is small",
        "Implemented",
    ),
    "UX-006": (
        "`crates/phairdlna-mac/src/main.rs::print_help`,"
        " `crates/phairdlna-mac/resources/en-US.strings`"
        " — the help of the command line of the application names the equivalent command in"
        " `phairdlna-cli`; the command line of the daemon has its own help (F-13)",
        "Implemented",
    ),
    "UX-007": (
        "`crates/phairdlna-mac/tests/menu.rs::a_running_server_shows_its_streams`,"
        " `::a_refusing_server_shows_recovery_actions_without_starting_it_again`,"
        " `crates/phairdlna-mac/src/menu.rs::status_line`"
        " — the state of the server is a sentence and never a coloured dot; the keyboard"
        " navigation of the menu is the one of the system and is checked by the manual"
        " protocol, which has not run yet; a server that refuses access offers recovery"
        " actions without offering a second start or stop",
        "Implemented",
    ),
    "UX-009": (
        "`crates/phairdlna-mac/src/platform/status_item.rs::set_accessibility`,"
        " `crates/phairdlna-mac/src/menu.rs::accessibility_value`"
        " — the item of the menu bar carries a label and a value that describe the state of"
        " the server in words; the announcement of VoiceOver has not been observed yet",
        "Implemented",
    ),
    "UX-010": (
        "`crates/phairdlna-mac/resources/en-US.strings`,"
        " `crates/phairdlna-mac/tests/strings.rs::no_user_facing_text_lives_in_the_code`"
        " — the texts of the user interface are plain: a media folder, a library, a server",
        "Implemented",
    ),
    "UX-014": (
        "`crates/phairdlna-mac/tests/menu.rs::a_running_server_shows_its_streams`,"
        " `crates/phairdlna-mac/tests/overview.rs::a_running_view_lists_every_folder_and_marks_a_missing_volume`,"
        " `::unavailable_tv_discovery_keeps_the_server_address_visible`"
        " — the menu shows state, folders and client activity; the overview lists every shared"
        " folder, marks unavailable volumes, hides stale data and gives the address when TV"
        " discovery is unavailable; scan progress is shown as server state",
        "Implemented",
    ),
    "UX-015": (
        "`crates/phairdlna-mac/tests/presentation_modes.rs::switching_the_view_changes_nothing_but_the_view`,"
        " `::both_views_show_the_same_state`"
        " — the mode is a preference of the application, it lives in a file of its own beside"
        " the configuration of the server, and switching it starts no work",
        "Verified",
    ),
    "UX-016": (
        "`crates/phairdlna-mac/tests/presentation_modes.rs::a_warning_stays_visible_in_the_short_view`"
        " — a warning of the server stays a line of the menu in the short view",
        "Verified",
    ),
    "UX-018": (
        "`crates/phairdlna-mac/tests/menu.rs::the_state_is_a_text_for_the_screen_reader`"
        " — the state of the server has a textual rendering that does not need the menu to"
        " be open",
        "Verified",
    ),
    "UX-020": (
        "`crates/phairdlna-mac/tests/strings.rs::every_language_carries_every_key`,"
        " `::every_literal_of_the_code_is_written_down`"
        " — every text of the user interface lives in a resource file per language, and the"
        " test fails when a text appears in the code; the list of the languages is the open"
        " question of F-15",
        "Verified",
    ),
    "MB-001": (
        "`crates/phairdlna-mac/tests/application.rs::nothing_of_the_last_answer_stays_when_the_server_is_gone`,"
        " `crates/phairdlna-mac/tests/overview.rs::stopped_and_refused_views_hide_stale_counts_folders_and_activity`,"
        " `crates/phairdlna-mac/tests/overview.rs::the_first_view_says_it_is_checking_in_every_section`,"
        " `crates/phairdlna-mac/tests/ipc_client.rs::a_server_that_was_never_started_sends_no_request`"
        " — the application holds no state of the server: it reads the state from the surface"
        " and shows nothing of the last answer when the surface is gone",
        "Verified",
    ),
    "MB-002": (
        "`crates/phairdlna-mac/src/worker.rs`, `crates/phairdlna-mac/src/platform/desktop.rs`"
        " — the requests run on a thread of their own, the main thread only draws and takes"
        " the answers, and the channel of the commands holds one command; the progress of a"
        " scan is the state of the server and not a progress of the application",
        "Implemented",
    ),
    "MB-004": (
        "`crates/phairdlna-mac/tests/presentation_modes.rs::switching_the_view_changes_nothing_but_the_view`"
        " — no view of the application starts an index or a scan of the network",
        "Verified",
    ),
    "MB-005": (
        "`crates/phairdlna-mac/tests/no_egress.rs::no_source_names_an_address_but_the_loopback`,"
        " `::the_transport_is_bound_to_the_loopback_interface`,"
        " `::the_manifest_holds_no_library_of_the_network`"
        " — the application talks to the administration surface on the loopback interface and"
        " to nothing else; the crate holds no library that could open a connection",
        "Verified",
    ),
    "MB-006": (
        "`crates/phairdlna-mac/src/platform/icon.rs::image_of`"
        " — the picture of the item is a template image of the system and therefore follows the"
        " appearance of the menu bar; the state is a text as well; the legibility on a small"
        " display has not been checked by the manual protocol yet",
        "Implemented",
    ),
    "MB-009": (
        "`crates/phairdlna-mac/tests/bundle_metadata.rs::the_bundle_is_a_regular_application`,"
        " `::the_bundle_explains_the_permission_of_the_local_network`"
        " — the bundle is a regular application and carries the text of the"
        " permission of the local network; the bundle itself is built and signed by the"
        " release, which is not done",
        "Implemented",
    ),
    "LIFE-010": (
        "`crates/phairdlna-mac/tests/bundle_metadata.rs::the_bundle_registers_no_helper`,"
        " `crates/phairdlna-cli/src/service/commands.rs::install`,"
        " `crates/phairdlna-cli/src/service/commands.rs::uninstall`"
        " — service installation is explicit and reversible via phairdlna service install/uninstall;"
        " the macOS application bundle installs no hidden privileged helper or launchd agent silently",
        "Verified",
    ),
    "PLT-001": (
        "`scripts/check-crate-boundaries.sh`"
        " — core crate contains no OS-specific dependencies and dependency direction is strictly one-way (QG-28)",
        "Verified",
    ),
    "PLT-002": (
        "`crates/phairdlna-core/tests/platform_matrix.rs::the_build_matrix_covers_all_required_platforms`,"
        " `.github/workflows/ci-matrix.yml`"
        " — build matrix covers macOS (arm64, x86_64), Linux (x86_64) and Windows (x86_64) via self-hosted runners",
        "Verified",
    ),
    "PLT-003": (
        "`crates/phairdlna-core/src/platform/traits.rs`,"
        " `crates/phairdlna-core/src/platform/unsupported.rs`"
        " — OS-specific behavior is isolated behind documented traits with explicit Unsupported fallbacks",
        "Verified",
    ),
    "PLT-004": (
        "`crates/phairdlna-mac/tests/apple_silicon_native.rs::application_target_matches_host_architecture`"
        " — native Apple Silicon (aarch64-apple-darwin) execution without Rosetta translation requirement",
        "Verified",
    ),
    "PLT-007": (
        "`crates/phairdlna-core/tests/path_containment.rs::a_traversal_attempt_names_no_object`,"
        " `crates/phairdlna-core/tests/path_containment.rs::a_traversal_attempt_is_never_served`,"
        " `crates/phairdlna-cli/tests/windows_manifest.rs`"
        " — path traversal attempts using both slash and backslash separators are rejected and long paths supported",
        "Verified",
    ),
    "PLT-010": (
        "`crates/phairdlna-core/tests/protocol_parity.rs::device_description_bytes_are_deterministic`,"
        " `::scpd_documents_are_identical_across_invocations`,"
        " `::soap_envelope_payload_and_ordering_are_deterministic`,"
        " `::didl_lite_item_ordering_and_xml_is_deterministic`"
        " — protocol behavior, ordering and identifiers are deterministic and byte-identical across platforms",
        "Verified",
    ),
    "PLT-014": (
        "`crates/phairdlna-mac/tests/universal_binary.rs::the_bundle_script_targets_universal_architectures`,"
        " `scripts/package-macos-app.sh`"
        " — macOS application is packaged as a universal binary containing both arm64 and x86_64 architectures",
        "Verified",
    ),
    "PLT-015": (
        "`crates/phairdlna-cli/tests/windows_manifest.rs::the_windows_manifest_declares_long_path_awareness`,"
        " `::the_documentation_declares_windows_long_paths_enabled_requirement`,"
        " `crates/phairdlna-cli/resources/phairdlna.manifest`"
        " — Windows executable manifest declares longPathAware and documentation specifies LongPathsEnabled requirement",
        "Verified",
    ),
    "PLT-016": (
        "`docs/08-releases/RELEASE-POLICY.md`,"
        " `docs/03-features/F-22-platform-portability.md`"
        " — primary Linux distribution channels must be native packages (deb/rpm/tarball), not sandboxed formats",
        "Verified",
    ),
    "UX-011": (
        "`crates/phairdlna-cli/tests/admin.rs::the_diagnostics_do_not_name_the_files_of_the_user`,"
        " `crates/phairdlna-admin/tests/admin_surface.rs::the_diagnostics_hide_the_paths`"
        " — the diagnostics of the surface name no file of the user; the report as plain text and the"
        " switch of the redaction are the work of the command line",
        "Implemented",
    ),
    "UX-012": (
        "`crates/phairdlna-cli/src/serve/transfers.rs::tests::a_transfer_is_counted_while_it_runs`,"
        " `crates/phairdlna-cli/src/serve/admin.rs`"
        " — the diagnostics name the client and the title of the item it is playing, and the status counts"
        " the transfers that run; the address of the client is not known yet, because the listener does not"
        " hand the address of the peer to the route of the media server",
        "Implemented",
    ),
    "SEC-071": (
        "`crates/phairdlna-cli/src/serve/guard.rs`,"
        " `crates/phairdlna-admin/tests/admin_surface.rs::a_request_that_names_another_host_is_refused`"
        " — the media server answers only to the host names of the configuration, and the administration"
        " surface only to the names of the loopback interface; a name the attacker controls is refused"
        " before a handler sees the request",
        "Verified",
    ),
    "SEC-072": (
        "`crates/phairdlna-admin/tests/admin_surface.rs::a_request_without_the_custom_header_is_refused`,"
        " `crates/phairdlna-admin/tests/admin_surface.rs::the_answer_never_carries_a_cross_origin_header`"
        " — every request of the administration surface carries the custom header, and no answer of the"
        " surface carries a header that would let another origin read it",
        "Verified",
    ),
    "RES-001": (
        "`crates/phairdlna-core/tests/scheduler_priority.rs::a_stream_runs_before_the_scan_that_waits_for_it`,"
        " `::a_control_request_runs_before_the_scan_that_waits_for_it`,"
        " `crates/phairdlna-cli/tests/resources.rs::the_daemon_answers_a_client_before_it_reads_the_library`"
        " — one scheduler in the core arbitrates the four classes, and a worker that becomes free takes the"
        " task of the most important class that may run one; the discovery of SSDP still runs on the socket"
        " task of the runtime and is not arbitrated by the scheduler yet",
        "Implemented",
    ),
    "RES-002": (
        "`crates/phairdlna-core/tests/resource_exhaustion.rs::a_full_queue_refuses_the_next_task_with_a_clear_error`,"
        " `::the_number_of_running_tasks_never_passes_the_bound`"
        " — every class and every queue has a documented bound, a task that does not fit is refused with an"
        " error that names the bound, and the tasks that have not started are dropped at the shutdown while"
        " the task that runs finishes",
        "Implemented",
    ),
    "RES-003": (
        "`crates/phairdlna-core/tests/scheduler_hysteresis.rs::the_scan_yields_while_the_machine_is_busy`,"
        " `::the_scan_starts_again_when_the_machine_has_been_calm`,"
        " `::a_value_inside_the_band_does_not_start_the_scan`"
        " — the thresholds, the band of the hysteresis and the quiet period are documented in the core, and"
        " the reading of the library yields while the machine is busy; the processor, the memory and the disk"
        " are not measured by a platform adapter yet, so only the number of the transfers and the time of"
        " the last answer are read",
        "Implemented",
    ),
    "RES-004": (
        "`crates/phairdlna-core/tests/scheduler_priority.rs::a_slow_client_does_not_hold_up_another_stream`"
        " — every class has its own bound on the tasks that run at the same time, so a client that stops"
        " reading slows down its own stream and nobody else",
        "Verified",
    ),
    "RES-005": (
        "`crates/phairdlna-core/src/scheduler/pressure.rs`"
        " — the thresholds, the bounds and the order of the classes live in the shared core, and a platform"
        " supplies hints through the trait `ResourceProbe` and never a decision; the daemon is the first"
        " provider of the hints, no platform adapter measures the processor, the memory or the disk yet",
        "Implemented",
    ),
    "RES-006": (
        "`crates/phairdlna-cli/tests/resources.rs::the_report_of_the_resources_names_no_private_data`"
        " — the counters of the scheduler are asked for, they name no file, no client and no address, and"
        " they never leave the machine; the report is not shown in the administration surface and not"
        " exported by a command yet",
        "Implemented",
    ),
    "RES-007": (
        "`crates/phairdlna-core/tests/resource_exhaustion.rs::a_full_queue_refuses_the_next_task_with_a_clear_error`,"
        " `crates/phairdlna-cli/src/serve/media_route.rs::artwork_body`"
        " — a task that does not fit is refused with an error that names the class and the bound, the route"
        " of the cover answers 503 with the reason, and a running stream is never dropped and no file of the"
        " library is changed",
        "Implemented",
    ),
    "RES-008": (
        "`crates/phairdlna-core/src/scheduler/pressure.rs::tests::an_energy_constrained_machine_yields_background_work`"
        " — under battery or thermal constraints background scanner yields while playback streams and exposure are preserved",
        "Verified",
    ),
    "RES-009": (
        "`crates/phairdlna-core/tests/resource_exhaustion.rs::a_stopping_scheduler_refuses_every_task`,"
        " `crates/phairdlna-core/src/scheduler/mod.rs::tests::a_scheduler_releases_its_workers`"
        " — the start-up builds the workers, the reading of the library gives its files back when the task"
        " ends, and the shutdown drops the tasks that have not started and joins the workers; the sleep and"
        " the wake do not release the workers of the scheduler yet",
        "Implemented",
    ),
    "SUP-005": (
        "`scripts/generate-release-artefacts.sh`,"
        " `crates/phairdlna-cli/tests/release_artefacts.rs::release_packaging_script_exists_and_implements_sbom_and_checksums`"
        " — release packaging script generates SHA-256 checksums and dependency SBOM metadata for distribution packages",
        "Verified",
    ),

    "INT-100": (
        "`crates/phairdlna-core/src/profile.rs`,"
        " `crates/phairdlna-core/tests/client_profiles.rs::profile_without_evidence_pointer_is_rejected_by_evidence_gate`"
        " — declarative profile store requires client family, observable symptoms, and mandatory evidence pointers",
        "Verified",
    ),
    "INT-101": (
        "`crates/phairdlna-core/src/profile.rs`,"
        " `crates/phairdlna-core/tests/client_profiles.rs::disabled_profiles_restore_plain_behavior_without_recompilation`"
        " — quirk workarounds are strictly scoped to output shaping adjustments and never violate normative UPnP/DLNA requirements for other clients",
        "Verified",
    ),

    "INT-102": (
        "`qa/device-matrix.toml`,"
        " `crates/phairdlna-core/tests/device_matrix.rs::device_matrix_records_have_mandatory_attributes`"
        " — every device-test result records device model, firmware, client application, media format, and observed outcome in machine-readable matrix",
        "Verified",
    ),
    "INT-103": (
        "`crates/phairdlna-cli/src/ssdp_cmd.rs::replay`, `crates/phairdlna-core/src/ssdp/replay.rs`,"
        " `crates/phairdlna-cli/tests/ssdp_replay.rs`"
        " — replay stored M-SEARCH requests and print exact byte-for-byte response datagrams for diagnostics",
        "Verified",
    ),
    "INT-104": (
        "`crates/phairdlna-core/src/profile.rs::ProfileStore::disable`,"
        " `crates/phairdlna-core/tests/client_profiles.rs::disabled_profiles_restore_plain_behavior_without_recompilation`"
        " — client compatibility profiles can be disabled globally via configuration to restore plain conforming behavior without recompiling",
        "Verified",
    ),

}


def parse_requirements(path: Path) -> list[dict]:
    """Extract (id, text, priority, section) from a requirement table document."""
    rows: list[dict] = []
    section = ""
    for line in path.read_text(encoding="utf-8").splitlines():
        heading = HEADING.match(line)
        if heading:
            section = heading.group(1).strip()
            continue
        match = ROW.match(line)
        if match:
            rid, text, priority = match.group(1), match.group(2).strip(), match.group(3)
            rows.append({"id": rid, "text": text, "priority": priority, "section": section})
    return rows


def short_title(text: str) -> str:
    """Turn a normative sentence into a short, readable title.

    The title is a label, not the normative text: the full sentence stays in the
    definition document. We therefore strip the leading subject and modal verb and
    keep the informative part of the sentence.
    """
    cleaned = re.sub(r"^The\s+(?:\w+\s+){1,4}?shall\s+", "", text)
    cleaned = re.sub(r"^The\s+(?:\w+\s+){1,4}?may\s+", "", cleaned)
    cleaned = re.sub(r"^The\s+(?:\w+\s+){1,4}?will\s+", "", cleaned)
    if cleaned == text:
        cleaned = text
    # Prefer a clause break, but only when the leading clause is substantial.
    head, sep, tail = cleaned.partition(", ")
    if sep and len(head) >= 45:
        cleaned = head
    if len(cleaned) > 96:
        cleaned = cleaned[:96].rsplit(" ", 1)[0] + "…"
    return cleaned[:1].upper() + cleaned[1:]


def in_range(rid: str, spec: str) -> bool:
    """Match a requirement ID against a spec such as 'INT-001' or 'INT-001..INT-015'."""
    if ".." not in spec:
        return rid == spec
    low, high = spec.split("..")
    prefix = low.split("-")[0]
    if not rid.startswith(prefix + "-"):
        return False
    return low.split("-")[1] <= rid.split("-")[1] <= high.split("-")[1]


def allocate(rows: list[dict]) -> dict[str, str]:
    """Assign each requirement to its owning feature.

    Product commitments (PRD-*) are cross-cutting by nature: they are proven by the
    product acceptance protocol and by audits across many features, so they are not
    forced into a single feature.
    """
    mapping: dict[str, str] = {}
    for row in rows:
        if row["id"].startswith("PRD-"):
            mapping[row["id"]] = "cross-cutting"
            continue
        for fid, _slug, specs in FEATURES:
            if any(in_range(row["id"], spec) for spec in specs):
                mapping[row["id"]] = fid
                break
    return mapping



# Milestone overrides: the roadmap schedules individual requirement groups inside
# a feature across different milestones. The index must agree with the roadmap,
# so the override table below is the authority for those groups.
MILESTONE_OVERRIDES = [
    # (prefix, low, high, milestone, reason)
    ("FUN", "017", "024", "M2", "external volumes, multiple roots and the organisation rules are M2"),
    ("FUN", "013", "016", "M2", "metadata, artwork and playlists are M2"),
    ("FUN", "020", "022", "M2", "playlists and subtitles are M2"),
    ("CFG", "001", "015", "M2", "the full configuration surface is M2"),
    ("OPS", "016", "027", "M2", "the administration surface and the source offer are M2"),
    ("OPS", "001", "015", "M1", "the command line arrives with the first milestone"),
    ("UX", "011", "020", "M2", "the interface experience is M2"),
    ("MB", "001", "009", "M2", "the macOS application is M2"),
    ("RES", "001", "009", "M2", "adaptive resource management is M2"),
    ("REL", "001", "006", "M4", "the reliability targets are proven in the hardening milestone"),
    ("PLT", "011", "011", "M2", "the volume events and the re-keying of the entries are the work of M2.1"),
    ("PLT", "011", "018", "M3", "the per-platform behaviour is delivered with Linux and Windows"),
    ("PLT", "013", "013", "M2", "the launchd daemon model belongs to the macOS application"),
    ("PLT", "001", "009", "M0", "portability and the build matrix are foundation work"),
    ("INT", "100", "104", "M4", "the client compatibility profiles are hardening work"),
    ("FUN", "047", "049", "M5", "transcoding is deferred"),
    ("PERF", "001", "015", "M1", "the budgets are measured in the first milestone"),
    ("SEC", "066", "072", "M1", "the isolation and the source offer are part of the security baseline"),
    ("SEC", "010", "065", "M1", "the security baseline is M1"),
    ("SEC", "090", "096", "M1", "the privacy guarantees are M1"),
    ("INT", "001", "104", "M1", "the protocol surface is M1"),
    ("DOC", "001", "012", "M0", "the documentation quality is foundation work"),
    ("SUP", "001", "009", "M0", "the licence and supply-chain policy is foundation work"),
    ("LIFE", "001", "011", "M2", "the lifecycle and recovery work is M2"),
    # The catalogue of M1 is clean and in memory; the index on disk belongs to
    # M2. The owner decided this on 2026-11-16 (DEC-037). This entry must stay
    # above the broad "FUN 001..012 is M1" rule below, because the first match
    # wins.
    ("FUN", "008", "010", "M2", "the persistent index on disk belongs to M2; M1 keeps a clean in-memory catalogue"),
    ("FUN", "001", "012", "M1", "the library core is M1"),
]


def milestone_for(row: dict) -> str:
    """The milestone of one requirement: the feature default, overridden by the
    roadmap's finer granularity where the two differ."""
    prefix = row["id"].split("-")[0]
    number = row["id"].split("-")[1]
    for pfx, low, high, milestone, _reason in MILESTONE_OVERRIDES:
        if pfx == prefix and low <= number <= high:
            return milestone
    return row["milestone"]


def assert_complete(rows: list[dict]) -> None:
    """Fail loudly when a requirement has no feature or no milestone.

    A silent default would let the index print a plausible-looking row for a
    requirement nobody owns - which is how 56 requirements ended up without an
    owner in an earlier revision of this script.
    """
    problems = []
    for row in rows:
        if row["feature"] in ("", "—", "-"):
            problems.append(f"{row['id']}: no owning feature")
        if not row["milestone"]:
            problems.append(f"{row['id']}: no milestone")
    if problems:
        raise SystemExit(
            "the requirement catalogue is incomplete:\n  "
            + "\n  ".join(problems)
            + "\n\nEvery requirement needs an owning feature and a milestone. "
            "Add it to FEATURES or MILESTONE_OVERRIDES in this script."
        )


def build_rows() -> list[dict]:
    rows = parse_requirements(PRODUCT) + parse_requirements(TECHNICAL)
    allocation = allocate(rows)
    for row in rows:
        row["feature"] = allocation.get(row["id"], "—")
        row["title"] = short_title(row["text"])
        row["verification"] = (
            "Protocol conformance + device test"
            if row["id"] in DEVICE_VERIFICATION
            else VERIFICATION[row["id"].split("-")[0]]
        )
        row["domain"] = row["id"].split("-")[0]
        row["milestone"] = FEATURE_MILESTONES.get(row["feature"], "M0")
    for row in rows:
        row["milestone"] = milestone_for(row)
    assert_complete(rows)
    rows.sort(key=lambda r: (r["domain"], r["id"]))
    return rows


def render_index(rows: list[dict]) -> str:
    out: list[str] = []
    out.append("# Requirements index — PhairDLNA\n")
    out.append(
        "Status: **Approved (v1.1 accessibility update, 2026-11-18; v1.0 baseline preserved)** · Owner role: Product Owner\n"
    )
    out.append(
        "This is the **cross-cutting view** of every requirement. The normative text of each\n"
        "requirement is defined exactly once — in `TECHNICAL-REQUIREMENTS.md` (technical\n"
        "domains) or `PRODUCT-REQUIREMENTS.md` (product commitments). This file adds the\n"
        "short title, priority, owning feature, milestone, verification method and status,\n"
        "and it is **generated** from those definitions by `scripts/generate_requirement_views.py`.\n"
        "Do not edit it by hand; edit the definition and re-run the script.\n"
    )
    out.append(
        "**Priority vocabulary.** `MUST` = required for correctness, safety or the promise of\n"
        "the product; a milestone with an open `MUST` cannot be accepted. `SHOULD` = strongly\n"
        "desirable, tracked, may delay a milestone. `COULD` = optional enhancement, outside\n"
        "the current baseline. Defined in\n"
        "[../00-governance/REQUIREMENTS-METHOD.md](../00-governance/REQUIREMENTS-METHOD.md) section 3.\n"
    )
    out.append(
        "\n**Status vocabulary.** `Baseline` = part of the approved requirement baseline; no\n"
        "implementation status is claimed. `Scheduled` = assigned to a milestone.\n"
        "`Implemented` = code exists.\n"
        "`Verified` = evidence recorded.\n"
    )
    out.append(
        "\nRequirement method, ID rules and the traceability chain are defined in\n"
        "[../00-governance/REQUIREMENTS-METHOD.md](../00-governance/REQUIREMENTS-METHOD.md).\n"
    )
    out.append("\n---\n")
    for domain, title in [
        ("PRD", "Product commitments"),
        ("FUN", "Functional behaviour"),
        ("INT", "Interoperability (DLNA / UPnP)"),
        ("SEC", "Security and privacy"),
        ("PERF", "Performance and resources"),
        ("UX", "Usability and accessibility"),
        ("OPS", "Operations and administration"),
        ("CFG", "Configuration"),
        ("PLT", "Platform and portability"),
        ("DOC", "Documentation and developer experience"),
        ("SUP", "Supply chain and licensing"),
        ("REL", "Reliability"),
        ("RES", "Adaptive resource management"),
        ("LIFE", "Service lifecycle and network recovery"),
        ("MB", "macOS menu bar application"),
    ]:
        selected = [r for r in rows if r["domain"] == domain]
        if not selected:
            continue
        out.append(f"\n## {domain} — {title}\n")
        out.append("| ID | Requirement | Priority | Feature | Milestone | Verification | Status |")
        out.append("| --- | --- | --- | --- | --- | --- | --- |")
        for r in selected:
            status = EVIDENCE.get(r["id"], ("", "Baseline"))[1]
            out.append(
                f"| `{r['id']}` | {r['title']} | {r['priority']} | `{r['feature']}` | "
                f"{r['milestone']} | {r['verification']} | {status} |"
            )
    counts: dict[str, int] = {}
    for r in rows:
        counts[r["priority"]] = counts.get(r["priority"], 0) + 1
    out.append("\n---\n")
    out.append(
        f"\n**Totals:** {len(rows)} requirements — "
        + ", ".join(f"{v} {k}" for k, v in sorted(counts.items()))
        + ".\n"
    )
    out.append(
        "\n## Coverage check\n\n"
        "A requirement is only *planned* when it has all of: an ID, a definition, a priority,\n"
        "an owning feature, a milestone and a verification method. A requirement missing any\n"
        "of these stays `Draft` and is not part of the baseline.\n"
    )
    out.append(
        "\n## Revision history\n\n"
        "| Version/date | Change |\n"
        "| --- | --- |\n"
        "| 1.1 / 2026-11-18 | Owner direction DEC-077 revises UX-008 and UX-017 for app-owned text scaling through 200% and reflow without clipping; the v1.0 baseline is preserved in `docs/99-archive/2026-11-18-larger-text-accessibility/`. |\n"
    )
    return "\n".join(out)


def render_trace(rows: list[dict]) -> str:
    out: list[str] = []
    out.append("# Requirements traceability matrix\n")
    out.append("Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Lead / QA\n")
    out.append(
        "Generated from the requirement definitions by\n"
        "`scripts/generate_requirement_views.py`. Do not edit by hand.\n"
    )
    out.append(
        "\n**How to read this table.** Every requirement is traced to the feature that owns\n"
        "it, the milestone that delivers it, the module area that will implement it, and the\n"
        "verification that will prove it. The *evidence* column is filled only with output\n"
        "that was actually observed; an empty cell means *not implemented*, not *passed*.\n"
    )
    out.append("\n| ID | Feature | Milestone | Module area | Verification | Evidence | Status |")
    out.append("| --- | --- | --- | --- | --- | --- | --- |")
    module_area = {
        "PRD": "—", "FUN": "crates/phairdlna-core/src/catalogue, crates/phairdlna-core/src/media",
        "INT": "crates/phairdlna-core/src/upnp, crates/phairdlna-core/src/didl",
        "SEC": ("crates/phairdlna-core/src/security, crates/phairdlna-core/src/http, "
                "crates/phairdlna-core/src/upnp"),
        "PERF": "crates/phairdlna-core/src/** (benchmarked)",
        "UX": "crates/phairdlna-cli, crates/phairdlna-mac",
        "OPS": "crates/phairdlna-cli, crates/phairdlna-admin",
        "CFG": "crates/phairdlna-core/src/config",
        "PLT": "crates/phairdlna-core/src/platform",
        "DOC": "docs/**", "SUP": "Cargo.toml, deny.toml, .github/workflows",
        "REL": "crates/phairdlna-core/tests/soak, crates/phairdlna-cli",
        "RES": "crates/phairdlna-core/src/scheduler",
        "LIFE": "crates/phairdlna-core/src/lifecycle, crates/phairdlna-cli",
        "MB": "crates/phairdlna-mac, docs/05-quality",
        "PRD": "cross-cutting (see docs/01-product/FEATURE-CATALOG.md)",
    }
    for r in rows:
        evidence, status = EVIDENCE.get(r["id"], ("—", "Not implemented"))
        out.append(
            f"| `{r['id']}` | `{r['feature']}` | {r['milestone']} | "
            f"`{module_area[r['domain']]}` | {r['verification']} | {evidence} | {status} |"
        )
    out.append(
        "\n---\n\n## How evidence is recorded\n\n"
        "When a requirement is implemented, the *Evidence* cell receives:\n\n"
        "- an automated test: `cargo test -p phairdlna-core catalogue::tests::…` (test name),\n"
        "- a gate: the gate ID and the observed output, or\n"
        "- a device record: the row in `../05-quality/INTEROPERABILITY-MATRIX.md`.\n\n"
        "A cell is only filled with output that was actually observed. A skipped check is\n"
        "written as `NOT RUN — <reason>` and never as a pass.\n"
    )
    return "\n".join(out)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check", action="store_true",
                        help="fail if the generated files are out of date")
    args = parser.parse_args()

    rows = build_rows()
    if not rows:
        print("no requirements parsed", file=sys.stderr)
        return 1
    for path, content in ((INDEX_OUT, render_index(rows)), (TRACE_OUT, render_trace(rows))):
        if args.check:
            if not path.exists() or path.read_text(encoding="utf-8") != content:
                print(f"OUT OF DATE: {path}", file=sys.stderr)
                return 1
            print(f"up to date: {path}")
        else:
            path.write_text(content, encoding="utf-8")
            print(f"written: {path}")
    print(f"{len(rows)} requirements indexed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
