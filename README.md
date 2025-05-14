# word-tally

[![Crates.io](https://img.shields.io/crates/v/word-tally?style=for-the-badge&label=word-tally)](https://crates.io/crates/word-tally)
[![docs.rs](https://img.shields.io/docsrs/word-tally?style=for-the-badge&link=https%3A%2F%2Fdocs.rs%2Fword-tally%2Flatest%2Fword_tally%2F)](https://docs.rs/word-tally/latest/word_tally/)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/havenwood/word-tally/rust.yml?style=for-the-badge)](https://github.com/havenwood/word-tally/actions/workflows/rust.yml)

Output a tally of the number of times unique unicode words appear in a source. Provides a command line and Rust library interface. I/O is streamed by default, but buffered and memory-mapped I/O are also supported to optimize for different file sizes and workloads. Memory-mapping is only supported for files with seekable file descriptors. Parallel processing can be enabled to take advantage of multiple CPU cores through parallelizing I/O processing and sorting.

## Installation

```sh
cargo install word-tally
```

## Usage

```
Usage: word-tally [OPTIONS] [PATH]

Arguments:
  [PATH]  File path to use as input rather than stdin ("-") [default: -]

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

## Examples

### Basic usage

```sh
word-tally README.md | head -n3
#>> word 48
#>> tally 47
#>> default 24

echo "one two two three three three" | word-tally
#>> three 3
#>> two 2
#>> one 1

echo "one two two three three three" | word-tally --verbose
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
#>> three 3
#>> two 2
#>> one 1
```

### I/O & parallelization

word-tally supports various combinations of I/O modes and parallel processing:

```sh
# Streamed I/O from stdin (`--io=streamed` is default unless another I/O is specified)
output | word-tally

# Streamed I/O from file
word-tally file.txt

# Streamed I/O with parallel processing
word-tally --parallel document.txt

# Buffered I/O with parallel processing
word-tally --io=buffered --parallel document.txt

# Memory-mapped I/O with efficient parallel processing (requires a file rather than stdin)
word-tally --io=mmap --parallel document.txt
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

## Environment Variables

The following environment variables configure various aspects of the library:

I/O and processing strategy configuration:

- `WORD_TALLY_IO` - I/O strategy (default: streamed, options: streamed, buffered, memory-mapped)
- `WORD_TALLY_PROCESSING` - Processing strategy (default: sequential, options: sequential, parallel)
- `WORD_TALLY_VERBOSE` - Enable verbose mode (default: false, options: true/1/yes/on)

Memory allocation and performance:

- `WORD_TALLY_UNIQUENESS_RATIO` - Divisor for estimating unique words from input size (default: 10)
- `WORD_TALLY_DEFAULT_CAPACITY` - Default initial capacity when there is no size hint (default: 1024)
- `WORD_TALLY_WORD_DENSITY` - Multiplier for estimating unique words per chunk (default: 15)

Parallel processing configuration:

- `WORD_TALLY_THREADS` - Number of threads for parallel processing (default: all available cores)
- `WORD_TALLY_CHUNK_SIZE` - Size of chunks for parallel processing in bytes (default: 65536)

## Library Usage

```toml
[dependencies]
word-tally = "0.24.0"
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

The library supports customization including case normalization, sorting, filtering, and I/O and processing strategies.

## Stability Notice

**Pre-release level stability**: This is prerelease software. Expect breaking interface changes at MINOR version (0.x.0) bumps until there is a stable release.

## Tests & Benchmarks

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
