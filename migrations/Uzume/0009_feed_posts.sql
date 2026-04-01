CREATE TABLE IF NOT EXISTS uzume.posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    author_alias TEXT NOT NULL,
    identity_id UUID NOT NULL,
    caption TEXT NOT NULL,
    like_count BIGINT NOT NULL DEFAULT 0,
    comment_count BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS posts_identity_id_created_at ON uzume.posts (identity_id, created_at DESC);
CREATE INDEX IF NOT EXISTS posts_created_at ON uzume.posts (created_at DESC);

CREATE TABLE IF NOT EXISTS uzume.post_likes (
    post_id UUID NOT NULL REFERENCES uzume.posts(id) ON DELETE CASCADE,
    liker_alias TEXT NOT NULL,
    liker_identity_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (post_id, liker_identity_id)
);

CREATE TABLE IF NOT EXISTS uzume.comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES uzume.posts(id) ON DELETE CASCADE,
    author_alias TEXT NOT NULL,
    author_identity_id UUID NOT NULL,
    content TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS comments_post_id ON uzume.comments (post_id, created_at DESC);
