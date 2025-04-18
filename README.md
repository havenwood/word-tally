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
  -s, --sort <ORDER>       Sort order [default: desc] [possible values: desc, asc, unsorted]
  -c, --case <FORMAT>      Case normalization [default: lower] [possible values: original, upper, lower]
  -m, --min-chars <COUNT>  Exclude words containing fewer than min chars
  -M, --min-count <COUNT>  Exclude words appearing fewer than min times
  -e, --exclude <WORDS>    Exclude words from a comma-delimited list
  -d, --delimiter <VALUE>  Delimiter between keys and values [default: " "]
  -o, --output <PATH>      Write output to file rather than stdout
  -f, --format <FORMAT>    Output format [default: text] [possible values: text, json, csv]
  -v, --verbose            Print verbose details
  -p, --parallel           Use parallel processing for word counting
  -h, --help               Print help
  -V, --version            Print version
```

## Examples

```sh
word-tally README.md | head -n3
#>> tally 22
#>> word 20
#>> https 11
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
word-tally = "0.19.0"
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
