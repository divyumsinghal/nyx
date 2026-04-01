use nyx_tests::sandbox::SandboxManager;

#[tokio::test]
#[ignore = "requires Docker"]
async fn content_flow_sandbox_storage_is_ready() {
    let sandbox = SandboxManager::new()
        .with_postgres()
        .await
        .expect("postgres must start")
        .with_minio()
        .await
        .expect("minio must start");

    let postgres = sandbox.postgres_url().await;
    let minio = sandbox.minio_config().await;

    assert!(postgres.contains("nyx_test"));
    assert!(minio.endpoint.starts_with("http://"));
}
