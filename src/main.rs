use anyhow::Result;
use coinbase_scraper::{scrape_coins_concurrent, Database};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    let pages = 20;
    
    // Initialize database
    println!("=== Database Setup ===");
    let db = Database::new("sqlite:cmc.db").await?;
    
    // Scrape data concurrently
    println!("\n=== Scraping Data ===");
    println!("Scraping {} pages concurrently...", pages);
    let start = Instant::now();
    let rows = scrape_coins_concurrent(pages, 10, 300).await?;
    let scrape_duration = start.elapsed();
    
    println!("Scraped {} coins in {:.2}s", rows.len(), scrape_duration.as_secs_f64());
    
    // Save to database
    println!("\n=== Database Storage ===");
    let start = Instant::now();
    db.save_coins(&rows).await?;
    let db_duration = start.elapsed();
    
    println!("Saved {} coins to database in {:.3}s", rows.len(), db_duration.as_secs_f64());
    
    // Get database stats
    let total_snapshots = db.get_snapshot_count().await?;
    println!("Total snapshots in database: {}", total_snapshots);
    
    // Demo: Show latest top 10 from database
    println!("\n=== Latest Top 10 from Database ===");
    let latest_coins = db.get_latest_coins(10).await?;
    for (i, coin) in latest_coins.iter().enumerate() {
        println!("{:>2}. {} ({}) - ${:.2} ({:+.2}%)", 
                 i + 1,
                 coin.name, 
                 coin.symbol,
                 coin.price_usd.unwrap_or(0.0),
                 coin.change_24h.unwrap_or(0.0));
    }
    
    // Demo: Show BTC history if available
    println!("\n=== BTC Price History ===");
    let btc_history = db.get_coin_history("BTC").await?;
    for point in btc_history.iter().take(5) {
        println!("{}: ${:.2}", 
                 point.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
                 point.price_usd.unwrap_or(0.0));
    }
    if btc_history.len() > 5 {
        println!("... and {} more data points", btc_history.len() - 5);
    }

    println!("{:<5} {:<18} {:<8} {:>14} {:>16} {:>10}", "Rank", "Name", "Symbol", "Price(USD)", "MktCap(USD)", "24h%");
    println!("{}", "-".repeat(80));
    for r in &rows {
        println!(
            "{:<5} {:<18} {:<8} {:>14} {:>16} {:>10}",
            r.rank.map_or("-".into(), |x| x.to_string()),
            r.name,
            r.symbol,
            r.price_usd.map(|x| format!("{:.2}", x)).unwrap_or_else(|| "-".into()),
            r.market_cap_usd.map(|x| format!("{:.0}", x)).unwrap_or_else(|| "-".into()),
            r.chg24h_pct.map(|x| format!("{:+.2}", x)).unwrap_or_else(|| "-".into()),
        );
    }

    Ok(())
}
