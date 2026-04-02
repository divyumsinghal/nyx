# Phase 4 Final Web Product - nyx.com + uzume.nyx.com

## TL;DR
> **Summary**: Deliver final web product (not MVP) in one plan: `nyx.com` account-only and `uzume.nyx.com` full social product + admin/moderation, using Expo web stack and completing backend gaps in the same phase.
> **Deliverables**:
> - pnpm + Turborepo + Expo monorepo with shared packages
> - nyx account web app
> - uzume full web app (feed, stories, reels, profiles, discover/search, messaging, notifications, admin/mod)
> - backend parity closure where endpoints are partial/missing
> **Effort**: XL
> **Parallel**: YES - 5 waves
> **Critical Path**: foundation/contracts -> backend parity -> core surfaces -> advanced/admin -> hardening

## Context
### Original Request
Build the final web UI for Uzume and Nyx in Phase 4, runnable locally end-to-end, with mobile later.

### Interview Summary
- SvelteKit placeholder direction is discarded.
- Fixed stack: Expo Router + NativeWind + Reanimated + FlashList + expo-image, pnpm + turbo.
- `nyx.com` is account management only.
- `uzume.nyx.com` contains all product features, including admin/moderation.
- Backend changes are explicitly allowed and expected.
- TDD is mandatory.

### Metis Review (gaps addressed)
- Boundary contract explicitly locked by domain.
- Endpoint-complete acceptance criteria added.
- Backend hardening included before parity claims.
- Zero-human QA evidence required for completion.

## Work Objectives
### Core Objective
Ship a complete locally runnable web product where users can do full Uzume social workflows and admins can perform moderation, while Nyx remains account-only.

### Deliverables
- `Maya/nyx-web`
- `Maya/uzume-web`
- `packages/ui`, `packages/api`, `packages/config`
- backend contract completion for required flows
- CI + e2e verification and runbook proof

### Definition of Done (verifiable)
- `pnpm -w install && pnpm -w turbo run build` passes
- `pnpm -w turbo run test` passes
- `pnpm -w turbo run e2e` passes
- `just ci` passes
- evidence confirms nyx account lifecycle and uzume full user/admin journeys

### Must Have
- NativeWind utility classes only
- FlashList only for lists, expo-image for images
- icons only from `packages/ui/icons`
- no mocked final behavior
- backend parity closure included in this phase

### Must NOT Have
- no MVP scope reduction
- no split into multiple plans
- no domain ownership drift
- no manual-only acceptance criteria

## Verification Strategy
> ZERO HUMAN INTERVENTION - all verification is agent-executed.
- Test decision: **TDD** with Vitest + Playwright + existing Rust tests
- QA policy: each task has happy/failure scenarios and evidence files
- Evidence path: `.sisyphus/evidence/task-{N}-{slug}.{ext}`

## Execution Strategy
### Parallel Execution Waves
- Wave 1: foundation + shared contracts
- Wave 2: backend parity closure
- Wave 3: core product surfaces
- Wave 4: messaging/notifications/admin surfaces
- Wave 5: hardening and release gates

### Dependency Matrix
- T1 blocks all
- T2/T3/T4/T5 block UI tasks
- T6/T7/T8/T9 block corresponding UI parity tasks
- T10/T11/T12/T13/T14 block T15
- T15 blocks Final Verification Wave

### Agent Dispatch Summary
- Wave 1: 5 tasks
- Wave 2: 4 tasks
- Wave 3: 5 tasks
- Wave 4: 3 tasks
- Wave 5: 1 task

