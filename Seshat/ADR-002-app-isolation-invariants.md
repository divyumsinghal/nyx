# ADR-002: App Isolation Invariants

## Status
Proposed

## Context
The Nyx platform is a modular monolith (deployment-wise) with app-specific logic partitioned by `app_id`. To ensure that App A cannot leak data into App B, we need strict invariants enforced at the database and service layers.

## Decisions

### 1. Mandatory `app_id` Requirement
Every request, every database record, and every event MUST be tagged with an `app_id`.
- **Database**: All tables in the `nyx` and app-specific schemas (e.g., `Uzume`) must include an `app_id` or `alias_id` that resolves back to an `app_id`.
- **API**: Every API endpoint must have a resolved `NyxApp` context.

### 2. App-Isolation Invariants
- **Data Partitioning**: Data from App A is physically or logically separated from App B. Query filters MUST always include `WHERE app_id = ?`.
- **Alias Scope**: An `AliasId` is only valid within its originating `app_id`.
- **Credential Isolation**: Sessions are app-scoped. Logging into Uzume does not automatically grant a session to Anteros (unless using a global SSO portal, and even then, app-specific tokens are issued).

### 3. Modular Monolith Boundary Rules
- **Platform Crates**: Crates in `Monad/` are the "Foundational Substrate." They are stateless and app-agnostic.
- **App Crates**: Crates in `apps/` are "Domain Services." They handle app-specific business logic.
- **Cross-App Communication**: Apps NEVER communicate directly with each other. They interact via `Monad/` platform services (NATS events or shared Platform DB schemas like `nyx.app_links`).

## Decision Table: Isolation Enforcement

| Enforcement Layer | Mechanism | Rule |
| :--- | :--- | :--- |
| **API Gateway** | `Monad/Heimdall` | Rejects any request that cannot resolve to a valid `app_id` via the hostname or `X-Nyx-App` header. |
| **Service Layer** | `Monad/nyx-api` | Extractors automatically inject `NyxApp` into handlers. |
| **Database** | RLS / Query Builders | All `sqlx` queries must be parameterized by the current `app_id` context. |
| **Event Bus** | `Monad/nyx-events` | NATS subjects are prefixed by `app_id` (e.g., `Uzume.post.created`). |

## Step-1 Non-Goals
- Multi-tenancy for unrelated organizations (Nyx is multi-app, single-platform).
- Cross-app transactionality (Consistency is eventual via NATS).
- Multi-region deployment isolation.

## Step-2 Deferred Items
- Dynamic `app_id` registration (currently a static enum in `Monad/Nun`).
- App-specific encryption keys.
- Granular per-app rate limiting configurations.
