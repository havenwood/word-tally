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
use word_tally::{Filters, Options, Performance, SizeHint, WordTally};

fn main() -> Result<()> {
    // Parse arguments.
    let args = Args::parse();

    let input = Input::from_args(args.get_input().as_str())?;
    let input_size = input.size();
    let size_hint = input_size.map_or_else(SizeHint::default, SizeHint::Bytes);
    let source = input.source();
    let reader = input.get_reader(&source)?;

    let formatting = args.get_formatting();
    let filters = Filters::create_from_args(
        &args.get_min_chars(),
        &args.get_min_count(),
        args.get_exclude_words(),
        args.get_exclude_patterns(),
        args.get_include_patterns(),
    )
    .with_context(|| "Failed to compile filter patterns")?;
    let concurrency = args.get_concurrency();
    let performance = Performance::default()
        .with_concurrency(concurrency)
        .with_size_hint(size_hint);
    let options = Options::new(formatting, filters, performance);

    // Create a WordTally instance.
    let word_tally = WordTally::new(reader, &options);

    // Process output.
    let delimiter = args.get_unescaped_delimiter()?;

    crate::verbose::handle_verbose_output(
        args.is_verbose(),
        options.format(),
        &word_tally,
        &delimiter,
        &source,
    )?;

    let mut output = Output::from_args(args.get_output())?;
    output.write_formatted_tally(word_tally.tally(), options.format(), &delimiter)?;

    Ok(())
}
