use anyhow::{Context, Result};
use chrono::Utc;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
};
use solana_transaction_status::{UiTransactionEncoding, EncodedTransactionWithStatusMeta};
use tracing::{info, warn};
use std::sync::Arc;
use std::collections::HashMap;

use crate::database::{Database, WriteLockEvent};
use crate::lock_detector::LockDetector;
use crate::live_tracker::LiveTracker;

pub struct RpcStream {
    rpc_endpoint: String,
    last_processed_slot: u64,
}

impl RpcStream {
    pub fn new(rpc_endpoint: String) -> Self {
        Self {
            rpc_endpoint,
            last_processed_slot: 0,
        }
    }

    #[allow(dead_code)]
    fn create_client(&self) -> RpcClient {
        RpcClient::new_with_commitment(
            self.rpc_endpoint.clone(),
            CommitmentConfig::confirmed(),
        )
    }

    /// Process slot with live tracking for real-time fee estimation
    pub async fn process_slot_live(
        &mut self, 
        database: &Database,
        live_tracker: &Arc<LiveTracker>,
    ) -> Result<()> {
        let endpoint = self.rpc_endpoint.clone();
        
        // Get current slot
        let current_slot = tokio::task::spawn_blocking(move || {
            let client = RpcClient::new_with_commitment(endpoint, CommitmentConfig::confirmed());
            client.get_slot()
        }).await??;

        if self.last_processed_slot == 0 {
            self.last_processed_slot = current_slot.saturating_sub(5);
            info!("🎯 Starting from slot {}", self.last_processed_slot);
        }

        // Process slots we haven't seen yet
        if current_slot > self.last_processed_slot {
            for slot in (self.last_processed_slot + 1)..=current_slot {
                match self.process_single_slot_live(slot, database, live_tracker).await {
                    Ok(num_events) => {
                        if num_events > 0 {
                            info!("✅ Slot {} processed: {} events (live)", slot, num_events);
                        }
                    }
                    Err(e) => {
                        let err_str = e.to_string();
                        if !err_str.contains("was skipped") && !err_str.contains("SlotSkipped") {
                            warn!("⚠️ Error processing slot {}: {}", slot, e);
                        }
                    }
                }
                self.last_processed_slot = slot;
            }
        }

        // Faster polling for real-time - 100ms instead of 200ms
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        Ok(())
    }

    async fn process_single_slot_live(
        &self, 
        slot: u64, 
        database: &Database,
        live_tracker: &Arc<LiveTracker>,
    ) -> Result<usize> {
        let endpoint = self.rpc_endpoint.clone();
        
        // Get block with transaction details
        let block = match tokio::task::spawn_blocking(move || {
            let client = RpcClient::new_with_commitment(endpoint, CommitmentConfig::confirmed());
            client.get_block_with_config(
                slot,
                solana_client::rpc_config::RpcBlockConfig {
                    encoding: Some(UiTransactionEncoding::Base64),
                    transaction_details: Some(
                        solana_transaction_status::TransactionDetails::Full,
                    ),
                    rewards: Some(false),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: Some(0),
                },
            )
        }).await? {
            Ok(block) => block,
            Err(_) => {
                return Ok(0);
            }
        };

        let mut detector = LockDetector::new();
        let mut events = Vec::new();
        
        // Track per-account stats for live tracker
        let mut account_stats: HashMap<String, (u32, i64, i64)> = HashMap::new(); // (count, sum_fee, max_fee)

        if let Some(transactions) = block.transactions {
            for tx_with_meta in &transactions {
                if let Some(transaction) = &tx_with_meta.transaction.decode() {
                    let signature = transaction.signatures[0].to_string();
                    let message = &transaction.message;

                    let writable_accounts: Vec<_> = message
                        .static_account_keys()
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| message.is_maybe_writable(*i))
                        .map(|(_, key)| *key)
                        .collect();

                    if !writable_accounts.is_empty() {
                        detector.track_transaction(&signature, &writable_accounts);
                    }

                    let priority_fee = extract_priority_fee(tx_with_meta);
                    let compute_units = extract_compute_units(tx_with_meta);
                    let success = tx_with_meta.meta.as_ref()
                        .and_then(|m| if m.status.is_ok() { Some(true) } else { None })
                        .unwrap_or(false);

                    // Extract program IDs from instructions
                    let program_ids: Vec<String> = message.instructions().iter()
                        .map(|ix| message.static_account_keys()[ix.program_id_index as usize].to_string())
                        .collect();

                    for account in &writable_accounts {
                        let account_str = account.to_string();
                        let contention = detector.calculate_contention(&account_str);
                        let fee = priority_fee.unwrap_or(0);

                        // Update per-account stats
                        let entry = account_stats.entry(account_str.clone()).or_insert((0, 0, 0));
                        entry.0 += 1; // tx count
                        entry.1 += fee; // sum fees
                        entry.2 = entry.2.max(fee); // max fee

                        // Find first non-system program ID (more interesting)
                        let program_id = program_ids.iter()
                            .find(|p| *p != "11111111111111111111111111111111" && 
                                      *p != "ComputeBudget111111111111111111111111111111")
                            .cloned();

                        let event = WriteLockEvent {
                            time: Utc::now(),
                            slot: slot as i64,
                            account_pubkey: account_str,
                            program_id,
                            transaction_signature: signature.clone(),
                            success,
                            lock_contention_score: contention,
                            priority_fee_lamports: priority_fee,
                            compute_units_consumed: compute_units,
                        };

                        events.push(event);
                    }
                }
            }
        }

