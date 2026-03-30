### platform/nyx-search

Meilisearch client wrapper. Depends on `Monad`.

Meilisearch client. Index convention: `{app}_{entity}`. Provides: index management (create, update settings), typed search queries, event-driven sync from NATS.

```
nyx-search/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── client.rs          # SearchClient (wraps meilisearch-sdk), connection config
│   ├── index.rs           # Index management: create, update settings, sync from DB
│   ├── query.rs           # Typed search request/response wrappers
│   └── sync.rs            # Event-driven index sync: listens to NATS, updates Meilisearch
└── tests/
```

Index naming convention: `{app}_{entity}` — `Uzume_users`, `Uzume_posts`, `Anteros_profiles`, `Themis_listings`.

