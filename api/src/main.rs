use anyhow::Result;
use axum::{
    routing::{get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::{
    cors::CorsLayer,
    compression::CompressionLayer,
    trace::TraceLayer,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod database;
mod routes;
mod websocket;
mod known_programs;

use database::Database;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "api=info,sqlx=warn".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    info!("🚀 Starting Solana Lock API Server");

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://solana:solana_dev_password@localhost:5432/solana_locks".to_string());
    
    let database = Database::new(&database_url).await?;
    info!("💾 Connected to database");

    // Build router
    let app = Router::new()
        .route("/", get(root_handler))
        .route("/health", get(health_handler))
        .route("/api/stats", get(routes::dashboard_stats))
        .route("/api/hot-accounts", get(routes::hot_accounts))
        .route("/api/accounts/:pubkey/stats", get(routes::account_stats))
        .route("/api/accounts/:pubkey/fee-now", get(routes::fee_now))
        .route("/api/priority-fees/estimate", post(routes::estimate_priority_fee))
        .route("/ws", get(websocket::ws_handler))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(database);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    info!("🌐 Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root_handler() -> &'static str {
    "Solana Lock Contention API v0.1.0"
}

async fn health_handler() -> &'static str {
    "OK"
}
