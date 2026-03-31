# Nyx + Uzume Step-1 Foundation (Monad/Nun)

## TL;DR
> **Summary**: Build a production-grade Nyx foundation for Uzume with strict app-isolated identity, chronological feed v1, and security/TDD guardrails that scale to future apps without redesign.
> **Deliverables**:
> - Architecture invariants + ADRs
> - Nun contracts (app identity + feed mode)
> - Heka Ory-Kratos integration + alias/link policy engine
> - Uzume-profiles + Uzume-feed (chronological-only runtime)
> - Security baseline + CI/TDD baseline + deferred event abstraction
> **Effort**: Large
> **Parallel**: YES - 3 waves
> **Critical Path**: 1 -> 2 -> 3 -> 6 -> 7 -> 8 -> 9 -> 10

## Context
### Original Request
- Build Nyx layer-by-layer, beginning with Monad/Nun and focusing on Uzume first.
- Keep architecture OSS-only, privacy-first, and production-capable.
- Preserve three separate public apps (Uzume/Themis/Anteros) over shared Nyx backend primitives.
- Implement chronological feed first, architect for additional feed modes later.
- Use Ory stack direction (Kratos first; Hydra/Oathkeeper as needed later).
- Defer event backbone provider choice.

### Interview Summary
- Repo is starter-grade and can be changed substantially.
- Existing `justfile` and CI workflows are placeholders and must be replaced.
- Super-account global identity is hidden by default; app alias is visible by default.
- Cross-app reveal/linking must be explicit, reversible, and app-selective.
- Initial deployment scope is single-region (Dublin/local), global later.
- Security-first and TDD-first are hard constraints.

### Metis Review (gaps addressed)
- Added anti-scope controls to prevent overbuilding in step-1.
- Added explicit linking semantics coverage (one-way, two-way, app-selective, revoke).
- Added CI/security/testing acceptance gates.
- Added sequencing guardrails to avoid circular dependencies.

## Work Objectives
### Core Objective
Deliver the first executable Nyx+Uzume architecture slice where identity isolation and chronological feed behavior are correct by construction and verified automatically.

### Deliverables
- ADRs that lock platform invariants and step boundaries.
- Nun-level contracts for app identity, linking semantics, and feed mode extensibility.
- Heka integration with Ory Kratos and policy checks.
- Uzume-profiles step-1 endpoints with app-scoped identity exposure.
- Uzume-feed step-1 endpoints with chronological-only behavior.
- Step-1 migrations for required `nyx` + `Uzume` tables.
- Production-grade CI/TDD command workflow replacing placeholders.
- Security baseline with mandatory scanning + abuse tests.
- Event abstraction boundary with provider deferred.

### Definition of Done (verifiable conditions with commands)
- `cargo test --workspace` passes.
- `cargo build --workspace` passes.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- `cargo fmt --all -- --check` passes.
- `cargo deny check` passes.
- `cargo audit` passes with no high/critical advisories.
- `cargo nextest run --workspace` passes (if installed).
- Cross-app default-deny tests pass for identity visibility.
- Chronological-only feed behavior tests pass.

### Must Have
- Strict TDD: RED -> GREEN -> REFACTOR for every implementation task.
- `app_id` isolation in identity, authorization, and data paths.
- Ory Kratos integration via Heka in step-1.
- Chronological feed mode implemented and exposed as sole runtime mode.
- Future feed mode contracts prepared without enabling non-chronological runtime.
- Security checks automated and reproducible locally.

### Must NOT Have (guardrails, AI slop patterns, scope boundaries)
- No ranking/personalization implementation in step-1.
- No final event backbone provider lock-in in step-1.
- No stories/reels/discover implementation in step-1.
- No multi-region/global routing implementation in step-1.
- No cross-app discoverability by default.

