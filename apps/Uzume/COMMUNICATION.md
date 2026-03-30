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

Uzume.post.created            → nyx-media, nyx-search, Ushas, Uzume-feed (fanout)
Uzume.post.liked              → Ushas, Uzume-feed (score)
Uzume.comment.created         → Ushas, Uzume-feed (score)
Uzume.user.followed           → Ushas, Uzume-feed (timeline)
Uzume.user.blocked            → Uzume-feed (filter), Uzume-discover (filter)
Uzume.story.created           → nyx-media, Ushas
Uzume.story.viewed            → Ushas
Uzume.reel.created            → nyx-media (transcode), nyx-search
Uzume.reel.viewed             → Uzume-reels (scoring)
Uzume.profile.updated         → nyx-search
Uzume.media.processed         → Uzume-feed / Uzume-stories / Uzume-reels (update URLs)
```

---