# Anteros

Anteros — the dating platform. Same internal structure as Uzume.

```
Anteros/
├── Cargo.toml
├── src/
│   ├── main.rs
│   ├── config.rs                  # Anteros config (fair-show window size, max distance, etc.)
│   ├── routes/
│   │   ├── mod.rs
│   │   ├── profiles.rs            # POST/GET/PATCH dating profiles
│   │   ├── discover.rs            # GET /discover (card stack of potential matches)
│   │   ├── swipe.rs               # POST /swipe (right/left)
│   │   ├── matches.rs             # GET /matches (matched users)
│   │   ├── preferences.rs         # GET/PATCH /preferences (age range, distance, gender, etc.)
│   │   └── health.rs
│   ├── handlers/
│   │   ├── mod.rs
│   │   ├── profiles.rs
│   │   ├── discover.rs
│   │   ├── swipe.rs
│   │   ├── matches.rs
│   │   └── preferences.rs
│   ├── services/
│   │   ├── mod.rs
│   │   ├── fair_show.rs           # The "Fair Show" algorithm: ensures right-swipers are shown
│   │   ├── discovery.rs           # Discovery feed builder (location + preferences + fair-show queue)
│   │   ├── matching.rs            # Match detection: mutual right swipe → create match + Matrix room
│   │   └── scoring.rs             # Profile compatibility scoring (distance, age, shared interests)
│   ├── models/
│   │   ├── mod.rs
│   │   ├── profile.rs             # DatingProfile, ProfileCreate, ProfileResponse
│   │   ├── swipe.rs               # Swipe, SwipeDirection
│   │   ├── match_.rs              # Match, MatchResponse
│   │   └── preference.rs          # DiscoveryPreferences
│   ├── queries/
│   │   ├── mod.rs
│   │   ├── profiles.rs
│   │   ├── swipes.rs
│   │   ├── matches.rs
│   │   └── fair_show.rs
│   └── workers/
│       ├── mod.rs
│       ├── fair_show_injector.rs   # Listens to Anteros.swipe.right → enqueues into fair_show_queue
│       └── search_sync.rs
└── tests/
    ├── api/
    │   ├── swipe_test.rs
    │   ├── discover_test.rs
    │   └── match_test.rs
    └── services/
        ├── fair_show_test.rs
        └── discovery_test.rs
```

**Anteros API surface** (all prefixed with `/api/Anteros`):

```
POST   /profiles                        # Create dating profile
GET    /profiles/me                     # Get own profile
PATCH  /profiles/me                     # Update profile (bio, photos, interests)

GET    /discover                        # Next batch of profiles to swipe on
POST   /swipe                           # Submit swipe { profile_id, direction: "right"|"left" }

GET    /matches                         # List all matches (cursor-paginated)
GET    /matches/{id}                    # Single match detail

GET    /preferences                     # Current discovery preferences
PATCH  /preferences                     # Update preferences (distance, age range, gender)

GET    /stats                           # Transparent stats: "shown to X people, Y liked you"

WS     /ws                              # WebSocket: new match notification, typing indicators
```