## Verification Strategy
> ZERO HUMAN INTERVENTION — all verification is agent-executed.
- Test decision: **TDD (RED-GREEN-REFACTOR)** using Rust unit + integration suites.
- QA policy: every task includes happy-path and failure/edge scenarios.
- Evidence: `.sisyphus/evidence/task-{N}-{slug}.{ext}`.

## Execution Strategy
### Parallel Execution Waves
Wave 1: Contracts, migrations, CI/TDD, security baseline (Tasks 1-5)
Wave 2: Identity + profile/feed implementation (Tasks 6-9)
Wave 3: Integration hardening + deferred boundary + runbook (Tasks 10-12)

### Dependency Matrix (full, all tasks)
| Task | Depends On | Blocks                   |
| ---- | ---------- | ------------------------ |
| 1    | -          | 2,3,4,5,6,7,8,9,10,11,12 |
| 2    | 1          | 3,6,7,8,9,10             |
| 3    | 1,2        | 6,7,8,9,10               |
| 4    | 1          | 6,8,9,10,11,12           |
| 5    | 1,4        | 10,11,12                 |
| 6    | 2,3,4      | 7,8,10,11                |
| 7    | 2,3,6      | 8,10,11                  |
| 8    | 2,3,4,6,7  | 9,10,11                  |
| 9    | 2,3,4,8    | 10,11                    |
| 10   | 6,7,8,9    | 11,12                    |
| 11   | 4,5,10     | 12                       |
| 12   | 5,10,11    | Final verification       |

### Agent Dispatch Summary (wave -> task count -> categories)
- Wave 1 -> 5 tasks -> writing, deep, unspecified-high
- Wave 2 -> 4 tasks -> deep, unspecified-high
- Wave 3 -> 3 tasks -> unspecified-high, writing, deep

## TODOs
> Implementation + Test = ONE task. Never separate.

- [x] 1. Lock architecture invariants and app-boundary ADRs

  **What to do**:
  - Create ADRs in `Seshat/` for: hidden-global-identity default, explicit linking/reveal policy, app-isolation invariants, modular-monolith boundaries.
  - Add state diagrams/tables for one-way, two-way, app-selective, and revoke flows.
  - Add explicit step-1 non-goals and step-2 deferred items.

  **Must NOT do**:
  - Do not define ranking algorithms beyond placeholders.
  - Do not hard-code event provider decisions.

  **Recommended Agent Profile**:
  - Category: `writing` — Reason: architectural decision quality.
  - Skills: `[]`.
  - Omitted: `['/playwright']` — no browser actions needed.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: 2,3,4,5,6,7,8,9,10,11,12 | Blocked By: none

  **References**:
  - Pattern: `Seshat/ARCHITECTURE.md:12-42` — platform/app split.
  - Pattern: `apps/Uzume/README.md:49-115` — profile/feed scope.
  - API/Type: `Monad/Nun/src/id.rs:174-193` — identity/link entity markers.

  **Acceptance Criteria**:
  - [ ] ADRs include invariants, non-goals, and step boundaries.
  - [ ] ADRs include default-private + reveal + revoke policy examples.

  **QA Scenarios**:
  ```
  Scenario: ADR completeness
    Tool: Bash
    Steps: verify ADR files include sections Invariants, Linking Modes, Non-Goals
    Expected: all required sections present
    Evidence: .sisyphus/evidence/task-1-adr-completeness.txt

  Scenario: Scope guardrails
    Tool: Bash
    Steps: inspect ADR text for explicit step-1 exclusions
    Expected: ranking/events-provider/multi-region excluded
    Evidence: .sisyphus/evidence/task-1-adr-guardrails.txt
  ```

  **Commit**: YES | Message: `docs(architecture): lock nyx-uzume step1 invariants` | Files: `Seshat/*.md`

