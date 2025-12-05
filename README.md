# Solana Write Lock Dashboard

Real-time monitoring for Solana write lock contention.

## Overview

Tracks write lock contention on Solana mainnet, showing which accounts have the highest contention and what priority fees are being paid.

```
Solana RPC → Indexer → TimescaleDB → API → Dashboard
```

## Requirements

- Rust 1.75+
- Node.js 20+
- Docker

## Quick Start

```bash
# 1. Start database
docker-compose up -d

# 2. Run indexer (in terminal 1)
SOLANA_RPC_URL="https://api.mainnet-beta.solana.com" \
DATABASE_URL="postgresql://solana:solana_dev_password@localhost:5432/solana_locks" \
cargo run --release --bin indexer

# 3. Run API (in terminal 2)
DATABASE_URL="postgresql://solana:solana_dev_password@localhost:5432/solana_locks" \
cargo run --release --bin api

# 4. Run dashboard (in terminal 3)
cd dashboard && npm install && npm run dev
```

Open http://localhost:3000

## Configuration

For better performance, use a paid RPC (Helius, QuickNode, Triton):

```bash
export SOLANA_RPC_URL="https://mainnet.helius-rpc.com/?api-key=YOUR_KEY"
```

## API

```bash
# Hot accounts (last 5 minutes)
GET /api/hot-accounts?limit=20&window=5

# Dashboard stats
GET /api/stats?window=5

# Account details
GET /api/accounts/:pubkey/stats?window=24

# Priority fee estimate
POST /api/priority-fees/estimate
{"accounts": ["pubkey1", "pubkey2"]}
```

WebSocket available at `/ws` for real-time updates.

## Structure

```
indexer/     # Rust - indexes Solana transactions
api/         # Rust - REST + WebSocket API
dashboard/   # Next.js - frontend
migrations/  # SQL schema
```

## License

MIT