## TODOs
- [ ] 1. Bootstrap pnpm/turbo/expo monorepo

  **What to do**: Create workspace/apps/packages structure and runnable scripts.
  **Must NOT do**: Leave placeholder manifests/scripts.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: repo-wide setup
  - Skills: [`verification-before-completion`] - ensure command contracts
  - Omitted: [`frontend-design`] - not visual

  **Parallelization**: Can Parallel: NO | Wave 1 | Blocks: 2-15 | Blocked By: none

  **References**:
  - Pattern: `Seshat/ARCHITECTURE.md`

  **Acceptance Criteria**:
  - [ ] `pnpm -w install` succeeds
  - [ ] `pnpm -w turbo run build` succeeds

  **QA Scenarios**:
  ```text
  Scenario: Happy bootstrap
    Tool: Bash
    Steps: run install and build commands
    Expected: exit 0
    Evidence: .sisyphus/evidence/task-1-bootstrap.txt

  Scenario: Broken workspace
    Tool: Bash
    Steps: run build with intentionally bad workspace ref
    Expected: deterministic dependency error
    Evidence: .sisyphus/evidence/task-1-bootstrap-error.txt
  ```

  **Commit**: YES | Message: `chore(web): bootstrap expo monorepo` | Files: root/workspace/apps/packages configs

- [ ] 2. Build shared config contracts

  **What to do**: Centralize ts/eslint/tailwind/nativewind/reanimated config in `packages/config`.
  **Must NOT do**: Allow inline styles, FlatList, or react-native Image in app code.

  **Recommended Agent Profile**:
  - Category: `quick` - Reason: config/rules setup
  - Skills: [`verification-before-completion`] - enforce policy checks
  - Omitted: [`frontend-design`] - non-visual

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: 10-15 | Blocked By: 1

  **References**:
  - Pattern: `AGENTS.md`

  **Acceptance Criteria**:
  - [ ] lint fails on inline style fixture
  - [ ] lint fails on forbidden list/image usage

  **QA Scenarios**:
  ```text
  Scenario: Policy violation
    Tool: Bash
    Steps: add temporary violating fixture, run lint
    Expected: lint failure with specific rule names
    Evidence: .sisyphus/evidence/task-2-config.txt

  Scenario: Clean policy pass
    Tool: Bash
    Steps: remove fixture, rerun lint
    Expected: lint exits 0
    Evidence: .sisyphus/evidence/task-2-config-pass.txt
  ```

  **Commit**: YES | Message: `chore(config): add shared lint/style contracts` | Files: `packages/config/**`

- [ ] 3. Build shared UI primitives and tokens

  **What to do**: Implement tokenized components and icons in `packages/ui`.
  **Must NOT do**: Use external icon libraries directly in apps.

  **Recommended Agent Profile**:
  - Category: `visual-engineering` - Reason: design system quality
  - Skills: [`frontend-design`] - anti-slop quality
  - Omitted: [`systematic-debugging`] - no bug context

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: 10-15 | Blocked By: 1,2

  **References**:
  - Pattern: `.sisyphus/drafts/phase4-web-ui-uzume-nyx.md`

  **Acceptance Criteria**:
  - [ ] tokenized primitives render in both apps
  - [ ] all icon imports originate from `packages/ui/icons`

  **QA Scenarios**:
  ```text
  Scenario: Token parity
    Tool: Playwright
    Steps: open style preview routes in both apps
    Expected: same tokenized styles and component shapes
    Evidence: .sisyphus/evidence/task-3-ui.png

  Scenario: Icon import guard
    Tool: Bash
    Steps: scan app imports for forbidden icon libs
    Expected: zero matches
    Evidence: .sisyphus/evidence/task-3-ui-icons.txt
  ```

  **Commit**: YES | Message: `feat(ui): add shared tokenized primitives` | Files: `packages/ui/**`

- [ ] 4. Build typed API contract layer via Heimdall routes

  **What to do**: Implement typed clients and error normalization in `packages/api` for `/api/nyx/*` and `/api/Uzume/*`.
  **Must NOT do**: bypass Heimdall with direct service URLs.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: integration contract layer
  - Skills: [`verification-before-completion`] - contract strictness
  - Omitted: [`frontend-design`] - non-visual

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: 10-15 | Blocked By: 1

  **References**:
  - Pattern: `Monad/Heimdall/README.md`
  - Pattern: `apps/Uzume/*/src/routes/mod.rs`

  **Acceptance Criteria**:
  - [ ] typed clients cover required nyx and uzume routes
  - [ ] protected route without token returns normalized auth error path

  **QA Scenarios**:
  ```text
  Scenario: Authenticated contract call
    Tool: Bash
    Steps: run contract test suite with valid token
    Expected: typed success parsing for key
endpoints
    Evidence: .sisyphus/evidence/task-4-api.txt

  Scenario: Missing token contract call
    Tool: Bash
    Steps: call protected endpoint without token
    Expected: normalized 401 path
    Evidence: .sisyphus/evidence/task-4-api-error.txt
  ```

  **Commit**: YES | Message: `feat(api): add typed Heimdall clients` | Files: `packages/api/**`