- [x] 2. Implement Nun app/identity/feed-mode contracts

  **What to do**:
  - Add/normalize Nun types for `NyxApp`, `FeedMode`, and link policy semantics.
  - Set chronological as default runtime mode while preserving extensible enum/contract design.
  - Add serde + validation + compatibility tests.

  **Must NOT do**:
  - Do not add app-specific business logic to Nun.
  - Do not add network/db operations in Nun.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: foundational shared contracts.
  - Skills: `[]`.
  - Omitted: `['/dev-browser']`.

  **Parallelization**: Can Parallel: NO | Wave 1 | Blocks: 3,6,7,8,9,10 | Blocked By: 1

  **References**:
  - Pattern: `Monad/Nun/src/id.rs:1-93` — typed id style.
  - Pattern: `Monad/Nun/src/validation.rs` — validation conventions.
  - Pattern: `Monad/Nun/src/lib.rs` — exports and module patterns.

  **Acceptance Criteria**:
  - [ ] Contracts compile and are publicly exported.
  - [ ] Chronological mode is default with future-compatible contract.
  - [ ] Contract tests pass.

  **QA Scenarios**:
  ```
  Scenario: Contract serialization
    Tool: Bash
    Steps: run Nun tests covering parse/serialize for app/mode/policy types
    Expected: stable values and no breaking serde behavior
    Evidence: .sisyphus/evidence/task-2-contract-serde.txt

  Scenario: Invalid policy rejection
    Tool: Bash
    Steps: execute tests for malformed policy values
    Expected: deterministic validation failures
    Evidence: .sisyphus/evidence/task-2-invalid-policy.txt
  ```

  **Commit**: YES | Message: `feat(nun): add app identity and feed-mode contracts` | Files: `Monad/Nun/src/*`

- [x] 3. Create step-1 migration baseline (`nyx` + `Uzume`)

  **What to do**:
  - Create migrations for step-1 tables only: `nyx.app_aliases`, `nyx.app_links`, minimal `Uzume.profiles`, minimal `Uzume.posts`.
  - Add reversible down migrations.
  - Enforce uniqueness and FK constraints required by privacy model.

  **Must NOT do**:
  - Do not include stories/reels/discover schema.
  - Do not include ranking-specific schema.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: schema correctness and isolation.
  - Skills: `[]`.
  - Omitted: `['/playwright']`.

  **Parallelization**: Can Parallel: PARTIAL | Wave 1 | Blocks: 6,7,8,9,10 | Blocked By: 1,2

  **References**:
  - Pattern: `Seshat/FUTURE.md:152-187` — migration structure intent.
  - Pattern: `apps/Uzume/README.md:53-88` — data model intent.

  **Acceptance Criteria**:
  - [ ] Up/down migrations execute cleanly.
  - [ ] Constraints enforce alias/link integrity.

  **QA Scenarios**:
  ```
  Scenario: Migration up/down
    Tool: Bash
    Steps: run migration apply, inspect schema, run revert, re-inspect
    Expected: clean apply and clean rollback
    Evidence: .sisyphus/evidence/task-3-migration-updown.txt

  Scenario: Duplicate alias prevention
    Tool: Bash
    Steps: attempt duplicate alias insertion in same app scope
    Expected: deterministic unique-constraint failure
    Evidence: .sisyphus/evidence/task-3-duplicate-alias.txt
  ```

  **Commit**: YES | Message: `feat(db): add step1 nyx-uzume migrations` | Files: `migrations/Monad/*`, `migrations/Uzume/*`

