-- =============================================================================
-- 0004_follows.up.sql
-- Follow graph + block/mute list.
--
-- Design notes:
--   FOLLOW STATUS:
--     'accepted' — normal follow (public account, or private account approved)
--     'pending'  — follow request sent to a private account, not yet approved
--     No 'rejected' state: a rejected follow simply has the row deleted.
--
--   BLOCK vs MUTE:
--     Block: bidirectional content isolation. Neither party sees the other.
--            Enforced at query level in feed + discover.
--     Mute:  Unilateral silence. Muter still appears in mutee's feed, but the
--            mutee's content is suppressed from the muter's feed/stories/reels.
--            The mutee cannot tell they've been muted (privacy-respecting).
--
--   FOLLOWER COUNTS:
--     Denormalized on profiles table (added here via ALTER TABLE).
--     Kept eventually consistent by uzume-profiles service via NATS events.
-- =============================================================================

-- ── Denormalized follower/following counts on profiles ────────────────────────
ALTER TABLE "Uzume".profiles
    ADD COLUMN IF NOT EXISTS follower_count  BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS following_count BIGINT NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS post_count      BIGINT NOT NULL DEFAULT 0;

-- ── Follows ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS "Uzume".follows (
    follower_profile_id     UUID        NOT NULL,
    followee_profile_id     UUID        NOT NULL,
    -- 'pending' only occurs when followee's account is private.
    status                  TEXT        NOT NULL DEFAULT 'accepted',
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- Timestamp of when a pending request was approved (NULL for auto-accepts).
    accepted_at             TIMESTAMPTZ,

    CONSTRAINT uzume_follows_pk
        PRIMARY KEY (follower_profile_id, followee_profile_id),

    CONSTRAINT uzume_follows_follower_fk
        FOREIGN KEY (follower_profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_follows_followee_fk
        FOREIGN KEY (followee_profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_follows_no_self
        CHECK (follower_profile_id <> followee_profile_id),

    CONSTRAINT uzume_follows_status_ck
        CHECK (status IN ('accepted', 'pending'))
);

-- "Who follows this person?" — used to build home feed fanout targets.
CREATE INDEX IF NOT EXISTS uzume_follows_followee_accepted_idx
    ON "Uzume".follows (followee_profile_id)
    WHERE status = 'accepted';

-- "Who is this person following?" — used for feed pull strategy.
CREATE INDEX IF NOT EXISTS uzume_follows_follower_accepted_idx
    ON "Uzume".follows (follower_profile_id)
    WHERE status = 'accepted';

-- "Pending requests for this account owner to approve/deny"
CREATE INDEX IF NOT EXISTS uzume_follows_followee_pending_idx
    ON "Uzume".follows (followee_profile_id, created_at DESC)
    WHERE status = 'pending';

-- ── Blocks ────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS "Uzume".blocks (
    blocker_profile_id  UUID        NOT NULL,
    blocked_profile_id  UUID        NOT NULL,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uzume_blocks_pk
        PRIMARY KEY (blocker_profile_id, blocked_profile_id),

    CONSTRAINT uzume_blocks_blocker_fk
        FOREIGN KEY (blocker_profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_blocks_blocked_fk
        FOREIGN KEY (blocked_profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_blocks_no_self
        CHECK (blocker_profile_id <> blocked_profile_id)
);

-- "Is profile B blocked by profile A?" — checked before showing any content.
CREATE INDEX IF NOT EXISTS uzume_blocks_blocker_idx
    ON "Uzume".blocks (blocker_profile_id);

CREATE INDEX IF NOT EXISTS uzume_blocks_blocked_idx
    ON "Uzume".blocks (blocked_profile_id);

-- ── Mutes ─────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS "Uzume".mutes (
    muter_profile_id    UUID        NOT NULL,
    muted_profile_id    UUID        NOT NULL,
    -- What to suppress: 'posts' | 'stories' | 'all'
    mute_scope          TEXT        NOT NULL DEFAULT 'all',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uzume_mutes_pk
        PRIMARY KEY (muter_profile_id, muted_profile_id),

    CONSTRAINT uzume_mutes_muter_fk
        FOREIGN KEY (muter_profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_mutes_muted_fk
        FOREIGN KEY (muted_profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_mutes_no_self
        CHECK (muter_profile_id <> muted_profile_id),

    CONSTRAINT uzume_mutes_scope_ck
        CHECK (mute_scope IN ('posts', 'stories', 'all'))
);

CREATE INDEX IF NOT EXISTS uzume_mutes_muter_idx
    ON "Uzume".mutes (muter_profile_id);
