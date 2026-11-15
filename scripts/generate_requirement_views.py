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
    "INT-020": (
        "`crates/phairdlna-core/tests/device_description.rs::the_root_device_carries_its_identity`"
        " — the document is generated; it is not served over HTTP yet",
        "Implemented",
    ),
    "INT-021": (
        "`crates/phairdlna-core/tests/device_description.rs::the_service_list_names_both_mandatory_services`",
        "Verified",
    ),
    "INT-022": (
        "`crates/phairdlna-core/tests/device_description.rs::every_service_entry_carries_its_addresses`"
        " — the URLs are absolute; their resolution against the request host needs the HTTP layer",
        "Implemented",
    ),
    "INT-023": (
        "`crates/phairdlna-core/tests/scpd.rs::every_state_variable_declares_its_data_type`"
        " — the service descriptions are generated; they are not served over HTTP yet",
        "Implemented",
    ),
    "INT-024": (
        "`crates/phairdlna-core/tests/device_description.rs::the_document_is_well_formed`",
        "Verified",
    ),
    "INT-027": (
        "`crates/phairdlna-core/tests/device_description.rs::a_name_with_reserved_characters_is_escaped`",
        "Verified",
    ),
    "INT-028": (
        "`crates/phairdlna-core/tests/device_description.rs::the_content_directory_is_advertised_in_version_one`"
        " — the service is advertised; serving it to a client needs the HTTP layer",
        "Implemented",
    ),
    "INT-029": (
        "`crates/phairdlna-core/tests/device_description.rs::the_connection_manager_is_advertised_without_transcoding`"
        " — the service is advertised; the reachability of its control URL needs the HTTP layer",
        "Implemented",
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
        "`crates/phairdlna-core/tests/content_directory.rs::the_system_update_identifier_grows_with_the_catalogue`"
        " — the handler returns the identifier of the catalogue; the catalogue owns its growth",
        "Implemented",
    ),
    "INT-038": (
        "`crates/phairdlna-core/tests/content_directory.rs::the_root_and_the_content_directory_root_are_known`"
        " — the placeholder catalogue answers; the real one is F-09",
        "Implemented",
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
        "`crates/phairdlna-core/tests/content_directory.rs::the_result_is_a_didl_lite_document_that_parses_on_its_own`"
        " — the placeholder writer of F-05 answers; the final writer is not written yet",
        "Implemented",
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
        "`crates/phairdlna-core/tests/connection_manager.rs::a_preparation_creates_no_transfer`"
        " — no connection is ever opened; the transfer that binds a connection to a media object needs the HTTP layer",
        "Implemented",
    ),
    "INT-055": (
        "`crates/phairdlna-core/tests/connection_manager.rs::every_entry_carries_the_wildcard_and_names_no_profile`",
        "Verified",
    ),
    "INT-056": (
        "`crates/phairdlna-core/tests/connection_manager.rs::no_entry_uses_the_push_protocol`",
        "Verified",
    ),
    "SEC-010": (
        "`crates/phairdlna-core/tests/xml_reader.rs::a_document_type_declaration_is_refused`",
        "Verified",
    ),
    "SEC-060": (
        "`crates/phairdlna-core/tests/xml_reader.rs::a_document_that_is_too_large_is_refused`",
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
        " — the attributes are written; the client that reads them is not tested yet",
        "Implemented",
    ),
    "INT-061": (
        "`crates/phairdlna-core/tests/didl_lite.rs::the_classes_are_the_ones_of_the_schema`",
        "Verified",
    ),
    "INT-062": (
        "`crates/phairdlna-core/tests/didl_lite.rs::the_identifier_is_stable_and_carries_no_path`"
        " — the identifier of the fixture is opaque; the hash of ARCHITECTURE §8 is F-08",
        "Implemented",
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
        "`crates/phairdlna-core/tests/didl_lite.rs::the_artwork_address_carries_no_dlna_attribute`"
        " — the address is built; the endpoint that serves it is F-10",
        "Implemented",
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
        " — the attribute carries the number the catalogue read; the unit is the catalogue's contract",
        "Implemented",
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
        " — the mode is chosen by the dispatcher; the client profile that selects it is F-07",
        "Implemented",
    ),
    "INT-079": (
        "`crates/phairdlna-core/tests/didl_resource.rs::nothing_is_claimed_that_the_server_cannot_prove`"
        " — the tokens are written as the requirement asks; the conformance check of a profile does not"
        " exist yet, so the catalogue claims no profile",
        "Implemented",
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
        "`crates/phairdlna-core/tests/http_media.rs::a_file_without_a_type_is_sent_as_octet_stream`"
        " — the type comes from the catalogue, which hands it over already refined; the sniffing is F-10",
        "Implemented",
    ),
    "INT-088": (
        "`crates/phairdlna-core/tests/http_media.rs::the_answer_carries_a_length_and_no_chunked_encoding`",
        "Verified",
    ),
    "INT-089": (
        "`crates/phairdlna-core/tests/http_media.rs::the_answer_names_the_file_to_read`"
        " — the transfer slot and the file descriptor belong to the runtime (F-13, F-18)",
        "Implemented",
    ),
    "INT-090": (
        "`crates/phairdlna-core/tests/http_media.rs::the_url_keeps_its_meaning`",
        "Verified",
    ),
    "INT-091": (
        "`crates/phairdlna-core/tests/http_media.rs::the_url_keeps_its_meaning`"
        " — reopening a file that was removed and restored is the file layer's (F-08, F-13)",
        "Implemented",
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
        "`crates/phairdlna-core/tests/http_path_containment.rs::a_traversal_attempt_is_answered_with_404`"
        " — the repetition of the check with the canonical path belongs to the runtime",
        "Implemented",
    ),
    "INT-099": (
        "`crates/phairdlna-core/tests/http_dlna_headers.rs::a_media_answer_carries_the_dlna_headers`",
        "Verified",
    ),
    "FUN-040": (
        "`crates/phairdlna-core/tests/http_media.rs::the_client_receives_the_bytes_of_the_file`"
        " — the bytes come from the file, never from a conversion",
        "Verified",
    ),
    "FUN-041": (
        "`crates/phairdlna-core/tests/http_media.rs::the_type_of_the_answer_is_the_type_of_the_file`"
        " — the codec is part of the type and of the DLNA profile token",
        "Implemented",
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
    out.append("Status: **Approved (v1.0 baseline, 2026-11-14)** · Owner role: Product Owner\n")
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
        "\n**Status vocabulary.** `Baseline` = defined, internally consistent and traceable,\n"
        "awaiting the owner's approval of the baseline as a whole (it is `Approved` minus the\n"
        "sign-off). `Scheduled` = assigned to a milestone. `Implemented` = code exists.\n"
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
