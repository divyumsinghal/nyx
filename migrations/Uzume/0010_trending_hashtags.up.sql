-- =============================================================================
-- 0010_trending_hashtags.up.sql
-- Trending hashtag snapshot table for Uzume-discover.
-- =============================================================================

CREATE TABLE IF NOT EXISTS "Uzume".trending_hashtags (
    hashtag     TEXT            NOT NULL,
    post_count  BIGINT          NOT NULL DEFAULT 0,
    score       DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    updated_at  TIMESTAMPTZ     NOT NULL DEFAULT NOW(),

    CONSTRAINT uzume_trending_hashtags_pk
        PRIMARY KEY (hashtag),

    CONSTRAINT uzume_trending_hashtags_post_count_ck
        CHECK (post_count >= 0),

    CONSTRAINT uzume_trending_hashtags_score_ck
        CHECK (score >= 0.0)
);

CREATE INDEX IF NOT EXISTS uzume_trending_hashtags_score_idx
    ON "Uzume".trending_hashtags (score DESC);
