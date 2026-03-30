
# Uzume

Uzume — the social media platform.

```
Uzume/
├── Cargo.toml
├── src/
│   ├── main.rs                    # Entry point: load config, build NyxServer, serve
│   ├── config.rs                  # Uzume-specific config (feed settings, story TTL, etc.)
│   ├── routes/
│   │   ├── mod.rs                 # Merges all route groups into one Axum Router
│   │   ├── posts.rs               # POST/GET/DELETE /posts, /posts/{id}
│   │   ├── feed.rs                # GET /feed (home timeline)
│   │   ├── comments.rs            # POST/GET /posts/{id}/comments
│   │   ├── likes.rs               # POST/DELETE /posts/{id}/like
│   │   ├── stories.rs             # POST/GET /stories, /stories/feed
│   │   ├── reels.rs               # POST/GET /reels, /reels/feed
│   │   ├── discover.rs            # GET /discover (explore page)
│   │   ├── profiles.rs            # GET/PATCH /profiles/{alias}, /profiles/me
│   │   ├── follow.rs              # POST/DELETE /profiles/{alias}/follow
│   │   ├── search.rs              # GET /search?q=...&type=users|posts|tags
│   │   ├── notifications.rs       # GET /notifications, WebSocket
│   │   └── health.rs              # GET /healthz
│   ├── handlers/                  # Request handler functions (one per route)
│   │   ├── mod.rs
│   │   ├── posts.rs
│   │   ├── feed.rs
│   │   ├── comments.rs
│   │   ├── likes.rs
│   │   ├── stories.rs
│   │   ├── reels.rs
│   │   ├── discover.rs
│   │   ├── profiles.rs
│   │   ├── follow.rs
│   │   ├── search.rs
│   │   └── notifications.rs
│   ├── services/                  # Business logic (pure functions, testable independently)
│   │   ├── mod.rs
│   │   ├── feed_builder.rs        # Hybrid push/pull feed generation
│   │   ├── story_lifecycle.rs     # 24h TTL management, highlights
│   │   ├── reel_ranker.rs         # Engagement-weighted reel scoring
│   │   ├── discover_ranker.rs     # Explore page content selection
│   │   └── follow_graph.rs        # Follower/following management, fan-out decisions
│   ├── models/                    # Domain types + sqlx query structs
│   │   ├── mod.rs
│   │   ├── post.rs                # Post, PostMedia, PostCreate, PostResponse
│   │   ├── comment.rs
│   │   ├── story.rs
│   │   ├── reel.rs
│   │   ├── profile.rs
│   │   ├── follow.rs
│   │   ├── like.rs
│   │   └── notification.rs
│   ├── queries/                   # Raw SQL queries (sqlx::query_as! macros)
│   │   ├── mod.rs
│   │   ├── posts.rs
│   │   ├── feed.rs
│   │   ├── comments.rs
│   │   ├── profiles.rs
│   │   └── follow.rs
│   └── workers/                   # Background tasks (run in separate tokio tasks within the same process)
│       ├── mod.rs
│       ├── feed_fanout.rs         # Listens to Uzume.post.created → writes to followers' timelines
│       ├── story_expiry.rs        # Periodic: expire stories past 24h TTL
│       └── search_sync.rs         # Listens to Uzume.* events → updates Meilisearch indexes
└── tests/
    ├── api/                       # Integration tests: spin up the server, hit real endpoints
    │   ├── posts_test.rs
    │   ├── feed_test.rs
    │   ├── stories_test.rs
    │   └── follow_test.rs
    └── services/                  # Unit tests for business logic (no HTTP, no DB)
        ├── feed_builder_test.rs
        └── reel_ranker_test.rs
```

**Uzume API surface** (all prefixed with `/api/Uzume` by the gateway):

```
POST   /posts                           # Create post (multipart: media[] + caption + tags)
GET    /posts/{id}                      # Get single post
DELETE /posts/{id}                      # Delete own post
POST   /posts/{id}/like                 # Like a post
DELETE /posts/{id}/like                 # Unlike a post
GET    /posts/{id}/comments             # List comments (cursor-paginated)
POST   /posts/{id}/comments             # Create comment
DELETE /posts/{id}/comments/{cid}       # Delete own comment

GET    /feed                            # Home timeline (cursor-paginated)

POST   /stories                         # Create story (multipart: media + stickers JSON)
GET    /stories/feed                    # Stories from followed users
GET    /stories/{id}/viewers            # View list for own story

POST   /reels                           # Create reel (multipart: video)
GET    /reels/feed                      # Algorithmic reel feed (cursor-paginated)

GET    /discover                        # Explore page (cursor-paginated)

GET    /profiles/{alias}                # Public profile
PATCH  /profiles/me                     # Update own profile
GET    /profiles/{alias}/followers      # Follower list (cursor-paginated)
GET    /profiles/{alias}/following      # Following list (cursor-paginated)
POST   /profiles/{alias}/follow         # Follow user
DELETE /profiles/{alias}/follow         # Unfollow user

GET    /search?q={query}&type={type}    # Search users, posts, hashtags

GET    /notifications                   # Notification list (cursor-paginated)
WS     /ws                              # WebSocket: real-time notifications
```

All list endpoints return cursor-paginated responses. All mutation endpoints require authentication (JWT via gateway). All responses follow the `ApiResponse<T>` envelope from `nyx-api`.
