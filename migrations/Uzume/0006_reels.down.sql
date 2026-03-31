ALTER TABLE "Uzume".reel_audio
    DROP CONSTRAINT IF EXISTS reel_audio_original_reel_fk;

DROP TABLE IF EXISTS "Uzume".reel_views;
DROP TABLE IF EXISTS "Uzume".reel_likes;
DROP TABLE IF EXISTS "Uzume".reels;
DROP TABLE IF EXISTS "Uzume".reel_audio;
