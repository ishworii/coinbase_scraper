use anyhow::Result;
use coinbase_scraper::{scrape_coins, scrape_coins_concurrent, save_to_csv, generate_filename};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<()> {
    let pages = 20;
    
    // Test sequential
    println!("=== Sequential Performance Test ===");
    println!("Scraping {} pages sequentially...", pages);
    let start = Instant::now();
    let sequential_rows = scrape_coins(pages).await?;
    let sequential_duration = start.elapsed();
    
    println!("Sequential: {:.2}s for {} coins", 
             sequential_duration.as_secs_f64(), 
             sequential_rows.len());
    
    // Test concurrent with batch size 10, 300ms pause (more aggressive)
    println!("\n=== Concurrent Performance Test ===");
    println!("Scraping {} pages concurrently (batch_size=10, pause=300ms)...", pages);
    let start = Instant::now();
    let concurrent_rows = scrape_coins_concurrent(pages, 10, 300).await?;
    let concurrent_duration = start.elapsed();
    
    println!("Concurrent: {:.2}s for {} coins", 
             concurrent_duration.as_secs_f64(), 
             concurrent_rows.len());
    
    // Performance comparison
    let speedup = sequential_duration.as_secs_f64() / concurrent_duration.as_secs_f64();
    println!("\nSpeedup: {:.2}x faster", speedup);
    
    // Use the concurrent results for display and CSV
    let rows = concurrent_rows;

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

    let filename = generate_filename();
    save_to_csv(&rows, &filename)?;
    println!("\nData saved to: {}", filename);

    Ok(())
}
