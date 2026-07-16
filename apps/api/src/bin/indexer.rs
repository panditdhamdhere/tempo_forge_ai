use sqlx::postgres::PgPoolOptions;
use std::time::Duration;
use tempoforge_blockchain::TempoClient;
use tempoforge_indexer::{Indexer, network_from_env};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://tempoforge:tempoforge@localhost:5432/tempoforge".into());

    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    sqlx::migrate!("../../migrations").run(&db).await?;

    let client = TempoClient::new(network_from_env());
    let start = std::env::var("INDEXER_START_BLOCK")
        .ok()
        .and_then(|v| v.parse().ok());

    let mut indexer = Indexer::bootstrap(client, db, start).await?;
    tracing::info!(
        network = indexer.network(),
        cursor = indexer.cursor(),
        "TempoForge indexer starting"
    );

    let interval_ms = std::env::var("INDEXER_POLL_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2_000u64);

    loop {
        match indexer.tick().await {
            Ok(blocks) if blocks.is_empty() => {
                tokio::time::sleep(Duration::from_millis(interval_ms)).await;
            }
            Ok(blocks) => {
                tracing::info!(count = blocks.len(), cursor = indexer.cursor(), "indexed batch");
            }
            Err(err) => {
                tracing::error!(error = %err, "indexer tick failed");
                tokio::time::sleep(Duration::from_millis(interval_ms.max(3_000))).await;
            }
        }
    }
}
