use anyhow::Result;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use std::sync::Arc;

mod config;
mod database;
mod rpc_stream;
mod lock_detector;
mod live_tracker;
mod known_programs;

use config::Config;
use database::Database;
use rpc_stream::RpcStream;
use live_tracker::LiveTracker;

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

    // Initialize live tracker (10-slot window)
    let live_tracker = Arc::new(LiveTracker::new(10));
    info!("📊 Live tracker initialized (10-slot window)");

    // Initialize RPC stream
    let mut rpc_stream = RpcStream::new(config.rpc_endpoint.clone());
    info!("🔗 Connected to Solana RPC");

    // Spawn cleanup task
    let tracker_cleanup = live_tracker.clone();
    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(30)).await;
            tracker_cleanup.cleanup_stale().await;
        }
    });

    // Main processing loop
    info!("📊 Starting real-time transaction monitoring...");
    loop {
        match rpc_stream.process_slot_live(&database, &live_tracker).await {
            Ok(_) => {}
            Err(e) => {
                warn!("Error processing slot: {}", e);
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            }
        }
    }
}

