use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CoinRow {
    pub id: u64,
    pub rank: Option<u64>,
    pub name: String,
    pub symbol: String,
    pub price_usd: Option<f64>,
    pub market_cap_usd: Option<f64>,
    pub chg24h_pct: Option<f64>,
    pub scraped_at: DateTime<Utc>,
}