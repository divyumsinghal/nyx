# Xtask

Developer tooling binary. Uses the Rust `xtask` pattern (a binary in the workspace that serves as a project-specific CLI).

Rust `xtask` binary. Commands: `migrate`, `db-reset`, `seed`, `openapi`, `new-app {name}`.

```
nyx-xtask/
├── Cargo.toml
├── src/
│   └── main.rs            # CLI: subcommands for common dev tasks
```

Commands:
- `cargo xtask migrate` — run all pending migrations (platform + all apps)
- `cargo xtask db-reset` — drop and recreate all schemas, re-run migrations
- `cargo xtask seed` — seed development data (test users, sample posts, etc.)
- `cargo xtask openapi` — generate merged OpenAPI spec from all services
- `cargo xtask new-app {name}` — scaffold a new app directory (for future apps)

---

## migrations/

```
migrations/
├── platform/          # nyx schema: aliases, links, push tokens
└── Uzume/              # Uzume schema: profiles, posts, interactions, follows,
                       #   stories, reels, notifications, timeline
```

One PostgreSQL, separate schemas. `cargo xtask migrate` runs all.

---