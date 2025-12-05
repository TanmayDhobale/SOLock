-- Enable TimescaleDB extension
CREATE EXTENSION IF NOT EXISTS timescaledb;

-- Main write lock events table
CREATE TABLE write_lock_events (
    time TIMESTAMPTZ NOT NULL,
    slot BIGINT NOT NULL,
    
    -- Account information
    account_pubkey TEXT NOT NULL,
    program_id TEXT,
    
    -- Transaction details
    transaction_signature TEXT NOT NULL,
    success BOOLEAN NOT NULL,
    
    -- Contention metrics
    lock_contention_score REAL NOT NULL,  -- Number of concurrent write attempts
    priority_fee_lamports BIGINT,
    compute_units_consumed INTEGER,
    
    -- Metadata
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Convert to hypertable (TimescaleDB magic for time-series optimization)
SELECT create_hypertable('write_lock_events', 'time');

-- Indexes for fast queries
CREATE INDEX idx_account_time ON write_lock_events (account_pubkey, time DESC);
CREATE INDEX idx_slot ON write_lock_events (slot);
CREATE INDEX idx_signature ON write_lock_events (transaction_signature);

-- Continuous aggregate: 5-minute hot accounts
-- This is like a materialized view that auto-updates
CREATE MATERIALIZED VIEW hot_accounts_5min
WITH (timescaledb.continuous) AS
SELECT
    time_bucket('5 minutes', time) AS bucket,
    account_pubkey,
    COUNT(*) AS lock_attempts,
    SUM(CASE WHEN success THEN 1 ELSE 0 END) AS successful_locks,
    AVG(priority_fee_lamports) AS avg_priority_fee,
    MAX(priority_fee_lamports) AS max_priority_fee,
    AVG(lock_contention_score) AS avg_contention,
    MAX(lock_contention_score) AS max_contention
FROM write_lock_events
GROUP BY bucket, account_pubkey
WITH NO DATA;

-- Refresh policy: update continuous aggregate every 1 minute
SELECT add_continuous_aggregate_policy('hot_accounts_5min',
    start_offset => INTERVAL '10 minutes',
    end_offset => INTERVAL '1 minute',
    schedule_interval => INTERVAL '1 minute');

-- Retention policy: keep raw data for 7 days
-- This auto-deletes old data to save space
SELECT add_retention_policy('write_lock_events', INTERVAL '7 days');

-- Account metadata table (for caching known accounts)
CREATE TABLE account_metadata (
    pubkey TEXT PRIMARY KEY,
    program_id TEXT,
    label TEXT,  -- e.g., "Jupiter Program", "USDC Token"
    is_monitored BOOLEAN DEFAULT false,
    first_seen TIMESTAMPTZ DEFAULT NOW(),
    last_seen TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_account_label ON account_metadata (label);
CREATE INDEX idx_monitored ON account_metadata (is_monitored) WHERE is_monitored = true;

-- Insert some well-known hot accounts for demo
INSERT INTO account_metadata (pubkey, label, is_monitored) VALUES
    ('EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v', 'USDC Token Mint', true),
    ('So11111111111111111111111111111111111111112', 'Wrapped SOL', true),
    ('JUP6LkbZbjS1jKKwapdHNy74zcZ3tLUZoi5QNyVTaV4', 'Jupiter Aggregator v6', true);
