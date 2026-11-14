# Documentation map — PhairDLNA

This is the map of the documentation tree. Every document has exactly one home, and
the structure follows [00-governance/DOCUMENTATION-POLICY.md](00-governance/DOCUMENTATION-POLICY.md).

```
docs/
├── 00-governance/   how the project works: standards, roles, decisions, exceptions, requirement method
├── 01-product/      what the product is and must do: vision, requirements, journeys, index
├── 02-architecture/ how it is built: components, data flows, decisions (adr/)
├── 03-features/     one specification per feature
├── 04-security/     the threat model, the privacy model, dependencies and licences
├── 05-quality/      gates, test strategy, interoperability evidence, performance budgets
├── 06-development/  setup, build, test, troubleshooting
├── 07-project/      plan, traceability, risks, status, decisions, CI, release blockers
├── 08-releases/     release policy, distribution, provenance
└── 99-archive/      immutable snapshots of superseded baselines
```

## Where to start

| You are … | Start at |
| --- | --- |
| a newcomer who wants to use it | [07-project/STATUS.md](07-project/STATUS.md) — it does not exist yet, and this says why |
| a newcomer who wants to build it | [06-development/GETTING-STARTED.md](06-development/GETTING-STARTED.md) |
| a contributor | [../WHERE_TO_CHANGE.md](../WHERE_TO_CHANGE.md) |
| a reviewer | [01-product/REQUIREMENTS-INDEX.md](01-product/REQUIREMENTS-INDEX.md) and [07-project/TRACEABILITY-MATRIX.md](07-project/TRACEABILITY-MATRIX.md) |
| the owner | [07-project/STATUS.md](07-project/STATUS.md) and [07-project/OPEN-DECISIONS.md](07-project/OPEN-DECISIONS.md) |

## The three documents that explain the most

1. **[01-product/TECHNICAL-REQUIREMENTS.md](01-product/TECHNICAL-REQUIREMENTS.md)** — the
   normative catalogue. Every requirement is defined here exactly once.
2. **[02-architecture/ARCHITECTURE.md](02-architecture/ARCHITECTURE.md)** — how the
   system is built and why.
3. **[05-quality/QUALITY-GATES.md](05-quality/QUALITY-GATES.md)** — what "done" means
   and how it is checked.

## Generated documents

Two documents are generated and must never be edited by hand:

| Document | Generator |
| --- | --- |
| [01-product/REQUIREMENTS-INDEX.md](01-product/REQUIREMENTS-INDEX.md) | `python3 scripts/generate_requirement_views.py` |
| [07-project/TRACEABILITY-MATRIX.md](07-project/TRACEABILITY-MATRIX.md) | same command |

Both are checked by `python3 scripts/generate_requirement_views.py --check`.
