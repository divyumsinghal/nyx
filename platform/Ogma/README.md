# Ogma: The Message

> Inventor of the Ogham script: the encoded communication protocol of the Celtic druids. Ogham wasn't just writing; it was a secret protocol used to send messages outsiders couldn't read. He didn't carry messages, he designed the encoding layer messages travel through.

Matrix homeserver integration layer. This is the privacy-isolated messaging system. Depends on `Monad`, `nyx-events`, `nyx-db`, `Heka`.

Matrix/Continuwuity integration. Creates **app-scoped rooms** tagged with `nyx.app` state event. Privacy enforcement: `list_rooms(user, app)` only returns rooms matching that app. Cross-app visibility requires explicit consent via `Heka` linking.

```rust
pub async fn create_app_room(&self, app: NyxApp, user_a: NyxId, user_b: NyxId) -> Result<MatrixRoomId>;
pub async fn list_rooms(&self, user: NyxId, app: NyxApp) -> Result<Vec<Room>>;
```

```
Ogma/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── client.rs          # MatrixClient: wraps HTTP calls to Continuwuity's client-server API
│   ├── rooms.rs           # Room lifecycle: create app-scoped room, invite, tag with app metadata
│   ├── messages.rs        # Send/receive messages, media attachments
│   ├── aliases.rs         # Matrix user alias management (maps NyxId + NyxApp → Matrix user)
│   ├── privacy.rs         # Privacy enforcement: filter rooms by app tag, cross-app link checks
│   └── types.rs           # Matrix event types, room metadata, message envelope
└── tests/
```

The privacy model implementation:

```rust
/// Create a new DM room scoped to a specific app.
/// Both participants join using their app-scoped Matrix aliases.
/// Room is tagged with `nyx.app = {app}` state event.
pub async fn create_app_room(
    &self,
    app: NyxApp,
    participant_a: NyxId,
    participant_b: NyxId,
) -> Result<MatrixRoomId>;

/// List all rooms visible to a user within a specific app context.
/// Only returns rooms tagged with the matching app.
/// Cross-app rooms are excluded unless both users have an active link.
pub async fn list_rooms(
    &self,
    user_id: NyxId,
    app: NyxApp,
) -> Result<Vec<Room>>;
```
