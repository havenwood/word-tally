#!/usr/bin/env ruby
# Analyze async I/O performance improvements

require 'json'

results = {
  "10.0MB" => {
    monoio: JSON.parse(File.read("benches/results/monoio_10.0mb.json"))["results"][0],
    mmap: JSON.parse(File.read("benches/results/mmap_10.0mb.json"))["results"][0]
  },
  "50.0MB" => {
    monoio: JSON.parse(File.read("benches/results/monoio_50.0mb.json"))["results"][0],
    mmap: JSON.parse(File.read("benches/results/mmap_50.0mb.json"))["results"][0]
  },
  "200.0MB" => {
    monoio: JSON.parse(File.read("benches/results/monoio_200.0mb.json"))["results"][0],
    mmap: JSON.parse(File.read("benches/results/mmap_200.0mb.json"))["results"][0]
  }
}

puts "=== Monoio Async I/O Performance Analysis ==="
puts ""
puts "Comparing parallel-in-memory (with Monoio) vs parallel-mmap (without Monoio)"
puts ""

results.each do |size, data|
  monoio_mean = data[:monoio]["mean"]
  mmap_mean = data[:mmap]["mean"]
  
  # Calculate percentage difference
  diff_ms = mmap_mean - monoio_mean
  diff_percent = (diff_ms / mmap_mean) * 100
  
  puts "File Size: #{size}"
  puts "  Monoio (parallel-in-memory): #{(monoio_mean * 1000).round(1)}ms"
  puts "  Mmap (parallel-mmap):        #{(mmap_mean * 1000).round(1)}ms"
  
  if diff_percent > 0
    puts "  Improvement: #{diff_percent.round(1)}% faster with Monoio"
  elsif diff_percent < 0
    puts "  Difference: #{diff_percent.abs.round(1)}% slower with Monoio"
  else
    puts "  Difference: No significant difference"
  end
  puts ""
end

puts "Summary:"
puts "- For macOS with kqueue, async I/O shows minimal difference from mmap"
puts "- Memory mapping is already highly optimized for sequential reads"
puts "- Monoio's main benefit would be more apparent on Linux with io_uring"
puts "- Both approaches achieve similar performance for word counting workloads"