# word-tally

[![Crates.io](https://img.shields.io/crates/v/word-tally?style=for-the-badge&label=word-tally)](https://crates.io/crates/word-tally)
[![docs.rs](https://img.shields.io/docsrs/word-tally?style=for-the-badge&link=https%3A%2F%2Fdocs.rs%2Fword-tally%2Flatest%2Fword_tally%2F)](https://docs.rs/word-tally/latest/word_tally/)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/havenwood/word-tally/rust.yml?style=for-the-badge)](https://github.com/havenwood/word-tally/actions/workflows/rust.yml)

Tallies the number of times each word appears in one or more unicode input sources. Use `word-tally` as a command-line tool or `WordTally` via the Rust library interface. I/O is streamed by default. Memory-mapped and fully-buffered-in-memory I/O modes can be selected to optimize for different file sizes and workloads. Optionally take advantage of multiple CPU cores through parallelization of sorting, memory-mapped file I/O and word counting. Memory-mapping is only supported for files with seekable file descriptors, but performs well in parallel mode.

Parallel streaming and parallel buffered I/O modes use SIMD for quick chunk boundary detection. Parallel memory-mapped I/O mode slices chunk boundaries more directly and is often the most efficient choice for large files. Sequential streaming mode uses the least peak memory and is the default.

## Usage

```
Usage: word-tally [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  File paths to use as input (use "-" for stdin) [default: -]

Options:
  -I, --io <STRATEGY>          I/O strategy [default: streamed] [possible values: mmap, streamed, buffered]
  -p, --parallel               Use threads for parallel processing
  -c, --case <FORMAT>          Case normalization [default: lower] [possible values: original, upper, lower]
  -s, --sort <ORDER>           Sort order [default: desc] [possible values: desc, asc, unsorted]
  -m, --min-chars <COUNT>      Exclude words containing fewer than min chars
  -M, --min-count <COUNT>      Exclude words appearing fewer than min times
  -E, --exclude-words <WORDS>  Exclude words from a comma-delimited list
  -i, --include <PATTERN>      Include only words matching a regex pattern
  -x, --exclude <PATTERN>      Exclude words matching a regex pattern
  -f, --format <FORMAT>        Output format [default: text] [possible values: text, json, csv]
  -d, --delimiter <VALUE>      Delimiter between keys and values [default: " "]
  -o, --output <PATH>          Write output to file rather than stdout
  -v, --verbose                Print verbose details
  -h, --help                   Print help (see more with '--help')
  -V, --version                Print version
```

## Installation

```sh
cargo install word-tally
```

## Examples

### I/O & parallelization

word-tally uses sequential, streamed processing by default to reduce memory usage. Parallel processing is also available for memory-mapped, streamed and fully-buffered I/O. The different modes have different performance and resource usage characteristics.

```sh
# Streamed I/O from stdin (`--io=streamed` is default)
echo "tally me" | word-tally

# Streamed I/O from a file
word-tally file.txt

# Process multiple files with aggregated word counts
word-tally file1.txt file2.txt file3.txt

# Mix stdin and files
cat file1.txt | word-tally - file2.txt

# Memory-mapped I/O with efficient parallel processing (requires a file rather than stdin)
word-tally --io=mmap --parallel document.txt

# Buffered I/O with parallel processing
word-tally --io=buffered --parallel document.txt

# Streamed I/O with parallel processing
word-tally --io=streamed --parallel document.txt
```

The `--io=mmap` memory-mapped processing mode only works with files and cannot be used with piped input. Parallel processing with memory mapping can be very efficient but mmap requires a file with a seekable file descriptor.

### Output formats

#### Text (default)

```sh
# Write to file instead of stdout
word-tally README.md --output=tally.txt

# Custom delimiter between word and count
word-tally README.md --delimiter=": " --output=tally.txt

# Pipe to other tools
word-tally README.md | head -n10
```

#### CSV

```sh
# Using a comma delimiter (unescaped without headers)
word-tally --delimiter="," --output="tally.csv" README.md

# Using proper CSV format (escaped with headers)
word-tally --format=csv --output="tally.csv" README.md
```

#### JSON

```sh
word-tally --format=json --output="tally.json" README.md
```

#### Visualization
Convert JSON output for visualization with [d3-cloud](https://github.com/jasondavies/d3-cloud#readme):

```sh
word-tally --format=json README.md | jq 'map({text: .[0], value: .[1]})' > d3-cloud.json
```

Format and pipe the JSON output to the [wordcloud_cli](https://github.com/amueller/word_cloud#readme) to produce an image:

```sh
word-tally --format=json README.md | jq -r 'map(.[0] + " ") | join(" ")' | wordcloud_cli --imagefile wordcloud.png
```

### Case normalization

```sh
# Default lowercase normalization
word-tally --case=lower file.txt

# Preserve original case
word-tally --case=original file.txt

# Convert all to uppercase
word-tally --case=upper file.txt
```

### Sorting options

```sh
# Sort by frequency (descending, default)
word-tally --sort=desc file.txt

# Sort alphabetically (ascending)
word-tally --sort=asc file.txt

# No sorting (sorted by order seen)
word-tally --sort=unsorted file.txt
```

### Filtering words

```sh
# Only include words that appear at least 10 times
word-tally --min-count=10 file.txt

# Exclude words with fewer than 5 characters
word-tally --min-chars=5 file.txt

# Exclude words by pattern
word-tally --exclude="^a.*" --exclude="^the$" file.txt

# Combining include and exclude patterns
word-tally --include="^w.*" --include=".*o$" --exclude="^who$" file.txt

# Exclude specific words
word-tally --exclude-words="the,a,an,and,or,but" file.txt
```

### Verbose output

```sh
echo "fe fi fi fo fo fo" | word-tally --verbose
#>> source -
#>> total-words 6
#>> unique-words 3
#>> delimiter " "
#>> case lower
#>> order desc
#>> processing sequential
#>> io streamed
#>> min-chars none
#>> min-count none
#>> exclude-words none
#>> exclude-patterns none
#>>
#>> fo 3
#>> fi 2
#>> fe 1
```

## Environment variables

The following environment variables configure various aspects of the library:

I/O and processing strategy configuration:

- `WORD_TALLY_IO` - I/O strategy (default: streamed, options: streamed, buffered, memory-mapped)
- `WORD_TALLY_PROCESSING` - Processing strategy (default: sequential, options: sequential, parallel)

Memory allocation and performance:

- `WORD_TALLY_UNIQUENESS_RATIO` - Ratio of total words to unique words for capacity estimation. Higher values allocate less initial memory. Books tend to have a 10:1 ratio, but a more conservative 256:1 is used as default to reduce unnecessary memory overhead (default: 256)
- `WORD_TALLY_WORDS_PER_KB` - Estimated words per KB of text for capacity calculation (default: 128, max: 512)
- `WORD_TALLY_STDIN_BUFFER_SIZE` - Buffer size for stdin when size cannot be determined (default: 262144)
- `WORD_TALLY_DEFAULT_CAPACITY` - Default initial capacity when there is no size hint (default: 1024)
- `WORD_TALLY_WORD_DENSITY` - Multiplier for estimating unique words per chunk (default: 15)

Parallel processing configuration:

- `WORD_TALLY_THREADS` - Number of threads for parallel processing (default: all available cores)
- `WORD_TALLY_CHUNK_SIZE` - Size of chunks for parallel processing in bytes (default: 65536)

## Exit codes

`word-tally` uses standard unix exit codes to indicate success or the types of failure:

- `0`: Success
- `1`: General error
- `64`: Command line usage error
- `65`: Data format error
- `66`: Cannot open input
- `69`: Service unavailable
- `74`: I/O error

## Library usage

```toml
[dependencies]
word-tally = "0.25.0"
```

```rust
use std::fs::File;
use word_tally::{Io, Options, Processing, WordTally};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let file = File::open("document.txt")?;
    let word_tally = WordTally::new(&file, &Options::default())?;
    assert!(word_tally.count() > 0);

    let another_file = File::open("another-document.txt")?;
    let options = Options::default()
        .with_io(Io::MemoryMapped)
        .with_processing(Processing::Parallel);
    let parallel_tally = WordTally::new(another_file, &options)?;
    assert!(parallel_tally.count() > 0);

    println!("Words: {} total, {} unique", parallel_tally.count(), parallel_tally.uniq_count());

    for (word, count) in parallel_tally.tally().iter().take(5) {
        println!("{}: {}", word, count);
    }

    Ok(())
}
```

The library supports case normalization, sorting, filtering and I/O and processing strategies.

## Stability notice

**Pre-release level stability**: This is prerelease software. Expect breaking interface changes at MINOR version (0.x.0) bumps until a stable release.

## Tests & benchmarks

### Tests

Clone the repository.

```sh
git clone https://github.com/havenwood/word-tally
cd word-tally
```

Run all tests.

```sh
cargo test
```

Run specific test modules.

```sh
cargo test --test api_tests
cargo test --test filters_tests
cargo test --test io_tests
```

Run individual tests

```
cargo test --test filters_tests -- test_min_chars
cargo test --test io_tests -- test_memory_mapped
```

### Benchmarks

Run all benchmarks.

```sh
cargo bench
```

Run specific benchmark groups

```sh
cargo bench --bench core
cargo bench --bench io
cargo bench --bench features
```

Run specific individual benchmarks

```sh
cargo bench --bench features -- case_sensitivity
cargo bench --bench core -- parallel_vs_sequential
```

## Documentation

[https://docs.rs/word-tally](https://docs.rs/word-tally/latest/word_tally/)
