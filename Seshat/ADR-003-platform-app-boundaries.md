# ADR-003: Platform App Boundaries

## Status
Proposed

## Context
A major design challenge for Nyx is where to draw the line between a shared platform service (Monad) and an application-specific implementation (App). This ADR defines the boundaries and how data crosses them.

## Decision: The Boundary Rule
Any logic that is required by TWO or more apps should live in `Monad/` but must be parameterized by `app_id`. Logic that is unique to ONE app (even if complex) lives in `apps/`.

## Deciding Factors for Platform vs App

| Topic | Decision | Logic Location |
| :--- | :--- | :--- |
| **Identity Creation** | Platform | `Monad/Heka` (shared across all apps). |
| **App-Scoped Alias** | Platform | `Monad/Heka` manages the `AliasId` mapping for any app. |
| **Profile Data** | App | `apps/Uzume/Uzume-profiles` (Uzume-specific fields like `bio`, `avatar_url`). |
| **Media Processing** | Platform | `Monad/Oya` (shared library for video/photo transcoding). |
| **Post/Feed Logic** | App | `apps/Uzume/Uzume-feed` (Algorithms and storage are app-specific). |
| **Messaging** | Platform | `Monad/Ogma` (Cross-app messaging protocols). |
| **Notifications** | Platform | `Monad/Ushas` (Shared dispatch mechanism). |

## Decision: Selective Reveal and Revoke Logic
When a user wants to link their identity across App A and App B:
1. **Initiation**: User A (in App A context) sends a "Link Request" targeting User B.
2. **Acceptance**: User B accepts the request.
3. **Record Storage**: The `nyx.app_links` table stores the link: `(actor_alias_id, target_alias_id, target_app_id, status)`.
4. **Visibility**: App A can now query Monad for the alias of User B in App B.
5. **Revoke**: Either user can delete the record in `nyx.app_links`. Once deleted, any cross-app visibility is instantly revoked.

## Invariants
- Apps MUST NOT reach into other app databases.
- The Platform schema `nyx` is the only shared data space for cross-app linking.
- Apps communicate via an event abstraction boundary for decoupling.

## Step-1 Non-Goals
- Global cross-app discovery.
- Cross-app feed aggregation.
- Event backbone provider selection (deferred).

## Step-2 Deferred Items
- Multi-region event replication.
- Shared asset library for users across apps.
- Dynamic app-level feature flags managed by the platform.