- [ ] 5. Set up TDD web test stack and CI jobs

  **What to do**: Add Vitest/Playwright projects, coverage thresholds, and CI jobs.
  **Must NOT do**: Keep tests local-only.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: cross-repo testing pipeline
  - Skills: [`webapp-testing`] - e2e reliability
  - Omitted: [`frontend-design`] - non-visual

  **Parallelization**: Can Parallel: YES | Wave 1 | Blocks: 15 | Blocked By: 1

  **References**:
  - Pattern: `.github/workflows/ci.yml`

  **Acceptance Criteria**:
  - [ ] `pnpm -w turbo run test` passes
  - [ ] `pnpm -w turbo run e2e` passes baseline suite

  **QA Scenarios**:
  ```text
  Scenario: CI parity pass
    Tool: Bash
    Steps: run local test + e2e commands with CI env
    Expected: deterministic pass
    Evidence: .sisyphus/evidence/task-5-test-infra.txt

  Scenario: Forced failure
    Tool: Bash
    Steps: add failing test and rerun
    Expected: pipeline fails with clear output
    Evidence: .sisyphus/evidence/task-5-test-infra-error.txt
  ```

  **Commit**: YES | Message: `test(web): add vitest and playwright CI` | Files: test configs and workflows

- [ ] 6. Complete stories interaction backend gaps

  **What to do**: implement missing/partial interaction handlers/services/queries/tests.
  **Must NOT do**: expose interaction UI without real persisted behavior.

  **Recommended Agent Profile**:
  - Category: `deep` - Reason: domain correctness and data integrity
  - Skills: [`test-driven-development`] - backend TDD discipline
  - Omitted: [`frontend-design`] - backend task

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: 12 | Blocked By: 4,5

  **References**:
  - Pattern: `apps/Uzume/Uzume-stories/src/routes/mod.rs`
  - Pattern: `apps/Uzume/Uzume-stories/src/handlers/stories.rs`

  **Acceptance Criteria**:
  - [ ] stories interaction API passes crate tests
  - [ ] invalid payload returns structured error

  **QA Scenarios**:
  ```text
  Scenario: Valid interaction submit
    Tool: Bash
    Steps: POST valid interaction payload
    Expected: 2xx and persisted result
    Evidence: .sisyphus/evidence/task-6-stories.txt

  Scenario: Invalid interaction submit
    Tool: Bash
    Steps: POST malformed payload
    Expected: 4xx validation response
    Evidence: .sisyphus/evidence/task-6-stories-error.txt
  ```

  **Commit**: YES | Message: `feat(stories): close interaction parity gaps` | Files: stories routes/handlers/services/tests

- [ ] 7. Complete reels comments and saves backend endpoints

  **What to do**: add/finalize reels comments/saves routes, handlers, services, queries, tests.
  **Must NOT do**: ship reels UI interaction buttons without endpoint parity.

  **Recommended Agent Profile**:
  - Category: `deep` - Reason: endpoint parity work
  - Skills: [`test-driven-development`] - test-first API behavior
  - Omitted: [`frontend-design`] - backend task

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: 12 | Blocked By: 4,5

  **References**:
  - Pattern: `apps/Uzume/Uzume-reels/src/routes/mod.rs`
  - Pattern: `apps/Uzume/Uzume-feed/src/handlers/interactions.rs`

  **Acceptance Criteria**:
  - [ ] reels comments/saves routes implemented and tested
  - [ ] auth/pagination behavior consistent with feed conventions

  **QA Scenarios**:
  ```text
  Scenario: Reel comment/save flow
    Tool: Bash
    Steps: create and read back comments/saves via API tests
    Expected: data persists and returns correctly
    Evidence: .sisyphus/evidence/task-7-reels.txt

  Scenario: Unauthorized reel action
    Tool: Bash
    Steps: call protected reels action without auth
    Expected: 401/403
    Evidence: .sisyphus/evidence/task-7-reels-error.txt
  ```

  **Commit**: YES | Message: `feat(reels): add comments and saves parity` | Files: reels backend files/tests

