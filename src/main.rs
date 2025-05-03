//! `word-tally` tallies and outputs the count of words from a given input.

pub(crate) mod args;
pub(crate) mod input;
pub(crate) mod output;
pub(crate) mod verbose;

use crate::input::Input;
use crate::output::Output;
use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use word_tally::{Filters, Format, Formatting, Options, Performance, SizeHint, WordTally};

fn main() -> Result<()> {
    // Parse arguments.
    let args = Args::parse();

    let input = Input::from_args(args.get_input().as_str())?;
    let input_size = input.size();
    let size_hint = input_size.map_or_else(SizeHint::default, SizeHint::Bytes);
    let source = input.source();
    let reader = input.get_reader(&source)?;

    let formatting = Formatting::new(args.get_case(), args.get_sort());
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
        args.get_format(),
        &word_tally,
        &delimiter,
        &source,
    )?;

    let mut output = Output::from_args(args.get_output())?;
    output.write_formatted_tally(word_tally.tally(), args.get_format(), &delimiter)?;

    Ok(())
}
