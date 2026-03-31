### Monad/events

Provider-agnostic event boundary for Step-1. Depends on `Nun` only.

This crate intentionally does NOT choose a concrete backbone provider yet. Service crates publish `EventEnvelope` values through the `EventPublisher` trait and remain decoupled from NATS or any other transport.

Current adapters:
- `NoopEventPublisher` for default runtime behavior
- `InMemoryEventPublisher` for deterministic tests

```
events/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── adapter.rs       # InMemoryEventPublisher + NoopEventPublisher
│   ├── envelope.rs      # DomainEvent trait + EventEnvelope + EventPublisher port
│   └── subjects.rs      # Subject constants following {app}.{entity}.{action}
└── tests/
    └── event_boundary.rs
```

`EventEnvelope` fields:
- `id`: UUIDv7 event identifier
- `app`: `NyxApp` origin
- `subject`: domain subject string
- `occurred_at`: UTC timestamp
- `payload`: JSON payload without provider-specific metadata

Subject convention remains `{app_or_nyx}.{entity}.{action}`.

Example subjects:
- `nyx.user.created`
- `nyx.user.deleted`
- `Uzume.profile.updated`
- `Uzume.post.created`
- `Uzume.post.deleted`

When a concrete provider is introduced later, it should be implemented as an adapter in this crate without changing service-domain code.
