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

// API Response DTOs
#[derive(Debug, Serialize)]
pub struct CoinResponse {
    pub id: u64,
    pub symbol: String,
    pub name: String,
    pub rank: Option<u64>,
    pub price_usd: Option<f64>,
    pub market_cap_usd: Option<f64>,
    pub change_24h: Option<f64>,
    pub ts_utc: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub symbol: String,
    pub series: Vec<(DateTime<Utc>, Option<f64>)>,
}

#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub ok: bool,
}