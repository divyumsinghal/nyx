# Nyx Step 2 — Uzume Stories Final (Monad-First, Production)

## TL;DR
> **Summary**: Deliver the final production Uzume Stories service end-to-end in a single release train, with shared capabilities implemented in Monad first and Uzume-specific domain logic implemented in Uzume-stories, while preserving all Step 1 contracts.
> **Deliverables**:
> - Monad shared stories primitives (storage/media/events/cache/repo helpers)
> - Uzume-stories service (image+video, feed, viewers, interactions, highlights)
> - Async event-driven media pipeline integration (upload → process → ready)
> - 24h expiry + retention workers
> - Security, abuse, rollback, and operational runbook evidence
> **Effort**: XL
> **Parallel**: YES - 4 waves
> **Critical Path**: 1 → 2 → 4 → 6 → 8 → 10 → 12

## Context
### Original Request
- Build Step 2 in parallel with Step 1.
- Treat Stories as final production implementation (no planned re-build later).
- Keep strict Monad/Uzume litmus split: shared/common in Monad, app-specific in Uzume.
- Follow strict TDD and security-first delivery.

### Interview Summary
- Scope locked to **full Stories feature**: image+video stories, 24h expiry worker, viewers list, interactions (poll/question/slider), highlights.
- Release model locked to **single release train**.
- **No mocks** permitted.
- Step 2 must not edit Step 1 scope or break Step 1 contracts.
- Async event-driven processing is preferred over synchronous media processing.

### Metis Review (gaps addressed)
- Front-loaded contract and compatibility gates before feature breadth.
- Added anti-scope guardrails to prevent Step 1 semantic drift.
- Added non-negotiable idempotency, rollback, and abuse-path requirements.
- Added explicit dependency protocol when Step 1 contracts are required.

## Work Objectives
### Core Objective
Ship a production-grade, final Uzume Stories capability with full media + interaction features by building reusable shared components in Monad and composing them in Uzume-stories without violating Step 1 identity/privacy/feed contracts.

### Deliverables
- Monad crates extended with Stories-ready shared functionality:
  - `Monad/Akash` storage APIs + path conventions for stories/higlights media.
  - `Monad/Oya` async image/video processing flows used by stories.
  - `Monad/events` typed story event subjects/payloads.
  - `Monad/Lethe` cache key + TTL policies for stories feeds/viewers/highlights.
  - `Monad/Mnemosyne` migration/repository helpers used by Uzume-stories.
  - `Monad/api` reusable middleware/extractors for stories upload/auth/request-id/rate limits.
- `apps/Uzume/Uzume-stories` service:
  - APIs: create story, stories feed, story read+view mark, viewers list, interactions, highlights CRUD/read.
  - Workers: expiry worker + retention cleanup + media state reconciler.
  - SQL schema/migrations for stories domain.
  - Comprehensive TDD suites (unit/integration/abuse).
- Gateway + routing integration:
  - `Monad/Heimdall` forwards `/api/Uzume/stories/*`.
- Production release controls:
  - rollback runbook, observability dashboards/checks, CI gates green.

### Definition of Done (verifiable conditions with commands)
- `just fmt-check` passes.
- `just lint` passes.
- `just security` passes.
- `just test` passes.
- `cargo nextest run --workspace -E 'package(uzume-stories)'` passes.
- `cargo nextest run --workspace -E 'package(oya)|package(akash)|package(events)|package(lethe)|package(mnemosyne)|package(api)'` passes.
- `cargo nextest run --workspace --test '*stories*'` passes.
- Rollback drill evidence exists and validates restore to pre-release behavior.

### Must Have
- Full stories scope in this step (no defer): image+video, interactions, highlights, viewer controls, expiry.
- Monad-first reuse: shared logic implemented once in Monad crates.
- Uzume-stories contains only Uzume domain behavior and service composition.
- Async event-driven media lifecycle.
- Strict authz and privacy checks on every endpoint.
- Idempotent write paths and retry-safe workers.

