//! `word-tally` tallies and outputs the count of words from a given input.

pub(crate) mod args;
pub(crate) mod errors;
pub(crate) mod input;
pub(crate) mod output;
pub(crate) mod verbose;

// Import formatting module from the crate
use word_tally::formatting;

use crate::errors::exit_code;
use crate::input::Input;
use crate::output::Output;
use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use std::fs::File;
use word_tally::{SizeHint, WordTally};

fn main() {
    match run() {
        Ok(()) => std::process::exit(exit_code::SUCCESS),
        Err(err) => {
            eprintln!("Error: {}", err);
            std::process::exit(exit_code(&err));
        }
    }
}

fn run() -> Result<()> {
    // Parse arguments and prepare an input reader.
    let args = Args::parse();

    let input = Input::new(args.get_input().as_str());
    let input_size = input.size();
    let size_hint = input_size.map_or_else(SizeHint::default, SizeHint::Bytes);
    let options = args.get_options(size_hint)?;

    let source = input.source();

    // Create WordTally instance based on input type
    let word_tally = match &input {
        Input::File(path) => {
            let file =
                File::open(path).with_context(|| format!("Failed to read from {}", source))?;

            // Use from_file for memory-mapped I/O, otherwise regular new
            if matches!(options.io(), word_tally::Io::MemoryMapped) {
                WordTally::from_file(&file, &options)?
            } else {
                WordTally::new(file, &options)?
            }
        }
        Input::Stdin => {
            let reader = input.get_reader(&source)?;
            WordTally::new(reader, &options)?
        }
    };

    // Process output.
    let delimiter = args.get_unescaped_delimiter()?;

    if args.is_verbose() {
        crate::verbose::handle_verbose_output(&word_tally, options.format(), &delimiter, &source)?;
    }

    let mut output = Output::new(args.get_output())?;
    output.write_formatted_tally(word_tally.tally(), options.format(), &delimiter)?;

    Ok(())
}
