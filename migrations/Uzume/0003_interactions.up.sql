-- =============================================================================
-- 0003_interactions.up.sql
-- Post interactions: likes, comments, saves.
-- Also extends the posts table with media + engagement counter columns.
--
-- Design notes:
--   POST MEDIA:
--     Stored separately (one-to-many). A post can have up to 10 media items
--     (like Instagram carousel). Raw upload keys are set immediately; processed
--     variant keys are filled in by Oya worker via Uzume.media.processed event.
--
--   DENORMALIZED COUNTERS:
--     like_count / comment_count / save_count on posts are denormalized for
--     feed rendering performance. Exact counts are derived from the child tables;
--     denormalized values are eventually consistent (updated via NATS events).
--
--   COMMENT TREE:
--     Flat parent_comment_id reference supports one level of reply threading
--     (top-level comment → reply). Deeper nesting is not supported per product spec.
-- =============================================================================

-- ── Extend posts with media + counters ────────────────────────────────────────
ALTER TABLE "Uzume".posts
    ADD COLUMN IF NOT EXISTS like_count     BIGINT      NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS comment_count  BIGINT      NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS save_count     BIGINT      NOT NULL DEFAULT 0,
    ADD COLUMN IF NOT EXISTS hashtags       TEXT[]      NOT NULL DEFAULT '{}',
    ADD COLUMN IF NOT EXISTS location_name  TEXT,
    ADD COLUMN IF NOT EXISTS updated_at     TIMESTAMPTZ NOT NULL DEFAULT NOW();

CREATE INDEX IF NOT EXISTS uzume_posts_author_created_idx
    ON "Uzume".posts (author_profile_id, created_at DESC);

CREATE INDEX IF NOT EXISTS uzume_posts_hashtags_idx
    ON "Uzume".posts USING GIN (hashtags);

-- ── Post media ────────────────────────────────────────────────────────────────
-- Stores one row per media item per post. Ordered by display_order.
CREATE TABLE IF NOT EXISTS "Uzume".post_media (
    id              UUID        NOT NULL DEFAULT gen_random_uuid(),
    post_id         UUID        NOT NULL,
    display_order   SMALLINT    NOT NULL DEFAULT 0,
    media_type      TEXT        NOT NULL,   -- 'image' | 'video'

    -- Raw upload (set immediately after client uploads to MinIO presigned URL)
    raw_key         TEXT        NOT NULL,

    -- Processed variants (filled in by Oya after transcoding)
    -- Keys follow: uzume/posts/{post_id}/{display_order}/{variant}.{ext}
    full_key        TEXT,   -- 1080px image / HLS manifest
    standard_key    TEXT,   -- 640px
    thumbnail_key   TEXT,   -- 150px thumbnail / video poster frame

    -- Metadata extracted from the original file
    width_px        INTEGER,
    height_px       INTEGER,
    duration_ms     INTEGER,    -- NULL for images
    blurhash        TEXT,       -- low-fi placeholder rendered before image loads

    processing_state TEXT NOT NULL DEFAULT 'pending',
    -- 'pending' → 'processing' → 'ready' | 'failed'

    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT post_media_pk
        PRIMARY KEY (id),

    CONSTRAINT post_media_post_fk
        FOREIGN KEY (post_id)
        REFERENCES "Uzume".posts (id)
        ON DELETE CASCADE,

    CONSTRAINT post_media_type_ck
        CHECK (media_type IN ('image', 'video')),

    CONSTRAINT post_media_state_ck
        CHECK (processing_state IN ('pending', 'processing', 'ready', 'failed')),

    CONSTRAINT post_media_order_ck
        CHECK (display_order BETWEEN 0 AND 9),  -- max 10 items per post

    CONSTRAINT post_media_order_unique
        UNIQUE (post_id, display_order)
);

CREATE INDEX IF NOT EXISTS post_media_post_idx
    ON "Uzume".post_media (post_id, display_order);

CREATE INDEX IF NOT EXISTS post_media_pending_idx
    ON "Uzume".post_media (created_at)
    WHERE processing_state = 'pending';

