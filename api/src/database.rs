use anyhow::Result;
use deadpool_postgres::{Manager, ManagerConfig, Pool, RecyclingMethod};
use tokio_postgres::NoTls;
use tracing::info;

#[derive(Debug, Clone)]
pub struct HotAccount {
    pub account_pubkey: String,
    pub lock_attempts: i64,
    pub successful_locks: i64,
    pub avg_priority_fee: Option<f64>,
    pub max_priority_fee: Option<i64>,
    pub avg_contention: Option<f64>,
    pub max_contention: Option<f64>,
}

#[derive(Debug, Clone)]
pub struct AccountStats {
    pub total_lock_attempts: i64,
    pub successful_locks: i64,
    pub failed_locks: i64,
    pub avg_contention: Option<f64>,
    pub avg_priority_fee: Option<f64>,
    pub max_priority_fee: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct DashboardStats {
    pub unique_accounts: i64,
    pub total_events: i64,
    pub high_contention_accounts: i64,
    pub avg_success_rate: f64,
}

#[derive(Clone)]
pub struct Database {
    pool: Pool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pg_config: tokio_postgres::Config = database_url.parse()?;
        let mgr_config = ManagerConfig {
            recycling_method: RecyclingMethod::Fast,
        };
        let mgr = Manager::from_config(pg_config, NoTls, mgr_config);
        let pool = Pool::builder(mgr).max_size(20).build()?;

        // Test connection
        let _ = pool.get().await?;

        info!("Database connection pool created");

        Ok(Self { pool })
    }

    /// Get dashboard overview stats
    pub async fn get_dashboard_stats(&self, minutes: i64) -> Result<DashboardStats> {
        let client = self.pool.get().await?;
        
        let rows = client.query(
            r#"
            SELECT
                COUNT(DISTINCT account_pubkey) as unique_accounts,
                COUNT(*) as total_events,
                COUNT(DISTINCT account_pubkey) FILTER (WHERE lock_contention_score >= 5) as high_contention_accounts,
                (COUNT(*) FILTER (WHERE success)::float8 / NULLIF(COUNT(*)::float8, 0) * 100) as avg_success_rate
            FROM write_lock_events
            WHERE time >= NOW() - ($1::text || ' minutes')::INTERVAL
            "#,
            &[&minutes.to_string()],
        ).await?;

        if rows.is_empty() {
            return Ok(DashboardStats {
                unique_accounts: 0,
                total_events: 0,
                high_contention_accounts: 0,
                avg_success_rate: 0.0,
            });
        }

        let row = &rows[0];
        Ok(DashboardStats {
            unique_accounts: row.get("unique_accounts"),
            total_events: row.get("total_events"),
            high_contention_accounts: row.get("high_contention_accounts"),
            avg_success_rate: row.get::<_, Option<f64>>("avg_success_rate").unwrap_or(0.0),
        })
    }


    /// Get hot accounts in the specified time window
    pub async fn get_hot_accounts(
        &self,
        limit: i64,
        minutes: i64,
    ) -> Result<Vec<HotAccount>> {
        let client = self.pool.get().await?;
        
        let rows = client.query(
            r#"
            SELECT
                time_bucket('5 minutes', time) as bucket,
                account_pubkey,
                COUNT(*) as lock_attempts,
                COUNT(*) FILTER (WHERE success) as successful_locks,
                AVG(priority_fee_lamports)::float8 as avg_priority_fee,
                MAX(priority_fee_lamports) as max_priority_fee,
                AVG(lock_contention_score)::float8 as avg_contention,
                MAX(lock_contention_score)::float8 as max_contention
            FROM write_lock_events
            WHERE time >= NOW() - ($1::text || ' minutes')::INTERVAL
            GROUP BY bucket, account_pubkey
            ORDER BY avg_contention DESC NULLS LAST, lock_attempts DESC
            LIMIT $2
            "#,
            &[&minutes.to_string(), &limit],
        ).await?;

        let accounts = rows.iter().map(|row| HotAccount {
            account_pubkey: row.get("account_pubkey"),
            lock_attempts: row.get("lock_attempts"),
            successful_locks: row.get("successful_locks"),
            avg_priority_fee: row.get("avg_priority_fee"),
            max_priority_fee: row.get("max_priority_fee"),
            avg_contention: row.get("avg_contention"),
            max_contention: row.get("max_contention"),
        }).collect();

        Ok(accounts)
    }

