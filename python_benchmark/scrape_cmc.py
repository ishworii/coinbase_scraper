import asyncio, json, time, argparse
from typing import List, Dict, Any
import aiohttp
from bs4 import BeautifulSoup

HEADERS = {
    "User-Agent": ("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 "
                   "(KHTML, like Gecko) Chrome/127.0.0.0 Safari/537.36"),
    "Accept": "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8",
    "Accept-Language": "en-US,en;q=0.9",
}

def build_url(page: int) -> str:
    return "https://coinmarketcap.com/" if page == 1 else f"https://coinmarketcap.com/?page={page}"

async def fetch_html(session: aiohttp.ClientSession, url: str) -> str:
    async with session.get(url, timeout=aiohttp.ClientTimeout(total=20)) as resp:
        resp.raise_for_status()
        return await resp.text()

def extract_crypto_list(html: str) -> List[Dict[str, Any]]:
    soup = BeautifulSoup(html, "html.parser")
    tag = soup.find("script", id="__NEXT_DATA__")
    if not tag or not tag.text:
        raise ValueError("__NEXT_DATA__ not found")

    data = json.loads(tag.text)

    # fast path (what you observed)
    try:
        queries = data["props"]["dehydratedState"]["queries"]
        for q in queries:
            listing = q.get("state", {}).get("data", {}).get("data", {}).get("listing", {})
            arr = listing.get("cryptoCurrencyList")
            if isinstance(arr, list):
                return arr
    except Exception:
        pass

    # fallback: recursive search for 'cryptoCurrencyList'
    def walk(v):
        if isinstance(v, dict):
            if "cryptoCurrencyList" in v and isinstance(v["cryptoCurrencyList"], list):
                return v["cryptoCurrencyList"]
            for vv in v.values():
                got = walk(vv)
                if got is not None:
                    return got
        elif isinstance(v, list):
            for vv in v:
                got = walk(vv)
                if got is not None:
                    return got
        return None

    arr = walk(data)
    if arr is None:
        raise ValueError("cryptoCurrencyList not found in __NEXT_DATA__")
    return arr

def to_row(obj: Dict[str, Any]) -> Dict[str, Any]:
    # tolerate both shapes: quote.USD or quotes[name='USD']
    usd = None
    if isinstance(obj.get("quote"), dict) and isinstance(obj["quote"].get("USD"), dict):
        usd = obj["quote"]["USD"]
    else:
        for q in obj.get("quotes", []):
            if q.get("name") == "USD":
                usd = q
                break

    return {
        "id": obj.get("id"),
        "name": obj.get("name"),
        "symbol": obj.get("symbol"),
        "cmcRank": obj.get("cmcRank") or obj.get("rank"),
        "price_usd": None if not usd else usd.get("price"),
        "market_cap_usd": None if not usd else usd.get("marketCap"),
        "change_24h": None if not usd else usd.get("percentChange24h"),
    }

async def fetch_page(session: aiohttp.ClientSession, page: int) -> List[Dict[str, Any]]:
    html = await fetch_html(session, build_url(page))
    coins = extract_crypto_list(html)
    return [to_row(c) for c in coins if isinstance(c, dict)]

async def scrape_concurrent(pages: int, batch_size: int, pause_ms: int) -> List[Dict[str, Any]]:
    results: List[Dict[str, Any]] = []
    conn = aiohttp.TCPConnector(limit_per_host=batch_size, limit=0, enable_cleanup_closed=True, ssl=False)
    async with aiohttp.ClientSession(headers=HEADERS, connector=conn) as session:
        for start in range(1, pages + 1, batch_size):
            chunk = list(range(start, min(start + batch_size, pages + 1)))
            tasks = [asyncio.create_task(fetch_page(session, p)) for p in chunk]
            pages_results = await asyncio.gather(*tasks, return_exceptions=True)
            for r in pages_results:
                if isinstance(r, Exception):
                    # keep going; you can log it
                    continue
                results.extend(r)
            if start + batch_size <= pages and pause_ms > 0:
                await asyncio.sleep(pause_ms / 1000.0)
    # Dedup by id (page overlaps shouldn't happen but let's be safe)
    seen = set()
    deduped = []
    for row in results:
        cid = row.get("id")
        if cid and cid not in seen:
            seen.add(cid)
            deduped.append(row)
    # sort by rank for parity with Rust
    deduped.sort(key=lambda x: x.get("cmcRank") if x.get("cmcRank") is not None else 10**9)
    return deduped

async def scrape_sequential(pages: int) -> List[Dict[str, Any]]:
    conn = aiohttp.TCPConnector(limit_per_host=1, limit=1, enable_cleanup_closed=True, ssl=False)
    async with aiohttp.ClientSession(headers=HEADERS, connector=conn) as session:
        all_rows = []
        for p in range(1, pages + 1):
            all_rows.extend(await fetch_page(session, p))
        all_rows.sort(key=lambda x: x.get("cmcRank") if x.get("cmcRank") is not None else 10**9)
        return all_rows

def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--pages", type=int, default=10, help="How many pages to fetch (100 coins per page)")
    ap.add_argument("--mode", choices=["sequential", "fast", "safe"], default="fast")
    ap.add_argument("--batch-size", type=int, default=10, help="Concurrent requests per batch (fast mode)")
    ap.add_argument("--pause-ms", type=int, default=300, help="Pause between batches (fast/safe mode)")
    args = ap.parse_args()

    t0 = time.perf_counter()
    if args.mode == "sequential":
        rows = asyncio.run(scrape_sequential(args.pages))
    else:
        # safe suggestion: batch=5, pause=500
        batch = 5 if args.mode == "safe" else args.batch_size
        pause = 500 if args.mode == "safe" else args.pause_ms
        rows = asyncio.run(scrape_concurrent(args.pages, batch, pause))
    dt = time.perf_counter() - t0

    print(f"Fetched {len(rows)} coins in {dt:.2f}s "
          f"({len(rows)/max(dt,1e-6):.0f} coins/sec) | mode={args.mode}")

    # Print first 10 rows as a sanity check
    print("{:<5} {:<15} {:<8} {:>12} {:>14} {:>8}".format("Rank","Name","Symbol","Price","MktCap","24h%"))
    print("-"*70)
    for r in rows[:10]:
        price = "-" if r["price_usd"] is None else f"{r['price_usd']:.2f}"
        mcap  = "-" if r["market_cap_usd"] is None else f"{r['market_cap_usd']:.0f}"
        chg   = "-" if r["change_24h"] is None else f"{r['change_24h']:+.2f}"
        print("{:<5} {:<15} {:<8} {:>12} {:>14} {:>8}".format(
            r.get("cmcRank","-"), r.get("name",""), r.get("symbol",""),
            price, mcap, chg
        ))

if __name__ == "__main__":
    main()