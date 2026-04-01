//! Integration tests for the discover service HTTP API.
//!
//! These tests spin up a real PostgreSQL container via testcontainers and
//! verify the Axum router behaviour against a real database.
//!
//! All tests are marked `#[ignore = "requires Docker"]` and run in CI only.
//! Run with: `cargo test -p Uzume-discover -- --ignored`

#![allow(clippy::unwrap_used)]

use sqlx::PgPool;
use testcontainers::runners::AsyncRunner;
use testcontainers::ImageExt;
use testcontainers_modules::postgres::Postgres;

// ── Test helpers ──────────────────────────────────────────────────────────────

async fn make_pool(url: &str) -> PgPool {
    PgPool::connect(url)
        .await
        .expect("failed to connect to test PostgreSQL")
}

/// Apply the minimal DDL needed by the discover service tests.
///
/// We do not run the full migration chain here because integration tests for
/// each service own only their own schema fragments. The subset below is
/// exactly what `uzume-discover` queries touch.
async fn apply_schema(pool: &PgPool) {
    // nyx schema — required by profiles FK
    sqlx::query(r#"CREATE SCHEMA IF NOT EXISTS nyx"#)
        .execute(pool)
        .await
        .ok();

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS nyx.app_aliases (
            nyx_identity_id UUID NOT NULL,
            app TEXT NOT NULL,
            alias TEXT NOT NULL,
            PRIMARY KEY (nyx_identity_id, app, alias)
        )"#,
    )
    .execute(pool)
    .await
    .ok();

    // Uzume schema
    sqlx::query(r#"CREATE SCHEMA IF NOT EXISTS "Uzume""#)
        .execute(pool)
        .await
        .expect("failed to create Uzume schema");

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS "Uzume".profiles (
            id               UUID PRIMARY KEY,
            nyx_identity_id  UUID NOT NULL,
            app              TEXT NOT NULL DEFAULT 'uzume',
            alias            TEXT NOT NULL,
            display_name     TEXT NOT NULL DEFAULT '',
            bio              TEXT NOT NULL DEFAULT '',
            is_private       BOOLEAN NOT NULL DEFAULT TRUE,
            follower_count   BIGINT NOT NULL DEFAULT 0,
            following_count  BIGINT NOT NULL DEFAULT 0,
            post_count       BIGINT NOT NULL DEFAULT 0,
            created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
    )
    .execute(pool)
    .await
    .expect("create profiles");

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS "Uzume".posts (
            id               UUID PRIMARY KEY,
            author_profile_id UUID NOT NULL REFERENCES "Uzume".profiles(id) ON DELETE CASCADE,
            caption          TEXT NOT NULL DEFAULT '',
            like_count       BIGINT NOT NULL DEFAULT 0,
            comment_count    BIGINT NOT NULL DEFAULT 0,
            save_count       BIGINT NOT NULL DEFAULT 0,
            hashtags         TEXT[] NOT NULL DEFAULT '{}',
            created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
    )
    .execute(pool)
    .await
    .expect("create posts");

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS "Uzume".post_media (
            id               UUID NOT NULL DEFAULT gen_random_uuid(),
            post_id          UUID NOT NULL REFERENCES "Uzume".posts(id) ON DELETE CASCADE,
            display_order    SMALLINT NOT NULL DEFAULT 0,
            media_type       TEXT NOT NULL DEFAULT 'image',
            raw_key          TEXT NOT NULL DEFAULT '',
            thumbnail_key    TEXT,
            processing_state TEXT NOT NULL DEFAULT 'pending',
            created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            PRIMARY KEY (id)
        )"#,
    )
    .execute(pool)
    .await
    .expect("create post_media");

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS "Uzume".follows (
            follower_profile_id UUID NOT NULL,
            followee_profile_id UUID NOT NULL,
            status TEXT NOT NULL DEFAULT 'accepted',
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            PRIMARY KEY (follower_profile_id, followee_profile_id)
        )"#,
    )
    .execute(pool)
    .await
    .expect("create follows");

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS "Uzume".blocks (
            blocker_profile_id UUID NOT NULL,
            blocked_profile_id UUID NOT NULL,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            PRIMARY KEY (blocker_profile_id, blocked_profile_id)
        )"#,
    )
    .execute(pool)
    .await
    .expect("create blocks");

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS "Uzume".reels (
            id               UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            author_profile_id UUID NOT NULL,
            caption          TEXT NOT NULL DEFAULT '',
            raw_key          TEXT NOT NULL DEFAULT '',
            duration_ms      INTEGER NOT NULL DEFAULT 1000,
            processing_state TEXT NOT NULL DEFAULT 'ready',
            score            DOUBLE PRECISION NOT NULL DEFAULT 0.0,
            created_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            updated_at       TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
    )
    .execute(pool)
    .await
    .expect("create reels");

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS "Uzume".reel_audio (
            id         UUID PRIMARY KEY DEFAULT gen_random_uuid(),
            title      TEXT NOT NULL,
            audio_key  TEXT NOT NULL DEFAULT '',
            duration_ms INTEGER NOT NULL DEFAULT 1000,
            use_count  BIGINT NOT NULL DEFAULT 0,
            created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
    )
    .execute(pool)
    .await
    .expect("create reel_audio");

    sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS "Uzume".trending_hashtags (
            hashtag    VARCHAR(100) PRIMARY KEY,
            post_count BIGINT NOT NULL DEFAULT 0,
            score      DOUBLE PRECISION NOT NULL DEFAULT 0.0,
            updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        )"#,
    )
    .execute(pool)
    .await
    .expect("create trending_hashtags");
}

