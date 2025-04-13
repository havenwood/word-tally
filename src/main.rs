//! `word-tally` tallies and outputs the count of words from a given input.

pub(crate) mod args;
pub(crate) mod input;
pub(crate) mod output;
pub(crate) mod verbose;

use anyhow::{Context, Result};
use args::{Args, Format};
use clap::Parser;
use input::Input;
use output::Output;
use unescaper::unescape;
use verbose::Verbose;
use word_tally::{Filters, Options, WordTally};

fn main() -> Result<()> {
    let args = Args::parse();

    let delimiter = unescape(&args.delimiter)?;
    let input = Input::from_args(&args.input)?;
    let source = input.source();

    let reader = input.get_reader(&source)?;
    let options = Options::new(args.case, args.sort);
    let filters = Filters::new(&args.min_chars, &args.min_count, args.exclude);
    let input_size = input.size();

    // Create word tally with either parallel or sequential processing based on args
    let word_tally = if args.parallel {
        WordTally::new_parallel_with_size(reader, options, filters, input_size)
    } else {
        WordTally::new_with_size(reader, options, filters, input_size)
    };

    if args.verbose {
        let mut stderr = Output::stderr();
        let mut verbose = Verbose::new(&mut stderr, &word_tally, &delimiter, &source);

        match args.format {
            Format::Json => {
                let json = verbose.to_json()?;
                stderr.write_line(&format!("{json}\n\n"))?;
            },
            Format::Csv => {
                verbose.log_csv()?;
                stderr.write_line("\n")?;
            },
            Format::Text => {
                verbose.log()?;
            }
        }
    }

    let mut output = Output::from_args(&args.output)?;

    match args.format {
        Format::Text => {
            for (word, count) in word_tally.tally() {
                output.write_line(&format!("{word}{delimiter}{count}\n"))?;
            }
        },
        Format::Json => {
            let json = serde_json::to_string(&word_tally.tally().iter().map(|(word, count)| (word.as_ref(), count)).collect::<Vec<_>>())
                .with_context(|| "Failed to serialize word tally to JSON")?;
            output.write_line(&format!("{json}\n"))?;
        },
        Format::Csv => {
            let mut wtr = csv::Writer::from_writer(Vec::new());
            wtr.write_record(["word", "count"])?;
            for (word, count) in word_tally.tally() {
                wtr.write_record([word.as_ref(), &count.to_string()])?;
            }
            let csv_data = String::from_utf8(wtr.into_inner()?)?;
            output.write_line(&csv_data)?;
        }
    }

    output.flush()?;

    Ok(())
}