# word-tally

[![Crates.io](https://img.shields.io/crates/v/word-tally?style=for-the-badge&label=word-tally)](https://crates.io/crates/word-tally)
[![docs.rs](https://img.shields.io/docsrs/word-tally?style=for-the-badge&link=https%3A%2F%2Fdocs.rs%2Fword-tally%2Flatest%2Fword_tally%2F)](https://docs.rs/word-tally/latest/word_tally/)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/havenwood/word-tally/rust.yml?style=for-the-badge)](https://github.com/havenwood/word-tally/actions/workflows/rust.yml)

Tallies the number of times each word appears in one or more unicode input sources. Use `word-tally` as a command-line tool or `WordTally` via the Rust library interface.

Four I/O strategies are available:
- **stream**: Sequential single-threaded streaming with minimal memory usage
- **parallel-stream** (default): Parallel streaming with balanced performance and memory usage
- **parallel-in-memory**: Load entire input into memory for parallel processing
- **parallel-mmap**: Memory-mapped I/O for best performance with large files

All parallel modes use SIMD-accelerated chunk boundary detection. Memory mapping requires seekable file descriptors and won't work with stdin or pipes.

## Usage

```
Usage: word-tally [OPTIONS] [PATHS]...

Arguments:
  [PATHS]...  File paths to use as input (use "-" for stdin) [default: -]

Options:
  -I, --io <STRATEGY>            I/O strategy [default: parallel-stream] [possible values: parallel-stream, stream, parallel-mmap, parallel-in-memory]
  -e, --encoding <ENCODING>      Text encoding mode [default: unicode] [possible values: unicode, ascii]
  -c, --case <FORMAT>            Case normalization [default: original] [possible values: original, upper, lower]
  -s, --sort <ORDER>             Sort order [default: desc] [possible values: desc, asc, unsorted]
  -m, --min-chars <COUNT>        Exclude words containing fewer than min chars
  -n, --min-count <COUNT>        Exclude words appearing fewer than min times
  -w, --exclude-words <WORDS>    Exclude words from a comma-delimited list
  -i, --include <PATTERNS>       Include only words matching a regex pattern
  -x, --exclude <PATTERNS>       Exclude words matching a regex pattern
  -f, --format <FORMAT>          Output format [default: text] [possible values: text, json, csv]
  -d, --field-delimiter <VALUE>  Delimiter between field and value [default: " "]
  -D, --entry-delimiter <VALUE>  Delimiter between entries [default: "\n"]
  -o, --output <PATH>            Write output to file rather than stdout
  -v, --verbose                  Print verbose details
  -h, --help                     Print help (see more with '--help')
  -V, --version                  Print version
```

## Installation

```sh
cargo install word-tally
```

## Examples

### I/O strategies

Choose an I/O strategy based on your performance and memory requirements:

```sh
# Default: Parallel streaming - balanced performance and memory for files or stdin
echo "tally me" | word-tally
word-tally file.txt

# Sequential streaming - minimal memory usage for files or stdin
word-tally --io=stream large-file.txt

# Parallel memory-mapped - performance for files with efficient memory usage
word-tally --io=parallel-mmap large-file.txt

# Parallel in-memory - performance for stdin with high memory usage
word-tally --io=parallel-in-memory document.txt
```

Additional features:
```sh
# Process multiple files
word-tally file1.txt file2.txt file3.txt

# Mix stdin and files
cat header.txt | word-tally - body.txt footer.txt

# ASCII encoding mode - validates input is ASCII-only, fails on non-ASCII bytes
# Uses simple ASCII word boundaries (faster than Unicode)
word-tally --encoding=ascii document.txt

# Unicode encoding mode (default) - accepts any UTF-8 text
# Uses ICU4X for proper Unicode word boundary detection
word-tally --encoding=unicode document.txt
```

**Note:** Memory mapping (`parallel-mmap`) requires seekable files and cannot be used with stdin or pipes.

### Output formats

#### Text (default)

```sh
# Write to file instead of stdout
word-tally README.md --output=tally.txt

# Custom delimiter between word and count
word-tally README.md --field-delimiter=": " --output=tally.txt

# Custom delimiter between entries (e.g., comma-separated)
word-tally README.md --field-delimiter=": " --entry-delimiter=", " --output=tally.txt

# Pipe to other tools
word-tally README.md | head -n10
```

#### Custom delimiters

```sh
# Tab-separated values without escaping
word-tally --field-delimiter="\t" README.md > tally.tsv

# Custom delimiters
word-tally --field-delimiter="|" --entry-delimiter=";" README.md
```

#### CSV

```sh
# CSV with proper escaping and headers
word-tally --format=csv README.md > tally.csv
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

### Encoding modes

The `--encoding` flag controls both text validation and word boundary detection:

**Unicode mode (default)**
- Accepts any valid UTF-8 text
- Uses ICU4X for Unicode-compliant word segmentation & case conversion

**ASCII mode**
- Accepts only ASCII text
- Uses simple ASCII word boundaries (alphanumeric + apostrophes)
- Faster processing for ASCII-only text

```sh
# Unicode mode - handles any UTF-8 text
echo "café naïve 你好" | word-tally --encoding=unicode

# ASCII mode - rejects non-ASCII input
echo "café" | word-tally --encoding=ascii  # Error: non-ASCII byte at position 3
```

### Case normalization

```sh
# Convert to lowercase
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
#>> entry-delimiter "\n"
#>> case original
#>> order desc
#>> io parallel-stream
#>> encoding unicode
#>> min-chars none
#>> min-count none
#>> exclude-words none
#>> exclude-patterns none
#>> include-patterns none
#>>
#>> fo 3
#>> fi 2
#>> fe 1
```

## Environment variables

The following environment variables configure various aspects of the library:

I/O and processing strategy configuration:

- `WORD_TALLY_IO` - I/O strategy (default: parallel-stream, options: stream, parallel-stream, parallel-in-memory, parallel-mmap)

Memory allocation and performance:

- `WORD_TALLY_UNIQUENESS_RATIO` - Ratio of total words to unique words for capacity estimation. Higher values allocate less initial memory. Books tend to have a 10:1 ratio, and a balanced 32:1 is used as default for better performance (default: 32)
- `WORD_TALLY_WORDS_PER_KB` - Estimated words per KB of text for capacity calculation (default: 128, max: 512)
- `WORD_TALLY_STDIN_BUFFER_SIZE` - Buffer size for stdin when size cannot be determined (default: 262144)

Parallel processing configuration:

- `WORD_TALLY_THREADS` - Number of threads for parallel processing (default: all available cores)
- `WORD_TALLY_CHUNK_SIZE` - Size of chunks for parallel processing in bytes (default: 65536)

## Exit codes

`word-tally` uses standard unix exit codes to indicate success or the types of failure:

- `0`: Success
- `1`: General failure
- `64`: Command line usage error
- `65`: Data format error
- `66`: Input not found
- `70`: Internal software error
- `73`: Output creation failed
- `74`: I/O error
- `77`: Permission denied

## Library usage

```toml
[dependencies]
word-tally = "0.27.0"
```

```rust
use std::collections::HashMap;
use word_tally::{Case, Filters, Io, Options, Reader, Serialization, TallyMap, View, WordTally};
use anyhow::Result;

fn main() -> Result<()> {
    // Basic usage
    let options = Options::default();
    let reader = Reader::try_from("document.txt")?;
    let tally_map = TallyMap::from_reader(&reader, &options)?;
    let word_tally = WordTally::from_tally_map(tally_map, &options);
    println!("Total words: {}", word_tally.count());

    // Custom configuration
    let options = Options::default()
        .with_case(Case::Lower)
        .with_filters(Filters::default().with_min_chars(3))
        .with_serialization(Serialization::Json)
        .with_io(Io::ParallelMmap);

    let view = View::try_from("large-file.txt")?;
    let tally_map = TallyMap::from_view(&view, &options)?;
    let tally = WordTally::from_tally_map(tally_map, &options);

    // Convert to `HashMap` for fast word lookups
    let lookup: HashMap<_, _> = tally.into();
    println!("Count of 'the': {}", *lookup.get("the").unwrap_or(&0));
    println!("Count of 'word': {}", *lookup.get("word").unwrap_or(&0));

    Ok(())
}
```

The library provides full control over case normalization, sorting, filtering, I/O strategies, and output formats.

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