- [ ] 8. Wire realtime notifications and websocket relay

  **What to do**: complete event relay/subscription path and websocket reliability for web notifications.
  **Must NOT do**: label realtime complete while only polling fallback exists.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: cross-service integration
  - Skills: [`systematic-debugging`] - reconnect/race handling
  - Omitted: [`frontend-design`] - integration focus

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: 13 | Blocked By: 4,5

  **References**:
  - Pattern: `Monad/Heimdall/README.md`
  - Pattern: `Monad/Ushas/**`

  **Acceptance Criteria**:
  - [ ] realtime events reach subscribed clients
  - [ ] reconnect path restores stream without duplicates/crash

  **QA Scenarios**:
  ```text
  Scenario: Realtime delivery
    Tool: Playwright
    Steps: trigger event from user A and observe user B notifications
    Expected: notification appears without refresh
    Evidence: .sisyphus/evidence/task-8-realtime.png

  Scenario: Drop and reconnect
    Tool: Playwright
    Steps: force disconnect then recover
    Expected: stream resumes cleanly
    Evidence: .sisyphus/evidence/task-8-realtime-error.png
  ```

  **Commit**: YES | Message: `feat(realtime): complete ws and notification relay` | Files: Heimdall/Ushas/related tests

- [ ] 9. Implement admin/moderation backend contracts

  **What to do**: add report/action endpoints and role checks required for admin web workflows.
  **Must NOT do**: allow destructive action without authz and audit metadata.

  **Recommended Agent Profile**:
  - Category: `deep` - Reason: security-sensitive contract work
  - Skills: [`systematic-debugging`] - policy-path rigor
  - Omitted: [`frontend-design`] - backend task

  **Parallelization**: Can Parallel: YES | Wave 2 | Blocks: 14 | Blocked By: 4,5

  **References**:
  - Pattern: `Monad/Heimdall/src/routes.rs`
  - Pattern: `apps/Uzume/*/src/routes/mod.rs`

  **Acceptance Criteria**:
  - [ ] admin-only routes reject non-admin principals
  - [ ] moderation actions persist with trace context

  **QA Scenarios**:
  ```text
  Scenario: Admin moderation action
    Tool: Bash
    Steps: invoke admin endpoint with admin token
    Expected: 2xx and persisted action
    Evidence: .sisyphus/evidence/task-9-admin.txt

  Scenario: Privilege violation
    Tool: Bash
    Steps: invoke same endpoint with non-admin token
    Expected: 403
    Evidence: .sisyphus/evidence/task-9-admin-error.txt
  ```

  **Commit**: YES | Message: `feat(admin): add moderation contracts and authz` | Files: backend routes/handlers/tests

- [ ] 10. Build nyx.com account web app

  **What to do**: implement register/login/session/account settings/logout, and strict route ownership.
  **Must NOT do**: include social/product routes in nyx app.

  **Recommended Agent Profile**:
  - Category: `visual-engineering` - Reason: user account UX
  - Skills: [`frontend-design`, `webapp-testing`] - UX quality + deterministic e2e
  - Omitted: [`systematic-debugging`] - unless blocker

  **Parallelization**: Can Parallel: YES | Wave 3 | Blocks: 15 | Blocked By: 1,2,3,4,5

  **References**:
  - Pattern: `Monad/Heimdall/README.md`

  **Acceptance Criteria**:
  - [ ] full account lifecycle succeeds on nyx domain
  - [ ] social routes are blocked/redirected away from nyx app

  **QA Scenarios**:
  ```text
  Scenario: Account lifecycle
    Tool: Playwright
    Steps: sign up -> sign in -> update settings -> sign out
    Expected: success and persistence
    Evidence: .sisyphus/evidence/task-10-nyx.png

  Scenario: Forbidden route ownership
    Tool: Playwright
    Steps: navigate nyx app to uzume social route
    Expected: deterministic redirect
    Evidence: .sisyphus/evidence/task-10-nyx-error.png
  ```

  **Commit**: YES | Message: `feat(nyx-web): account-only web experience` | Files: `Maya/nyx-web/**`

