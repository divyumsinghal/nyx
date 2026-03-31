DROP TABLE IF EXISTS "Uzume".saves;
DROP TABLE IF EXISTS "Uzume".comment_likes;
DROP TABLE IF EXISTS "Uzume".comments;
DROP TABLE IF EXISTS "Uzume".likes;
DROP TABLE IF EXISTS "Uzume".post_media;

ALTER TABLE "Uzume".posts
    DROP COLUMN IF EXISTS like_count,
    DROP COLUMN IF EXISTS comment_count,
    DROP COLUMN IF EXISTS save_count,
    DROP COLUMN IF EXISTS hashtags,
    DROP COLUMN IF EXISTS location_name,
    DROP COLUMN IF EXISTS updated_at;