- [x] 4. Replace placeholder TDD/CI command workflow

  **What to do**:
  - Rewrite `justfile` into project-accurate commands (unit/integration/security/migration checks).
  - Implement `.github/workflows/ci.yml` with fmt/clippy/tests/deny/audit/migration checks.
  - Add deterministic fallback when `cargo-nextest` is unavailable.

  **Must NOT do**:
  - Do not keep placeholder or non-existent path commands.
  - Do not create flaky CI steps.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: CI and workflow reliability.
  - Skills: `[]`.
  - Omitted: `['/frontend-ui-ux']`.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: 6,8,9,10,11,12 | Blocked By: 1

  **References**:
  - Pattern: `justfile:1-27` — current placeholder commands.
  - Pattern: `.github/workflows/ci.yml` — currently empty file.

  **Acceptance Criteria**:
  - [ ] CI enforces fmt/clippy/tests/deny/audit/migrations.
  - [ ] Local command set mirrors CI checks.
  - [ ] CI fails predictably on regressions.

  **QA Scenarios**:
  ```
  Scenario: Local/CI parity
    Tool: Bash
    Steps: run local command bundle equivalent to CI stages
    Expected: same pass/fail behavior as CI
    Evidence: .sisyphus/evidence/task-4-ci-parity.txt

  Scenario: Gate failure behavior
    Tool: Bash
    Steps: introduce known failing check in temporary branch
    Expected: CI exits non-zero and blocks merge
    Evidence: .sisyphus/evidence/task-4-gate-failure.txt
  ```

  **Commit**: YES | Message: `chore(ci): replace placeholder workflow and commands` | Files: `justfile`, `.github/workflows/ci.yml`

- [x] 5. Establish security-first baseline (threat model + scanners)

  **What to do**:
  - Create security baseline document with threat categories, controls, and abuse-case expectations.
  - Integrate secret/dependency/vulnerability checks into local and CI paths.
  - Define required cross-app unauthorized access test gates.

  **Must NOT do**:
  - Do not defer security checks to post-MVP.
  - Do not rely on manual-only security reviews.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: security enforcement setup.
  - Skills: `[]`.
  - Omitted: `['/frontend-ui-ux']`.

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: 10,11,12 | Blocked By: 1,4

  **References**:
  - Pattern: `.github/README.md` — intended quality checks.
  - Pattern: `Seshat/CONTRIBUTING.md` — quality guardrail style.

  **Acceptance Criteria**:
  - [ ] Security baseline doc exists and is actionable.
  - [ ] deny/audit/secret scans run in CI.
  - [ ] cross-app abuse checks are mandatory pass gates.

  **QA Scenarios**:
  ```
  Scenario: Security scan gate
    Tool: Bash
    Steps: run deny/audit/secret scanning command set
    Expected: all checks pass with deterministic output
    Evidence: .sisyphus/evidence/task-5-security-scans.txt

  Scenario: Unauthorized cross-app attempt
    Tool: Bash
    Steps: execute tests calling app-B endpoint with app-A context
    Expected: denied with 401/403
    Evidence: .sisyphus/evidence/task-5-crossapp-abuse.txt
  ```

  **Commit**: YES | Message: `chore(security): add baseline and scan gates` | Files: `Seshat/*`, `.github/workflows/ci.yml`, `justfile`

- [ ] 6. Implement Heka Ory Kratos client core

  **What to do**:
  - Implement Heka modules for Kratos client, session validation, identity lookup, and typed response handling.
  - Map provider and network failures into Nun standardized error model.
  - Add tests for valid/expired/malformed session paths.

  **Must NOT do**:
  - Do not implement Hydra/Oathkeeper in step-1.
  - Do not leak Ory internal payloads into app service contracts.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: external identity integration correctness.
  - Skills: `[]`.
  - Omitted: `['/playwright']`.

  **Parallelization**: Can Parallel: PARTIAL | Wave 2 | Blocks: 7,8,10,11 | Blocked By: 2,3,4

  **References**:
  - Pattern: `Monad/Heka/README.md:8-49` — intended API surface.
  - Pattern: `Monad/Nun/src/error.rs` — error taxonomy.
  - External: `https://www.ory.sh/docs/kratos/reference/api`.

  **Acceptance Criteria**:
  - [ ] Heka API methods match documented contracts.
  - [ ] Error mapping is deterministic and standardized.
  - [ ] Session validation tests cover happy + failure paths.

  **QA Scenarios**:
  ```
  Scenario: Valid session handling
    Tool: Bash
    Steps: run Heka tests with valid session fixture
    Expected: authenticated identity returned
    Evidence: .sisyphus/evidence/task-6-valid-session.txt

  Scenario: Invalid session handling
    Tool: Bash
    Steps: run Heka tests with expired/malformed token fixtures
    Expected: unauthorized error classification
    Evidence: .sisyphus/evidence/task-6-invalid-session.txt
  ```

  **Commit**: YES | Message: `feat(heka): implement kratos client core` | Files: `Monad/Heka/src/*`, `Monad/Heka/tests/*`

