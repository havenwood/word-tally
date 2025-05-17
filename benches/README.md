# Word-Tally Benchmarks

Benchmarks for measuring word-tally performance across different strategies.

## Benchmark Structure

- `common.rs` - Shared benchmark utilities and text generation functions
- `core.rs` - Benchmarks for sorting and filtering strategies
- `io.rs` - I/O and processing strategy benchmarks (Streamed, Buffered, Memory-mapped)
- `features.rs` - Processing strategy benchmarks (Sequential vs Parallel)
- `fixtures/` - Test data files for benchmarking

## What's Being Benchmarked

### Core Benchmarks
- **Sorting strategies**: Unsorted vs Descending sort performance
- **Filtering strategies**: Min characters, min count, and combined filters

### I/O Benchmarks
- **I/O strategies**: Performance comparison between Streamed, Buffered, and Memory-mapped I/O
- **Processing strategies**: Sequential vs Parallel processing for each I/O method
- **File sizes**: Small (10KB), Medium (75KB), and Large (500KB) in release mode

### Features Benchmarks
- **Processing comparison**: Sequential vs Parallel processing for different text sizes
- **Regex patterns**: Performance with no patterns, few patterns, and many patterns
- **Text sizes**: Small (~30KB) and Medium (~100KB) samples

## Running Benchmarks

```sh
# Run all benchmarks
cargo bench

# Run specific benchmark groups
cargo bench --bench core     # Sorting and filtering benchmarks
cargo bench --bench io       # I/O and processing strategy benchmarks
cargo bench --bench features # Processing strategy benchmarks

# Run specific benchmark tests by group name
cargo bench -- core/sorting_strategies
cargo bench -- core/filtering_strategies
cargo bench -- features/processing_comparison
cargo bench -- features/regex_patterns
cargo bench -- io_strategies/file_size_10kb
cargo bench -- io_strategies/file_size_75kb
```

## Benchmark Generation
- Uses the `fake` crate to generate semirealistic test data with random words
- Creates temporary files of specified sizes for I/O testing
- Wraps options in `Arc` for efficient sharing between benchmark iterations

## Standard Configuration
- 60 samples per benchmark
- 7 second measurement time
- 3 second warm-up time
- Large input batching to minimize overhead
