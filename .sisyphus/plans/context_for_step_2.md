# Context for Step 2 (Parallel Track)

## Purpose
This document is the handoff contract between:
- **Step 1 track**: Nyx + Uzume foundation (identity/privacy/contracts/chronological feed/security/TDD)
- **Step 2 track**: next parallel build stream that must remain compatible with Step 1 outputs

Goal: enable fast parallel work without collisions, regressions, or architecture drift.

## Current Program Constraints
- Production-grade quality bar from day 1.
- OSS/public codebase: security-first and explicit threat handling.
- Three separate public products: Uzume, Themis, Anteros (separate apps/domains/stores).
- Shared Nyx backend primitives with strict app isolation by default.
- Single-region initial operating model (Dublin/local), global later.

## What Step 2 SHOULD do (parallel-safe)
1. Build **non-breaking extension points** around Step 1 contracts.
2. Prepare **feed mode expansion scaffolding** (ranking/custom) without exposing modes yet.
3. Prepare **Ory stack expansion interfaces** (Hydra/Oathkeeper integration points) without coupling Step 1 runtime.
4. Build **operational hardening artifacts** that do not change Step 1 API contracts (observability, runbook improvements, security checks).
5. Build **privacy policy test matrix expansion** (edge-case suites) against Step 1 identity/linking semantics.

## What Step 2 can EXPECT to be done by Step 1
Step 1 is expected to deliver:
- Core Nyx identity model with app-scoped visibility defaults.
- Alias/link policy baseline with explicit link + revoke flows.
- Uzume profile baseline + chronological feed mode only.
- Security baseline and CI quality gates.
- TDD-first command/workflow baseline.
- Event bus decision deferred behind abstraction.

Step 2 may assume these are stable contracts unless explicitly marked experimental.

## DO NOT TOUCH (Step 2 guardrails)
Step 2 must not modify these without explicit contract update:
1. Default privacy invariant: global Nyx identity hidden by default.
2. Step 1 public endpoint semantics for profiles/feed (chronological behavior).
3. Step 1 link/revoke behavior semantics.
4. Step 1 CI quality gates (fmt/clippy/tests/security checks).
5. Step 1 migration assumptions for identity/profile/feed baseline tables.

Step 2 must not:
- Enable ranking mode in user-visible API/UI.
- Choose/finalize event backbone provider.
- Introduce cross-app discoverability-by-default behavior.
- Add multi-region runtime assumptions into Step 1 path.

## Integration Contract: Feed Modes
- Mode contract can include: `chronological`, `ranking`, `custom`.
- **Only `chronological` is executable/visible in Step 1**.
- Step 2 may add adapters/interfaces/tests for future modes but must keep mode exposure flag off.

## Integration Contract: Identity & Linking
Canonical user story (must remain true):
1. User has hidden global Nyx identity and app-visible alias by default.
2. Without linking, global handle is not searchable in other apps.
3. With explicit link policy, global handle can be revealed/searchable per policy scope.
4. Revoke must return visibility to private default behavior.
5. Directional, bilateral, and app-selective linking behaviors are supported by policy model.

## Security-First Requirements (both tracks)
- Threat model must be kept current with architecture changes.
- Every new behavior requires abuse-case tests (not only happy path).
- No new endpoint ships without authz + data exposure checks.
- Dependency, vulnerability, and secret scanning remain mandatory gates.

## Parallel Change Protocol
When Step 2 needs a contract change:
1. Add `PROPOSED CHANGE` section in this document.
2. Include: reason, impacted contracts, migration risk, rollback plan.
3. Mark status: `proposed` -> `accepted` -> `implemented`.
4. Step 1 implementers update plan/contracts before Step 2 consumes change.

## Real-Time Update Template
Use this block for communication updates:

```
## Update <timestamp>
Track: Step1 | Step2
Change: <what changed>
Impact: <contracts/endpoints/policies affected>
Action Required: <none | specific>
Owner: <name/agent>
Status: proposed|accepted|implemented|blocked
```

## Decisions Deferred Intentionally
- Final event backbone provider selection.
- User-visible ranking/custom feed mode rollout.
- Multi-region/global distribution topology.
- Full Hydra/Oathkeeper production wiring (only extension points in parallel).

## Exit Criteria for Step 2 readiness
Step 2 is considered parallel-safe when:
- No Step 1 contract is broken.
- All Step 2 changes pass Step 1 CI/security gates.
- Step 2 adds measurable capability without increasing privacy leakage risk.
- Handoff notes in this file are updated for every cross-track change.
