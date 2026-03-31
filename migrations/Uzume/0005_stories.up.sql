-- =============================================================================
-- 0005_stories.up.sql
-- Stories, story views, and highlights.
--
-- Design notes:
--   TTL ENFORCEMENT:
--     expires_at is set to NOW() + 24h at insert time.
--     The story_expiry worker (Uzume-stories service) runs every 60s and
--     soft-deletes rows where expires_at < NOW() AND highlight_count = 0.
--     Stories in a highlight are exempt from expiry (highlight_count > 0).
--
--   MEDIA:
--     Same raw_key / variant_keys pattern as post_media. Oya processes the
--     upload and fills in the variant keys via Uzume.media.processed event.
--
--   PRIVACY:
--     Story viewer list is only accessible to the story author.
--     Close-friends lists are a future feature (not in Phase 0).
--
--   HIGHLIGHTS:
--     A highlight is a named collection of stories shown permanently on a
--     profile. Stories in a highlight are excluded from expiry.
-- =============================================================================

-- ── Stories ───────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS "Uzume".stories (
    id                  UUID        NOT NULL DEFAULT gen_random_uuid(),
    author_profile_id   UUID        NOT NULL,
    media_type          TEXT        NOT NULL,   -- 'image' | 'video'

    -- Storage keys (same naming convention as post_media)
    raw_key             TEXT        NOT NULL,
    full_key            TEXT,       -- processed: 1080px image / HLS manifest
    thumbnail_key       TEXT,       -- 150px preview shown in story ring

    width_px            INTEGER,
    height_px           INTEGER,
    duration_ms         INTEGER,    -- NULL for images (client uses 5 000ms display)
    blurhash            TEXT,

    processing_state    TEXT        NOT NULL DEFAULT 'pending',
    -- 'pending' → 'processing' → 'ready' | 'failed'

    -- Caption / stickers (stored as structured JSONB for flexibility)
    -- Schema: [{"type": "text", "content": "hello", "x": 0.5, "y": 0.3}, ...]
    overlays            JSONB       NOT NULL DEFAULT '[]'::jsonb,

    -- View count — updated by story_views insert trigger for denormalization
    view_count          BIGINT      NOT NULL DEFAULT 0,

    -- How many highlights contain this story (>0 = exempt from 24h expiry)
    highlight_count     INTEGER     NOT NULL DEFAULT 0,

    expires_at          TIMESTAMPTZ NOT NULL DEFAULT NOW() + INTERVAL '24 hours',
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uzume_stories_pk
        PRIMARY KEY (id),

    CONSTRAINT uzume_stories_author_fk
        FOREIGN KEY (author_profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_stories_media_type_ck
        CHECK (media_type IN ('image', 'video')),

    CONSTRAINT uzume_stories_state_ck
        CHECK (processing_state IN ('pending', 'processing', 'ready', 'failed')),

    CONSTRAINT uzume_stories_highlight_count_ck
        CHECK (highlight_count >= 0)
);

-- Story ring: "give me all non-expired stories for author P, newest last"
CREATE INDEX IF NOT EXISTS uzume_stories_author_active_idx
    ON "Uzume".stories (author_profile_id, created_at ASC)
    WHERE expires_at > NOW();

-- Expiry worker: "give me stories whose TTL has elapsed and are not in highlights"
CREATE INDEX IF NOT EXISTS uzume_stories_expiry_idx
    ON "Uzume".stories (expires_at)
    WHERE highlight_count = 0;

-- ── Story views ───────────────────────────────────────────────────────────────
-- One row per (story, viewer) pair. Insert-only; no updates.
CREATE TABLE IF NOT EXISTS "Uzume".story_views (
    story_id            UUID        NOT NULL,
    viewer_profile_id   UUID        NOT NULL,
    viewed_at           TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uzume_story_views_pk
        PRIMARY KEY (story_id, viewer_profile_id),

    CONSTRAINT uzume_story_views_story_fk
        FOREIGN KEY (story_id)
        REFERENCES "Uzume".stories (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_story_views_viewer_fk
        FOREIGN KEY (viewer_profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE
);

-- "Who has viewed story S?" — shown only to the author.
CREATE INDEX IF NOT EXISTS uzume_story_views_story_idx
    ON "Uzume".story_views (story_id, viewed_at DESC);

-- ── Highlights ────────────────────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS "Uzume".highlights (
    id                  UUID        NOT NULL DEFAULT gen_random_uuid(),
    author_profile_id   UUID        NOT NULL,
    title               TEXT        NOT NULL,
    -- Cover is usually the first story thumbnail, but user can override.
    cover_media_key     TEXT,
    display_order       INTEGER     NOT NULL DEFAULT 0,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at          TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uzume_highlights_pk
        PRIMARY KEY (id),

    CONSTRAINT uzume_highlights_author_fk
        FOREIGN KEY (author_profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE,

    CONSTRAINT uzume_highlights_title_len
        CHECK (char_length(title) <= 150)
);

CREATE INDEX IF NOT EXISTS uzume_highlights_author_idx
    ON "Uzume".highlights (author_profile_id, display_order);

-- ── Highlight → story junction ────────────────────────────────────────────────
CREATE TABLE IF NOT EXISTS "Uzume".highlight_stories (
    highlight_id        UUID        NOT NULL,
    story_id            UUID        NOT NULL,
    display_order       INTEGER     NOT NULL DEFAULT 0,
    added_at            TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uzume_highlight_stories_pk
        PRIMARY KEY (highlight_id, story_id),

    CONSTRAINT uzume_highlight_stories_highlight_fk
        FOREIGN KEY (highlight_id)
        REFERENCES "Uzume".highlights (id)
        ON DELETE CASCADE,

    -- Story is kept alive (highlight_count > 0); cascade delete would leave
    -- the highlight with a dangling reference, so restrict instead.
    CONSTRAINT uzume_highlight_stories_story_fk
        FOREIGN KEY (story_id)
        REFERENCES "Uzume".stories (id)
        ON DELETE RESTRICT
);

CREATE INDEX IF NOT EXISTS uzume_highlight_stories_highlight_idx
    ON "Uzume".highlight_stories (highlight_id, display_order);
