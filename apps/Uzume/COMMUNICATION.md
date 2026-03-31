## Inter-service communication

**REST (HTTP/JSON) everywhere.** One protocol. Shared Rust types via library crates for compile-time safety without protobuf.

### Synchronous

Service-to-service HTTP calls. Discovered via config (env / TOML). Cached in DragonflyDB.

```toml
# Uzume-feed config
[upstream]
profiles = "http://Uzume-profiles:3001"
```

### Asynchronous

NATS JetStream. At-least-once, consumer groups, replay on restart.

```
nyx.user.created              → Uzume-profiles (create stub)
nyx.user.deleted              → all Uzume services (cascade)

Uzume.post.created            → Oya, Brizo, Ushas, Uzume-feed (fanout)
Uzume.post.liked              → Ushas, Uzume-feed (score)
Uzume.comment.created         → Ushas, Uzume-feed (score)
Uzume.user.followed           → Ushas, Uzume-feed (timeline)
Uzume.user.blocked            → Uzume-feed (filter), Uzume-discover (filter)
Uzume.story.created           → Oya, Ushas
Uzume.story.viewed            → Ushas
Uzume.reel.created            → Oya (transcode), Brizo
Uzume.reel.viewed             → Uzume-reels (scoring)
Uzume.profile.updated         → Brizo
Uzume.media.processed         → Uzume-feed / Uzume-stories / Uzume-reels (update URLs)
```

---