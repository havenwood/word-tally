#!/bin/bash
# Benchmark script to compare Monoio async I/O performance

set -euo pipefail

# Test files
SMALL_FILE="benches/scripts/generated_text/test_10mb.txt"
MEDIUM_FILE="benches/scripts/generated_text/test_50.0mb.txt"
LARGE_FILE="benches/scripts/generated_text/large_book_200mb.txt"

# Check if test files exist
for file in "$SMALL_FILE" "$MEDIUM_FILE" "$LARGE_FILE"; do
    if [ ! -f "$file" ]; then
        echo "Error: Test file $file not found"
        exit 1
    fi
done

echo "=== Word-Tally Monoio Async I/O Benchmark ==="
echo "Testing with Monoio-enhanced I/O"
echo ""

# Function to get file size in MB
get_size_mb() {
    local size=$(wc -c < "$1")
    echo "scale=1; $size / 1048576" | bc
}

# Test parallel-in-memory mode (uses our new async read)
echo "Testing parallel-in-memory mode (uses Monoio async I/O):"
echo ""

for file in "$SMALL_FILE" "$MEDIUM_FILE" "$LARGE_FILE"; do
    size_mb=$(get_size_mb "$file")
    echo "File: $file (${size_mb}MB)"
    
    echo "  Warming up..."
    ./target/release/word-tally --io=parallel-in-memory "$file" > /dev/null 2>&1
    
    echo "  Running benchmark..."
    hyperfine --warmup 3 --min-runs 10 \
        --export-json "benches/results/monoio_${size_mb}mb.json" \
        "./target/release/word-tally --io=parallel-in-memory '$file'" 
    echo ""
done

# Compare with parallel-mmap (doesn't use Monoio)
echo "Comparing with parallel-mmap mode (memory-mapped, no Monoio):"
echo ""

for file in "$SMALL_FILE" "$MEDIUM_FILE" "$LARGE_FILE"; do
    size_mb=$(get_size_mb "$file")
    echo "File: $file (${size_mb}MB)"
    
    echo "  Running benchmark..."
    hyperfine --warmup 3 --min-runs 10 \
        --export-json "benches/results/mmap_${size_mb}mb.json" \
        "./target/release/word-tally --io=parallel-mmap '$file'"
    echo ""
done

echo "Benchmark complete! Results saved to benches/results/"