#!/usr/bin/env python3
"""Test repeated scraping operations to check for memory leaks and performance degradation"""

import time
import subprocess
import sys
from typing import List

def run_python_test(script: str, iterations: int = 5, pages: int = 10) -> List[float]:
    """Run Python scraper multiple times and measure performance"""
    times = []
    print(f"\n=== Testing {script} ({iterations} iterations, {pages} pages each) ===")
    
    for i in range(iterations):
        start = time.perf_counter()
        try:
            result = subprocess.run([
                'python', script, '--pages', str(pages), '--mode', 'fast'
            ], capture_output=True, text=True, timeout=120)
            
            if result.returncode == 0:
                # Extract time from output
                for line in result.stdout.split('\n'):
                    if 'Fetched' in line and 'coins/sec' in line:
                        # Parse: "Fetched 997 coins in 0.64s (1546 coins/sec)"
                        parts = line.split()
                        time_str = parts[4]  # "0.64s"
                        duration = float(time_str.rstrip('s'))
                        times.append(duration)
                        print(f"Run {i+1}: {duration:.2f}s")
                        break
            else:
                print(f"Run {i+1}: ERROR - {result.stderr}")
        except subprocess.TimeoutExpired:
            print(f"Run {i+1}: TIMEOUT")
        except Exception as e:
            print(f"Run {i+1}: ERROR - {e}")
    
    if times:
        avg_time = sum(times) / len(times)
        print(f"Average: {avg_time:.2f}s, Min: {min(times):.2f}s, Max: {max(times):.2f}s")
        print(f"Performance variance: {((max(times) - min(times)) / avg_time * 100):.1f}%")
    
    return times

def run_rust_test(iterations: int = 5, pages: int = 10) -> List[float]:
    """Run Rust scraper multiple times and measure performance"""
    times = []
    print(f"\n=== Testing Rust ({iterations} iterations, {pages} pages each) ===")
    
    for i in range(iterations):
        try:
            result = subprocess.run([
                'cargo', 'run', '--release', '--', 'scrape', '--pages', str(pages)
            ], capture_output=True, text=True, timeout=120, cwd='/Users/ishworkhanal/Desktop/rust_practice/coinbase_scraper')
            
            if result.returncode == 0:
                # Extract scraping time from output
                for line in result.stdout.split('\n'):
                    if 'Scraped' in line and 'coins in' in line:
                        # Parse: "Scraped 997 coins in 0.43s"
                        parts = line.split()
                        time_str = parts[4].rstrip('s')
                        duration = float(time_str)
                        times.append(duration)
                        print(f"Run {i+1}: {duration:.2f}s")
                        break
            else:
                print(f"Run {i+1}: ERROR - {result.stderr}")
        except subprocess.TimeoutExpired:
            print(f"Run {i+1}: TIMEOUT")
        except Exception as e:
            print(f"Run {i+1}: ERROR - {e}")
    
    if times:
        avg_time = sum(times) / len(times)
        print(f"Average: {avg_time:.2f}s, Min: {min(times):.2f}s, Max: {max(times):.2f}s")
        print(f"Performance variance: {((max(times) - min(times)) / avg_time * 100):.1f}%")
    
    return times

def main():
    iterations = 5
    pages = 10
    
    print("=== Repeated Scraping Test ===")
    print(f"Testing repeated operations to check for memory leaks and performance consistency")
    print(f"Each test: {iterations} iterations of {pages} pages")
    
    # Test all implementations
    aiohttp_times = run_python_test('scrape_cmc.py', iterations, pages)
    requests_times = run_python_test('scrape_requests.py', iterations, pages)
    rust_times = run_rust_test(iterations, pages)
    
    # Summary
    print(f"\n=== Summary ===")
    if aiohttp_times:
        print(f"Python aiohttp:  avg={sum(aiohttp_times)/len(aiohttp_times):.2f}s, variance={((max(aiohttp_times) - min(aiohttp_times)) / (sum(aiohttp_times)/len(aiohttp_times)) * 100):.1f}%")
    if requests_times:
        print(f"Python requests: avg={sum(requests_times)/len(requests_times):.2f}s, variance={((max(requests_times) - min(requests_times)) / (sum(requests_times)/len(requests_times)) * 100):.1f}%")
    if rust_times:
        print(f"Rust:            avg={sum(rust_times)/len(rust_times):.2f}s, variance={((max(rust_times) - min(rust_times)) / (sum(rust_times)/len(rust_times)) * 100):.1f}%")

if __name__ == "__main__":
    main()