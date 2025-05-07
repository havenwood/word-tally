# word-tally

[![Crates.io](https://img.shields.io/crates/v/word-tally?style=for-the-badge&label=word-tally)](https://crates.io/crates/word-tally)
[![docs.rs](https://img.shields.io/docsrs/word-tally?style=for-the-badge&link=https%3A%2F%2Fdocs.rs%2Fword-tally%2Flatest%2Fword_tally%2F)](https://docs.rs/word-tally/latest/word_tally/)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/havenwood/word-tally/rust.yml?style=for-the-badge)](https://github.com/havenwood/word-tally/actions/workflows/rust.yml)

Output a tally of the number of times unique words appear in source input.

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

## Stability Notice

**Pre-release level stability**: This project is currently in pre-release stage. Expect breaking interface changes at MINOR version bumps (0.x.0) as the API evolves. The library will maintain API stability once it reaches 1.0.0.

## Examples

### Basic Usage

```sh
word-tally README.md | head -n3
#>> tally 22
#>> word 20
#>> https 11

echo "one two two three three three" | word-tally
#>> three 3
#>> two 2
#>> one 1

word-tally README.md --output=words.txt
```

### Filtering Words

```sh
# Only include words that appear at least 10 times
word-tally --min-count=10 book.txt

# Exclude words with fewer than 5 characters
word-tally --min-chars=5 book.txt

# Exclude words by pattern
word-tally --exclude="^a.*" --exclude="^the$" book.txt

# Combining include and exclude patterns
word-tally --include="^w.*" --include=".*o$" --exclude="^who$" book.txt

# Exclude specific words
word-tally --exclude-words="the,a,an,and,or,but" book.txt
```

CSV output:
```sh
# Using delimiter (manual CSV)
word-tally --delimiter="," --output="tally.csv" README.md

# Using CSV format (with headers)
word-tally --format=csv --output="tally.csv" README.md
```

JSON output:
```sh
word-tally --format=json --output="tally.json" README.md
```

Transform JSON output for visualization with [d3-cloud](https://github.com/jasondavies/d3-cloud#readme):
```sh
word-tally --format=json README.md | jq 'map({text: .[0], value: .[1]})' > d3-cloud.json
```

Transform and pipe the JSON output to the [wordcloud_cli](https://github.com/amueller/word_cloud#readme) to produce an image:
```sh
word-tally --format=json README.md | jq -r 'map(.[0] + " ") | join(" ")' | wordcloud_cli --imagefile wordcloud.png
```

### I/O and Processing Strategies

word-tally supports various I/O modes and parallel processing:

```sh
output | word-tally # `--io=streamed` is the default I/O strategy

word-tally -Istreamed file.txt # `-I` is a short form alias for `--io`

word-tally --io=streamed --parallel large-file.txt

word-tally --io=mmap --parallel large-file.txt

word-tally --io=mmap file.txt

word-tally --io=buffered file.txt
```

#### Performance Considerations

Synthetic enchmarks with semi-realistic data suggest these strategies based on file size:

| File Size | Best for Speed | Best for Memory | Balanced Approach |
|-----------|----------------|-----------------|-------------------|
| Small (<1MB) | Sequential + Memory-mapped | Sequential + Streamed | Sequential + Streamed |
| Medium (1-80MB) | Sequential + Memory-mapped | Sequential + Streamed | Sequential + Memory-mapped |
| Large (>80MB) | Parallel + Memory-mapped | Parallel + Streamed | Parallel + Memory-mapped |
| Very Large (>1GB) | Parallel + Buffered | Parallel + Streamed | Parallel + Streamed |

Anecdotal insights:

- The inflection point where parallel processing becomes faster for me is around 80MB
- At this point, parallel processing may be several times faster than sequential
- For pipes and non-seekable sources, streaming I/O is required
- Memory-mapped I/O provides excellent performance but requires a seekable file
- Sequential streaming processing remains memory-efficient for files under 80MB

Performance can be further tuned through environment variables (detailed below).

### Environment Variables

The following environment variables configure various aspects of the library:

Memory allocation and performance in all modes:

- `WORD_TALLY_UNIQUENESS_RATIO` - Divisor for estimating unique words from input size (default: 10)
- `WORD_TALLY_DEFAULT_CAPACITY` - Default initial capacity when there is no size hint (default: 1024)
- `WORD_TALLY_WORD_DENSITY` - Multiplier for estimating unique words per chunk (default: 15)
- `WORD_TALLY_RESERVE_THRESHOLD` - Base threshold for capacity reservation when merging maps (default: 1000, scales with input size)

Parallel processing configuration:

- `WORD_TALLY_THREADS` - Number of threads for parallel processing (default: all available cores)
- `WORD_TALLY_CHUNK_SIZE` - Size of chunks for parallel processing in bytes (default: 65536, 64KB)

I/O and processing strategy configuration:

- `WORD_TALLY_IO` - I/O strategy (default: streamed, options: streamed, buffered, memory-mapped)
- `WORD_TALLY_PROCESSING` - Processing strategy (default: sequential, options: sequential, parallel)
- `WORD_TALLY_VERBOSE` - Enable verbose mode (default: false, options: true/1/yes/on)

## Installation

```sh
cargo install word-tally
```

## Library Usage

```toml
[dependencies]
word-tally = "0.22.0"
```

```rust
use std::fs::File;
use word_tally::{Io, Options, Processing, WordTally};

fn main() -> std::io::Result<()> {
    // Create a word tally with default options (Streamed I/O, Sequential processing)
    let file = File::open("document.txt")?;
    let word_tally = WordTally::new(file, &Options::default());

    // Or customize I/O and processing strategies
    let file = File::open("large-document.txt")?;
    let options = Options::default()
        .with_io(Io::MemoryMapped)  // Use memory-mapped I/O for better performance with large files
        .with_processing(Processing::Parallel); // Use parallel processing for multi-core efficiency

    // For memory-mapped I/O, use try_from_file to handle potential errors
    let word_tally = WordTally::try_from_file(file, &options).expect("Failed to process file");

    // Print basic statistics
    println!("Words: {} total, {} unique", word_tally.count(), word_tally.uniq_count());

    // Print the top 5 words and the count of times each appear
    for (word, count) in word_tally.tally().iter().take(5) {
        println!("{}: {}", word, count);
    }

    Ok(())
}
```

The library supports customization including case normalization, sorting, filtering, and I/O and processing strategies.

## Documentation

[https://docs.rs/word-tally](https://docs.rs/word-tally/latest/word_tally/)

## Tests & Benchmarks

Clone the repository.

```sh
git clone https://github.com/havenwood/word-tally
cd word-tally
```

Run the tests.

```sh
cargo test
```

Run the benchmarks.

```sh
cargo bench
```

### Benchmarks

The project includes comprehensive benchmarks for measuring performance across different strategies:

```sh
# Run specific benchmark groups
cargo bench --bench core
cargo bench --bench io
cargo bench --bench features

# Run specific benchmark tests
cargo bench --bench io -- size_10kb
cargo bench --bench io -- size_75kb
```