-- ── Likes ─────────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS "Uzume".likes (
    post_id         UUID        NOT NULL,
    profile_id      UUID        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uzume_likes_pk
        PRIMARY KEY (post_id, profile_id),

    CONSTRAINT uzume_likes_post_fk
        FOREIGN KEY (post_id)
        REFERENCES "Uzume".posts (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_likes_profile_fk
        FOREIGN KEY (profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE
);

-- "Did profile P like any of these post IDs?" — used in feed rendering.
CREATE INDEX IF NOT EXISTS uzume_likes_profile_idx
    ON "Uzume".likes (profile_id, created_at DESC);

-- ── Comments ──────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS "Uzume".comments (
    id                  UUID        NOT NULL DEFAULT gen_random_uuid(),
    post_id             UUID        NOT NULL,
    author_profile_id   UUID        NOT NULL,
    -- One level of nesting: top-level comments have NULL parent.
    parent_comment_id   UUID,
    body                TEXT        NOT NULL,
    -- Soft-delete: body is replaced with NULL, row stays for reply threading.
    is_deleted          BOOLEAN     NOT NULL DEFAULT FALSE,
    like_count          BIGINT      NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uzume_comments_pk
        PRIMARY KEY (id),

    CONSTRAINT uzume_comments_post_fk
        FOREIGN KEY (post_id)
        REFERENCES "Uzume".posts (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_comments_author_fk
        FOREIGN KEY (author_profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE,

    -- Only one level of nesting; replies point to a top-level comment.
    CONSTRAINT uzume_comments_parent_fk
        FOREIGN KEY (parent_comment_id)
        REFERENCES "Uzume".comments (id)
        ON DELETE SET NULL,

    CONSTRAINT uzume_comments_body_len
        CHECK (char_length(body) <= 2200),

    -- A comment cannot reply to itself
    CONSTRAINT uzume_comments_no_self_reply
        CHECK (id <> parent_comment_id)
);

CREATE INDEX IF NOT EXISTS uzume_comments_post_idx
    ON "Uzume".comments (post_id, created_at DESC)
    WHERE is_deleted = FALSE;

CREATE INDEX IF NOT EXISTS uzume_comments_parent_idx
    ON "Uzume".comments (parent_comment_id, created_at DESC)
    WHERE parent_comment_id IS NOT NULL AND is_deleted = FALSE;

-- ── Comment likes ─────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS "Uzume".comment_likes (
    comment_id      UUID        NOT NULL,
    profile_id      UUID        NOT NULL,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uzume_comment_likes_pk
        PRIMARY KEY (comment_id, profile_id),

    CONSTRAINT uzume_comment_likes_comment_fk
        FOREIGN KEY (comment_id)
        REFERENCES "Uzume".comments (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_comment_likes_profile_fk
        FOREIGN KEY (profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE
);

-- ── Saves (bookmarks) ─────────────────────────────────────────────────────────
-- A user can save a post to a named collection (default = "Saved").
CREATE TABLE IF NOT EXISTS "Uzume".saves (
    id                  UUID        NOT NULL DEFAULT gen_random_uuid(),
    post_id             UUID        NOT NULL,
    profile_id          UUID        NOT NULL,
    -- User-created collection name. "Saved" is the default visible collection.
    collection_name     TEXT        NOT NULL DEFAULT 'Saved',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uzume_saves_pk
        PRIMARY KEY (id),

    CONSTRAINT uzume_saves_post_fk
        FOREIGN KEY (post_id)
        REFERENCES "Uzume".posts (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_saves_profile_fk
        FOREIGN KEY (profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_saves_collection_name_len
        CHECK (char_length(collection_name) <= 100),

    -- A post can only appear once in the same collection.
    CONSTRAINT uzume_saves_unique
        UNIQUE (post_id, profile_id, collection_name)
);

CREATE INDEX IF NOT EXISTS uzume_saves_profile_collection_idx
    ON "Uzume".saves (profile_id, collection_name, created_at DESC);
