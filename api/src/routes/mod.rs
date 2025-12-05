use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};

use crate::database::Database;

// GET /api/hot-accounts?limit=20&window=5
#[derive(Debug, Deserialize)]
pub struct HotAccountsQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    
    #[serde(default = "default_window")]
    pub window: i64,  // minutes
}

fn default_limit() -> i64 {
    20
}

fn default_window() -> i64 {
    5
}

#[derive(Debug, Serialize)]
pub struct HotAccountResponse {
    pub account_pubkey: String,
    pub lock_attempts: i64,
    pub successful_locks: i64,
    pub success_rate: f64,
    pub avg_contention: f64,
    pub max_contention: f64,
    pub avg_priority_fee: i64,
    pub max_priority_fee: i64,
}

pub async fn hot_accounts(
    Query(params): Query<HotAccountsQuery>,
    State(db): State<Database>,
) -> Result<Json<Vec<HotAccountResponse>>, StatusCode> {
    let accounts = db
        .get_hot_accounts(params.limit, params.window)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let response: Vec<HotAccountResponse> = accounts
        .into_iter()
        .map(|acc| HotAccountResponse {
            account_pubkey: acc.account_pubkey,
            lock_attempts: acc.lock_attempts,
            successful_locks: acc.successful_locks,
            success_rate: if acc.lock_attempts > 0 {
                (acc.successful_locks as f64 / acc.lock_attempts as f64) * 100.0
            } else {
                0.0
            },
            avg_contention: acc.avg_contention.unwrap_or(0.0),
            max_contention: acc.max_contention.unwrap_or(0.0),
            avg_priority_fee: acc.avg_priority_fee.unwrap_or(0.0) as i64,
            max_priority_fee: acc.max_priority_fee.unwrap_or(0),
        })
        .collect();

    Ok(Json(response))
}

// GET /api/stats?window=5
#[derive(Debug, Serialize)]
pub struct DashboardStatsResponse {
    pub unique_accounts: i64,
    pub total_events: i64,
    pub high_contention_accounts: i64,
    pub avg_success_rate: f64,
}

pub async fn dashboard_stats(
    Query(params): Query<HotAccountsQuery>,
    State(db): State<Database>,
) -> Result<Json<DashboardStatsResponse>, StatusCode> {
    let stats = db
        .get_dashboard_stats(params.window)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(DashboardStatsResponse {
        unique_accounts: stats.unique_accounts,
        total_events: stats.total_events,
        high_contention_accounts: stats.high_contention_accounts,
        avg_success_rate: stats.avg_success_rate,
    }))
}

// GET /api/accounts/:pubkey/stats?window=24
#[derive(Debug, Deserialize)]
pub struct AccountStatsQuery {
    #[serde(default = "default_stats_window")]
    pub window: i64,  // hours
}

fn default_stats_window() -> i64 {
    24
}

#[derive(Debug, Serialize)]
pub struct AccountStatsResponse {
    pub pubkey: String,
    pub total_lock_attempts: i64,
    pub successful_locks: i64,
    pub failed_locks: i64,
    pub success_rate: f64,
    pub avg_contention: f64,
    pub avg_priority_fee: i64,
    pub max_priority_fee: i64,
}

pub async fn account_stats(
    Path(pubkey): Path<String>,
    Query(params): Query<AccountStatsQuery>,
    State(db): State<Database>,
) -> Result<Json<AccountStatsResponse>, StatusCode> {
    let stats = db
        .get_account_stats(&pubkey, params.window)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match stats {
        Some(s) => Ok(Json(AccountStatsResponse {
            pubkey,
            total_lock_attempts: s.total_lock_attempts,
            successful_locks: s.successful_locks,
            failed_locks: s.failed_locks,
            success_rate: if s.total_lock_attempts > 0 {
                (s.successful_locks as f64 / s.total_lock_attempts as f64) * 100.0
            } else {
                0.0
            },
            avg_contention: s.avg_contention.unwrap_or(0.0),
            avg_priority_fee: s.avg_priority_fee.unwrap_or(0.0) as i64,
            max_priority_fee: s.max_priority_fee.unwrap_or(0),
        })),
        None => Err(StatusCode::NOT_FOUND),
    }
}

// POST /api/priority-fees/estimate
#[derive(Debug, Deserialize)]
pub struct PriorityFeeRequest {
    pub accounts: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct PriorityFeeResponse {
    pub recommended_fee_lamports: i64,
    pub recommended_fee_sol: f64,
}

pub async fn estimate_priority_fee(
    State(db): State<Database>,
    Json(payload): Json<PriorityFeeRequest>,
) -> Result<Json<PriorityFeeResponse>, StatusCode> {
    if payload.accounts.is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let fee = db
        .get_recommended_priority_fee(&payload.accounts)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(PriorityFeeResponse {
        recommended_fee_lamports: fee,
        recommended_fee_sol: fee as f64 / 1_000_000_000.0,
    }))
}