- [ ] 7. Implement alias/link policy engine (one-way/two-way/app-selective)

  **What to do**:
  - Implement alias resolution and policy evaluation for one-way, two-way, app-selective linking.
  - Implement revoke semantics returning visibility to private default.
  - Define deterministic conflict precedence.

  **Must NOT do**:
  - Do not make global identity visible by default.
  - Do not allow implicit link creation.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: privacy-critical correctness.
  - Skills: `[]`.
  - Omitted: `['/frontend-ui-ux']`.

  **Parallelization**: Can Parallel: NO | Wave 2 | Blocks: 8,10,11 | Blocked By: 2,3,6

  **References**:
  - Pattern: `Monad/Heka/README.md:5-18` — alias/link semantics.
  - API/Type: `Monad/Nun/src/id.rs:184-193` — alias/link entities.

  **Acceptance Criteria**:
  - [ ] Default-private behavior enforced.
  - [ ] One-way/two-way/app-selective tests pass.
  - [ ] Revocation immediately affects policy outcome.

  **QA Scenarios**:
  ```
  Scenario: Default-private policy
    Tool: Bash
    Steps: run policy tests with no links
    Expected: cross-app reveal denied
    Evidence: .sisyphus/evidence/task-7-default-private.txt

  Scenario: Link + revoke behavior
    Tool: Bash
    Steps: apply link policies then revoke
    Expected: visibility transitions match policy table
    Evidence: .sisyphus/evidence/task-7-link-revoke.txt
  ```

  **Commit**: YES | Message: `feat(identity): add alias-link policy engine` | Files: `Monad/Heka/src/*`, `Monad/Heka/tests/*`, `migrations/Monad/*`

- [ ] 8. Implement Uzume-profiles step-1 service

  **What to do**:
  - Implement step-1 profiles endpoints (`GET /me`, `PATCH /me`, public profile read).
  - Integrate app-scoped identity context and policy checks.
  - Add tests for authorized/unauthorized/forbidden flows.

  **Must NOT do**:
  - Do not implement follow/block/mute unless required by step-1 feed baseline.
  - Do not expose global identity in response payloads.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: service + auth integration.
  - Skills: `[]`.
  - Omitted: `['/playwright']`.

  **Parallelization**: Can Parallel: PARTIAL | Wave 2 | Blocks: 9,10,11 | Blocked By: 2,3,4,6,7

  **References**:
  - Pattern: `apps/Uzume/README.md:49-76` — profiles scope.
  - Pattern: `Monad/Nun/src/testing.rs` — test utility conventions.

  **Acceptance Criteria**:
  - [ ] Profiles endpoints compile and pass tests.
  - [ ] Public payloads show app alias only.
  - [ ] Unauthorized requests return standardized errors.

  **QA Scenarios**:
  ```
  Scenario: Profile lifecycle
    Tool: Bash
    Steps: create/fetch/update profile in API tests
    Expected: success responses with app-scoped fields only
    Evidence: .sisyphus/evidence/task-8-profile-lifecycle.txt

  Scenario: Unauthorized access
    Tool: Bash
    Steps: call protected endpoints without valid auth context
    Expected: 401/403 with standard error body
    Evidence: .sisyphus/evidence/task-8-profile-unauthorized.txt
  ```

  **Commit**: YES | Message: `feat(uzume-profiles): implement step1 service` | Files: `apps/Uzume/Uzume-profiles/*`

