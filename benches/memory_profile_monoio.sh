#!/bin/bash
# Detailed memory profiling for Monoio integration

set -euo pipefail

TEST_FILE="benches/scripts/generated_text/large_book_200mb.txt"
FILE_SIZE_MB=200

echo "=== Memory Usage Profiling for 200MB File ==="
echo ""

# Function to profile memory with time command and extract RSS
profile_memory() {
    local mode="$1"
    local cmd="./target/release/word-tally --io=$mode $TEST_FILE"
    
    # Run with time command and capture stderr
    local output=$( (/usr/bin/time -l $cmd > /dev/null) 2>&1 )
    
    # Extract maximum resident set size (in bytes on macOS)
    local rss_bytes=$(echo "$output" | grep "maximum resident set size" | awk '{print $1}')
    local rss_mb=$(echo "scale=1; $rss_bytes / 1048576" | bc)
    local rss_percent=$(echo "scale=0; ($rss_mb / $FILE_SIZE_MB) * 100" | bc | sed 's/^$/0/')
    
    echo "$mode: ${rss_mb}MB (${rss_percent}% of file)"
}

# Profile each mode
for mode in stream parallel-stream parallel-in-memory parallel-mmap; do
    profile_memory "$mode"
done

echo ""
echo "Running Ruby memory profiler for detailed analysis..."

# Use the Ruby memory profiler for more detailed analysis
ruby -e "
require 'open3'

puts ''
puts 'Detailed Memory Analysis:'
puts '------------------------'

modes = ['stream', 'parallel-stream', 'parallel-in-memory', 'parallel-mmap']
file = 'benches/scripts/generated_text/large_book_200mb.txt'
file_size_mb = 200

results = {}

modes.each do |mode|
  cmd = \"./target/release/word-tally --io=#{mode} #{file}\"
  
  # Run command and capture memory usage
  stdout, stderr, status = Open3.capture3(\"/usr/bin/time -l #{cmd} > /dev/null 2>&1\")
  output = stderr
  
  # Parse memory statistics
  if output =~ /maximum resident set size\\s+(\\d+)/
    rss_bytes = \\$1.to_i
    rss_mb = rss_bytes / 1024.0 / 1024.0
    rss_percent = (rss_mb / file_size_mb * 100).round
    
    results[mode] = {
      rss_mb: rss_mb.round(1),
      rss_percent: rss_percent
    }
  end
end

# Display results in a table
puts ''
puts '| Mode | Memory (MB) | % of File | Notes |'
puts '|------|-------------|-----------|-------|'

results.each do |mode, stats|
  notes = case mode
  when 'parallel-in-memory'
    'Uses Monoio async I/O'
  when 'parallel-mmap'
    'Memory mapped, no async'
  else
    ''
  end
  
  puts \"| #{mode} | #{stats[:rss_mb]} | #{stats[:rss_percent]}% | #{notes} |\"
end
"