- [ ] 11. Build uzume feed and post interactions

  **What to do**: implement feed/timeline/post detail + like/comment/save/share.
  **Must NOT do**: leave optimistic state without rollback on failure.

  **Recommended Agent Profile**:
  - Category: `visual-engineering` - Reason: core product surface
  - Skills: [`frontend-design`, `webapp-testing`] - UX + e2e
  - Omitted: [`skill-creator`] - irrelevant

  **Parallelization**: Can Parallel: YES | Wave 3 | Blocks: 15 | Blocked By: 1,2,3,4,5,7

  **References**:
  - Pattern: `apps/Uzume/Uzume-feed/src/routes/mod.rs`

  **Acceptance Criteria**:
  - [ ] feed and interactions fully functional against live APIs
  - [ ] rollback and failure states are deterministic

  **QA Scenarios**:
  ```text
  Scenario: Feed interaction happy path
    Tool: Playwright
    Steps: like/comment/save on feed post
    Expected: consistent UI and persisted API state
    Evidence: .sisyphus/evidence/task-11-feed.png

  Scenario: Interaction API failure
    Tool: Playwright
    Steps: trigger failed mutation
    Expected: optimistic UI rollback + error message
    Evidence: .sisyphus/evidence/task-11-feed-error.png
  ```

  **Commit**: YES | Message: `feat(uzume-web): feed and post interactions` | Files: `Maya/uzume-web/**`

- [ ] 12. Build uzume stories and reels web experiences

  **What to do**: implement stories tray/viewer/progression/interactions and reels playback/audio/interactions.
  **Must NOT do**: rely on native-only behavior without web fallback.

  **Recommended Agent Profile**:
  - Category: `visual-engineering` - Reason: animation-heavy surfaces
  - Skills: [`frontend-design`, `webapp-testing`] - polish + validation
  - Omitted: [`requesting-code-review`] - not needed in task body

  **Parallelization**: Can Parallel: YES | Wave 3 | Blocks: 15 | Blocked By: 1,2,3,4,5,6,7

  **References**:
  - Pattern: `apps/Uzume/Uzume-stories/src/routes/mod.rs`
  - Pattern: `apps/Uzume/Uzume-reels/src/routes/mod.rs`

  **Acceptance Criteria**:
  - [ ] story progression and interactions persist via API
  - [ ] reels playback and interactions work on web

  **QA Scenarios**:
  ```text
  Scenario: Stories and reels flow
    Tool: Playwright
    Steps: consume stories then reels and perform interactions
    Expected: persisted outcomes and stable transitions
    Evidence: .sisyphus/evidence/task-12-stories-reels.png

  Scenario: Media fallback path
    Tool: Playwright
    Steps: load unsupported/problematic media
    Expected: graceful fallback state
    Evidence: .sisyphus/evidence/task-12-stories-reels-error.png
  ```

  **Commit**: YES | Message: `feat(uzume-web): stories and reels parity surfaces` | Files: `Maya/uzume-web/**`

- [ ] 13. Build profiles/discover/search/messaging/notifications

  **What to do**: implement profile/follow/privacy, discover/search/trending, messaging UI, notifications center with realtime.
  **Must NOT do**: move messaging ownership to nyx app.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: broad integration wave
  - Skills: [`webapp-testing`] - cross-flow validation
  - Omitted: [`using-git-worktrees`] - optional workflow

  **Parallelization**: Can Parallel: YES | Wave 4 | Blocks: 15 | Blocked By: 1,2,3,4,5,8

  **References**:
  - Pattern: `apps/Uzume/Uzume-profiles/src/routes/mod.rs`
  - Pattern: `apps/Uzume/Uzume-discover/src/routes/mod.rs`
  - Pattern: `Monad/Heimdall/README.md`

  **Acceptance Criteria**:
  - [ ] all listed surfaces functional with live endpoint data
  - [ ] realtime notifications remain stable across navigation

  **QA Scenarios**:
  ```text
  Scenario: Cross-feature flow
    Tool: Playwright
    Steps: profile update -> search -> message -> notification receipt
    Expected: persisted state and stable UI
    Evidence: .sisyphus/evidence/task-13-surfaces.png

  Scenario: Auth guard failure path
    Tool: Playwright
    Steps: access protected surface without session
    Expected: redirect to auth route
    Evidence: .sisyphus/evidence/task-13-surfaces-error.png
  ```

  **Commit**: YES | Message: `feat(uzume-web): profiles discover messaging notifications` | Files: `Maya/uzume-web/**`

