#!/usr/bin/env python3
"""Test to verify timing parameters match Rust implementation"""

import time
import asyncio
import argparse

async def mock_batch_with_timing(pages: int, batch_size: int, pause_ms: int):
    """Mock the batching logic to see actual timing"""
    print(f"Testing batching: {pages} pages, batch_size={batch_size}, pause_ms={pause_ms}")
    
    total_start = time.perf_counter()
    batch_times = []
    
    for start in range(1, pages + 1, batch_size):
        batch_start = time.perf_counter()
        chunk = list(range(start, min(start + batch_size, pages + 1)))
        
        # Simulate work (like HTTP requests)
        await asyncio.sleep(0.1)  # Mock 100ms per batch processing
        
        batch_end = time.perf_counter()
        batch_duration = batch_end - batch_start
        batch_times.append(batch_duration)
        
        print(f"Batch {len(batch_times)}: pages {chunk[0]}-{chunk[-1]} took {batch_duration:.3f}s")
        
        # Apply pause between batches (if not the last batch)
        if start + batch_size <= pages and pause_ms > 0:
            print(f"  Pausing for {pause_ms}ms...")
            await asyncio.sleep(pause_ms / 1000.0)
    
    total_duration = time.perf_counter() - total_start
    expected_pauses = max(0, (pages // batch_size + (1 if pages % batch_size else 0)) - 1)
    expected_pause_time = expected_pauses * (pause_ms / 1000.0)
    
    print(f"\nSummary:")
    print(f"Total time: {total_duration:.3f}s")
    print(f"Expected pause time: {expected_pause_time:.3f}s ({expected_pauses} pauses)")
    print(f"Processing time: {total_duration - expected_pause_time:.3f}s")

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--pages", type=int, default=10)
    parser.add_argument("--batch-size", type=int, default=10)
    parser.add_argument("--pause-ms", type=int, default=300)
    args = parser.parse_args()
    
    print("=== Timing Verification Test ===")
    asyncio.run(mock_batch_with_timing(args.pages, args.batch_size, args.pause_ms))
    
    print("\n=== Rust Equivalent ===")
    print(f"Rust uses: pages={args.pages}, batch_size=10, pause_ms=300")
    print(f"Python should use same for fair comparison")

if __name__ == "__main__":
    main()