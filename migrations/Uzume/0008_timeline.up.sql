-- =============================================================================
-- 0008_timeline.up.sql
-- Materialized user feed timeline (push-based fanout).
--
-- Design notes:
--   STRATEGY: HYBRID PUSH/PULL
--     Push (fanout-on-write): When a user creates a post, Uzume-feed's
--       feed_fanout worker writes one row per follower to user_timeline.
--       This is fast for feed reads but expensive for mega-accounts.
--     Pull fallback: For accounts with > FANOUT_THRESHOLD followers (e.g. 100k),
--       the worker skips push; instead, feed reads pull directly from the posts
--       table and merge with the pre-populated timeline for normal accounts.
--       FANOUT_THRESHOLD is a runtime config value, not encoded in the schema.
--
--   CURSOR PAGINATION:
--     Feeds are paginated by (score DESC, inserted_at DESC, post_id).
--     The cursor is an opaque base64 token encoding these three values.
--     Using a score+timestamp compound cursor prevents the "item appears twice"
--     problem that pure timestamp cursors have when items are inserted mid-scroll.
--
--   RETENTION:
--     Timeline rows are soft-bounded: the feed_fanout worker caps the timeline
--     to the N most recent posts per user (default N = 500). Older rows are
--     deleted in the same transaction as the insert via DELETE ... WHERE ctid IN
--     (SELECT ctid FROM user_timeline WHERE profile_id = ... ORDER BY score ASC
--      LIMIT <overflow>).
--     This prevents unbounded table growth without a separate cleanup job.
--
--   SCORE:
--     Initial score = 1.0 (chronological order).
--     Uzume-feed score_updater worker recalculates score when engagement events
--     arrive (post_liked, post_commented, etc.) and UPDATE the row.
--     Score updates are batched via DragonflyDB INCR + periodic flush.
-- =============================================================================

-- ── User timeline ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS "Uzume".user_timeline (
    profile_id          UUID            NOT NULL,
    post_id             UUID            NOT NULL,
    author_profile_id   UUID            NOT NULL,
    -- Composite ranking score: higher = shown earlier in feed.
    -- Seeded at 1.0; updated by engagement events.
    score               DOUBLE PRECISION NOT NULL DEFAULT 1.0,
    inserted_at         TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT uzume_user_timeline_pk
        PRIMARY KEY (profile_id, post_id),

    CONSTRAINT uzume_user_timeline_profile_fk
        FOREIGN KEY (profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_user_timeline_post_fk
        FOREIGN KEY (post_id)
        REFERENCES "Uzume".posts (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_user_timeline_author_fk
        FOREIGN KEY (author_profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE
);

-- PRIMARY feed read: "give me the top N posts for profile P (score DESC)"
-- This index is hot — it's read on every feed request.
CREATE INDEX IF NOT EXISTS uzume_user_timeline_feed_idx
    ON "Uzume".user_timeline (profile_id, score DESC, inserted_at DESC, post_id DESC);

-- Fanout cleanup: "oldest/lowest-scored entries for profile P" (for eviction)
CREATE INDEX IF NOT EXISTS uzume_user_timeline_eviction_idx
    ON "Uzume".user_timeline (profile_id, score ASC, inserted_at ASC);

-- Backfill: "all timeline entries for posts by author A" (used on unfollow to clean up)
CREATE INDEX IF NOT EXISTS uzume_user_timeline_author_idx
    ON "Uzume".user_timeline (author_profile_id);