### Must NOT Have (guardrails, AI slop patterns, scope boundaries)
- No Step 1 contract changes (privacy defaults, chronological feed semantics, link/revoke semantics, baseline migration assumptions).
- No ranking/custom mode exposure.
- No event provider finalization.
- No cross-app discoverability by default.
- No mocks or fake contract shims.
- No synchronous heavy media processing in request path.

## Verification Strategy
> ZERO HUMAN INTERVENTION — all verification is agent-executed.
- Test decision: **TDD (RED-GREEN-REFACTOR)** for every task.
- QA policy: every task includes happy path + failure/abuse scenario.
- Evidence: `.sisyphus/evidence/task-{N}-{slug}.{ext}`.
- Release gate: all tests + abuse + rollback evidence required.

## Execution Strategy
### Parallel Execution Waves
Wave 1: Contract + shared Monad primitives + schema baseline (Tasks 1-4)
Wave 2: Uzume-stories core API + authz/privacy + feed/viewers (Tasks 5-8)
Wave 3: Interactions + highlights + workers + media async finalization (Tasks 9-12)
Wave 4: Security hardening + rollback + release validation (Tasks 13-14)

### Dependency Matrix (full, all tasks)
| Task | Depends On | Blocks |
| ---- | ---------- | ------ |
| 1 | - | 2,3,4,5,6,7,8,9,10,11,12,13,14 |
| 2 | 1 | 5,6,7,8,9,10,11,12 |
| 3 | 1 | 5,9,10,11,12 |
| 4 | 1 | 5,6,7,8,9,10,11,12 |
| 5 | 1,2,3,4 | 6,7,8,9,10,11,12,13 |
| 6 | 5 | 7,8,13 |
| 7 | 5,6 | 8,13 |
| 8 | 5,6,7 | 9,10,13 |
| 9 | 5,8 | 10,12,13 |
| 10 | 5,8,9 | 11,12,13 |
| 11 | 3,5,10 | 12,13 |
| 12 | 9,10,11 | 13,14 |
| 13 | 6,7,8,9,10,11,12 | 14 |
| 14 | 13 | Final verification |

### Agent Dispatch Summary (wave → task count → categories)
- Wave 1 → 4 tasks → deep, unspecified-high
- Wave 2 → 4 tasks → unspecified-high, deep
- Wave 3 → 4 tasks → deep, unspecified-high
- Wave 4 → 2 tasks → unspecified-high, writing

## TODOs
> Implementation + Test = ONE task. Never separate.

- [ ] 1. Contract lock + Step 1 compatibility guardrails

  **What to do**:
  - Encode explicit Step 1 compatibility checks for identity/privacy/feed invariants and CI/security gates.
  - Add contract-check tests for no semantic drift in Step 1 endpoints and link/revoke behavior.
  - Wire dependency protocol: if Step 1 contract gap appears, record via `context_for_step_2.md` update format.

  **Must NOT do**:
  - Do not alter Step 1 APIs or semantics.
  - Do not add temporary contract forks.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: compatibility-critical boundary definition.
  - Skills: `[]`
  - Omitted: `['/playwright']`

  **Parallelization**: Can Parallel: NO | Wave 1 | Blocks: 2-14 | Blocked By: none

  **References**:
  - Pattern: `.sisyphus/plans/context_for_step_2.md:35-47` — do-not-touch contract boundaries.
  - Pattern: `.sisyphus/plans/context_for_step_2.md:49-60` — feed mode + identity/linking invariants.
  - Pattern: `.sisyphus/plans/nyx-uzume-monad-nun-foundation.md:64-78` — Step 1 must-have/must-not.

  **Acceptance Criteria**:
  - [ ] Contract tests fail on any Step 1 semantic drift.
  - [ ] Compatibility checks run in CI/local gates.

  **QA Scenarios**:
  ```
  Scenario: Step 1 semantic guard
    Tool: Bash
    Steps: run contract suite verifying privacy defaults, link/revoke semantics, chronological feed-only behavior
    Expected: all invariants pass, no regression
    Evidence: .sisyphus/evidence/task-1-step1-compat.txt

  Scenario: Drift rejection
    Tool: Bash
    Steps: run negative test fixture that simulates semantic change in protected contract
    Expected: suite fails with deterministic mismatch report
    Evidence: .sisyphus/evidence/task-1-drift-reject.txt
  ```

  **Commit**: YES | Message: `test(contracts): lock step1 compatibility for step2` | Files: `contracts/*`, `tests/contracts/*`, `justfile` (if needed)

