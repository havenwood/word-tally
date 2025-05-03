# word-tally

[![Crates.io](https://img.shields.io/crates/v/word-tally?style=for-the-badge&label=word-tally)](https://crates.io/crates/word-tally)
[![docs.rs](https://img.shields.io/docsrs/word-tally?style=for-the-badge&link=https%3A%2F%2Fdocs.rs%2Fword-tally%2Flatest%2Fword_tally%2F)](https://docs.rs/word-tally/latest/word_tally/)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/havenwood/word-tally/rust.yml?style=for-the-badge)](https://github.com/havenwood/word-tally/actions/workflows/rust.yml)

Output a tally of the number of times unique words appear in source input.

## Stability Notice

**Pre-release level stability**: This project is currently in pre-release stage. Expect breaking interface changes at MINOR version bumps (0.x.0) as the API evolves. The library will maintain API stability once it reaches 1.0.0.

## Usage

```
Usage: word-tally [OPTIONS] [PATH]

Arguments:
  [PATH]  File path to use as input rather than stdin ("-") [default: -]

Options:
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
  -p, --parallel               Use parallel processing for word counting
  -h, --help                   Print help
  -V, --version                Print version
```

## Examples

### Basic Usage

```sh
# Count words from a file
word-tally README.md | head -n3
#>> tally 22
#>> word 20
#>> https 11

# Count words from stdin
echo "one two two three three three" | word-tally
#>> three 3
#>> two 2
#>> one 1

# Count words and output to a file
word-tally README.md --output=words.txt
```

### Filtering Words

```sh
# Exclude words starting with "a" or exactly matching "the"
word-tally --exclude="^a.*" --exclude="^the$" book.txt

# Include only words starting with "h" or "w"
word-tally --include="^h.*" --include="^w.*" book.txt

# Exclude words with fewer than 5 characters
word-tally --min-chars=5 book.txt

# Only include words that appear at least 10 times
word-tally --min-count=10 book.txt

# Combining include and exclude patterns
word-tally --include="^h.*" --include="^w.*" --exclude="^who$" book.txt

# Exclude specific words
word-tally --exclude-words="the,a,an,and,or,but" book.txt
```

CSV output:
```sh
# Using delimiter (manual CSV)
word-tally --delimiter="," --output="tally.csv" README.md

# Using CSV format (with header)
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

### Stream Processing and Parallelization

word-tally efficiently processes text input of any size through streaming, reading line-by-line to maintain minimal memory usage. For large inputs, the `--parallel` flag enables multi-threaded processing that can significantly improve performance by utilizing multiple CPU cores.

```sh
# Process large input using parallel mode
word-tally --parallel large-file.txt
```

Performance can be further tuned through environment variables (detailed in the Environment Variables section below).

### Environment Variables

These variables affect memory allocation and performance in all modes:

- `WORD_TALLY_UNIQUENESS_RATIO` - Divisor for estimating unique words from input size (default: 10)
- `WORD_TALLY_DEFAULT_CAPACITY` - Default initial capacity when there is no size hint (default: 1024)

These variables tune parallel processing when using the `--parallel` flag:

- `WORD_TALLY_THREADS` - Number of threads for parallel processing (default: number of cores)
- `WORD_TALLY_CHUNK_SIZE` - Size of chunks for parallel processing in bytes (default: 16384)
- `WORD_TALLY_WORD_DENSITY` - Multiplier for estimating unique words per chunk (default: 15)

## Installation

```sh
cargo install word-tally
```

## Library Usage

```toml
[dependencies]
word-tally = "0.21.0"
```

```rust
use std::fs::File;
use word_tally::{Options, WordTally};

fn main() -> std::io::Result<()> {
    // Create a word tally by streaming data from a file
    let file = File::open("document.txt")?;
    let word_tally = WordTally::new(file, &Options::default());

    // Print basic statistics about the word tally
    println!("Words: {} total, {} unique", word_tally.count(), word_tally.uniq_count());

    // Print the top 5 words and the count of times each appear
    for (word, count) in word_tally.tally().iter().take(5) {
        println!("{}: {}", word, count);
    }

    Ok(())
}
```

The library supports customization including case normalization, sorting, filtering, and parallel processing.

## Documentation

[https://docs.rs/word-tally](https://docs.rs/word-tally/latest/word_tally/)

## Tests & benchmarks

Clone the repository.

```sh
git clone https://github.com/havenwood/word-tally
cd word-tally
```

Run the tests.

```sh
cargo test
```

And run the benchmarks.

```sh
cargo bench
```
