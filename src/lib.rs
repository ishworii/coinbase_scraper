pub mod model;
pub mod fetch;
pub mod export;

pub use model::CoinRow;
pub use fetch::scrape_coins;
pub use export::{save_to_csv, append_to_csv, generate_filename};