        // Update live tracker with per-account contention data
        for (account, (tx_count, sum_fee, max_fee)) in account_stats {
            let avg_fee = if tx_count > 0 { sum_fee / tx_count as i64 } else { 0 };
            let contention = detector.calculate_contention(&account);
            
            live_tracker.record_slot(
                &account,
                slot,
                contention,
                tx_count,
                avg_fee,
                max_fee,
            ).await;
        }

        // Batch insert events
        if !events.is_empty() {
            info!("📝 Inserting {} events for slot {}", events.len(), slot);
            database.insert_events(&events).await?;
        }

        Ok(events.len())
    }

    #[allow(dead_code)]
    pub async fn process_slot(&mut self, database: &Database) -> Result<()> {
        let endpoint = self.rpc_endpoint.clone();
        
        let current_slot = tokio::task::spawn_blocking(move || {
            let client = RpcClient::new_with_commitment(endpoint, CommitmentConfig::confirmed());
            client.get_slot()
        }).await??;

        if self.last_processed_slot == 0 {
            self.last_processed_slot = current_slot.saturating_sub(5);
            info!("🎯 Starting from slot {}", self.last_processed_slot);
        }

        if current_slot > self.last_processed_slot {
            for slot in (self.last_processed_slot + 1)..=current_slot {
                match self.process_single_slot(slot, database).await {
                    Ok(num_events) => {
                        if num_events > 0 {
                            info!("✅ Slot {} processed: {} write lock events recorded", slot, num_events);
                        }
                    }
                    Err(e) => {
                        let err_str = e.to_string();
                        if !err_str.contains("was skipped") && !err_str.contains("SlotSkipped") {
                            warn!("⚠️ Error processing slot {}: {}", slot, e);
                        }
                    }
                }
                self.last_processed_slot = slot;
            }
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        Ok(())
    }

    #[allow(dead_code)]
    async fn process_single_slot(&self, slot: u64, database: &Database) -> Result<usize> {
        let endpoint = self.rpc_endpoint.clone();
        
        let block = match tokio::task::spawn_blocking(move || {
            let client = RpcClient::new_with_commitment(endpoint, CommitmentConfig::confirmed());
            client.get_block_with_config(
                slot,
                solana_client::rpc_config::RpcBlockConfig {
                    encoding: Some(UiTransactionEncoding::Base64),
                    transaction_details: Some(
                        solana_transaction_status::TransactionDetails::Full,
                    ),
                    rewards: Some(false),
                    commitment: Some(CommitmentConfig::confirmed()),
                    max_supported_transaction_version: Some(0),
                },
            )
        }).await? {
            Ok(block) => block,
            Err(_) => {
                return Ok(0);
            }
        };

        let mut detector = LockDetector::new();
        let mut events = Vec::new();

        if let Some(transactions) = block.transactions {
            for tx_with_meta in &transactions {
                if let Some(transaction) = &tx_with_meta.transaction.decode() {
                    let signature = transaction.signatures[0].to_string();
                    let message = &transaction.message;

                    let writable_accounts: Vec<_> = message
                        .static_account_keys()
                        .iter()
                        .enumerate()
                        .filter(|(i, _)| message.is_maybe_writable(*i))
                        .map(|(_, key)| *key)
                        .collect();

                    if !writable_accounts.is_empty() {
                        detector.track_transaction(&signature, &writable_accounts);
                    }

                    let priority_fee = extract_priority_fee(tx_with_meta);
                    let compute_units = extract_compute_units(tx_with_meta);
                    let success = tx_with_meta.meta.as_ref()
                        .and_then(|m| if m.status.is_ok() { Some(true) } else { None })
                        .unwrap_or(false);

                    for account in &writable_accounts {
                        let account_str = account.to_string();
                        let contention = detector.calculate_contention(&account_str);

                        let event = WriteLockEvent {
                            time: Utc::now(),
                            slot: slot as i64,
                            account_pubkey: account_str,
                            program_id: None,
                            transaction_signature: signature.clone(),
                            success,
                            lock_contention_score: contention,
                            priority_fee_lamports: priority_fee,
                            compute_units_consumed: compute_units,
                        };

                        events.push(event);
                    }
                }
            }
        }

        if !events.is_empty() {
            info!("📝 Inserting {} events for slot {}", events.len(), slot);
            database.insert_events(&events).await?;
        }

        Ok(events.len())
    }
}

/// Extract priority fee from transaction metadata
fn extract_priority_fee(tx: &EncodedTransactionWithStatusMeta) -> Option<i64> {
    tx.meta.as_ref().map(|m| m.fee as i64)
}

/// Extract compute units from transaction metadata
fn extract_compute_units(tx: &EncodedTransactionWithStatusMeta) -> Option<i32> {
    use solana_transaction_status::option_serializer::OptionSerializer;
    tx.meta.as_ref().and_then(|m| match m.compute_units_consumed {
        OptionSerializer::Some(cu) => Some(cu as i32),
        _ => None,
    })
}