// ── Smoke test: database layer only (no Axum) ─────────────────────────────────

/// Verify that the trending hashtags query returns rows seeded via upsert.
#[tokio::test]
#[ignore = "requires Docker"]
async fn test_upsert_and_read_trending_hashtags() {
    let container = Postgres::default()
        .with_db_name("nyx_test")
        .with_user("nyx")
        .with_password("nyx")
        .start()
        .await
        .expect("failed to start postgres container");

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!("postgres://nyx:nyx@{host}:{port}/nyx_test");
    let pool = make_pool(&db_url).await;
    apply_schema(&pool).await;

    use uzume_discover::queries::trending::{get_trending_hashtags, upsert_trending_hashtag};

    // Seed two hashtags
    upsert_trending_hashtag(&pool, "sunsets", 100, 42.5)
        .await
        .expect("upsert sunsets");
    upsert_trending_hashtag(&pool, "travel", 50, 20.0)
        .await
        .expect("upsert travel");

    let rows = get_trending_hashtags(&pool, 10).await.expect("get trending");

    assert_eq!(rows.len(), 2);
    // sunsets should be first (higher score)
    assert_eq!(rows[0].hashtag, "sunsets");
    assert!((rows[0].score - 42.5).abs() < f64::EPSILON);
}

/// Verify that `get_trending_posts_for_explore` returns an empty list when no
/// posts have been created within the 48h window.
#[tokio::test]
#[ignore = "requires Docker"]
async fn test_explore_trending_posts_empty_on_fresh_db() {
    let container = Postgres::default()
        .with_db_name("nyx_test")
        .with_user("nyx")
        .with_password("nyx")
        .start()
        .await
        .expect("failed to start postgres container");

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!("postgres://nyx:nyx@{host}:{port}/nyx_test");
    let pool = make_pool(&db_url).await;
    apply_schema(&pool).await;

    use uzume_discover::queries::trending::get_trending_posts_for_explore;

    let items = get_trending_posts_for_explore(&pool, 20, None, None)
        .await
        .expect("query should succeed on empty DB");

    assert!(items.is_empty(), "no posts → empty explore feed");
}

