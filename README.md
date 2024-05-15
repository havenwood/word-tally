# word-tally

Output a tally of the number of times unique words appear in source input.

```
Usage: word-tally [OPTIONS] [INPUT]

Arguments:
  [INPUT]  File path to use as input rather than stdin ("-") [default: -]

Options:
  -O, --output <PATH>          Write output to file rather than stdout
  -D, --delimiter <DELIMITER>  Delimiter between keys and values [default: ": "]
  -c, --case <SENSITIVITY>     Case sensitivity [default: insensitive] [possible values: insensitive, sensitive]
  -o, --order <SORT>           Sort order [default: desc] [possible values: desc, asc, unsorted]
  -v, --verbose                Print verbose details
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
