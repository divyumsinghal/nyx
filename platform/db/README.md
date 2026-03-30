### platform/nyx-db

Database connection management and shared query utilities. Depends on `Monad`.

One PostgreSQL instance, multiple schemas (`nyx`, `Uzume`, `Anteros`, `Themis`). Provides `PgPool` builder, per-schema migration runner, transaction helpers with auto-rollback, bulk insert helper.

```
nyx-db/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── pool.rs            # Connection pool builder (PgPool), config from Monad
│   ├── migrate.rs         # Migration runner — discovers and runs migrations per-schema
│   ├── transaction.rs     # Transaction helper with automatic rollback on error
│   └── ext.rs             # sqlx extensions: bulk insert helper, RETURNING helpers
└── tests/
```

Each app service gets its own PostgreSQL **schema** (not database). This means one PgPool, multiple schemas:

```sql
-- platform schema (shared tables)
CREATE SCHEMA IF NOT EXISTS nyx;

-- per-app schemas
CREATE SCHEMA IF NOT EXISTS Uzume;
CREATE SCHEMA IF NOT EXISTS Anteros;
CREATE SCHEMA IF NOT EXISTS Themis;
```

The `nyx` schema holds cross-app data: app-link consents, shared counters, feature flags. Each app schema holds only that app's domain tables.