    /// Get detailed stats for a specific account
    pub async fn get_account_stats(
        &self,
        pubkey: &str,
        hours: i64,
    ) -> Result<Option<AccountStats>> {
        let client = self.pool.get().await?;
        
        let rows = client.query(
            r#"
            SELECT
                COUNT(*) as total_lock_attempts,
                COUNT(*) FILTER (WHERE success) as successful_locks,
                COUNT(*) FILTER (WHERE NOT success) as failed_locks,
                AVG(lock_contention_score)::float8 as avg_contention,
                AVG(priority_fee_lamports)::float8 as avg_priority_fee,
                MAX(priority_fee_lamports) as max_priority_fee
            FROM write_lock_events
            WHERE account_pubkey = $1
              AND time >= NOW() - ($2::text || ' hours')::INTERVAL
            "#,
            &[&pubkey, &hours.to_string()],
        ).await?;

        if rows.is_empty() {
            return Ok(None);
        }

        let row = &rows[0];
        Ok(Some(AccountStats {
            total_lock_attempts: row.get("total_lock_attempts"),
            successful_locks: row.get("successful_locks"),
            failed_locks: row.get("failed_locks"),
            avg_contention: row.get("avg_contention"),
            avg_priority_fee: row.get("avg_priority_fee"),
            max_priority_fee: row.get("max_priority_fee"),
        }))
    }

    /// Get current average priority fee for accounts with high contention
    pub async fn get_recommended_priority_fee(
        &self,
        accounts: &[String],
    ) -> Result<i64> {
        if accounts.is_empty() {
            return Ok(0);
        }

        let client = self.pool.get().await?;
        
        let rows = client.query(
            r#"
            SELECT
                PERCENTILE_CONT(0.75) WITHIN GROUP (ORDER BY priority_fee_lamports) as recommended_fee
            FROM write_lock_events
            WHERE account_pubkey = ANY($1)
              AND time >= NOW() - INTERVAL '1 hour'
              AND priority_fee_lamports IS NOT NULL
              AND success = true
            "#,
            &[&accounts],
        ).await?;

        if rows.is_empty() {
            return Ok(0);
        }

        let fee: Option<f64> = rows[0].get("recommended_fee");
        Ok(fee.unwrap_or(0.0) as i64)
    }

    /// Get LIVE fee estimate for an account (P90 of last 10 slots + 20% buffer)
    pub async fn get_live_fee_estimate(
        &self,
        pubkey: &str,
    ) -> Result<LiveFeeEstimate> {
        let client = self.pool.get().await?;
        
        // Get data from last 10 slots (~4 seconds)
        let rows = client.query(
            r#"
            SELECT
                slot,
                COUNT(*) as tx_count,
                AVG(lock_contention_score)::float8 as avg_contention,
                MAX(priority_fee_lamports) as max_fee,
                PERCENTILE_CONT(0.9) WITHIN GROUP (ORDER BY priority_fee_lamports) as p90_fee
            FROM write_lock_events
            WHERE account_pubkey = $1
              AND time >= NOW() - INTERVAL '30 seconds'
              AND priority_fee_lamports IS NOT NULL
            GROUP BY slot
            ORDER BY slot DESC
            LIMIT 10
            "#,
            &[&pubkey],
        ).await?;

        if rows.is_empty() {
            return Ok(LiveFeeEstimate {
                account: pubkey.to_string(),
                queue_depth: 0,
                p90_fee: 0,
                recommended_fee: 0,
                avg_contention: 0.0,
                slots_observed: 0,
            });
        }

        // Calculate aggregates
        let queue_depth: i64 = rows.iter()
            .map(|r| r.get::<_, i64>("tx_count"))
            .sum();
        
        let avg_contention: f64 = rows.iter()
            .map(|r| r.get::<_, Option<f64>>("avg_contention").unwrap_or(0.0))
            .sum::<f64>() / rows.len() as f64;

        // Get P90 across all recent slots
        let mut all_fees: Vec<i64> = rows.iter()
            .filter_map(|r| r.get::<_, Option<f64>>("p90_fee").map(|f| f as i64))
            .collect();
        all_fees.sort();
        
        let p90_idx = (all_fees.len() as f64 * 0.9).ceil() as usize;
        let p90_fee = all_fees.get(p90_idx.saturating_sub(1)).copied().unwrap_or(0);
        
        // Recommended = P90 + 20% buffer
        let recommended_fee = (p90_fee as f64 * 1.2) as i64;

        Ok(LiveFeeEstimate {
            account: pubkey.to_string(),
            queue_depth: queue_depth as u32,
            p90_fee,
            recommended_fee,
            avg_contention,
            slots_observed: rows.len(),
        })
    }
}

/// Live fee estimate for real-time prediction
#[derive(Debug, Clone)]
pub struct LiveFeeEstimate {
    pub account: String,
    pub queue_depth: u32,
    pub p90_fee: i64,
    pub recommended_fee: i64,
    pub avg_contention: f64,
    pub slots_observed: usize,
}