- [ ] 9. Implement Uzume-feed chronological v1 + latent multi-mode contract

  **What to do**:
  - Implement step-1 feed endpoints for create/get/delete posts + home feed.
  - Enforce deterministic chronological ordering.
  - Preserve mode contract extensibility while exposing only chronological runtime behavior.

  **Must NOT do**:
  - Do not implement ranking/personalization in step-1.
  - Do not implement hashtag/discover/celebrity fanout logic in step-1.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: feed correctness and future-compatibility.
  - Skills: `[]`.
  - Omitted: `['/playwright']`.

  **Parallelization**: Can Parallel: NO | Wave 2 | Blocks: 10,11 | Blocked By: 2,3,4,8

  **References**:
  - Pattern: `apps/Uzume/README.md:77-115` — feed scope.
  - Pattern: `Monad/Nun/src/pagination.rs` — cursor behavior.
  - Pattern: `Monad/Nun/src/id.rs:40-44` — time-sortable IDs.

  **Acceptance Criteria**:
  - [ ] Home feed returns newest-first deterministic order.
  - [ ] Only chronological mode is exposed in step-1 runtime.
  - [ ] Tests verify no ranking path is used.

  **QA Scenarios**:
  ```
  Scenario: Chronological ordering
    Tool: Bash
    Steps: seed posts with known timestamps and fetch feed
    Expected: strict descending order with stable tie-breaks
    Evidence: .sisyphus/evidence/task-9-feed-chronological.txt

  Scenario: Unsupported mode behavior
    Tool: Bash
    Steps: request non-chronological mode in tests
    Expected: deterministic reject or fallback to chronological
    Evidence: .sisyphus/evidence/task-9-feed-mode-handling.txt
  ```

  **Commit**: YES | Message: `feat(uzume-feed): implement chronological v1` | Files: `apps/Uzume/Uzume-feed/*`

- [ ] 10. Add identity/privacy integration and abuse test suite

  **What to do**:
  - Add integration tests across Heka + profiles + feed for default-deny and explicit-link paths.
  - Add abuse tests for forged app context, revoked links, unauthorized alias resolution.
  - Ensure deterministic CI execution.

  **Must NOT do**:
  - Do not rely on unit tests alone for privacy invariants.
  - Do not skip failure-path assertions.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: cross-service policy verification.
  - Skills: `[]`.
  - Omitted: `['/frontend-ui-ux']`.

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: 11,12 | Blocked By: 6,7,8,9

  **References**:
  - Pattern: `Monad/Heka/README.md` — policy API.
  - Pattern: `Monad/Nun/src/error.rs` — standardized error contracts.

  **Acceptance Criteria**:
  - [ ] Default-deny cross-app behavior proven by integration tests.
  - [ ] Explicit linking grants only intended visibility.
  - [ ] Revocation reverts visibility immediately.

  **QA Scenarios**:
  ```
  Scenario: Deny-by-default matrix
    Tool: Bash
    Steps: run integration suite with no active links
    Expected: cross-app access denied in all cases
    Evidence: .sisyphus/evidence/task-10-deny-default.txt

  Scenario: Revoke enforcement
    Tool: Bash
    Steps: grant link, validate access, revoke, re-validate
    Expected: immediate access denial after revoke
    Evidence: .sisyphus/evidence/task-10-revoke-enforcement.txt
  ```

  **Commit**: YES | Message: `test(identity): add cross-app privacy integration suite` | Files: `Monad/Heka/tests/*`, `apps/Uzume/*/tests/*`

