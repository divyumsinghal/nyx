use nyx_tests::sandbox::SandboxManager;

#[tokio::test]
#[ignore = "requires Docker"]
async fn sandbox_starts_postgres_redis_and_minio() {
    let sandbox = SandboxManager::new()
        .with_postgres()
        .await
        .expect("postgres should start")
        .with_redis()
        .await
        .expect("redis should start")
        .with_minio()
        .await
        .expect("minio should start");

    let postgres_url = sandbox.postgres_url().await;
    let redis_url = sandbox.redis_url().await;
    let minio = sandbox.minio_config().await;

    assert!(postgres_url.starts_with("postgres://"));
    assert!(redis_url.starts_with("redis://"));
    assert!(minio.endpoint.starts_with("http://"));
}
