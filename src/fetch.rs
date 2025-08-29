use anyhow::{anyhow, Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT, ACCEPT, ACCEPT_LANGUAGE};
use scraper::{Html, Selector};
use serde_json::Value;
use std::collections::{BTreeMap, HashSet};
use chrono::Utc;
use futures::future::join_all;
use tokio::time::{sleep, Duration};

use crate::model::CoinRow;

pub async fn scrape_coins(pages: u32) -> Result<Vec<CoinRow>> {
    let mut seen = HashSet::new();
    let mut rows = Vec::new();
    let scraped_at = Utc::now();

    for page in 1..=pages {
        let url = if page == 1 {
            "https://coinmarketcap.com/".to_string()
        } else {
            format!("https://coinmarketcap.com/?page={page}")
        };
        let html = fetch_html(&url).await?;
        let page_rows = extract_home_coins(&html, scraped_at)
            .with_context(|| format!("failed to parse page {}", page))?;
        for r in page_rows {
            if seen.insert(r.id) {
                rows.push(r);
            }
        }
    }

    rows.sort_by_key(|r| r.rank.unwrap_or(u64::MAX));
    Ok(rows)
}

pub async fn scrape_coins_concurrent(pages: u32, batch_size: u32, pause_ms: u64) -> Result<Vec<CoinRow>> {
    let mut seen = HashSet::new();
    let mut rows = Vec::new();
    let scraped_at = Utc::now();

    let page_numbers: Vec<u32> = (1..=pages).collect();
    
    for chunk in page_numbers.chunks(batch_size as usize) {
        let tasks: Vec<_> = chunk.iter()
            .map(|&page| {
                let scraped_at = scraped_at;
                tokio::spawn(async move {
                    let url = if page == 1 {
                        "https://coinmarketcap.com/".to_string()
                    } else {
                        format!("https://coinmarketcap.com/?page={page}")
                    };
                    
                    let html = fetch_html(&url).await
                        .with_context(|| format!("Failed to fetch page {}", page))?;
                    
                    extract_home_coins(&html, scraped_at)
                        .with_context(|| format!("Failed to parse page {}", page))
                })
            })
            .collect();

        let results = join_all(tasks).await;
        
        for res in results {
            match res {
                Ok(Ok(page_rows)) => {
                    for r in page_rows {
                        if seen.insert(r.id) {
                            rows.push(r);
                        }
                    }
                }
                Ok(Err(e)) => eprintln!("Scrape error: {}", e),
                Err(e) => eprintln!("Task join error: {}", e),
            }
        }

        if chunk.len() < page_numbers.len() {
            sleep(Duration::from_millis(pause_ms)).await;
        }
    }

    rows.sort_by_key(|r| r.rank.unwrap_or(u64::MAX));
    Ok(rows)
}

async fn fetch_html(url: &str) -> Result<String> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_static(
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36"));
    headers.insert(ACCEPT, HeaderValue::from_static("text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8"));
    headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .redirect(reqwest::redirect::Policy::limited(5))
        .gzip(true).brotli(true).deflate(true)
        .build()?;

    let html = client.get(url)
        .send().await?
        .error_for_status()?
        .text().await?;

    Ok(html)
}

fn extract_home_coins(html: &str, scraped_at: chrono::DateTime<chrono::Utc>) -> Result<Vec<CoinRow>> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse(r#"script#__NEXT_DATA__"#).unwrap();
    let json_text = doc.select(&sel).next()
        .and_then(|n| n.text().next())
        .ok_or_else(|| anyhow!("__NEXT_DATA__ not found"))?;

    let v: Value = serde_json::from_str(json_text)?;
    let list = locate_crypto_list(&v)?;
    let mut out = Vec::with_capacity(list.len());

    for coin in list {
        if let Some(row) = parse_coin_object(coin, scraped_at) {
            out.push(row);
        }
    }
    Ok(out)
}

fn locate_crypto_list(root: &Value) -> Result<&Vec<Value>> {
    let props = &root["props"];
    let queries = &props["dehydratedState"]["queries"];
    let arr = queries.as_array().ok_or_else(|| anyhow!("queries not array"))?;
    for q in arr {
        let path = &q["state"]["data"]["data"]["listing"]["cryptoCurrencyList"];
        if let Some(list) = path.as_array() {
            return Ok(list);
        }
    }
    Err(anyhow!("cryptoCurrencyList not found"))
}

fn parse_coin_object(v: &Value, scraped_at: chrono::DateTime<chrono::Utc>) -> Option<CoinRow> {
    let id = v.get("id").and_then(Value::as_u64)?;
    let name = v.get("name").and_then(Value::as_str)?.to_string();
    let symbol = v.get("symbol").and_then(Value::as_str)?.to_string();
    let rank = v.get("cmcRank").and_then(Value::as_u64);

    let mut price_usd = None;
    let mut mcap_usd = None;
    let mut chg24 = None;

    if let Some(qusd) = v.get("quote").and_then(|q| q.get("USD")) {
        price_usd = qusd.get("price").and_then(Value::as_f64);
        mcap_usd = qusd.get("marketCap").and_then(Value::as_f64);
        chg24 = qusd.get("percentChange24h").and_then(Value::as_f64);
    }

    if price_usd.is_none() {
        if let Some(quotes) = v.get("quotes").and_then(Value::as_array) {
            let mut map = BTreeMap::new();
            for q in quotes {
                if let Some(k) = q.get("name").and_then(Value::as_str) {
                    map.insert(k.to_string(), q);
                }
            }
            if let Some(usd) = map.get("USD") {
                price_usd = usd.get("price").and_then(Value::as_f64);
                mcap_usd  = usd.get("marketCap").and_then(Value::as_f64);
                chg24     = usd.get("percentChange24h").and_then(Value::as_f64);
            }
        }
    }

    Some(CoinRow {
        id,
        rank,
        name,
        symbol,
        price_usd,
        market_cap_usd: mcap_usd,
        chg24h_pct: chg24,
        scraped_at,
    })
}