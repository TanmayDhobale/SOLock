use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rpc_endpoint: String,
    pub database_url: String,
    
    #[allow(dead_code)]
    #[serde(default = "default_poll_interval_ms")]
    pub poll_interval_ms: u64,
    
    #[allow(dead_code)]
    #[serde(default = "default_batch_size")]
    pub batch_size: usize,
}

fn default_poll_interval_ms() -> u64 {
    400
}

fn default_batch_size() -> usize {
    500
}

impl Config {
    pub fn load() -> Result<Self> {
        let config_path = "config.toml";
        
        if std::path::Path::new(config_path).exists() {
            let content = fs::read_to_string(config_path)
                .context("Failed to read config.toml")?;
            toml::from_str(&content)
                .context("Failed to parse config.toml")
        } else {
            // Use environment variables as fallback
            Ok(Self {
                rpc_endpoint: std::env::var("SOLANA_RPC_URL")
                    .unwrap_or_else(|_| "https://api.mainnet-beta.solana.com".to_string()),
                database_url: std::env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgresql://solana:solana_dev_password@localhost:5432/solana_locks".to_string()),
                poll_interval_ms: default_poll_interval_ms(),
                batch_size: default_batch_size(),
            })
        }
    }
}
