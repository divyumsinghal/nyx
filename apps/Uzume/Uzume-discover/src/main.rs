use tracing::info;
use uzume_discover::{config, routes, state::AppState, workers};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "uzume_discover=info,nyx_api=info".into()),
        )
        .json()
        .init();

    let nyx_config = config::load()?;
    info!(
        environment = %nyx_config.environment,
        addr = %nyx_config.server.addr(),
        "uzume-discover starting"
    );

    let db = mnemosyne::build_pool_from_config(&nyx_config.database).await?;
    info!("PostgreSQL pool ready");

    let cache = lethe::CacheClient::connect(nyx_config.cache.url.expose()).await?;
    info!("DragonflyDB cache ready");

    let nats = nyx_events::NatsClient::connect(&nyx_config.nats.url).await?;
    info!("NATS JetStream ready");

    let storage = akash::connect(&nyx_config.storage)?;
    info!("Storage client ready");

    let search = brizo::connect(&nyx_config.search);
    info!("Meilisearch search client ready");

    let state = AppState {
        db,
        cache,
        nats,
        storage,
        search,
    };

    let router = routes::router(state.clone());

    let server = nyx_api::NyxServer::builder()
        .with_config(nyx_config)
        .with_routes(router)
        .build()?;

    tokio::spawn(workers::trending_updater::run(state.clone()));
    tokio::spawn(workers::search_sync::run(state.clone()));
    info!("background workers spawned");

    server.serve().await?;

    Ok(())
}
