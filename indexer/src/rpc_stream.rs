use anyhow::{Context, Result};
use chrono::Utc;
use solana_client::rpc_client::RpcClient;
use solana_sdk::{
    commitment_config::CommitmentConfig,
};
use solana_transaction_status::{UiTransactionEncoding, EncodedTransactionWithStatusMeta};
use tracing::{info, warn};

use crate::database::{Database, WriteLockEvent};
use crate::lock_detector::LockDetector;

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

    pub async fn process_slot(&mut self, database: &Database) -> Result<()> {
        let endpoint = self.rpc_endpoint.clone();
        
        // Get current slot (blocking call wrapped in spawn_blocking)
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
            let slots_to_process = current_slot - self.last_processed_slot;
            if slots_to_process > 10 {
                info!("📦 Processing {} slots (from {} to {})", slots_to_process, self.last_processed_slot + 1, current_slot);
            }
            
            for slot in (self.last_processed_slot + 1)..=current_slot {
                match self.process_single_slot(slot, database).await {
                    Ok(num_events) => {
                        if num_events > 0 {
                            info!("✅ Slot {} processed: {} write lock events recorded", slot, num_events);
                        }
                    }
                    Err(e) => {
                        // Only log some errors - skipped slots are normal
                        let err_str = e.to_string();
                        if !err_str.contains("was skipped") && !err_str.contains("SlotSkipped") {
                            warn!("⚠️ Error processing slot {}: {}", slot, e);
                        }
                    }
                }
                self.last_processed_slot = slot;
            }
        }

        // Small delay to avoid hammering the RPC
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        Ok(())
    }

    async fn process_single_slot(&self, slot: u64, database: &Database) -> Result<usize> {
        let endpoint = self.rpc_endpoint.clone();
        
        // Get block with transaction details (blocking call wrapped in spawn_blocking)
        let block = match tokio::task::spawn_blocking(move || {
            let client = RpcClient::new_with_commitment(endpoint, CommitmentConfig::confirmed());
            client.get_block_with_config(
                slot,
                solana_client::rpc_config::RpcBlockConfig {
                    encoding: Some(UiTransactionEncoding::Base64),  // Use Base64 for decode() compatibility
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
                // Slot might not have a block (skipped slot), that's normal
                return Ok(0);
            }
        };

        let mut detector = LockDetector::new();
        let mut events = Vec::new();

        // Process transactions in the block
        if let Some(transactions) = block.transactions {
            let tx_count = transactions.len();
            if tx_count > 0 && slot % 50 == 0 {
                info!("🔍 Slot {} has {} transactions", slot, tx_count);
            }
            
            for tx_with_meta in &transactions {
                if let Some(transaction) = &tx_with_meta.transaction.decode() {
                    let signature = transaction.signatures[0].to_string();
                    let message = &transaction.message;

                    // Track writable accounts
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

                    // Extract priority fee and compute units from transaction
                    let priority_fee = extract_priority_fee(tx_with_meta);
                    let compute_units = extract_compute_units(tx_with_meta);
                    let success = tx_with_meta.meta.as_ref()
                        .and_then(|m| if m.status.is_ok() { Some(true) } else { None })
                        .unwrap_or(false);

                    // Create events for each writable account
                    for account in &writable_accounts {
                        let account_str = account.to_string();
                        let contention = detector.calculate_contention(&account_str);

                        let event = WriteLockEvent {
                            time: Utc::now(),
                            slot: slot as i64,
                            account_pubkey: account_str.clone(),
                            program_id: None,  // Could be extracted from instructions
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

        // Batch insert events
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

///Extract compute units from transaction metadata
fn extract_compute_units(tx: &EncodedTransactionWithStatusMeta) -> Option<i32> {
    use solana_transaction_status::option_serializer::OptionSerializer;
    tx.meta.as_ref().and_then(|m| match m.compute_units_consumed {
        OptionSerializer::Some(cu) => Some(cu as i32),
        _ => None,
    })
}