- [ ] 2. Build Monad storage primitives for stories media (`Akash`)

  **What to do**:
  - Implement reusable stories storage path + object lifecycle helpers in `Monad/Akash`.
  - Add pre-signed upload/download flows for stories and highlights media.
  - Add content-type/size guardrails and secure key naming conventions.

  **Must NOT do**:
  - Do not place Uzume domain rules inside Akash.
  - Do not expose unsafe wildcard object paths.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: shared infra library used by multiple services.
  - Skills: `[]`
  - Omitted: `['/frontend-ui-ux']`

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: 5-12 | Blocked By: 1

  **References**:
  - Pattern: `Monad/README.md:5-17` — crate role and reuse model.
  - Pattern: `apps/Uzume/README.md:120-125` — stories/highlights media needs.

  **Acceptance Criteria**:
  - [ ] Akash exposes typed stories/higlights media APIs.
  - [ ] Pre-signed URL generation validates constraints.

  **QA Scenarios**:
  ```
  Scenario: Valid presign flow
    Tool: Bash
    Steps: run Akash tests for stories upload/download presign with allowed type/size
    Expected: signed URL generated and validated
    Evidence: .sisyphus/evidence/task-2-akash-presign.txt

  Scenario: Invalid media rejection
    Tool: Bash
    Steps: run tests with disallowed MIME and oversize payload metadata
    Expected: deterministic validation error
    Evidence: .sisyphus/evidence/task-2-akash-reject.txt
  ```

  **Commit**: YES | Message: `feat(akash): add stories media storage primitives` | Files: `Monad/Akash/src/*`, `Monad/Akash/tests/*`

- [ ] 3. Build Monad async media pipeline primitives (`Oya` + `events`)

  **What to do**:
  - Implement shared async processing contract: upload accepted → event emitted → media processed → ready event.
  - Add typed event payloads/subjects for stories media lifecycle in `Monad/events`.
  - Implement Oya story image/video processing adapters (resize/transcode/thumbnail metadata contracts).

  **Must NOT do**:
  - Do not process full video synchronously in API request path.
  - Do not hard-wire event provider choice.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: async correctness, retries, and cross-service contracts.
  - Skills: `[]`
  - Omitted: `['/dev-browser']`

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: 5,9,10,11,12 | Blocked By: 1

  **References**:
  - Pattern: `.sisyphus/plans/context_for_step_2.md:88-92` — provider deferred.
  - Pattern: `apps/Uzume/COMMUNICATION.md:17-34` — event-driven communication pattern.

  **Acceptance Criteria**:
  - [ ] Typed story media events compile and serialize stably.
  - [ ] Oya pipeline contracts cover image and video processing states.

  **QA Scenarios**:
  ```
  Scenario: Async lifecycle happy path
    Tool: Bash
    Steps: run integration tests: emit media-upload event -> process -> emit media-ready event
    Expected: deterministic state transition accepted→processing→ready
    Evidence: .sisyphus/evidence/task-3-oya-lifecycle.txt

  Scenario: Retry safety
    Tool: Bash
    Steps: run duplicate event delivery test for same media job id
    Expected: idempotent processing, no duplicate side effects
    Evidence: .sisyphus/evidence/task-3-oya-idempotent.txt
  ```

  **Commit**: YES | Message: `feat(oya-events): add async stories media pipeline contracts` | Files: `Monad/Oya/src/*`, `Monad/events/src/*`, `Monad/Oya/tests/*`, `Monad/events/tests/*`

