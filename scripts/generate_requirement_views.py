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
    "INT-044": (
        "`crates/phairdlna-core/tests/device_description.rs::the_service_list_names_both_mandatory_services`",
        "Verified",
    ),
    "INT-045": (
        "`crates/phairdlna-core/tests/device_description.rs::the_document_holds_exactly_one_device_element`",
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
        "SEC": "crates/phairdlna-core/src/security, crates/phairdlna-core/src/http",
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
