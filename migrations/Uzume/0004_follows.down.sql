DROP TABLE IF EXISTS "Uzume".mutes;
DROP TABLE IF EXISTS "Uzume".blocks;
DROP TABLE IF EXISTS "Uzume".follows;

ALTER TABLE "Uzume".profiles
    DROP COLUMN IF EXISTS follower_count,
    DROP COLUMN IF EXISTS following_count,
    DROP COLUMN IF EXISTS post_count;
