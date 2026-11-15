# Multi-agent task — evaluate every competitor feature

Status: Requirements work order. **No feature implementation until baseline approval.**

## Objective
Read docs/01-product/COMPETITOR-FEATURE-ATLAS.md and original sources. Extend/verify the per-product inventories, starting with Elmedia, FireStream and JustStream, then UMS, Serviio, Gerbera, Jellyfin, Plex, ReadyMedia, Rygel, VuIO and Spritz.

## Deliverables
1. A versioned, source-backed feature inventory per competitor (not just a summary), with edition/platform constraints, uncertainty and current source URLs.
2. A cross-product feature matrix linking each capability to one or more user outcomes.
3. One evaluation record per meaningful feature using docs/05-quality/FEATURE-EVALUATION-TEMPLATE.md.
4. CORE/OPTIONAL/LATER/REJECT recommendations with evidence, impact, cost, security/privacy/performance and differentiation.
5. Proposed PhairDLNA requirements with stable IDs and measurable acceptance tests for approved candidates.
6. Simple and Advanced UX mapping, opt-in defaults, off-switch and no hidden background work.
7. Independent architecture/security/QA/UX review and an owner decision list.
8. Updated roadmap, traceability, WHERE_TO_CHANGE and archived superseded baselines as applicable.

## Guardrails
Do not simply clone competitor features. Propose a genuinely better user outcome; when improvement is not credible, say so. Distinguish server functions from player/caster/screen-mirroring scope. Prefer minimal core and optional modular features. Never add forced accounts, internet requests, tracking, advertising, invasive permissions, auto port forwarding or unbounded background processing. Security and streaming performance are non-negotiable gates. Any external metadata/subtitle service is opt-in with explicit privacy disclosures. Do not claim format/device compatibility without real tests.

## Agent division
Product/UX: value and user journeys.
Protocol specialist: standards and real device behaviors.
Architect: module boundaries and platform fit.
Security/licensing: threats, permissions, dependencies and provenance.
Performance/QA: reproducible benchmarks and acceptance.
Beginner documentation: comprehensibility, code size and navigation.

## Completion
Commit documentation and verified research. Produce short report: coverage, recommendations, risks, open owner decisions and next milestone. No releases/tags. Manual-only CI. No actual feature development before requirements baseline sign-off.
