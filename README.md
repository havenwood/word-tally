# word-tally

Output the number of times each word that appears in a file or stdin source.

```
Usage: word-tally [OPTIONS] [INPUT]

Arguments:
  [INPUT]  Source input for tallying words [default: -]

Options:
  -D, --delimiter <DELIMITER>  Delimiter between word and count [default: ": "]
  -c, --case-sensitive         Tally with case sensitivity
  -n, --no-sort                Unsorted word count order
  -d, --debug                  Additional debugging information
  -v, --verbose                Verbose command details
  -o, --output <OUTPUT>        Output to file rather than stdout
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
