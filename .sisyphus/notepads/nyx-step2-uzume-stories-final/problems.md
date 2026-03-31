

## Step 2 Task 3 Blockers (2026-03-31)

### Critical Blockers

1. **No implementation in Monad/events crate**
   - No `src/` directory exists
   - Cannot implement event patterns without envelope/subject/publisher/subscriber
   - Blocking: All downstream async workflows depend on event infrastructure

2. **No implementation in Monad/Oya crate**
   - No `src/` directory exists
   - Cannot implement media processing without worker/pipeline/image/video modules
   - Blocking: Media upload → ready flow requires Oya processing

3. **Missing media status types in Nun**
   - No `MediaStatus` enum (for Accepted → Processing → Ready)
   - No media-specific ID markers
   - Blocking: Need typed state machine before Task 3 can be deterministic

### Minor Blockers

4. **Idempotency pattern not yet codified**
   - Only documented in learnings (duplicate delivery handling)
   - Needs concrete implementation: job_id tracking in DB or Redis
   - Non-blocking for exploration, blocking for implementation

5. **No NATS infrastructure configured**
   - Events crate depends on NATS JetStream
   - No docker-compose or configuration for local dev
   - Non-blocking for exploration, blocking for integration testing

### Assumptions Required for Task 3

- Provider finalization deferred → events must be provider-agnostic
- Contract guardrails from Task 1 must remain unchanged → no media status locks needed yet
- Async flow must be deterministic → state transitions must be atomic

### Unresolved Design Risks (Task 3 State Machine & Outbox)

6. **Outbox Relay Latency & Polling Overhead**
   - Using a Transactional Outbox pattern requires a relay worker to poll or tail the database (e.g., via Postgres logical replication/WAL or simple polling).
   - *Risk:* If polling is used, it introduces latency to the `Accepted -> Processing` notification. If logical replication is used, it adds deployment complexity to the infrastructure.

7. **Poison Message & DLQ Deadlocks in Inbox Pattern**
   - If a message crashes the worker *after* inserting the `idempotency_key` but *before* committing the domain state, the next retry might see the idempotency key and assume success, dropping the message.
   - *Risk:* The inbox transaction MUST wrap the domain state change in the exact same DB transaction. If the media processing is asynchronous or spans multiple systems, this requires a distributed saga or two-phase commit, making deterministic state machines harder to strictly enforce.

8. **Concurrent "Accepted" Race Conditions**
   - If two different worker nodes pick up the same `Accepted` event simultaneously (due to broker redelivery), both might try to transition it to `Processing`.
   - *Risk:* Need explicit `SELECT ... FOR UPDATE SKIP LOCKED` or optimistic concurrency control (version ID) to prevent race conditions during the `Accepted -> Processing` state transition.
