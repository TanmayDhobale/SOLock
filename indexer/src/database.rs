use anyhow::Result;
use chrono::{DateTime, Utc};
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;
use tracing::info;

#[derive(Debug, Clone)]
pub struct WriteLockEvent {
    pub time: DateTime<Utc>,
    pub slot: i64,
    pub account_pubkey: String,
    pub program_id: Option<String>,
    pub transaction_signature: String,
    pub success: bool,
    pub lock_contention_score: f32,
    pub priority_fee_lamports: Option<i64>,
    pub compute_units_consumed: Option<i32>,
}

pub struct Database {
    pool: Pool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        // Parse database URL
        let pg_config: tokio_postgres::Config = database_url.parse()?;
        let mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
        let pool = Pool::builder(mgr).max_size(16).build()?;
        
        // Test connection
        let _ = pool.get().await?;
        
        info!("Database connection established");

        Ok(Self { pool })
    }

    pub async fn insert_events(&self, events: &[WriteLockEvent]) -> Result<()> {
        if events.is_empty() {
            return Ok(());
        }

        let client = self.pool.get().await?;

        for event in events {
            client
                .execute(
                    r#"
                    INSERT INTO write_lock_events (
                        time, slot, account_pubkey, program_id, transaction_signature,
                        success, lock_contention_score, priority_fee_lamports, compute_units_consumed
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                    "#,
                    &[
                        &event.time,
                        &event.slot,
                        &event.account_pubkey,
                        &event.program_id,
                        &event.transaction_signature,
                        &event.success,
                        &event.lock_contention_score,
                        &event.priority_fee_lamports,
                        &event.compute_units_consumed,
                    ],
                )
                .await?;
        }

        Ok(())
    }

    #[allow(dead_code)]
    pub async fn upsert_account_metadata(
        &self,
        pubkey: &str,
        program_id: Option<&str>,
        label: Option<&str>,
    ) -> Result<()> {
        let client = self.pool.get().await?;
        
        client
            .execute(
                r#"
                INSERT INTO account_metadata (pubkey, program_id, label, last_seen)
                VALUES ($1, $2, $3, NOW())
                ON CONFLICT (pubkey)
                DO UPDATE SET
                    program_id = COALESCE($2, account_metadata.program_id),
                    label = COALESCE($3, account_metadata.label),
                    last_seen = NOW()
                "#,
                &[&pubkey, &program_id, &label],
            )
            .await?;

        Ok(())
    }
}
