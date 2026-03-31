# Ushas

### Monad/Ushas

> Ushas is the Vedic goddess of dawn — one of the most hymned deities in the Rig Veda, with over twenty dedicated verses. Each morning she drives her chariot ahead of the sun, opening the door of darkness, awakening sleeping humans, and announcing what comes next whether they want it or not. She doesn't carry a specific message — she is the signal itself. The Rig Veda calls her "she who opens the door of darkness."

Notification dispatch. Library + background worker binary. Depends on `Nun`, `nyx-events`, `Lethe`.

Gorush HTTP client for APNs/FCM push. In-app notification storage (PostgreSQL). Grouping: "X and 42 others liked your post". User preference management (per-app, per-event-type mute).
Worker: NATS subscriber → check preferences → persist → push.

```
Ushas/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── bin/
│   │   └── worker.rs      # Binary: NATS subscriber, dispatches notifications
│   ├── gorush.rs          # Gorush HTTP client: send push to APNs/FCM via Gorush API
│   ├── in_app.rs          # In-app notification storage (PostgreSQL) + WebSocket push
│   ├── grouping.rs        # Notification grouping: "X and 42 others liked your post"
│   ├── preferences.rs     # User notification preferences (per-app, per-event-type)
│   └── types.rs           # Notification types, delivery status
└── tests/
```
