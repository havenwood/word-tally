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
  -s, --sort <ORDER>           Sort order [default: desc] [possible values: desc, asc, unsorted]
  -c, --case <FORMAT>          Case normalization [default: lower] [possible values: original, upper, lower]
  -m, --min-chars <COUNT>      Exclude words containing fewer than min chars
  -M, --min-count <COUNT>      Exclude words appearing fewer than min times
  -E, --exclude-words <WORDS>  Exclude words from a comma-delimited list
  -x, --exclude <PATTERN>      Exclude words matching a regex pattern
  -i, --include <PATTERN>      Include only words matching a regex pattern
  -d, --delimiter <VALUE>      Delimiter between keys and values [default: " "]
  -o, --output <PATH>          Write output to file rather than stdout
  -v, --verbose                Print verbose details
  -f, --format <FORMAT>        Output format [default: text] [possible values: text, json, csv]
  -p, --parallel               Use parallel processing for word counting
  -h, --help                   Print help
  -V, --version                Print version
```

## Examples

```sh
word-tally README.md | head -n3
#>> tally 22
#>> word 20
#>> https 11

# Using regex patterns to exclude words
word-tally --exclude="^a.*" --exclude="^the$" book.txt

# Using regex patterns to include only specific words
word-tally --include="^h.*" --include="^w.*" book.txt

# Combining include and exclude patterns
word-tally --include="^h.*" --include="^w.*" --exclude="^who$" book.txt
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

Parallel processing can be much faster for large files:
```sh
word-tally --parallel README.md

# Tune with environment variables
WORD_TALLY_THREADS=4 WORD_TALLY_CHUNK_SIZE=32768 word-tally --parallel huge-file.txt
```

### Environment Variables

- `WORD_TALLY_UNIQUENESS_RATIO` - Divisor for estimating unique words from input size (default: 10)
- `WORD_TALLY_DEFAULT_CAPACITY` - Default initial capacity when there is no size hint (default: 1024)

These variables only affect the program when using the `--parallel` flag:

- `WORD_TALLY_THREADS` - Number of threads for parallel processing (default: number of cores)
- `WORD_TALLY_CHUNK_SIZE` - Size of chunks for parallel processing in bytes (default: 16384)
- `WORD_TALLY_WORD_DENSITY` - Multiplier for estimating unique words per chunk (default: 15)

## Installation

```sh
cargo install word-tally
```

## Cargo.toml

Add `word-tally` as a dependency.

```toml
[dependencies]
word-tally = "0.20.0"
```

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
