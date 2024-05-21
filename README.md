# word-tally

[![Crates.io](https://img.shields.io/crates/v/word-tally?style=for-the-badge&label=word-tally)](https://crates.io/crates/word-tally)
[![docs.rs](https://img.shields.io/docsrs/word-tally?style=for-the-badge&link=https%3A%2F%2Fdocs.rs%2Fword-tally%2Flatest%2Fword_tally%2F)](https://docs.rs/word-tally/latest/word_tally/)
[![GitHub Actions Workflow Status](https://img.shields.io/github/actions/workflow/status/havenwood/word-tally/rust.yml?style=for-the-badge)](https://github.com/havenwood/word-tally/actions/workflows/rust.yml)

Output a tally of the number of times unique words appear in source input.

## Usage

```
Usage: word-tally [OPTIONS] [INPUT]

Arguments:
  [INPUT]  File path to use as input rather than stdin ("-") [default: -]

Options:
  -s, --sort <ORDER>       Order [default: desc] [possible values: desc, asc, unsorted]
  -c, --case <FORMAT>      Normalization [default: lower] [possible values: original, upper, lower]
  -D, --delimiter <VALUE>  Delimiter between keys and values [default: " "]
  -o, --output <PATH>      Write output to file rather than stdout
  -v, --verbose            Print verbose details
  -d, --debug              Print debugging information
  -h, --help               Print help
  -V, --version            Print version
```

## Examples

```sh
> word-tally README.md | head -n6
tally 9
word 8
input 5
default 4
print 4
output 4
```

```sh
> word-tally --delimiter="," --output="tally.csv" README.md
```

## Installation

```sh
cargo install word-tally
```

## Documentation

[https://docs.rs/word-tally](https://docs.rs/word-tally/latest/word_tally/)

## Tests

```sh
git clone https://github.com/havenwood/word-tally
cd word-tally
cargo test
```
