// Known Solana account labels for popular programs and accounts
export const KNOWN_ACCOUNTS: Record<string, { label: string; category: string; color: string }> = {
    // Raydium
    "675kPX9MHTjS2zt1qfr1NYHuzeLXfQM9H24wFSUt1Mp8": { label: "Raydium AMM", category: "DEX", color: "#9945FF" },
    "5Q544fKrFoe6tsEbD7S8EmxGTJYAKtTVhAW5Q5pge4j1": { label: "Raydium CLMM", category: "DEX", color: "#9945FF" },
    "CAMMCzo5YL8w4VFF8KVHrK22GGUsp5VTaW7grrKgrWqK": { label: "Raydium CPMM", category: "DEX", color: "#9945FF" },

    // Jupiter
    "JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4": { label: "Jupiter v6", category: "DEX Aggregator", color: "#00D18C" },
    "JUP4Fb2cqiRUcaTHdrPC8h2gNsA2ETXiPDD33WcGuJB": { label: "Jupiter v4", category: "DEX Aggregator", color: "#00D18C" },
    "jupoNjAxXgZ4rjzxzPMP4oxduvQsQtZzyknqvzYNrNu": { label: "Jupiter Limit Order", category: "DEX", color: "#00D18C" },
    "DCA265Vj8a9CEuX1eb1LWRnDT7uK6q1xMipnNyatn23M": { label: "Jupiter DCA", category: "DEX", color: "#00D18C" },

    // Orca
    "whirLbMiicVdio4qvUfM5KAg6Ct8VwpYzGff3uctyCc": { label: "Orca Whirlpool", category: "DEX", color: "#00B4D8" },
    "9W959DqEETiGZocYWCQPaJ6sBmUzgfxXfqGeTEdp3aQP": { label: "Orca Legacy", category: "DEX", color: "#00B4D8" },

    // Pump.fun
    "6EF8rrecthR5Dkzon8Nwu78hRvfCKubJ14M5uBEwF6P": { label: "Pump.fun", category: "Launchpad", color: "#FF6B6B" },

    // Tensor
    "TSWAPaqyCSx2KABk68Shruf4rp7CxcNi8hAsbdwmHbN": { label: "Tensor Swap", category: "NFT", color: "#E040FB" },
    "TCMPhJdwDryooaGtiocG1u3xcYbRpiJzb283XfCZsDp": { label: "Tensor cNFT", category: "NFT", color: "#E040FB" },

    // Magic Eden
    "M2mx93ekt1fmXSVkTrUL9xVFHkmME8HTUi5Cyc5aF7K": { label: "Magic Eden v2", category: "NFT", color: "#E040FB" },

    // Marinade
    "MarBmsSgKXdrN1egZf5sqe1TMai9K1rChYNDJgjq7aD": { label: "Marinade Finance", category: "Staking", color: "#14F195" },

    // Jito
    "Jito4APyf642JPZPx3hGc6WWJ8uzSDkCGXCvQ23nhbR": { label: "Jito Tip", category: "MEV", color: "#FF9900" },

    // System Programs
    "11111111111111111111111111111111": { label: "System Program", category: "System", color: "#718096" },
    "TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA": { label: "Token Program", category: "System", color: "#718096" },
    "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb": { label: "Token-2022", category: "System", color: "#718096" },
    "ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL": { label: "Associated Token", category: "System", color: "#718096" },
    "ComputeBudget111111111111111111111111111111": { label: "Compute Budget", category: "System", color: "#718096" },

    // Vote
    "Vote111111111111111111111111111111111111111": { label: "Vote Program", category: "System", color: "#718096" },

    // Metaplex
    "metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s": { label: "Metaplex Token Metadata", category: "NFT", color: "#E040FB" },

    // Serum/OpenBook
    "srmqPvymJeFKQ4zGQed1GFppgkRHL9kaELCbyksJtPX": { label: "Serum v3", category: "DEX", color: "#00CED1" },
    "opnb2LAfJYbRMAHHvqjCwQxanZn7ReEHp1k81EohpZb": { label: "OpenBook", category: "DEX", color: "#00CED1" },

    // Lending
    "So1endDq2YkqhipRh3WViPa8hdiSpxWy6z3Z6tMCpAo": { label: "Solend", category: "Lending", color: "#FFD166" },
    "MFv2hWf31Z9kbCa1snEPYctwafyhdvnV7FZnsebVacA": { label: "Marginfi", category: "Lending", color: "#FFD166" },
    "KLend2g3cP87ber41GXWsSn1Ld3adxjAGjVvH8S4mbP": { label: "Kamino", category: "Lending", color: "#FFD166" },

    // Drift
    "dRiftyHA39MWEi3m9aunc5MzRF1JYuBsbn6VPcn33UH": { label: "Drift Protocol", category: "Perps", color: "#6366F1" },
};

// Get label info for an account, returns undefined if not found
export function getAccountLabel(pubkey: string): { label: string; category: string; color: string } | undefined {
    return KNOWN_ACCOUNTS[pubkey];
}

// Get display name (label or truncated pubkey)
export function getAccountDisplayName(pubkey: string): string {
    const known = KNOWN_ACCOUNTS[pubkey];
    return known ? known.label : `${pubkey.slice(0, 4)}...${pubkey.slice(-4)}`;
}

// Category colors for badges
export const CATEGORY_COLORS: Record<string, string> = {
    "DEX": "bg-purple-500/20 text-purple-400 border-purple-500/30",
    "DEX Aggregator": "bg-emerald-500/20 text-emerald-400 border-emerald-500/30",
    "NFT": "bg-pink-500/20 text-pink-400 border-pink-500/30",
    "Staking": "bg-green-500/20 text-green-400 border-green-500/30",
    "MEV": "bg-orange-500/20 text-orange-400 border-orange-500/30",
    "System": "bg-gray-500/20 text-gray-400 border-gray-500/30",
    "Lending": "bg-yellow-500/20 text-yellow-400 border-yellow-500/30",
    "Launchpad": "bg-red-500/20 text-red-400 border-red-500/30",
    "Perps": "bg-indigo-500/20 text-indigo-400 border-indigo-500/30",
};
