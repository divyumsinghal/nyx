# Brizo

> Brizo was an obscure prophetic goddess of the island of Delos, worshipped almost exclusively there. Sailors and fishermen left small model boats as offerings to her and received in return guidance for navigating unknown waters. She gave cryptic but accurate information about what lay ahead. She was specifically the goddess of navigation-by-knowledge: not of the sea itself, but of knowing where to go in the sea.


### Monad/Brizo

Meilisearch client wrapper. Depends on `Nun`.

Meilisearch client. Index convention: `{app}_{entity}`. Provides: index management (create, update settings), typed search queries, event-driven sync from NATS.

```
Brizo/
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

