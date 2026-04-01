use nyx_tests::sandbox::SandboxManager;

#[tokio::test]
#[ignore = "requires Docker"]
async fn registration_flow_sandbox_bootstrap_is_ready() {
    let sandbox = SandboxManager::new()
        .with_postgres()
        .await
        .expect("postgres must start")
        .with_redis()
        .await
        .expect("redis must start")
        .with_nats()
        .await
        .expect("nats must start");

    let postgres = sandbox.postgres_url().await;
    let redis = sandbox.redis_url().await;
    let nats = sandbox.nats_url().await;

    assert!(postgres.starts_with("postgres://"));
    assert!(redis.starts_with("redis://"));
    assert!(nats.starts_with("nats://"));
}
