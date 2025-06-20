#!/bin/bash
# Test streaming modes to see if async I/O helps more there

set -euo pipefail

MEDIUM_FILE="benches/scripts/generated_text/test_50.0mb.txt"

echo "=== Testing Streaming Modes with Async I/O ==="
echo ""

# Test stream mode (should benefit from async reads in fill_stream_buffer)
echo "1. Sequential stream mode:"
hyperfine --warmup 2 --min-runs 5 \
    "./target/release/word-tally --io=stream '$MEDIUM_FILE'"

echo ""
echo "2. Parallel stream mode:"  
hyperfine --warmup 2 --min-runs 5 \
    "./target/release/word-tally --io=parallel-stream '$MEDIUM_FILE'"

echo ""
echo "3. Parallel in-memory mode (uses async read_all):"
hyperfine --warmup 2 --min-runs 5 \
    "./target/release/word-tally --io=parallel-in-memory '$MEDIUM_FILE'"

echo ""
echo "4. Parallel mmap mode (no async I/O):"
hyperfine --warmup 2 --min-runs 5 \
    "./target/release/word-tally --io=parallel-mmap '$MEDIUM_FILE'"