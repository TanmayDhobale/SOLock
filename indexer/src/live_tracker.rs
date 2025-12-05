use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

/// Live contention tracker - keeps last N slots in memory for real-time queries
#[derive(Clone)]
pub struct LiveTracker {
    /// Account -> (slot, contention_score, priority_fees)
    state: Arc<RwLock<HashMap<String, AccountLiveState>>>,
    /// How many slots to keep in memory
    window_size: usize,
}

#[derive(Debug, Clone)]
pub struct AccountLiveState {
    /// Recent slot data: (slot, contention_score, avg_priority_fee)
    pub recent_slots: Vec<SlotData>,
    /// Last updated timestamp
    pub last_seen: DateTime<Utc>,
}

#[derive(Debug, Clone)]
pub struct SlotData {
    pub slot: u64,
    pub contention_score: f32,
    pub tx_count: u32,
    pub avg_priority_fee: i64,
    pub max_priority_fee: i64,
}

#[derive(Debug, Clone)]
pub struct LiveFeeEstimate {
    pub account: String,
    pub queue_depth: u32,           // How many txs in recent slots
    pub p90_fee: i64,               // 90th percentile of recent fees
    pub recommended_fee: i64,       // P90 + 20% buffer
    pub avg_contention: f32,
    pub slots_observed: usize,
}

impl LiveTracker {
    pub fn new(window_size: usize) -> Self {
        Self {
            state: Arc::new(RwLock::new(HashMap::new())),
            window_size,
        }
    }

    /// Record contention data for a slot
    pub async fn record_slot(
        &self,
        account: &str,
        slot: u64,
        contention_score: f32,
        tx_count: u32,
        avg_priority_fee: i64,
        max_priority_fee: i64,
    ) {
        let mut state = self.state.write().await;
        
        let entry = state.entry(account.to_string()).or_insert_with(|| AccountLiveState {
            recent_slots: Vec::new(),
            last_seen: Utc::now(),
        });

        // Add new slot data
        entry.recent_slots.push(SlotData {
            slot,
            contention_score,
            tx_count,
            avg_priority_fee,
            max_priority_fee,
        });

        // Keep only last N slots
        if entry.recent_slots.len() > self.window_size {
            entry.recent_slots.remove(0);
        }

        entry.last_seen = Utc::now();
    }

    /// Get live fee estimate for an account
    pub async fn get_live_estimate(&self, account: &str) -> Option<LiveFeeEstimate> {
        let state = self.state.read().await;
        
        let entry = state.get(account)?;
        if entry.recent_slots.is_empty() {
            return None;
        }

        // Calculate queue depth (total txs in window)
        let queue_depth: u32 = entry.recent_slots.iter().map(|s| s.tx_count).sum();

        // Calculate P90 of priority fees
        let mut fees: Vec<i64> = entry.recent_slots.iter()
            .map(|s| s.max_priority_fee)
            .collect();
        fees.sort();
        
        let p90_idx = (fees.len() as f64 * 0.9).ceil() as usize;
        let p90_fee = fees.get(p90_idx.saturating_sub(1)).copied().unwrap_or(0);
        
        // Recommended = P90 + 20% buffer
        let recommended_fee = (p90_fee as f64 * 1.2) as i64;

        // Average contention
        let avg_contention = entry.recent_slots.iter()
            .map(|s| s.contention_score)
            .sum::<f32>() / entry.recent_slots.len() as f32;

        Some(LiveFeeEstimate {
            account: account.to_string(),
            queue_depth,
            p90_fee,
            recommended_fee,
            avg_contention,
            slots_observed: entry.recent_slots.len(),
        })
    }

    /// Get all hot accounts (sorted by contention)
    pub async fn get_hot_accounts(&self, limit: usize) -> Vec<LiveFeeEstimate> {
        let state = self.state.read().await;
        
        let mut estimates: Vec<LiveFeeEstimate> = state.keys()
            .filter_map(|account| {
                let entry = state.get(account)?;
                if entry.recent_slots.is_empty() {
                    return None;
                }

                let queue_depth: u32 = entry.recent_slots.iter().map(|s| s.tx_count).sum();
                let mut fees: Vec<i64> = entry.recent_slots.iter()
                    .map(|s| s.max_priority_fee)
                    .collect();
                fees.sort();
                
                let p90_idx = (fees.len() as f64 * 0.9).ceil() as usize;
                let p90_fee = fees.get(p90_idx.saturating_sub(1)).copied().unwrap_or(0);
                let recommended_fee = (p90_fee as f64 * 1.2) as i64;
                let avg_contention = entry.recent_slots.iter()
                    .map(|s| s.contention_score)
                    .sum::<f32>() / entry.recent_slots.len() as f32;

                Some(LiveFeeEstimate {
                    account: account.clone(),
                    queue_depth,
                    p90_fee,
                    recommended_fee,
                    avg_contention,
                    slots_observed: entry.recent_slots.len(),
                })
            })
            .collect();

        // Sort by contention (highest first)
        estimates.sort_by(|a, b| b.avg_contention.partial_cmp(&a.avg_contention).unwrap());
        estimates.truncate(limit);
        estimates
    }

    /// Clean stale accounts (not seen in last minute)
    pub async fn cleanup_stale(&self) {
        let mut state = self.state.write().await;
        let cutoff = Utc::now() - chrono::Duration::seconds(60);
        state.retain(|_, v| v.last_seen > cutoff);
    }
}