- [ ] 4. Build Monad cache/db/api helpers for stories composition (`Lethe`, `Mnemosyne`, `api`)

  **What to do**:
  - Add shared cache key namespaces + TTL rules for stories feed/viewers/highlights in Lethe.
  - Add Mnemosyne helpers for stories transaction boundaries and pagination queries.
  - Add reusable api middleware/extractors needed by stories endpoints (auth context, request-id, validated payload wrappers).

  **Must NOT do**:
  - Do not implement Uzume business logic in shared crates.
  - Do not add global cache keys without app scoping.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: shared infra correctness and consistency.
  - Skills: `[]`
  - Omitted: `['/playwright']`

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: 5-12 | Blocked By: 1

  **References**:
  - Pattern: `Monad/Nun/src/types/app.rs:3-10` — app-scoped enum foundation.
  - Pattern: `Monad/Nun/src/pagination.rs` — cursor contract to preserve.

  **Acceptance Criteria**:
  - [ ] Shared helpers compile and are reusable by uzume-stories.
  - [ ] Cache keys are app-scoped and TTL-tested.

  **QA Scenarios**:
  ```
  Scenario: Cache namespace correctness
    Tool: Bash
    Steps: run Lethe tests verifying app-scoped key generation and TTL assignment
    Expected: no key collision across apps
    Evidence: .sisyphus/evidence/task-4-cache-namespace.txt

  Scenario: DB helper transactional safety
    Tool: Bash
    Steps: run Mnemosyne integration tests for commit/rollback around stories writes
    Expected: rollback restores pre-transaction state
    Evidence: .sisyphus/evidence/task-4-tx-safety.txt
  ```

  **Commit**: YES | Message: `feat(monad): add stories cache-db-api shared helpers` | Files: `Monad/Lethe/src/*`, `Monad/Mnemosyne/src/*`, `Monad/api/src/*`, tests

- [ ] 5. Create `Uzume-stories` service scaffold and contract wiring

  **What to do**:
  - Create full service structure under `apps/Uzume/Uzume-stories/` (main/config/routes/handlers/services/models/queries/workers/tests).
  - Wire dependencies only through Monad shared crates + Nun contracts.
  - Register workspace/gateway routing integration points required for stories APIs.

  **Must NOT do**:
  - Do not bypass shared Monad layers for duplicated utilities.
  - Do not add stories logic to unrelated Uzume services.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: foundational service architecture and wiring.
  - Skills: `[]`
  - Omitted: `['/frontend-ui-ux']`

  **Parallelization**: Can Parallel: NO | Wave 2 | Blocks: 6-13 | Blocked By: 1,2,3,4

  **References**:
  - Pattern: `apps/Uzume/README.md:30-47` — expected service module structure.
  - Pattern: `Monad/Heimdall/README.md:58-63` — stories route mapping.

  **Acceptance Criteria**:
  - [ ] Service compiles and boots with health route and config loading.
  - [ ] Route registration exists for `/api/Uzume/stories/*`.

  **QA Scenarios**:
  ```
  Scenario: Service boot
    Tool: Bash
    Steps: run uzume-stories startup test with test config and health probe
    Expected: service starts and returns healthy status
    Evidence: .sisyphus/evidence/task-5-service-boot.txt

  Scenario: Route contract check
    Tool: Bash
    Steps: run route table tests verifying all stories endpoints are registered
    Expected: all expected methods/paths present
    Evidence: .sisyphus/evidence/task-5-routes.txt
  ```

  **Commit**: YES | Message: `feat(uzume-stories): scaffold service and routing` | Files: `apps/Uzume/Uzume-stories/**`, `Cargo.toml`, `Monad/Heimdall/src/*`

