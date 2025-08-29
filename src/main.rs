use anyhow::Result;
use coinbase_scraper::{scrape_coins_concurrent, Database, start_server};
use std::time::Instant;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "coinbase_scraper")]
#[command(about = "A high-performance cryptocurrency data scraper with database storage and REST API")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scrape cryptocurrency data and save to database
    Scrape {
        /// Number of pages to scrape
        #[arg(short, long, default_value_t = 10)]
        pages: u32,
        /// Database path
        #[arg(short, long, default_value = "sqlite:cmc.db")]
        db: String,
    },
    /// Start the REST API server
    Serve {
        /// Port to run the server on
        #[arg(short, long, default_value_t = 8080)]
        port: u16,
        /// Database path
        #[arg(short, long, default_value = "sqlite:cmc.db")]
        db: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();

    match cli.command {
        Commands::Scrape { pages, db } => {
            scrape_command(pages, &db).await?;
        },
        Commands::Serve { port, db } => {
            serve_command(port, &db).await?;
        },
    }

    Ok(())
}

async fn scrape_command(pages: u32, db_url: &str) -> Result<()> {
    // Initialize database
    println!("=== Database Setup ===");
    let db = Database::new(db_url).await?;
    
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

    Ok(())
}

async fn serve_command(port: u16, db_url: &str) -> Result<()> {
    println!("=== Starting API Server ===");
    let db = Database::new(db_url).await?;
    
    println!("Database connected. Starting server...");
    start_server(db, port).await?;
    
    Ok(())
}