- [ ] 11. Add event abstraction boundary (provider deferred)

  **What to do**:
  - Define provider-agnostic event interface used by step-1 services.
  - Provide in-memory/no-op adapter for step-1 tests and runtime.
  - Ensure service code depends only on abstraction.

  **Must NOT do**:
  - Do not choose or optimize for a concrete event provider in step-1.
  - Do not leak provider metadata into domain logic.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: architecture decoupling.
  - Skills: `[]`.
  - Omitted: `['/playwright']`.

  **Parallelization**: Can Parallel: PARTIAL | Wave 3 | Blocks: 12 | Blocked By: 4,5,10

  **References**:
  - Pattern: `Monad/events/` — existing events crate location.
  - Pattern: `apps/Uzume/README.md:109-113` — event touchpoints.

  **Acceptance Criteria**:
  - [ ] Event abstraction is documented and implemented.
  - [ ] Step-1 services compile against abstraction only.
  - [ ] In-memory adapter supports deterministic tests.

  **QA Scenarios**:
  ```
  Scenario: Provider-agnostic compile
    Tool: Bash
    Steps: build workspace with abstraction adapter only
    Expected: successful build without provider-specific deps
    Evidence: .sisyphus/evidence/task-11-provider-agnostic-build.txt

  Scenario: In-memory event behavior
    Tool: Bash
    Steps: run tests using in-memory event adapter
    Expected: deterministic publish/consume behavior
    Evidence: .sisyphus/evidence/task-11-inmemory-events.txt
  ```

  **Commit**: YES | Message: `refactor(events): add provider-agnostic boundary` | Files: `Monad/events/*`, `apps/Uzume/*`

- [ ] 12. Single-region local deployment/runbook hardening for step-1

  **What to do**:
  - Create operational runbook for single-region startup, health checks, backup/restore drill, and incident triage.
  - Ensure commands and paths match actual repository state.
  - Document assumptions for later multi-region evolution without implementing it.

  **Must NOT do**:
  - Do not add global routing/distribution implementation.
  - Do not document non-existent procedures as complete.

  **Recommended Agent Profile**:
  - Category: `writing` — Reason: operations clarity and reliability.
  - Skills: `[]`.
  - Omitted: `['/playwright']`.

  **Parallelization**: Can Parallel: NO | Wave 3 | Blocks: Final verification | Blocked By: 5,10,11

  **References**:
  - Pattern: `Seshat/FUTURE.md:62-82` — infra process intent.
  - Pattern: `justfile:1-7` — local startup command baseline.

  **Acceptance Criteria**:
  - [ ] Runbook includes concrete startup and verification commands.
  - [ ] Health checks and incident triage procedures are defined.
  - [ ] Backup/restore drill steps are executable.

  **QA Scenarios**:
  ```
  Scenario: Fresh bring-up
    Tool: Bash
    Steps: follow runbook from clean environment
    Expected: required services start and health checks pass
    Evidence: .sisyphus/evidence/task-12-bringup.txt

  Scenario: Recovery drill
    Tool: Bash
    Steps: execute backup and restore drill in local environment
    Expected: restored state is valid and services return healthy
    Evidence: .sisyphus/evidence/task-12-recovery.txt
  ```

  **Commit**: YES | Message: `docs(ops): add step1 single-region runbook` | Files: `Seshat/*`, `Prithvi/*` (if created)

## Final Verification Wave (MANDATORY — after ALL implementation tasks)
> 4 review agents run in PARALLEL. ALL must APPROVE. Wait for explicit user okay before marking complete.
- [ ] F1. Plan Compliance Audit — oracle
- [ ] F2. Code Quality Review — unspecified-high
- [ ] F3. Real Manual QA — unspecified-high (+ playwright if UI)
- [ ] F4. Scope Fidelity Check — deep

## Commit Strategy
- Use atomic commits by concern and keep commit size small.
- Follow test-first flow where practical (RED commit -> GREEN commit -> REFACTOR commit).
- Commit message format: `type(scope): description`.
- Never merge with failing quality/security gates.

## Success Criteria
- Hidden-by-default global identity with explicit reveal/revoke behavior is enforced by tests.
- Uzume feed is chronological-only in user-visible behavior for step-1.
- CI/TDD/security gates are active and block regressions.
- Architecture remains extensible for additional apps and feed modes.
