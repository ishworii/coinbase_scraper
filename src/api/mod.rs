pub mod routes;

use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;
use tower_http::cors::CorsLayer;

use crate::database::Database;

pub type SharedDatabase = Arc<Database>;

pub fn create_router(db: SharedDatabase) -> Router {
    Router::new()
        .route("/health", get(routes::health))
        .route("/coins", get(routes::get_coins))
        .route("/coin/:symbol/latest", get(routes::get_coin_latest))
        .route("/coin/:symbol/history", get(routes::get_coin_history))
        .with_state(db)
        .layer(CorsLayer::permissive()) // Allow all origins for dev
}

pub async fn start_server(db: Database, port: u16) -> anyhow::Result<()> {
    let db = Arc::new(db);
    let app = create_router(db);
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    
    tracing::info!("Server running on http://localhost:{}", port);
    axum::serve(listener, app).await?;
    
    Ok(())
}