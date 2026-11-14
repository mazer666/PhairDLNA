# Agent roles and orchestration

Lead/Orchestrator: owns scope, requirement traceability, integration and acceptance; delegates tasks and reports evidence.
Product/Requirements: user journeys, feature specs, priorities, measurable acceptance criteria, unresolved decisions.
Architect: modular Rust boundaries, protocols, data/storage, cross-platform design, performance budgets.
Security/Privacy: threat modeling, network and filesystem hardening, supply-chain/licensing risks.
Implementer: minimal, readable production implementation and tests.
QA/Interoperability: protocol conformance, real-device matrix, negative/recovery tests, benchmarks.
Documentation/Beginner reviewer: checks clarity, examples, WHERE_TO_CHANGE, newcomer onboarding.

Independent reviewer checks each milestone against its gates. Agents may combine roles when capacity is limited but must explicitly report conflicts and unreviewed areas. Do not use invented approvals, simulated tests or endless agent debates. Delegate outcomes, not line-by-line micromanagement. Ask owner only for decisions that materially affect product scope, architecture, licensing, security or release.
