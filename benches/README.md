# Word-Tally Benchmarks

Performance benchmarks for word-tally I/O strategies and core functionality.

## Benchmark Structure

- `common.rs` - Shared benchmark utilities with unified I/O handling and text generation
- `core.rs` - Sorting strategy benchmarks with small text samples
- `features.rs` - Processing and encoding mode benchmarks
- `io.rs` - I/O strategy benchmarks across different file sizes
- `multi_file.rs` - Multiple file input processing benchmarks

## Benchmarks

- **Core**: Sorting strategies (unsorted vs descending) with ~15KB text samples
- **I/O**: All I/O strategies across file sizes (10KB, 50KB in release builds)
  - `stream`, `parallel-stream`, `parallel-in-memory`, `parallel-mmap`
- **Features**: Processing modes (default vs sequential) and encoding (Unicode vs ASCII)
- **Multi-file**: File aggregation with different I/O strategies

## Running Benchmarks

```sh
# Run all benchmarks
cargo bench

# Run specific benchmark groups
cargo bench --bench core       # Sorting and filtering benchmarks
cargo bench --bench io         # I/O and processing strategy benchmarks
cargo bench --bench features   # Processing strategy benchmarks
cargo bench --bench multi_file # Multiple file input benchmarks

# Run specific benchmark tests by group name
cargo bench -- core/sorting_strategies
cargo bench -- core/filtering_strategies
cargo bench -- features/processing_comparison
cargo bench -- features/encoding_comparison
cargo bench -- io_strategies/file_size_10kb
cargo bench -- io_strategies/file_size_50kb      # Release builds only
cargo bench -- multi_file/processing
```

## Configuration

- **Sample size**: 15 samples per benchmark
- **Timing**: 3 seconds measurement time, 1 second warm-up
- **Test data**: Generated using `fake` crate for realistic text patterns
- **Batching**: `BatchSize::LargeInput` to minimize setup overhead
- **I/O handling**: Unified benchmark functions with dedicated handlers for each I/O mode