- [ ] 6. Implement stories schema + migrations + repository layer

  **What to do**:
  - Add stories domain tables: stories, story_views, story_interactions, highlights, highlight_items.
  - Implement repository/query layer with cursor pagination and strict ownership filters.
  - Enforce expiry, uniqueness, and foreign-key constraints.

  **Must NOT do**:
  - Do not mutate Step 1 baseline tables semantics.
  - Do not use offset pagination.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: data integrity and migration safety.
  - Skills: `[]`
  - Omitted: `['/dev-browser']`

  **Parallelization**: Can Parallel: NO | Wave 2 | Blocks: 7,8,13 | Blocked By: 5

  **References**:
  - Pattern: `apps/Uzume/README.md:120-126` — stories/highlights data intent.
  - Pattern: `Monad/Nun/src/pagination.rs` — cursor pattern.

  **Acceptance Criteria**:
  - [ ] Up/down migrations succeed.
  - [ ] Repository tests verify ownership and expiry filtering.

  **QA Scenarios**:
  ```
  Scenario: Migration lifecycle
    Tool: Bash
    Steps: apply stories migrations, validate schema, rollback, re-apply
    Expected: deterministic up/down success
    Evidence: .sisyphus/evidence/task-6-migrations.txt

  Scenario: Unauthorized row access
    Tool: Bash
    Steps: run repository tests with non-owner access attempts
    Expected: no data leak, deterministic forbidden/not-found handling
    Evidence: .sisyphus/evidence/task-6-repo-authz.txt
  ```

  **Commit**: YES | Message: `feat(stories-db): add stories schema and repositories` | Files: `migrations/Uzume/*stories*`, `apps/Uzume/Uzume-stories/src/{models,queries,services}/*`

- [ ] 7. Implement core stories APIs (create/feed/read/viewers)

  **What to do**:
  - Implement endpoints: create story, stories feed, read story (mark viewed), viewers list.
  - Enforce authz/privacy rules (owner-only viewers, follow/private account checks).
  - Integrate request-id, rate limiting, and structured error mapping.

  **Must NOT do**:
  - Do not reveal hidden global identity.
  - Do not include non-follower private stories in feed.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: API correctness + policy integration.
  - Skills: `[]`
  - Omitted: `['/playwright']`

  **Parallelization**: Can Parallel: NO | Wave 2 | Blocks: 8,13 | Blocked By: 6

  **References**:
  - Pattern: `.sisyphus/plans/context_for_step_2.md:55-60` — identity/linking invariants.
  - Pattern: `Monad/Nun/src/error.rs` — standardized error responses.

  **Acceptance Criteria**:
  - [ ] Endpoint behavior matches stories contract and privacy rules.
  - [ ] Mark-view logic is idempotent.

  **QA Scenarios**:
  ```
  Scenario: Story feed visibility
    Tool: Bash
    Steps: integration test with public, private-followed, and private-not-followed accounts
    Expected: only authorized stories returned
    Evidence: .sisyphus/evidence/task-7-feed-visibility.txt

  Scenario: Viewers privacy enforcement
    Tool: Bash
    Steps: non-owner requests viewers list for story
    Expected: forbidden response with standard error body
    Evidence: .sisyphus/evidence/task-7-viewers-authz.txt
  ```

  **Commit**: YES | Message: `feat(uzume-stories): implement core stories APIs` | Files: `apps/Uzume/Uzume-stories/src/{routes,handlers,services}/*`, tests

- [ ] 8. Implement media upload/ready flow with async event pipeline

  **What to do**:
  - Implement create-story API to accept media metadata and initiate async processing workflow.
  - Emit accepted/processing/ready states through shared event contracts.
  - Finalize story publication only when media-ready state is confirmed.

  **Must NOT do**:
  - Do not block create request on full video transcode.
  - Do not publish story as ready before processing completes.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: asynchronous consistency + state machine correctness.
  - Skills: `[]`
  - Omitted: `['/frontend-ui-ux']`

  **Parallelization**: Can Parallel: NO | Wave 2 | Blocks: 9,10,13 | Blocked By: 7

  **References**:
  - Pattern: `apps/Uzume/COMMUNICATION.md:17-34` — event-driven async pattern.
  - Pattern: `.sisyphus/plans/context_for_step_2.md:62-66` — security-first endpoint requirements.

  **Acceptance Criteria**:
  - [ ] Media lifecycle state transitions are deterministic and auditable.
  - [ ] Duplicate processing events do not duplicate side effects.

  **QA Scenarios**:
  ```
  Scenario: End-to-end media readiness
    Tool: Bash
    Steps: create story with image and video fixtures, process async events, fetch story state
    Expected: transitions accepted→processing→ready with valid media URLs
    Evidence: .sisyphus/evidence/task-8-media-e2e.txt

  Scenario: Duplicate event delivery
    Tool: Bash
    Steps: replay same media-ready event twice for one story
    Expected: idempotent final state, no duplicate updates
    Evidence: .sisyphus/evidence/task-8-media-idempotent.txt
  ```

  **Commit**: YES | Message: `feat(uzume-stories): wire async media lifecycle` | Files: `apps/Uzume/Uzume-stories/src/*`, `Monad/Oya/*`, `Monad/events/*`

