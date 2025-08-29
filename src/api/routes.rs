use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json, response::Result,
};
use serde::Deserialize;
use std::collections::HashMap;

use crate::api::SharedDatabase;
use crate::model::{HealthResponse, CoinResponse, HistoryResponse};

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse { ok: true })
}

#[derive(Debug, Deserialize)]
pub struct CoinsQuery {
    limit: Option<u32>,
}

pub async fn get_coins(
    Query(params): Query<CoinsQuery>,
    State(db): State<SharedDatabase>
) -> Result<Json<Vec<CoinResponse>>, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(100).min(500) as i64;
    
    match db.get_latest_coins_api(limit).await {
        Ok(coins) => Ok(Json(coins)),
        Err(err) => {
            tracing::error!("Failed to get latest coins: {}", err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()))
        }
    }
}

pub async fn get_coin_latest(
    Path(symbol): Path<String>,
    State(db): State<SharedDatabase>
) -> Result<Json<CoinResponse>, (StatusCode, String)> {
    match db.get_coin_latest_api(&symbol).await {
        Ok(Some(coin)) => Ok(Json(coin)),
        Ok(None) => Err((StatusCode::NOT_FOUND, "Symbol not found".to_string())),
        Err(err) => {
            tracing::error!("Failed to get coin {}: {}", symbol, err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()))
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct HistoryQuery {
    since: Option<String>,
    limit: Option<u32>,
}

pub async fn get_coin_history(
    Path(symbol): Path<String>,
    Query(params): Query<HistoryQuery>,
    State(db): State<SharedDatabase>
) -> Result<Json<HistoryResponse>, (StatusCode, String)> {
    let limit = params.limit.unwrap_or(500).min(2000);
    
    // For now, just get all history (ignore 'since' parameter for simplicity)
    match db.get_coin_history(&symbol).await {
        Ok(history) => {
            let series: Vec<_> = history.into_iter()
                .take(limit as usize)
                .map(|point| (point.timestamp, point.price_usd))
                .collect();
                
            Ok(Json(HistoryResponse {
                symbol: symbol.to_uppercase(),
                series,
            }))
        },
        Err(err) => {
            tracing::error!("Failed to get history for {}: {}", symbol, err);
            Err((StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()))
        }
    }
}