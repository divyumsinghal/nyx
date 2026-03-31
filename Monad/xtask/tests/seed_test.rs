//! Seed command tests — unit (JSON parsing) and integration (DB insert via testcontainers).

use nyx_xtask::commands::seed::{SeedPost, SeedUser};

// ---------------------------------------------------------------------------
// Cycle 1 — Unit: pure JSON parsing, zero I/O
// ---------------------------------------------------------------------------

#[test]
fn parse_users_json_has_correct_count() {
    let json = include_str!("../../../tools/seed-data/users.json");
    let users: Vec<SeedUser> = serde_json::from_str(json).unwrap();
    assert_eq!(users.len(), 10);
}

#[test]
fn parse_users_json_first_user_correct() {
    let json = include_str!("../../../tools/seed-data/users.json");
    let users: Vec<SeedUser> = serde_json::from_str(json).unwrap();
    assert_eq!(users[0].username, "alice.nyx");
    assert_eq!(users[0].display_name, "Alice Chen");
    assert!(!users[0].is_private);
}

#[test]
fn parse_posts_json_not_empty() {
    let json = include_str!("../../../tools/seed-data/uzume_posts.json");
    let posts: Vec<SeedPost> = serde_json::from_str(json).unwrap();
    assert!(!posts.is_empty());
}

#[test]
fn parse_posts_json_first_post_has_author() {
    let json = include_str!("../../../tools/seed-data/uzume_posts.json");
    let posts: Vec<SeedPost> = serde_json::from_str(json).unwrap();
    assert_eq!(posts[0].author_username, "alice.nyx");
    assert!(!posts[0].caption.is_empty());
}

// ---------------------------------------------------------------------------
// Cycle 6 — Integration: DB insert via testcontainers
// ---------------------------------------------------------------------------

#[cfg(feature = "integration")]
mod integration {
    use std::path::PathBuf;
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;

    async fn start_pg() -> (testcontainers::ContainerAsync<Postgres>, String) {
        let container = Postgres::default().start().await.unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
        (container, url)
    }

    fn find_workspace_root() -> PathBuf {
        nyx_xtask::commands::migrate::find_workspace_root().unwrap()
    }

    #[tokio::test]
    async fn seed_inserts_all_users_as_profiles() {
        let (_container, url) = start_pg().await;
        nyx_xtask::commands::migrate::run(&url).await.unwrap();

        let workspace_root = find_workspace_root();
        nyx_xtask::commands::seed::run(&url, &workspace_root.join("tools/seed-data"))
            .await
            .unwrap();

        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        let row: (i64,) = sqlx::query_as(r#"SELECT COUNT(*) FROM "Uzume".profiles"#)
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(row.0, 10);
    }

    #[tokio::test]
    async fn seed_inserts_app_aliases() {
        let (_container, url) = start_pg().await;
        nyx_xtask::commands::migrate::run(&url).await.unwrap();
        let workspace_root = find_workspace_root();
        nyx_xtask::commands::seed::run(&url, &workspace_root.join("tools/seed-data"))
            .await
            .unwrap();

        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM nyx.app_aliases")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(row.0, 10);
    }

    #[tokio::test]
    async fn seed_is_idempotent() {
        let (_container, url) = start_pg().await;
        nyx_xtask::commands::migrate::run(&url).await.unwrap();
        let workspace_root = find_workspace_root();
        let seed_dir = workspace_root.join("tools/seed-data");
        nyx_xtask::commands::seed::run(&url, &seed_dir).await.unwrap();
        // Second run must not error (ON CONFLICT DO NOTHING)
        nyx_xtask::commands::seed::run(&url, &seed_dir).await.unwrap();
    }
}