- [ ] 9. Implement interactions API (poll/question/slider)

  **What to do**:
  - Implement interaction endpoint with strict schema validation per interaction type.
  - Enforce one-vote/one-response constraints where required.
  - Add aggregation/read models for owner-visible interaction results.

  **Must NOT do**:
  - Do not allow arbitrary JSON payloads without validation.
  - Do not leak private responses to non-owners.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: domain validation and abuse resistance.
  - Skills: `[]`
  - Omitted: `['/playwright']`

  **Parallelization**: Can Parallel: YES | Wave 3 | Blocks: 10,12,13 | Blocked By: 8

  **References**:
  - Pattern: `apps/Uzume/README.md:123-124` — interaction types.
  - Pattern: `Monad/Nun/src/validation.rs` — validation style.

  **Acceptance Criteria**:
  - [ ] All interaction types validate and persist correctly.
  - [ ] Owner-only visibility of results is enforced.

  **QA Scenarios**:
  ```
  Scenario: Valid interaction matrix
    Tool: Bash
    Steps: submit poll vote, question reply, slider value with valid payloads
    Expected: accepted and queryable by owner
    Evidence: .sisyphus/evidence/task-9-interactions-valid.txt

  Scenario: Malformed interaction abuse
    Tool: Bash
    Steps: send invalid type, invalid slider range, oversize payload
    Expected: deterministic validation errors
    Evidence: .sisyphus/evidence/task-9-interactions-invalid.txt
  ```

  **Commit**: YES | Message: `feat(uzume-stories): add interactions endpoints` | Files: `apps/Uzume/Uzume-stories/src/*`, tests

- [ ] 10. Implement highlights API + ownership and ordering guarantees

  **What to do**:
  - Implement create/update highlights and add/remove/reorder highlight items.
  - Enforce owner-only mutation and visibility rules.
  - Ensure referenced stories are valid for highlight inclusion policy.

  **Must NOT do**:
  - Do not allow users to mutate other users’ highlights.
  - Do not allow invalid story references.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: business invariants and authorization.
  - Skills: `[]`
  - Omitted: `['/frontend-ui-ux']`

  **Parallelization**: Can Parallel: YES | Wave 3 | Blocks: 11,12,13 | Blocked By: 8,9

  **References**:
  - Pattern: `apps/Uzume/README.md:124-139` — highlights requirements.
  - Pattern: `Monad/Nun/src/id.rs` — typed ID and ownership-safe patterns.

  **Acceptance Criteria**:
  - [ ] Highlight CRUD and ordering is stable and deterministic.
  - [ ] Ownership checks enforced for all mutating endpoints.

  **QA Scenarios**:
  ```
  Scenario: Highlight lifecycle
    Tool: Bash
    Steps: create highlight, add stories, reorder, fetch public highlights
    Expected: deterministic ordering and expected visibility
    Evidence: .sisyphus/evidence/task-10-highlights-lifecycle.txt

  Scenario: Cross-user mutation attempt
    Tool: Bash
    Steps: authenticated user attempts patch/add on another user's highlight
    Expected: forbidden response and no data mutation
    Evidence: .sisyphus/evidence/task-10-highlights-authz.txt
  ```

  **Commit**: YES | Message: `feat(uzume-stories): implement highlights feature` | Files: `apps/Uzume/Uzume-stories/src/*`, tests

