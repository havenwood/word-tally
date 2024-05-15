# word-tally

Output a tally of the number of times unique words appear in source input.

## Usage

```
Usage: word-tally [OPTIONS] [INPUT]

Arguments:
  [INPUT]  File path to use as input rather than stdin ("-") [default: -]

Options:
  -s, --sort <ORDER>            Sort order [default: desc] [possible values: desc, asc, unsorted]
  -c, --case <SENSITIVITY>      Case sensitivity [default: insensitive] [possible values: insensitive, sensitive]
  -D, --delimiter <CHARACTERS>  Delimiter characters between keys and values [default: ": "]
  -o, --output <PATH>           Write output to file rather than stdout
  -v, --verbose                 Print verbose details
  -d, --debug                   Print debugging information
  -h, --help                    Print help
  -V, --version                 Print version
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

## Installation

```sh
git clone https://github.com/havenwood/word-tally
cd word-tally
cargo build --release
sudo ln -s target/release/word-tally /usr/local/bin/word-tally
```
