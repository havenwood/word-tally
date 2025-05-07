//! `word-tally` tallies and outputs the count of words from a given input.

pub(crate) mod args;
pub(crate) mod input;
pub(crate) mod output;
pub(crate) mod verbose;

// Import formatting module from the crate
use word_tally::formatting;

use crate::input::Input;
use crate::output::Output;
use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use std::fs::File;
use word_tally::{SizeHint, WordTally};

fn main() -> Result<()> {
    // Parse arguments and prepare an input reader.
    let args = Args::parse();

    let input = Input::new(args.get_input().as_str())?;
    let input_size = input.size();
    let size_hint = input_size.map_or_else(SizeHint::default, SizeHint::Bytes);
    let options = args.get_options(size_hint)?;

    let source = input.source();

    // Create WordTally instance based on input type
    let word_tally = match &input {
        Input::File(path) => {
            let file =
                File::open(path).with_context(|| format!("Failed to read from {}", source))?;
            // Use try_from_file only for memory-mapped I/O, otherwise use new()
            if matches!(options.io(), word_tally::Io::MemoryMapped) {
                WordTally::try_from_file(file, &options)?
            } else {
                WordTally::new(file, &options)
            }
        }
        Input::Stdin => {
            // For stdin, use the standard new method with a reader
            let reader = input.get_reader(&source)?;
            WordTally::new(reader, &options)
        }
    };

    // Process output.
    let delimiter = args.get_unescaped_delimiter()?;

    crate::verbose::handle_verbose_output(
        args.is_verbose(),
        options.format(),
        &word_tally,
        &delimiter,
        &source,
    )?;

    let mut output = Output::new(args.get_output())?;
    output.write_formatted_tally(word_tally.tally(), options.format(), &delimiter)?;

    Ok(())
}