- [ ] 11. Implement expiry/retention/reconciliation workers

  **What to do**:
  - Build 24h expiry worker to transition stories out of active visibility.
  - Build retention cleanup worker for expired media lifecycle.
  - Build reconciliation worker to repair stuck processing states.

  **Must NOT do**:
  - Do not hard-delete active stories.
  - Do not run unbounded cleanup jobs without batching.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: background job safety and consistency.
  - Skills: `[]`
  - Omitted: `['/dev-browser']`

  **Parallelization**: Can Parallel: PARTIAL | Wave 3 | Blocks: 12,13 | Blocked By: 10

  **References**:
  - Pattern: `apps/Uzume/README.md:142` — periodic story expiry worker expectation.
  - Pattern: `.sisyphus/plans/context_for_step_2.md:63-66` — abuse/security requirement.

  **Acceptance Criteria**:
  - [ ] Expired stories removed from active feeds at 24h boundary.
  - [ ] Cleanup and reconciliation jobs are idempotent and bounded.

  **QA Scenarios**:
  ```
  Scenario: Expiry boundary
    Tool: Bash
    Steps: seed stories around expiry threshold and run expiry worker tick
    Expected: only expired stories transitioned out of active set
    Evidence: .sisyphus/evidence/task-11-expiry-boundary.txt

  Scenario: Reconciliation recovery
    Tool: Bash
    Steps: inject stuck processing state and run reconciler
    Expected: state repaired to deterministic terminal state
    Evidence: .sisyphus/evidence/task-11-reconcile.txt
  ```

  **Commit**: YES | Message: `feat(uzume-stories): add expiry retention reconciliation workers` | Files: `apps/Uzume/Uzume-stories/src/workers/*`, tests

- [ ] 12. Add full abuse/security/privacy test matrix

  **What to do**:
  - Add abuse cases: spam attempts, enumeration, payload bombs, replayed events, unauthorized viewers, interaction tampering.
  - Add security checks for authz/data exposure on every stories endpoint.
  - Add privacy tests ensuring no global identity leakage.

  **Must NOT do**:
  - Do not rely only on happy-path integration tests.
  - Do not ship endpoint without abuse coverage.

  **Recommended Agent Profile**:
  - Category: `deep` — Reason: security-critical validation.
  - Skills: `[]`
  - Omitted: `['/playwright']`

  **Parallelization**: Can Parallel: PARTIAL | Wave 3 | Blocks: 13 | Blocked By: 9,10,11

  **References**:
  - Pattern: `.sisyphus/plans/context_for_step_2.md:62-67` — mandatory security requirements.
  - Pattern: `.sisyphus/plans/context_for_step_2.md:55-60` — canonical identity privacy story.

  **Acceptance Criteria**:
  - [ ] Abuse matrix covers all public and authenticated stories routes.
  - [ ] All unauthorized access paths return expected 401/403 with standard body.

  **QA Scenarios**:
  ```
  Scenario: Endpoint abuse sweep
    Tool: Bash
    Steps: execute abuse test suite against all stories endpoints
    Expected: all abuse vectors blocked with deterministic responses
    Evidence: .sisyphus/evidence/task-12-abuse-sweep.txt

  Scenario: Identity leakage check
    Tool: Bash
    Steps: run response snapshot tests ensuring no hidden global identity fields exposed
    Expected: only app-scoped alias/public fields visible
    Evidence: .sisyphus/evidence/task-12-privacy-leak.txt
  ```

  **Commit**: YES | Message: `test(security): add stories abuse and privacy matrix` | Files: `apps/Uzume/Uzume-stories/tests/*`, `security/tests/*`