/// Verify that posts seeded with hashtags appear in the hashtag window query.
#[tokio::test]
#[ignore = "requires Docker"]
async fn test_hashtag_window_counts_recent_posts() {
    let container = Postgres::default()
        .with_db_name("nyx_test")
        .with_user("nyx")
        .with_password("nyx")
        .start()
        .await
        .expect("failed to start postgres container");

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!("postgres://nyx:nyx@{host}:{port}/nyx_test");
    let pool = make_pool(&db_url).await;
    apply_schema(&pool).await;

    // Seed a profile and two posts with hashtags
    let profile_id = uuid::Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO "Uzume".profiles (id, nyx_identity_id, alias, display_name)
           VALUES ($1, $1, 'test_user', 'Test User')"#,
    )
    .bind(profile_id)
    .execute(&pool)
    .await
    .expect("insert profile");

    let post_a = uuid::Uuid::now_v7();
    let post_b = uuid::Uuid::now_v7();

    sqlx::query(
        r#"INSERT INTO "Uzume".posts (id, author_profile_id, hashtags)
           VALUES ($1, $2, ARRAY['sunset', 'travel'])"#,
    )
    .bind(post_a)
    .bind(profile_id)
    .execute(&pool)
    .await
    .expect("insert post_a");

    sqlx::query(
        r#"INSERT INTO "Uzume".posts (id, author_profile_id, hashtags)
           VALUES ($1, $2, ARRAY['sunset'])"#,
    )
    .bind(post_b)
    .bind(profile_id)
    .execute(&pool)
    .await
    .expect("insert post_b");

    use uzume_discover::queries::engagement::get_hashtag_counts_window;
    let counts = get_hashtag_counts_window(&pool, 24)
        .await
        .expect("get hashtag counts");

    // 'sunset' should have count = 2, 'travel' should have count = 1
    let sunset = counts.iter().find(|(h, _)| h == "sunset");
    let travel = counts.iter().find(|(h, _)| h == "travel");

    assert!(sunset.is_some(), "sunset should appear in hashtag counts");
    assert_eq!(sunset.unwrap().1, 2, "sunset count should be 2");
    assert!(travel.is_some(), "travel should appear in hashtag counts");
    assert_eq!(travel.unwrap().1, 1, "travel count should be 1");
}

/// Verify that the `compute_trending_hashtags_raw` query only returns hashtags
/// with at least 2 usages (per the HAVING clause).
#[tokio::test]
#[ignore = "requires Docker"]
async fn test_compute_raw_filters_single_use_hashtags() {
    let container = Postgres::default()
        .with_db_name("nyx_test")
        .with_user("nyx")
        .with_password("nyx")
        .start()
        .await
        .expect("failed to start postgres container");

    let host = container.get_host().await.unwrap();
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let db_url = format!("postgres://nyx:nyx@{host}:{port}/nyx_test");
    let pool = make_pool(&db_url).await;
    apply_schema(&pool).await;

    let profile_id = uuid::Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO "Uzume".profiles (id, nyx_identity_id, alias)
           VALUES ($1, $1, 'user2')"#,
    )
    .bind(profile_id)
    .execute(&pool)
    .await
    .expect("insert profile");

    // Post with hashtag 'once' (only used once — should be filtered out)
    let post_id = uuid::Uuid::now_v7();
    sqlx::query(
        r#"INSERT INTO "Uzume".posts (id, author_profile_id, hashtags)
           VALUES ($1, $2, ARRAY['once'])"#,
    )
    .bind(post_id)
    .bind(profile_id)
    .execute(&pool)
    .await
    .expect("insert post");

    use uzume_discover::queries::trending::compute_trending_hashtags_raw;
    let raw = compute_trending_hashtags_raw(&pool)
        .await
        .expect("compute raw trending");

    // 'once' appears only once — should not pass the HAVING COUNT(*) >= 2 filter
    let once = raw.iter().find(|(h, _, _)| h == "once");
    assert!(
        once.is_none(),
        "hashtag used only once should be excluded from raw trending"
    );
}
