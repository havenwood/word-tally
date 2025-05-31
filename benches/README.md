# Word-Tally Benchmarks

Performance benchmarks across different strategies.

## Benchmark Structure

- `common.rs` - Shared benchmark utilities and text generation functions
- `core.rs` - Benchmarks for sorting and filtering strategies
- `io.rs` - I/O strategy benchmarks (Stream, Parallel Stream, Parallel In-Memory, Parallel Mmap, Parallel Bytes)
- `features.rs` - Processing strategy benchmarks (Sequential vs Parallel)
- `multi_file.rs` - Multiple file input benchmarks

## Benchmarks

- **Core**: Sorting (unsorted vs descending) and filtering (none vs combined)
- **I/O**: 3 key strategies across file sizes (10KB, 50KB)
  - `stream`, `parallel-stream`, `parallel-mmap`
- **Features**: Sequential vs parallel, Unicode vs ASCII
- **Multi-file**: File aggregation only

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
cargo bench -- features/regex_patterns
cargo bench -- features/encoding_comparison
cargo bench -- io_strategies/file_size_10kb
cargo bench -- io_strategies/file_size_75kb
cargo bench -- multi_file_processing
cargo bench -- multi_file_scaling/2_files
cargo bench -- multi_file_scaling/4_files
cargo bench -- multi_file_scaling/8_files
```

## Configuration
- 15 samples, 3s measurement, 1s warm-up
- Uses `fake` crate for realistic test data
- Large input batching to minimize overhead
