//! db-reset command integration tests — requires Docker (testcontainers).

#[cfg(feature = "integration")]
mod integration {
    use testcontainers::runners::AsyncRunner;
    use testcontainers_modules::postgres::Postgres;

    async fn start_pg() -> (testcontainers::ContainerAsync<Postgres>, String) {
        let container = Postgres::default().start().await.unwrap();
        let port = container.get_host_port_ipv4(5432).await.unwrap();
        let url = format!("postgres://postgres:postgres@127.0.0.1:{port}/postgres");
        (container, url)
    }

    #[tokio::test]
    async fn db_reset_drops_and_recreates_schemas() {
        let (_container, url) = start_pg().await;
        // First: migrate
        nyx_xtask::commands::migrate::run(&url).await.unwrap();

        // Insert a row to confirm data exists
        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        sqlx::query(
            "INSERT INTO nyx.app_aliases (nyx_identity_id, app, alias) VALUES ($1, 'uzume', 'testuser')",
        )
        .bind(uuid::Uuid::now_v7())
        .execute(&pool)
        .await
        .unwrap();

        // Confirm the row is there
        let before: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM nyx.app_aliases")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(before.0, 1, "pre-reset: should have 1 alias row");

        // Now reset
        nyx_xtask::commands::db_reset::run(&url).await.unwrap();

        // Tables must exist but be empty
        let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM nyx.app_aliases")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(row.0, 0, "post-reset: app_aliases should be empty");
    }

    #[tokio::test]
    async fn db_reset_is_idempotent() {
        let (_container, url) = start_pg().await;
        nyx_xtask::commands::migrate::run(&url).await.unwrap();
        nyx_xtask::commands::db_reset::run(&url).await.unwrap();
        // Second reset must not error
        nyx_xtask::commands::db_reset::run(&url).await.unwrap();
    }
}
