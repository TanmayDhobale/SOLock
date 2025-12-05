use anyhow::Result;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod config;
mod database;
mod rpc_stream;
mod lock_detector;

use config::Config;
use database::Database;
use rpc_stream::RpcStream;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "indexer=info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🚀 Starting Solana Lock Indexer");

    // Load configuration
    let config = Config::load()?;
    info!("📝 Loaded config: RPC endpoint = {}", config.rpc_endpoint);

    // Initialize database
    let database = Database::new(&config.database_url).await?;
    info!("💾 Connected to database");

    // Initialize RPC stream
    let mut rpc_stream = RpcStream::new(config.rpc_endpoint.clone());
    info!("🔗 Connected to Solana RPC");

    // Main processing loop
    info!("📊 Starting transaction monitoring...");
    loop {
        match rpc_stream.process_slot(&database).await {
            Ok(_) => {}
            Err(e) => {
                warn!("Error processing slot: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        }
    }
}
