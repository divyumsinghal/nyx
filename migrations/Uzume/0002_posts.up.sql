CREATE TABLE IF NOT EXISTS "Uzume".posts (
    id UUID PRIMARY KEY,
    author_profile_id UUID NOT NULL,
    caption TEXT NOT NULL DEFAULT '',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT uzume_posts_author_fk FOREIGN KEY (author_profile_id)
        REFERENCES "Uzume".profiles (id)
        ON DELETE CASCADE
);