- [ ] 14. Build uzume admin/moderation web interfaces

  **What to do**: implement admin report review/action flows consuming T9 backend contracts.
  **Must NOT do**: expose admin routes to non-admin users.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: policy-sensitive UI
  - Skills: [`webapp-testing`, `systematic-debugging`] - authz and failure rigor
  - Omitted: [`frontend-design`] - correctness priority

  **Parallelization**: Can Parallel: YES | Wave 4 | Blocks: 15 | Blocked By: 1,2,3,4,5,9

  **References**:
  - Pattern: T9 endpoint contracts

  **Acceptance Criteria**:
  - [ ] admin lifecycle (review/action/resolve) works end-to-end
  - [ ] non-admin route access is blocked

  **QA Scenarios**:
  ```text
  Scenario: Admin moderation lifecycle
    Tool: Playwright
    Steps: login as admin -> process report -> verify resolution state
    Expected: action succeeds and state persists
    Evidence: .sisyphus/evidence/task-14-admin-ui.png

  Scenario: Non-admin route access
    Tool: Playwright
    Steps: non-admin opens admin URL
    Expected: block/redirect and no privileged action
    Evidence: .sisyphus/evidence/task-14-admin-ui-error.png
  ```

  **Commit**: YES | Message: `feat(uzume-web): admin and moderation interfaces` | Files: `Maya/uzume-web/**`

- [ ] 15. Hardening, regression, and release gate completion

  **What to do**: run full regression, fix edge defects, enforce perf/a11y gate, validate local runbook and CI.
  **Must NOT do**: mark phase complete with failing checks or missing evidence.

  **Recommended Agent Profile**:
  - Category: `unspecified-high` - Reason: end-state stabilization
  - Skills: [`verification-before-completion`, `webapp-testing`] - evidence-first completion
  - Omitted: [`frontend-design`] - verification wave

  **Parallelization**: Can Parallel: NO | Wave 5 | Blocks: F1-F4 | Blocked By: 5,10,11,12,13,14

  **References**:
  - Pattern: CI workflows + all prior evidence

  **Acceptance Criteria**:
  - [ ] `pnpm -w turbo run test && pnpm -w turbo run e2e && pnpm -w turbo run build` pass
  - [ ] `just ci` passes
  - [ ] evidence index covers all mandatory flows

  **QA Scenarios**:
  ```text
  Scenario: Full gate pass
    Tool: Bash
    Steps: execute full command gate set
    Expected: all pass, no manual intervention needed
    Evidence: .sisyphus/evidence/task-15-regression.txt

  Scenario: Gate failure block
    Tool: Bash
    Steps: force one failing check and rerun gate
    Expected: completion blocked
    Evidence: .sisyphus/evidence/task-15-regression-error.txt
  ```

  **Commit**: YES | Message: `chore(release): finalize hardening and full verification` | Files: fixes/tests/workflows/runbook

## Final Verification Wave (MANDATORY - after ALL implementation tasks)
- [ ] F1. Plan Compliance Audit - oracle
- [ ] F2. Code Quality Review - unspecified-high
- [ ] F3. Real Manual QA - unspecified-high (+ playwright if UI)
- [ ] F4. Scope Fidelity Check - deep

## Commit Strategy
- Atomic commits per task with passing tests
- No completion claim before F1-F4 approval and user explicit okay

## Success Criteria
- nyx.com account-only contract enforced
- uzume.nyx.com full user + admin/moderation experience complete
- backend parity blockers closed
- all automated command gates pass
- local run path demonstrates complete usable product
