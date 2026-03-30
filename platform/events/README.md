
### platform/nyx-events

NATS JetStream client wrapper. Depends on `Monad`.

NATS JetStream typed pub/sub. `NyxEvent<T>` envelope: `{ id, subject, app, timestamp, payload: T }`. Subject convention: `{app}.{entity}.{action}`. Typed publisher + subscriber. See the inter-service communication section for the full event map.

```
nyx-events/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── client.rs          # NatsClient: connect, publish, subscribe, create streams
│   ├── subjects.rs        # All event subject constants (nyx.user.created, Uzume.post.liked, etc.)
│   ├── envelope.rs        # NyxEvent<T> envelope: { id, subject, app, timestamp, payload: T }
│   ├── publisher.rs       # Typed publisher: publish(subject, payload) → serializes to JSON
│   └── subscriber.rs      # Typed subscriber: subscribe<T>(subject) → Stream<NyxEvent<T>>
└── tests/
```

Event subjects follow a strict convention: `{app_or_nyx}.{entity}.{action}`

```rust
pub mod subjects {
    // Platform-level
    pub const USER_CREATED: &str = "nyx.user.created";
    pub const USER_DELETED: &str = "nyx.user.deleted";
    pub const APPS_LINKED: &str  = "nyx.apps.linked";

    // Uzume
    pub const Uzume_POST_CREATED: &str    = "Uzume.post.created";
    pub const Uzume_POST_LIKED: &str      = "Uzume.post.liked";
    pub const Uzume_COMMENT_CREATED: &str = "Uzume.comment.created";
    pub const Uzume_STORY_CREATED: &str   = "Uzume.story.created";
    pub const Uzume_USER_FOLLOWED: &str   = "Uzume.user.followed";

    // Anteros
    pub const Anteros_SWIPE: &str         = "Anteros.swipe";
    pub const Anteros_MATCH_CREATED: &str = "Anteros.match.created";

    // Themis
    pub const Themis_LISTING_CREATED: &str = "Themis.listing.created";
    pub const Themis_REVIEW_CREATED: &str  = "Themis.review.created";
}
```

New apps add their own subjects to this file. The convention guarantees no collisions.
