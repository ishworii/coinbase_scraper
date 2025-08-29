use anyhow::Result;
use coinbase_scraper::{scrape_coins, save_to_csv, generate_filename};

#[tokio::main]
async fn main() -> Result<()> {
    let pages = 10;
    let rows = scrape_coins(pages).await?;

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
