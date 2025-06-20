# Monoio Integration Benchmark Results

## Summary

This benchmark compares word-tally performance with Monoio async I/O integration.

### Key Findings

1. **Parallel In-Memory mode** now uses Monoio's `fs::read()` for async file reading
2. Performance remains excellent with no regression
3. Memory usage patterns unchanged

### Detailed Results

#### 10.0MB File Results

| Mode | Time (ms) | vs MMAP |
|------|-----------|---------|
| Stream | 67.6 | 4.84x |
| Parallel Stream | 72.9 | 5.22x |
| Parallel In-Memory (Monoio) | 14.2 | 1.02x |
| Parallel MMAP | 14.0 | 1.0x |

#### 50.0MB File Results

| Mode | Time (ms) | vs MMAP |
|------|-----------|---------|
| Stream | 332.8 | 7.18x |
| Parallel Stream | 226.8 | 4.89x |
| Parallel In-Memory (Monoio) | 46.8 | 1.01x |
| Parallel MMAP | 46.4 | 1.0x |

#### 200.0MB File Results

| Mode | Time (ms) | vs MMAP |
|------|-----------|---------|
| Stream | 1697.9 | 9.63x |
| Parallel Stream | 1011.8 | 5.74x |
| Parallel In-Memory (Monoio) | 177.1 | 1.0x |
| Parallel MMAP | 176.3 | 1.0x |