- [ ] 13. Release hardening: observability, rollback, and runbook

  **What to do**:
  - Implement stories operational metrics/logging/tracing and alert thresholds.
  - Define and automate rollback drill for single release train.
  - Create runbook for incident triage, failed media pipeline recovery, and data repair paths.

  **Must NOT do**:
  - Do not claim production readiness without tested rollback.
  - Do not leave worker failures without observable signals.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` — Reason: production readiness and safe operations.
  - Skills: `[]`
  - Omitted: `['/frontend-ui-ux']`

  **Parallelization**: Can Parallel: NO | Wave 4 | Blocks: 14 | Blocked By: 6,7,8,9,10,11,12

  **References**:
  - Pattern: `.sisyphus/plans/context_for_step_2.md:95-99` — step2 readiness criteria.
  - Pattern: `justfile:42-44` — CI-equivalent local gate command.

  **Acceptance Criteria**:
  - [ ] Observability signals exist for stories APIs and workers.
  - [ ] Rollback drill completes with validated restore outcome.

  **QA Scenarios**:
  ```
  Scenario: Rollback drill
    Tool: Bash
    Steps: deploy candidate build in test environment, execute rollback procedure, rerun smoke + contract tests
    Expected: system returns to pre-release behavior with no contract drift
    Evidence: .sisyphus/evidence/task-13-rollback-drill.txt

  Scenario: Failure observability
    Tool: Bash
    Steps: inject worker failure and verify error metrics/logs/alerts emitted
    Expected: alerts and traces contain actionable failure context
    Evidence: .sisyphus/evidence/task-13-observability.txt
  ```

  **Commit**: YES | Message: `chore(ops): harden stories observability and rollback` | Files: `ops/*`, `apps/Uzume/Uzume-stories/*`, `.github/workflows/ci.yml` (if needed)

- [ ] 14. Single release train validation and cut criteria

  **What to do**:
  - Execute full release gate bundle and publish consolidated evidence package.
  - Validate no Step 1 contract regressions and no unresolved dependency risks.
  - Prepare final release checklist and signoff artifact.

  **Must NOT do**:
  - Do not waive failing gates.
  - Do not ship with unresolved contract or rollback failures.

  **Recommended Agent Profile**:
  - Category: `writing` — Reason: final release package and evidence synthesis.
  - Skills: `[]`
  - Omitted: `['/playwright']`

  **Parallelization**: Can Parallel: NO | Wave 4 | Blocks: Final verification | Blocked By: 13

  **References**:
  - Pattern: `.sisyphus/plans/context_for_step_2.md:95-99` — readiness gates.
  - Pattern: `.github/workflows/ci.yml:46-62` — CI checks to mirror.

  **Acceptance Criteria**:
  - [ ] Full gate suite passes in CI-equivalent run.
  - [ ] Evidence bundle contains tests + abuse + rollback outputs.

  **QA Scenarios**:
  ```
  Scenario: Full gate run
    Tool: Bash
    Steps: run just ci + stories-specific integration + abuse suite
    Expected: all checks pass with zero critical warnings
    Evidence: .sisyphus/evidence/task-14-full-gate.txt

  Scenario: Contract regression check
    Tool: Bash
    Steps: run Step1 compatibility suite against release candidate
    Expected: no protected contract regression
    Evidence: .sisyphus/evidence/task-14-contract-regression.txt
  ```

  **Commit**: YES | Message: `chore(release): finalize stories single-train readiness` | Files: `.sisyphus/evidence/*`, release checklist docs

## Final Verification Wave (MANDATORY — after ALL implementation tasks)
> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
> **Do NOT auto-proceed after verification. Wait for user's explicit approval before marking work complete.**
> **Never mark F1-F4 as checked before getting user's okay.**
- [ ] F1. Plan Compliance Audit — oracle
- [ ] F2. Code Quality Review — unspecified-high
- [ ] F3. Real Manual QA — unspecified-high (+ playwright if UI)
- [ ] F4. Scope Fidelity Check — deep

## Commit Strategy
- Use atomic commits by crate boundary: Monad shared crates first, then Uzume-stories, then ops/release hardening.
- Keep RED-GREEN-REFACTOR progression visible in commit sequence when feasible.
- Commit message format: `type(scope): description`.
- Never merge while any security/abuse/rollback gate is failing.

## Success Criteria
- Stories is production-complete in one release: image+video, interactions, highlights, expiry, viewers.
- Shared logic is implemented in Monad and reused by Uzume-stories (no cross-service duplication).
- Step 1 contracts remain intact and verified.
- Release is backed by passing tests, abuse validation, and proven rollback.


<!-- OMO_INTERNAL_INITIATOR -->