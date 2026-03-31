# ADR-001: Identity Visibility and Linking

## Status
Proposed

## Context
Nyx is a multi-app platform where users (Identities) maintain different personas (Aliases) across different applications (e.g., Uzume for social, Anteros for dating). A core value proposition is "privacy by default" combined with "controlled reveal."

We need a formal model for how these identities interact and when a user's global identity or cross-app presence becomes visible to others.

## Decision: Hidden Global Identity by Default

1. **Global Identity (`IdentityId`)** is never exposed in any public API. It is the "true name" used only for internal linking and authentication (Monad/Heka).
2. **App-Scoped Alias (`AliasId`)** is the primary identifier for all app-level interactions.
3. **App-Isolation**: By default, an actor in App A has no way to know that an actor in App B is the same human.

## Decision Table: Visibility and Linking

| Mode | Scopes Visible | Identifier Shown | Narrative Example |
| :--- | :--- | :--- | :--- |
| **Default Private** | Single App | App-specific Alias | Emily uses Uzume. Bob sees `@emily_u`. Bob has no idea Emily is also on Anteros. |
| **One-Way Reveal** | App A -> App B | Alias B (to A) | Emily trusts Bob on Uzume and clicks "Reveal Anteros." Bob can now see Emily's Anteros profile link from her Uzume profile. Emily cannot see Bob's Anteros profile unless he reveals it too. |
| **Two-Way Reveal** | App A <-> App B | Both Aliases | Emily and Bob both reveal their Anteros profiles to each other on Uzume. They now see the "Mutual Match" connection across apps. |
| **App-Selective** | Specific Apps | Selected Aliases | Emily reveals her "Work" app profile to Bob, but keeps her "Dating" app profile hidden. |
| **Revoke** | None (Reset) | App-specific Alias | Emily revokes Bob's access. Bob can no longer see the link between Emily's Uzume and Anteros personas. Existing data in App B remains, but the *link* is severed. |

## Invariants
- `IdentityId` MUST NOT appear in JSON responses.
- `AliasId` is unique within an `app_id` but may be shared across identities if a user changes handles (though the underlying ID remains constant for that persona).
- Linking records live in `Monad/Heka` (or a dedicated `nyx.app_links` table) and require explicit opt-in from the resource owner.

## Step-1 Non-Goals
- Automated cross-app friend suggestions.
- Global search for a user across all apps by their real name.
- Ranking/Personalization based on cross-app activity (deferred to Step-2).

## Step-2 Deferred Items
- Verifiable Credentials (VCs) for identity proofing.
- Decentralized Identity (DID) integration.
- Group-based visibility policies.
