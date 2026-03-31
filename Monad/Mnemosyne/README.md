# Mnemosyne

> Titaness of memory: the river in the underworld whose waters gave permanent recall to those who drank them. She is the direct opposite of Lethe (forgetfulness/cache). Those who drank from Mnemosyne's pool retained all memories of all past lives and were released from the cycle of reincarnation. A database is Mnemosyne: it remembers everything, permanently, so services don't have to hold state themselves.


### Monad/Mnemosyne

Database connection management and shared query utilities. Depends on `Nun`.

One PostgreSQL instance, multiple schemas (`nyx`, `Uzume`, `Anteros`, `Themis`). Provides `PgPool` builder, per-schema migration runner, transaction helpers with auto-rollback, bulk insert helper.

```
Mnemosyne/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── pool.rs            # Connection pool builder (PgPool), config from Nun
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


