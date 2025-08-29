use anyhow::Result;
use chrono::Utc;
use csv::Writer;
use std::fs::OpenOptions;
use std::path::Path;

use crate::model::CoinRow;

pub fn save_to_csv<P: AsRef<Path>>(data: &[CoinRow], file_path: P) -> Result<()> {
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(file_path)?;
    
    let mut wtr = Writer::from_writer(file);
    
    for row in data {
        wtr.serialize(row)?;
    }
    
    wtr.flush()?;
    Ok(())
}

pub fn append_to_csv<P: AsRef<Path>>(data: &[CoinRow], file_path: P) -> Result<()> {
    let file_exists = file_path.as_ref().exists();
    
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .append(true)
        .open(file_path)?;
    
    let mut wtr = Writer::from_writer(file);
    
    if !file_exists {
        wtr.write_record(&["id", "rank", "name", "symbol", "price_usd", "market_cap_usd", "chg24h_pct", "scraped_at"])?;
    }
    
    for row in data {
        wtr.serialize(row)?;
    }
    
    wtr.flush()?;
    Ok(())
}

pub fn generate_filename() -> String {
    let now = Utc::now();
    format!("coinbase_data_{}.csv", now.format("%Y%m%d_%H%M%S"))
}