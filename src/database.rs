use anyhow::Result;
use sqlx::{sqlite::SqlitePool, Row};
use chrono::{DateTime, Utc};

use crate::model::{CoinRow, CoinResponse};

pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self> {
        let pool = SqlitePool::connect(database_url).await?;
        
        // Create tables manually instead of migrations for now
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS coins (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL,
                symbol TEXT NOT NULL,
                UNIQUE(id)
            );
            "#
        ).execute(&pool).await?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS snapshots (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                coin_id INTEGER NOT NULL,
                ts_utc TEXT NOT NULL,
                cmc_rank INTEGER,
                price_usd REAL,
                market_cap_usd REAL,
                change_24h REAL,
                FOREIGN KEY (coin_id) REFERENCES coins (id)
            );
            "#
        ).execute(&pool).await?;
        
        // Create indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_snapshots_coin_id ON snapshots(coin_id);")
            .execute(&pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_snapshots_ts_utc ON snapshots(ts_utc);")
            .execute(&pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_snapshots_coin_ts ON snapshots(coin_id, ts_utc);")
            .execute(&pool).await?;
        
        Ok(Self { pool })
    }

    pub async fn save_coins(&self, coins: &[CoinRow]) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        for coin in coins {
            // Upsert coin (insert if not exists)
            sqlx::query("INSERT OR IGNORE INTO coins (id, name, symbol) VALUES (?, ?, ?)")
                .bind(coin.id as i64)
                .bind(&coin.name)
                .bind(&coin.symbol)
                .execute(&mut *tx)
                .await?;

            // Insert snapshot
            sqlx::query("INSERT INTO snapshots (coin_id, ts_utc, cmc_rank, price_usd, market_cap_usd, change_24h) VALUES (?, ?, ?, ?, ?, ?)")
                .bind(coin.id as i64)
                .bind(coin.scraped_at.to_rfc3339())
                .bind(coin.rank.map(|r| r as i64))
                .bind(coin.price_usd)
                .bind(coin.market_cap_usd)
                .bind(coin.chg24h_pct)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    pub async fn get_latest_coins(&self, limit: i64) -> Result<Vec<CoinSummary>> {
        let rows = sqlx::query(
            r#"
            SELECT c.name, c.symbol, s.cmc_rank, s.price_usd, s.market_cap_usd, s.change_24h
            FROM snapshots s
            JOIN coins c ON s.coin_id = c.id
            WHERE s.ts_utc = (SELECT MAX(ts_utc) FROM snapshots)
            AND s.cmc_rank IS NOT NULL
            ORDER BY s.cmc_rank ASC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut coins = Vec::new();
        for row in rows {
            coins.push(CoinSummary {
                name: row.get("name"),
                symbol: row.get("symbol"), 
                rank: row.get::<Option<i64>, _>("cmc_rank").map(|r| r as u64),
                price_usd: row.get("price_usd"),
                market_cap_usd: row.get("market_cap_usd"),
                change_24h: row.get("change_24h"),
            });
        }

        Ok(coins)
    }

    pub async fn get_latest_coins_api(&self, limit: i64) -> Result<Vec<CoinResponse>> {
        let rows = sqlx::query(
            r#"
            SELECT c.id, c.name, c.symbol, s.cmc_rank, s.price_usd, s.market_cap_usd, s.change_24h, s.ts_utc
            FROM snapshots s
            JOIN coins c ON s.coin_id = c.id
            WHERE s.ts_utc = (SELECT MAX(ts_utc) FROM snapshots)
            AND s.cmc_rank IS NOT NULL
            ORDER BY s.cmc_rank ASC
            LIMIT ?
            "#
        )
        .bind(limit)
        .fetch_all(&self.pool)
        .await?;

        let mut coins = Vec::new();
        for row in rows {
            let ts_utc: String = row.get("ts_utc");
            coins.push(CoinResponse {
                id: row.get::<i64, _>("id") as u64,
                name: row.get("name"),
                symbol: row.get("symbol"),
                rank: row.get::<Option<i64>, _>("cmc_rank").map(|r| r as u64),
                price_usd: row.get("price_usd"),
                market_cap_usd: row.get("market_cap_usd"),
                change_24h: row.get("change_24h"),
                ts_utc: DateTime::parse_from_rfc3339(&ts_utc)?.with_timezone(&Utc),
            });
        }

        Ok(coins)
    }

    pub async fn get_coin_latest_api(&self, symbol: &str) -> Result<Option<CoinResponse>> {
        let row = sqlx::query(
            r#"
            SELECT c.id, c.name, c.symbol, s.cmc_rank, s.price_usd, s.market_cap_usd, s.change_24h, s.ts_utc
            FROM snapshots s
            JOIN coins c ON s.coin_id = c.id
            WHERE c.symbol = ? AND s.ts_utc = (SELECT MAX(ts_utc) FROM snapshots WHERE coin_id = c.id)
            "#
        )
        .bind(symbol.to_uppercase())
        .fetch_optional(&self.pool)
        .await?;

        if let Some(row) = row {
            let ts_utc: String = row.get("ts_utc");
            Ok(Some(CoinResponse {
                id: row.get::<i64, _>("id") as u64,
                name: row.get("name"),
                symbol: row.get("symbol"),
                rank: row.get::<Option<i64>, _>("cmc_rank").map(|r| r as u64),
                price_usd: row.get("price_usd"),
                market_cap_usd: row.get("market_cap_usd"),
                change_24h: row.get("change_24h"),
                ts_utc: DateTime::parse_from_rfc3339(&ts_utc)?.with_timezone(&Utc),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn get_coin_history(&self, symbol: &str) -> Result<Vec<HistoryPoint>> {
        let rows = sqlx::query(
            r#"
            SELECT s.ts_utc, s.price_usd, s.market_cap_usd
            FROM snapshots s
            JOIN coins c ON s.coin_id = c.id
            WHERE c.symbol = ?
            ORDER BY s.ts_utc ASC
            "#
        )
        .bind(symbol)
        .fetch_all(&self.pool)
        .await?;

        let mut history = Vec::new();
        for row in rows {
            let ts_utc: String = row.get("ts_utc");
            history.push(HistoryPoint {
                timestamp: DateTime::parse_from_rfc3339(&ts_utc)?.with_timezone(&Utc),
                price_usd: row.get("price_usd"),
                market_cap_usd: row.get("market_cap_usd"),
            });
        }

        Ok(history)
    }

    pub async fn get_snapshot_count(&self) -> Result<i64> {
        let row = sqlx::query("SELECT COUNT(*) as count FROM snapshots")
            .fetch_one(&self.pool)
            .await?;
        
        Ok(row.get("count"))
    }
}

#[derive(Debug)]
pub struct CoinSummary {
    pub name: String,
    pub symbol: String,
    pub rank: Option<u64>,
    pub price_usd: Option<f64>,
    pub market_cap_usd: Option<f64>,
    pub change_24h: Option<f64>,
}

#[derive(Debug)]
pub struct HistoryPoint {
    pub timestamp: DateTime<Utc>,
    pub price_usd: Option<f64>,
    pub market_cap_usd: Option<f64>,
}