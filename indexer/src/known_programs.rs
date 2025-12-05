use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Known Solana program IDs → human-readable labels
pub static KNOWN_PROGRAMS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    // DEX / AMM Programs
    m.insert("675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8", "Raydium AMM");
    m.insert("CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK", "Raydium CLMM");
    m.insert("routeUGWgWzqBWFcrCfv8tritsqukccJPu3q5GPP3xS", "Raydium Route");
    m.insert("whirLbMiicVdio4qvUfM5KAg6Ct8VwpYC1LDsqNuNmM", "Orca Whirlpool");
    m.insert("9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP", "Orca Swap V2");
    m.insert("JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4", "Jupiter V6");
    m.insert("JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB", "Jupiter V4");
    m.insert("PhoeNiXZ8ByJGLkxNfZRnkUfjvmuYqLR89jjFHGqdXY", "Phoenix DEX");
    m.insert("LBUZKhRxPF3XUpBCjp4YzTKgLccjZhTSDM9YuVaPwxo", "Meteora DLMM");
    m.insert("Eo7WjKq67rjJQSZxS6z3YkapzY3eMj6Xy8X5EQVn5UG", "Meteora Pools");
    m.insert("srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX", "Serum DEX V3");
    m.insert("opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb", "Openbook V2");
    
    // Lending / Borrowing
    m.insert("So1endDq2YkqhipRh3WViPa8hdiSpxWy6z3Z6tMCpAo", "Solend");
    m.insert("MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA", "Marginfi");
    m.insert("KLend2g3cP87ber41yPrLSQn3UNsXM3x4vjGj8AvH7p", "Kamino Lend");
    m.insert("DjVE6JNiYqPL2QXyCUUh8rNjHrbz9hXHNYt99MQ59qw1", "Drift");
    
    // Staking / Liquid Staking
    m.insert("MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD", "Marinade");
    m.insert("SPo1eCN1YNa4YhkYDgG9sP1xFYT8p3YYryNVvPbja71", "Jito Stake Pool");
    m.insert("LST8uQcJ8uKhRxrAKq4pEjTxrJS2a3eVv2zXwQVxonr", "Sanctum LST");
    
    // NFT / Metaplex
    m.insert("metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s", "Metaplex Token Metadata");
    m.insert("M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K", "Magic Eden V2");
    m.insert("TSWAPaqyCSx2KABk68Shruf4rp7CxcNi8hAsbdwmHbN", "Tensor Swap");
    m.insert("TCMPhJdwDryooaGtiocG1u3xcYbRpiJzb283XfCZsDp", "Tensor Compressed");
    
    // Infrastructure
    m.insert("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA", "Token Program");
    m.insert("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb", "Token-2022");
    m.insert("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL", "Associated Token");
    m.insert("11111111111111111111111111111111", "System Program");
    m.insert("ComputeBudget111111111111111111111111111111", "Compute Budget");
    m.insert("memo1UhkJRfHyvLMcVucJwxXeuD728EqVDDwQDxFMNo", "Memo");
    
    // Oracles
    m.insert("FsJ3A3u2vn5cTVofAjvy6y5kwABJAqYWpe4975bi2epH", "Pyth Oracle");
    m.insert("SW1TCH7qEPTdLsDHRgPuMQjbQxKdH2aBStViMFnt64f", "Switchboard V2");
    
    // Governance
    m.insert("GovER5Lthms3bLBqWub97yVrMmEogzX7xNjdXpPPCVZw", "Realms Governance");
    m.insert("jdaoMN6xD3oSJz4VtCNqUhYHxsDK6EvPeQi14ZFKBuR", "Jupiter DAO");
    
    m
});

/// Get label for a program ID
pub fn get_program_label(program_id: &str) -> Option<&'static str> {
    KNOWN_PROGRAMS.get(program_id).copied()
}

/// Check if account is a known program
pub fn is_known_program(pubkey: &str) -> bool {
    KNOWN_PROGRAMS.contains_key(pubkey)
}

/// Known high-contention accounts (popular pools, vaults, etc.)
pub static KNOWN_ACCOUNTS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut m = HashMap::new();
    
    // Raydium SOL-USDC pool
    m.insert("58oQChx4yWmvKdwLLZzBi4ChoCc2fqCUWBkwMihLYQo2", "Raydium SOL-USDC");
    m.insert("8sLbNZoA1cfnvMJLPfp98ZLAnFSYCFApfJKMbiXNLwxj", "Raydium SOL-USDC AMM");
    
    // Jupiter aggregator accounts
    m.insert("JUPjXmP6pxXbcFqWLt2cxLPPvNhqDqXYX9LMqM16TgP", "Jupiter Fee Account");
    
    // Orca Whirlpool popular pools
    m.insert("7qbRF6YsyGuLUVs6Y1q64bdVrfe4ZcUUz1JRdoVNUJnm", "Orca SOL-USDC Whirlpool");
    
    m
});

/// Get label for a known account
pub fn get_account_label(pubkey: &str) -> Option<&'static str> {
    KNOWN_ACCOUNTS.get(pubkey).copied()
}
