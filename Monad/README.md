## Monad: The Monad

```
Monad/
├── Nun/               # Foundation: types, errors, config, IDs
├── nyx-api/             # Axum framework: middleware, extractors, server builder
├── Mnemosyne/              # PostgreSQL: pool, migrations, transaction helpers
├── Heka/                # Ory Kratos client + app-scoped alias management
├── nyx-events/          # NATS JetStream: typed pub/sub
├── Lethe/           # DragonflyDB: cache-aside, rate limiting, sessions
├── Akash/         # MinIO/S3: upload, download, presigned URLs
├── Brizo/          # Meilisearch: index management, typed queries
├── Ogma/                # Matrix/Continuwuity: app-scoped rooms, privacy isolation
├── Oya/           # Media processing pipeline (library + worker binary)
├── Ushas/               # Notification dispatch (library + worker binary)
├── Heimdall/            # API gateway binary: routing, auth, rate limiting
└── nyx-xtask/           # Developer CLI: migrate, seed, openapi, new-app scaffold
```

- These are **library crates** (except `Heimdall` and `nyx-xtask` which are binaries).
- App services depend on these crates via Cargo workspace dependencies. They are not separate running services, they are compiled into the app binaries.
- The only Monad *processes* are:
    - `Heimdall` (the API gateway),
    - `Oya` (which also builds a worker binary for background media processing), and
    - `Ushas` (which also builds a worker binary for notification dispatch).
