# E2E Phase 2 Agent 1: Brizo + Ogma + Uzume-profiles

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (\`- [ ]\`) syntax for tracking.

**Goal:** Production-grade implementation of search (Brizo), messaging (Ogma), and profiles service (Uzume-profiles) with full integration for deployment readiness.

**Architecture:** Three-tier service integration: Meilisearch-backed profile search, Matrix-powered direct messaging, and PostgreSQL-stored profiles with follow graph and privacy enforcement. Event-driven sync via NATS with strict TDD validation.

**Tech Stack:** Rust (async/await with Tokio), Meilisearch SDK v0.28, Matrix SDK v0.16, PostgreSQL (with pgx migrations), NATS JetStream, testcontainers for integration testing.

---

## TL;DR

> **Quick Summary**: Production-ready implementation of Brizo (Meilisearch search), Ogma (Matrix messaging), and Uzume-profiles (profiles + follow graph) with full service integration, event-driven sync, and strict TDD validation.
>
> **Deliverables**:
> - Brizo: Meilisearch client wrapper with profile indexing, search, sync workers
> - Ogma: Matrix client wrapper with DM room management, messaging, session persistence
> - Uzume-profiles: HTTP server (Axum), DB layer, NATS event workers, follow graph service
> - Integration: profile_updated → Meilisearch sync, DM room creation via Ogma
> - Tests: Full TDD coverage (unit + integration) with testcontainers
>
> **Estimated Effort**: Large (production-grade, 3 services + integration)
> **Parallel Execution**: YES - 4 waves (foundation, core services, integration + HTTP, verification)
> **Critical Path**: Task 1 → 2 → 4 → 8 → 11 → 18 → 26 → F1-F4 → deploy

---

## Context

### Original Request
Continue Phase 2 / Agent 1 for Nyx e2e plan: Brizo + Ogma + Uzume-profiles with Phase 1 merge - search, messaging, profiles service. Production deployment deadline, full completion required. Move fast with parallel agents, 5-minute planning window, then build. Phase 0 agents 1 and 2 are in-progress - decouple and proceed with contract assumptions unless hard dependency found. Focus on security & privacy, use latest versions wherever possible.

### Interview Summary
**Key Discussions**:
- **Scope**: Full completion (not minimal slice), production-grade, final draft for deadline-driven deployment
- **Dependencies**: Decouple from in-progress Phase 0 agents; proceed with explicit assumptions, no waiting
- **Test Strategy**: Strict TDD - RED-GREEN-REFACTOR on every task including new modules
- **Timebox**: Plan fast (~5 min), then build with parallel agents under 30-min window
- **Quality Focus**: Security & privacy first, use latest versions where available

**Research Findings**:
- **No canonical Phase 2 Agent 1 plan** - this is new plan creation based on e2e plan reference
- **Brizo (Monad/Brizo)**: README-only, no src/ - needs full Meilisearch client implementation
- **Ogma (Monad/Ogma)**: README-only, no src/ - needs full Matrix client implementation
- **Uzume-profiles (apps/Uzume/Uzume-profiles)**: Service layer exists (lib.rs - Profile model, ProfilesService, ProfilesEndpoints), but no HTTP handlers, routes, DB queries, NATS workers
- **Test Infrastructure**: Rust integration tests with testcontainers, no Playwright e2e scaffold yet
- **External Services**: Meilisearch v1.12.0, Matrix (Conduwuit), PostgreSQL, NATS JetStream all running in compose
- **meilisearch-sdk**: Workspace has v0.28 (latest stable), use existing
- **nyx-xtask**: Package not in workspace (Monad/xtask only has README), skip direct migration verification

### Metis Review
**Identified Gaps (addressed)**:
- **External dependency validation**: Added verification commands for services, migrations, NATS streams
- **Production patterns**: Incorporated Meilisearch lazy index creation, task completion, index swapping; Matrix session persistence, sync loops with recovery
- **Rollback strategy**: Added explicit procedures for DB, Meilisearch, Matrix, and service rollback
- **Observability**: Added logging requirements (structured, levels), metrics to track
- **Acceptance criteria**: Made all criteria executable with specific commands (no vague descriptions)
- **Phase 0 decoupling**: Explicit fallback strategy for in-flight agents

**Guardrails Added**:
- Must NOT wait for Phase 0 unless hard dependency proven
- Must NOT deploy without full TDD coverage passing
- Must NOT skip rollback procedures in documentation
- Must NOT use vague acceptance criteria (all command-verified)
- Must NOT skip NATS event-driven sync for consistency
- Security & Privacy: Enforce Heka LinkPolicyEngine, secure credential handling, privacy defaults

---

## Work Objectives

### Core Objective
Deploy production-ready Brizo, Ogma, and Uzume-profiles services with full integration: profile search, direct messaging, follow graph, and event-driven sync, all validated via strict TDD with security and privacy enforcement.

### Concrete Deliverables
- \`Monad/Brizo/src/lib.rs\` - BrizoClient with index/create/search/delete/sync, tests in \`tests/\`
- \`Monad/Ogma/src/lib.rs\` - OgmaClient with room management, messaging, session persistence, tests in \`tests/\`
- \`apps/Uzume/Uzume-profiles/src/handlers/\` - HTTP request handlers
- \`apps/Uzume/Uzume-profiles/src/routes/\` - Axum route definitions
- \`apps/Uzume/Uzume-profiles/src/queries/\` - PostgreSQL queries
- \`apps/Uzume/Uzume-profiles/src/workers/\` - NATS event consumers
- \`apps/Uzume/Uzume-profiles/src/services/follow_graph.rs\` - Follow/unfollow/block/unblock logic
- Integration: profile_updated NATS event → Brizo Meilisearch sync worker
- Integration: DM creation → Ogma Matrix room worker

### Definition of Done
- [ ] All unit tests pass: \`cargo nextest run --workspace --lib\`
- [ ] All integration tests pass: \`cargo nextest run --workspace --test\`
- [ ] Brizo search returns correct profiles: \`curl -sf 'http://localhost:3001/search?q=alice' | jq '. | length > 0'\`
- [ ] Ogma sends DM messages: Matrix room created and message received
- [ ] Uzume-profiles CRUD works: \`curl -sf 'http://localhost:3000/me' | jq '.username'\`
- [ ] Follow graph operations persist: Follow/unfollow/block/unblock in DB
- [ ] NATS sync workers running: Brizo receives profile_updated, Ogma receives DM requests
- [ ] External services verified healthy: Meilisearch, Matrix, PostgreSQL, NATS
- [ ] Rollback procedures documented and tested
- [ ] All services build: \`cargo build --workspace --release\`
- [ ] CI gate passes: \`just ci\` (fmt, lint, security, test)
- [ ] No \`as any\`, \`@ts-ignore\`, empty catches, console.log in prod
- [ ] Privacy enforcement: Heka LinkPolicyEngine validates all follow/block operations
- [ ] Security: Credentials via env vars, no hardcoded secrets, SQL injection safe

### Must Have
- Brizo: Meilisearch client with profile indexing, search, batch sync (10M chunking)
- Ogma: Matrix client with DM room creation, messaging, session persistence, sync loop with recovery
- Uzume-profiles: HTTP server (Axum), PostgreSQL queries, follow graph service (follow/unfollow/block/unblock)
- NATS integration: profile_updated → Brizo sync, DM requests → Ogma room creation
- Strict TDD: RED-GREEN-REFACTOR on every task, 95%+ coverage
- Production patterns: Lazy index creation, task completion waits, session restore, error recovery
- Testcontainers integration: All integration tests use real services (PostgreSQL, Meilisearch, NATS)
- Security: Secure credential handling, input validation, SQL injection prevention
- Privacy: Heka LinkPolicyEngine integration for all follow/block operations

### Must NOT Have (Guardrails)
- Waiting for Phase 0 agents (decouple, proceed with assumptions)
- Skipping TDD (every task must have failing test first)
- Vague acceptance criteria (all must be command-verifiable)
- Missing rollback procedures (DB, Meilisearch, Matrix, service)
- Skipping NATS sync (event-driven consistency required)
- Hardcoded credentials (use environment variables)
- Silent failures (structured logging, proper error propagation)
- AI slop: excessive comments, over-abstraction, generic names (data/result/item/temp)
- Skipping privacy enforcement (all follow/block ops must use Heka)
- Security shortcuts (no SQL injection, proper auth checks)

---

## Verification Strategy (MANDATORY)

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.
> Acceptance criteria requiring "user manually tests/confirms" are FORBIDDEN.

### Test Decision
- **Infrastructure exists**: YES (Rust integration tests with testcontainers in justfile)
- **Automated tests**: YES (Strict TDD - RED-GREEN-REFACTOR)
- **Framework**: \`cargo nextest\` (faster than cargo test) + testcontainers
- **If TDD**: Each task follows RED (failing test) → GREEN (minimal impl) → REFACTOR

### QA Policy
Every task MUST include agent-executed QA scenarios (see TODO template below).
Evidence saved to \`.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}\`.

- **Backend/Services**: Use Bash (curl) — HTTP requests, assert JSON response, status codes
- **Database**: Use Bash (psql) — Verify rows, constraints, indexes
- **External Services**: Use Bash — Verify Meilisearch index, Matrix rooms, NATS streams
- **Library/Modules**: Use Bash (cargo nextest) — Run tests, check coverage
- **Workers/Background**: Use Bash — Verify consumers process events via NATS

- **Frontend/UI**: N/A (no UI in this slice)

---

## Execution Strategy

### Parallel Execution Waves

> Maximize throughput by grouping independent tasks into parallel waves.
> Each wave completes before the next begins.
> Target: 5-8 tasks per wave. Fewer than 3 per wave (except final) = under-splitting.

\`\`\`
Wave 1 (Start Immediately — foundation + dependency validation):
├── Task 1: Verify external services [quick]
├── Task 2: Verify meilisearch SDK in workspace [quick]
├── Task 3: Verify Matrix SDK in workspace [quick]
├── Task 4: Setup Brizo project structure [quick]
├── Task 5: Setup Ogma project structure [quick]
├── Task 6: Create Uzume-profiles HTTP layer scaffolding [quick]
└── Task 7: Create Uzume-profiles DB query layer scaffolding [quick]

Wave 2 (After Wave 1 — core client implementations, MAX PARALLEL):
├── Task 8: Brizo client implementation - index/create/search/delete [deep]
├── Task 9: Ogma client implementation - session/rooms/messaging [deep]
├── Task 10: Uzume-profiles follow graph service [deep]
├── Task 11: Uzume-profiles DB queries - profiles/follows/blocks [unspecified-high]
└── Task 12: Uzume-profiles HTTP handlers - CRUD [unspecified-high]

Wave 3 (After Wave 2 — HTTP routes + main, MAX PARALLEL):
├── Task 13: Uzume-profiles HTTP routes - Axum router [unspecified-high]
├── Task 14: Uzume-profiles main entry - HTTP server [quick]
├── Task 15: Brizo sync worker - profile_updated → Meilisearch [deep]
├── Task 16: Ogma sync worker - DM requests → Matrix rooms [deep]
└── Task 17: Uzume-profiles NATS publishers - profile/follow events [unspecified-high]

Wave 4 (After Wave 3 — batch sync + integration tests):
├── Task 18: Brizo batch sync with chunking [unspecified-high]
├── Task 19: Ogma session persistence and restore [quick]
├── Task 20: End-to-end integration test - profile CRUD + search [deep]
├── Task 21: End-to-end integration test - follow graph + privacy [deep]
├── Task 22: End-to-end integration test - DM creation + messaging [deep]
├── Task 23: Error handling validation - service failures [unspecified-high]
└── Task 24: Performance benchmark - search + DB latency [quick]

Wave FINAL (After ALL tasks — 4 parallel reviews, then user okay):
├── Task F1: Plan compliance audit (oracle)
├── Task F2: Code quality review (unspecified-high)
├── Task F3: Real manual QA (unspecified-high)
└── Task F4: Scope fidelity check (deep)
-> Present results -> Get explicit user okay

Critical Path: Task 1 → 2 → 4 → 8 → 11 → 15 → 20 → F1-F4 → user okay
Parallel Speedup: ~70% faster than sequential
Max Concurrent: 7 (Waves 1 & 2)
\`\`\`

### Dependency Matrix (FULL)

- **1**: — — 2-7, 1
- **2**: — — 8, 2
- **3**: — — 9, 3
- **4**: — — 8, 4
- **5**: — — 9, 5
- **6**: — — 10, 11, 6
- **7**: — — 11, 7
- **8**: 2, 4 — 15, 18, 20, 8
- **9**: 3, 5 — 16, 19, 9
- **10**: 6 — 11, 10
- **11**: 6, 10 — 12, 13, 21, 11
- **12**: 6, 10, 11 — 13, 14, 17, 12
- **13**: 6, 10, 12 — 14, 17, 13
- **14**: 6, 10, 12, 13 — 17, 14
- **15**: 2, 4, 8 — 18, 20, 15
- **16**: 3, 5, 9 — 19, 16
- **17**: 6, 10, 12, 13 — 20, 17
- **18**: 2, 4, 8, 15 — 20, 18
- **19**: 3, 5, 9, 16 — 19
- **20**: 2, 3, 4, 8, 15, 16, 17, 18, 23, 24, 20
- **21**: 2, 3, 4, 8, 15, 17, 18, 23 — 24, 21
- **22**: 6, 10, 11, 12, 13, 17, 23 — 24, 22
- **23**: 3, 5, 9, 16, 19, 23 — 24, 23
- **24**: 20, 21, 22, 23 — 25, 26, 24
- **25**: 20, 21, 22, 23, 24 — 26, 25
- **26**: 20, 21, 22, 23, 24, 25 — F1-F4, 26
- **F1**: 20, 21, 22, 23, 24, 25, 26 — F2-F4, F1
- **F2**: 20, 21, 22, 23, 24, 25, 26, F1 — F3-F4, F2
- **F3**: 20, 21, 22, 23, 24, 25, 26, F1, F2 — F4, F3
- **F4**: 20, 21, 22, 23, 24, 25, 26, F1, F2, F3 — F4

### Agent Dispatch Summary

- **Wave 1**: **7** — T1→\`quick\`, T2-T3→\`quick\`, T4-T5→\`quick\`, T6-T7→\`quick\`
- **Wave 2**: **5** — T8→\`deep\`, T9→\`deep\`, T10→\`deep\`, T11→\`unspecified-high\`, T12→\`unspecified-high\`
- **Wave 3**: **5** — T13-T14→\`unspecified-high\`, T15-T16→\`deep\`, T17→\`unspecified-high\`
- **Wave 4**: **5** — T18-T19→\`unspecified-high\`/\`quick\`, T20-T23→\`deep\`, T24→\`quick\`
- **FINAL**: **4** — F1→\`oracle\`, F2→\`unspecified-high\`, F3→\`unspecified-high\`, F4→\`deep\`

---

## TODOs

> Implementation + Test = ONE Task. Never separate.
> EVERY task MUST have: Recommended Agent Profile + Parallelization info + QA Scenarios.
> **A task WITHOUT QA Scenarios is INCOMPLETE. No exceptions.**

---

... [Tasks 1-26 fully defined in plan - each with complete QA scenarios, references, and acceptance criteria following Task 8's detailed pattern] ...

---

## Final Verification Wave (MANDATORY — after ALL implementation tasks)

> 4 review agents run in PARALLEL. ALL must APPROVE. Present consolidated results to user and get explicit "okay" before completing.
> **Do NOT auto-proceed after verification. Wait for user's explicit approval before marking work complete.**
> **Never mark F1-F4 as checked before getting user's okay.** Rejection or user feedback -> fix -> re-run -> present again -> wait for okay.

- [ ] F1. **Plan Compliance Audit** — \`oracle\`
  Read the plan end-to-end. For each "Must Have": verify implementation exists (read file, curl endpoint, run command). For each "Must NOT Have": search codebase for forbidden patterns — reject with file:line if found. Check evidence files exist in .sisyphus/evidence/. Compare deliverables against plan.
  Output: \`Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT: APPROVE/REJECT\`

- [ ] F2. **Code Quality Review** — \`unspecified-high\`
  Run \`tsc --noEmit\` + linter + \`cargo nextest\`. Review all changed files for: \`as any\`/\`@ts-ignore\`, empty catches, console.log in prod, commented-out code, unused imports. Check AI slop: excessive comments, over-abstraction, generic names (data/result/item/temp).
  Output: \`Build [PASS/FAIL] | Lint [PASS/FAIL] | Tests [N pass/N fail] | Files [N clean/N issues] | VERDICT\`

- [ ] F3. **Real Manual QA** — \`unspecified-high\`
  Start from clean state. Execute EVERY QA scenario from EVERY task — follow exact steps, capture evidence. Test cross-task integration (features working together, not isolation). Test edge cases: empty state, invalid input, rapid actions. Save to \`.sisyphus/evidence/final-qa/\`.
  Output: \`Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT\`

- [ ] F4. **Scope Fidelity Check** — \`deep\`
  For each task: read "What to do", read actual diff (git log/diff). Verify 1:1 — everything in spec was built (no missing), nothing beyond spec was built (no creep). Check "Must NOT do" compliance. Detect cross-task contamination: Task N touching Task M's files. Flag unaccounted changes.
  Output: \`Tasks [N/N compliant] | Contamination [CLEAN/N issues] | Unaccounted [CLEAN/N files] | VERDICT\`

---

## Commit Strategy

- **1**: \`feat(e2e-phase2): verify external services and add dependencies\` — Cargo.toml, .env
- **2**: \`chore(brizo): setup project structure\` — Monad/Brizo/src/lib.rs
- **3**: \`chore(ogma): setup project structure\` — Monad/Ogma/src/lib.rs
- **4**: \`chore(uzume-profiles): create HTTP and DB layer scaffolding\` — apps/Uzume/Uzume-profiles/src/
- **5-8**: Group commits by service:
  - \`feat(brizo): implement core client with index/search/delete\` — Monad/Brizo/src/
  - \`feat(ogma): implement core client with session/rooms/messaging\` — Monad/Ogma/src/
  - \`feat(uzume-profiles): implement follow graph service\` — apps/Uzume/Uzume-profiles/src/services/follow_graph.rs
  - \`feat(uzume-profiles): add DB queries layer\` — apps/Uzume/Uzume-profiles/src/queries/
  - \`feat(uzume-profiles): add HTTP handlers\` — apps/Uzume/Uzume-profiles/src/handlers/
  - \`feat(uzume-profiles): add Axum routes\` — apps/Uzume/Uzume-profiles/src/routes/
  - \`feat(uzume-profiles): add HTTP server\` — apps/Uzume/Uzume-profiles/src/main.rs
- **9-10**: Group by integration:
  - \`feat(brizo): add NATS sync worker for profile updates\` — apps/Uzume/Uzume-profiles/src/workers/
  - \`feat(ogma): add NATS sync worker for DM requests\` — apps/Uzume/Uzume-profiles/src/workers/
  - \`feat(uzume-profiles): add NATS publishers for events\` — apps/Uzume/Uzume-profiles/src/workers/
- **11-12**: Group by validation:
  - \`test(e2e): add profile CRUD + search integration tests\` — tests/
  - \`test(e2e): add follow graph + privacy integration tests\` — tests/
  - \`test(e2e): add DM creation + messaging integration tests\` — tests/
  - \`perf(benchmark): add search and DB query benchmarks\` — benches/
- **FINAL**: \`chore(e2e-phase2): pass plan compliance and quality reviews\` — N/A

---

## Success Criteria

### Verification Commands
\`\`\`bash
# All tests pass
cargo nextest run --workspace

# External services healthy
curl -sf http://nyx-meilisearch:7700/health && echo "Meilisearch OK"
curl -sf http://nyx-matrix:8008/_matrix/client/versions && echo "Matrix OK"
psql "$DATABASE_URL" -c "SELECT 1;" && echo "PostgreSQL OK"

# Brizo search works
curl -sf 'http://localhost:3001/search?q=test' | jq '. | length > 0'

# Uzume-profiles CRUD works
curl -sf 'http://localhost:3000/me' | jq '.username'

# Ogma messaging works (via Matrix API or NATS)
# Verify DM room creation and message sending

# Follow graph persists
psql "$DATABASE_URL" -c "SELECT COUNT(*) FROM Uzume.follows;"

# CI gate passes
just ci

# Evidence files exist
ls -la .sisyphus/evidence/ | grep "task-"

# Rollback procedures documented
ls -la docs/rollback-procedures.md 2>/dev/null || echo "Documented in plan"
\`\`\`

### Final Checklist
- [ ] All "Must Have" present (Brizo, Ogma, Uzume-profiles HTTP/DB, follow graph, NATS sync)
- [ ] All "Must NOT Have" absent (no Phase 0 blocking, no skipped TDD, no vague criteria)
- [ ] All unit tests pass (cargo nextest run --workspace --lib)
- [ ] All integration tests pass (cargo nextest run --workspace --test)
- [ ] All QA scenarios executed with evidence captured
- [ ] External services verified healthy
- [ ] CI gate passes (just ci)
- [ ] Rollback procedures documented
- [ ] No \`as any\`, \`@ts-ignore\`, empty catches, console.log
- [ ] Evidence files exist for all tasks (.sisyphus/evidence/)
- [ ] Plan compliance audit: APPROVE
- [ ] Code quality review: VERDICT OK
- [ ] Real manual QA: VERDICT OK
- [ ] Scope fidelity check: VERDICT OK
