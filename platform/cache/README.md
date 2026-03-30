
### platform/nyx-cache

DragonflyDB/Redis client wrapper. Depends on `Monad`.

DragonflyDB client. Key convention: `{app}:{entity}:{id}`. Provides: token bucket rate limiter, session cache, `get_or_set` cache-aside helper, TTL constants.

```
nyx-cache/
├── Cargo.toml
├── src/
│   ├── lib.rs
│   ├── client.rs          # CacheClient (wraps fred::Client), connection pool
│   ├── keys.rs            # Key namespace convention: {app}:{entity}:{id}
│   ├── rate_limit.rs      # Token bucket implementation (INCR + EXPIRE)
│   ├── session.rs         # Session cache (store/retrieve validated Kratos sessions)
│   └── helpers.rs         # get_or_set (cache-aside pattern), TTL constants
└── tests/
```

Key namespace convention ensures no collisions across apps:

```
Uzume:user:{id}          → cached Uzume user profile
Uzume:feed:{id}          → cached home feed post IDs
Anteros:profile:{id}     → cached Anteros dating profile
Themis:listing:{id}     → cached Themis listing
nyx:session:{token}     → cached Kratos session
nyx:rate:{ip}:{route}   → rate limit counter
```
