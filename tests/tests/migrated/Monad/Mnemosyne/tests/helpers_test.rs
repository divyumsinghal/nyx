use mnemosyne::{
    build_cursor_pagination_query, discover_schema_migrations, run_migrations_for_schemas,
};
use sqlx::types::Uuid;

#[test]
fn cursor_query_uses_desc_tuple_seek() {
    let sql = build_cursor_pagination_query(
        "Uzume.stories",
        &["id", "author_id", "created_at"],
        "created_at",
        "id",
    );

    assert!(sql.contains("SELECT id, author_id, created_at FROM Uzume.stories"));
    assert!(sql.contains("($1::timestamptz IS NULL OR (created_at, id) < ($1, $2))"));
    assert!(sql.contains("ORDER BY created_at DESC, id DESC"));
    assert!(sql.contains("LIMIT $3"));
}

#[test]
fn migration_discovery_finds_schema_dirs() {
    let base = std::env::temp_dir().join(format!("mnemosyne-migrations-{}", Uuid::now_v7()));
    std::fs::create_dir_all(base.join("Uzume")).unwrap();
    std::fs::create_dir_all(base.join("nyx")).unwrap();
    std::fs::write(base.join("README.md"), "ignore").unwrap();

    let schemas = discover_schema_migrations(&base).unwrap();
    assert_eq!(schemas, vec!["Uzume".to_string(), "nyx".to_string()]);

    std::fs::remove_dir_all(base).unwrap();
}

#[tokio::test]
async fn run_migrations_skips_missing_schema_paths() {
    let base = std::env::temp_dir().join(format!("mnemosyne-empty-{}", Uuid::now_v7()));
    std::fs::create_dir_all(base.join("Uzume")).unwrap();

    let pool = sqlx::postgres::PgPoolOptions::new()
        .connect_lazy("postgres://postgres:postgres@localhost/nyx")
        .unwrap();

    let result = run_migrations_for_schemas(&pool, &base, ["DoesNotExist"])
        .await
        .unwrap();
    assert!(result.is_empty());

    std::fs::remove_dir_all(base).unwrap();
}
