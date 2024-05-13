# word-tally

Output a tally of the number of times unique words appear in source input.

```
Usage: word-tally [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Path to file to use as input rather than stdin ("-") [default: -]

Options:
  -o, --output <OUTPUT>        Write output to specified file rather than stdout
  -D, --delimiter <DELIMITER>  Delimiter between keys and values [default: ": "]
  -c, --case-sensitive         Switch to tallying words with case sensitivity
  -n, --no-sort                Skip sorting by descending word count order
  -v, --verbose                Print verbose source details
  -d, --debug                  Print debugging information
  -h, --help                   Print help
  -V, --version                Print version
```

## Example

```sh
> word-tally README.md | head -n6
word: 10
tally: 9
output: 5
count: 4
delimiter: 4
input: 4
```

## Installation

```sh
git clone https://github.com/havenwood/word-tally
cd word-tally
cargo build --release
sudo ln -s target/release/word-tally /usr/local/bin/word-tally
```
