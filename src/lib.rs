pub mod model;
pub mod fetch;
pub mod export;
pub mod database;
pub mod api;

pub use model::{CoinRow, CoinResponse, HistoryResponse, HealthResponse};
pub use fetch::{scrape_coins, scrape_coins_concurrent};
pub use export::{save_to_csv, append_to_csv, generate_filename};
pub use database::{Database, CoinSummary, HistoryPoint};
pub use api::start_server;