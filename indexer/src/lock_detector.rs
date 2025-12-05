use std::collections::HashMap;
use solana_sdk::pubkey::Pubkey;

pub struct LockDetector {
    write_attempts: HashMap<String, Vec<String>>,
}

impl LockDetector {
    pub fn new() -> Self {
        Self {
            write_attempts: HashMap::new(),
        }
    }

    /// Track a transaction's writable accounts
    pub fn track_transaction(&mut self, signature: &str, writable_accounts: &[Pubkey]) {
        for account in writable_accounts {
            let account_key = account.to_string();
            self.write_attempts
                .entry(account_key)
                .or_default()
                .push(signature.to_string());
        }
    }

    pub fn calculate_contention(&self, account: &str) -> f32 {
        self.write_attempts
            .get(account)
            .map(|txs| txs.len() as f32)
            .unwrap_or(1.0)
    }
    /// Get all accounts with contention (more than 1 transaction)
    #[allow(dead_code)]
    pub fn get_contended_accounts(&self) -> Vec<(String, f32)> {
        self.write_attempts
            .iter()
            .filter(|(_, txs)| txs.len() > 1)
            .map(|(account, txs)| (account.clone(), txs.len() as f32))
            .collect()
    }

    /// Clear state for next slot
    #[allow(dead_code)]
    pub fn reset(&mut self) {
        self.write_attempts.clear();
    }

    /// Check if an account is "hot" (high contention)
    #[allow(dead_code)]
    pub fn is_hot_account(&self, account: &str, threshold: f32) -> bool {
        self.calculate_contention(account) >= threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_sdk::pubkey::Pubkey;

    #[test]
    fn test_simple_contention() {
        let mut detector = LockDetector::new();
        let account = Pubkey::new_unique();
        
        detector.track_transaction("tx1", &[account]);
        detector.track_transaction("tx2", &[account]);
        detector.track_transaction("tx3", &[account]);
        
        assert_eq!(detector.calculate_contention(&account.to_string()), 3.0);
        assert!(detector.is_hot_account(&account.to_string(), 2.0));
    }

    #[test]
    fn test_no_contention() {
        let mut detector = LockDetector::new();
        let account = Pubkey::new_unique();
        
        detector.track_transaction("tx1", &[account]);
        
        assert_eq!(detector.calculate_contention(&account.to_string()), 1.0);
        assert!(!detector.is_hot_account(&account.to_string(), 2.0));
    }
}
