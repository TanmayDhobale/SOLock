# Solana Write Lock Dashboard

Real-time monitoring for Solana write lock contention with predictive fee estimation.

## Demo


https://github.com/user-attachments/assets/ac4175b9-7f3b-4d56-9db9-ab7489729112



## Overview

Tracks write lock contention on Solana mainnet, showing which accounts have the highest contention and recommending priority fees.

```
Solana RPC → Indexer → TimescaleDB → API → Dashboard
```

## Features

- **Real-time contention tracking** - 100ms polling, 10-slot sliding window
- **Predictive fee estimation** - P90 of recent fees + 20% buffer
- **Queue depth per account** - How many txs competing for same account
- **Live dashboard** - Updates every 5 seconds via WebSocket

## Requirements

- Rust 1.75+
- Node.js 20+
- Docker

## Quick Start

```bash
# 1. Start database
docker-compose up -d

# 2. Run indexer (terminal 1)
SOLANA_RPC_URL="https://api.mainnet-beta.solana.com" \
DATABASE_URL="postgresql://solana:solana_dev_password@localhost:5432/solana_locks" \
cargo run --release --bin indexer

# 3. Run API (terminal 2)
DATABASE_URL="postgresql://solana:solana_dev_password@localhost:5432/solana_locks" \
cargo run --release --bin api

# 4. Run dashboard (terminal 3)
cd dashboard && npm install && npm run dev
```

Open http://localhost:3000

## API

```bash
# Hot accounts (last 5 minutes)
GET /api/hot-accounts?limit=20&window=5

# Dashboard stats
GET /api/stats?window=5

# Account details
GET /api/accounts/:pubkey/stats?window=24

# LIVE fee estimate (P90 + 20% buffer)
GET /api/accounts/:pubkey/fee-now

# Priority fee estimate (historical P75)
POST /api/priority-fees/estimate
{"accounts": ["pubkey1", "pubkey2"]}
```

WebSocket available at `/ws` for real-time updates.

## Limitations

**This is based on landed blocks, not the mempool.** True real-time fee prediction would require:
- Geyser plugin for pending transactions
- TPU/QUIC stream for mempool visibility

Current approach is "very recent history" (~30 seconds), not "what's pending right now."

## Roadmap

- [ ] Geyser plugin integration for true mempool visibility
- [ ] Per-slot streaming (vs batch inserts)
- [ ] Program ID → Account label mapping
- [ ] Instruction-level write set parsing

## Structure

```
indexer/     # Rust - indexes Solana transactions (100ms polling)
api/         # Rust - REST + WebSocket API
dashboard/   # Next.js - frontend
migrations/  # SQL schema
```

## License

MIT
