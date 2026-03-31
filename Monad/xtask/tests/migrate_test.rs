//! Migrate command integration tests — requires Docker (testcontainers).
//!
//! These tests are gated behind `#[cfg(feature = "integration")]` so that
//! `cargo test --lib` remains fast and CI can opt-in with `--features integration`.

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
    async fn migrate_creates_nyx_schema() {
        let (_container, url) = start_pg().await;
        nyx_xtask::commands::migrate::run(&url).await.unwrap();

        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM information_schema.schemata WHERE schema_name = 'nyx')",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(row.0, "nyx schema should exist");
    }

    #[tokio::test]
    async fn migrate_creates_uzume_schema() {
        let (_container, url) = start_pg().await;
        nyx_xtask::commands::migrate::run(&url).await.unwrap();

        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM information_schema.schemata WHERE schema_name = 'Uzume')",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(row.0, "Uzume schema should exist");
    }

    #[tokio::test]
    async fn migrate_creates_app_aliases_table() {
        let (_container, url) = start_pg().await;
        nyx_xtask::commands::migrate::run(&url).await.unwrap();

        let pool = sqlx::PgPool::connect(&url).await.unwrap();
        let row: (bool,) = sqlx::query_as(
            "SELECT EXISTS(SELECT 1 FROM information_schema.tables WHERE table_schema = 'nyx' AND table_name = 'app_aliases')",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert!(row.0);
    }

    #[tokio::test]
    async fn migrate_is_idempotent() {
        let (_container, url) = start_pg().await;
        nyx_xtask::commands::migrate::run(&url).await.unwrap();
        // Second run must not error
        nyx_xtask::commands::migrate::run(&url).await.unwrap();
    }
}
