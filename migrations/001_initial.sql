-- Coins reference table (static info)
CREATE TABLE IF NOT EXISTS coins (
    id INTEGER PRIMARY KEY,              -- CMC ID
    name TEXT NOT NULL,                  -- Bitcoin
    symbol TEXT NOT NULL,                -- BTC
    UNIQUE(id)
);

-- Snapshots table (time series data)
CREATE TABLE IF NOT EXISTS snapshots (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    coin_id INTEGER NOT NULL,            -- FK to coins.id
    ts_utc TEXT NOT NULL,                -- ISO 8601 timestamp
    cmc_rank INTEGER,                    -- NULL if unranked
    price_usd REAL,                      -- NULL if no price
    market_cap_usd REAL,                 -- NULL if no market cap
    change_24h REAL,                     -- NULL if no change data
    FOREIGN KEY (coin_id) REFERENCES coins (id)
);

-- Indexes for common queries
CREATE INDEX IF NOT EXISTS idx_snapshots_coin_id ON snapshots(coin_id);
CREATE INDEX IF NOT EXISTS idx_snapshots_ts_utc ON snapshots(ts_utc);
CREATE INDEX IF NOT EXISTS idx_snapshots_coin_ts ON snapshots(coin_id, ts_utc);