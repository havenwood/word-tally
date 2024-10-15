//! `word-tally` tallies and outputs the count of words from a given input.
pub(crate) mod args;
pub(crate) mod input;
pub(crate) mod output;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;
use input::Input;
use output::Output;
use unescaper::unescape;
use word_tally::{Case, ExcludeWords, Filters, MinChars, MinCount, Options, Sort, WordTally};

fn main() -> Result<()> {
    let args = Args::parse();
    let delimiter = unescape(&args.delimiter)?;

    let input = Input::from_args(args.input)?;
    let input_file_name = input.file_name().unwrap_or("-").to_string();
    let reader = input
        .get_reader()
        .context(format!("Failed to read from {}.", input_file_name))?;

    let options = Options {
        case: args.case,
        sort: args.sort,
    };

    let filters = Filters {
        min_chars: args.min_chars.map(MinChars),
        min_count: args.min_count.map(MinCount),
        exclude: args.exclude.map(ExcludeWords),
    };

    let word_tally = WordTally::new(reader, options, filters);

    if args.verbose || args.debug {
        let log_config = LogConfig {
            verbose: args.verbose,
            debug: args.debug,
            case: args.case,
            sort: args.sort,
            min_chars: args.min_chars,
            min_count: args.min_count,
        };

        let mut stderr_output = Output::stderr();
        log_details(
            &mut stderr_output,
            &word_tally,
            &delimiter,
            &input_file_name,
            log_config,
        )?;
    }

    let mut output = Output::from_args(args.output)?;

    for (word, count) in word_tally.tally() {
        output.write_line(&format!("{word}{delimiter}{count}\n"))?;
    }

    output.flush()?;

    Ok(())
}

/// Log a formatted details line.
fn log_detail<T: std::fmt::Display>(
    w: &mut Output,
    label: &str,
    delimiter: &str,
    value: T,
) -> Result<()> {
    w.write_line(&format!("{label}{delimiter}{value}\n"))
}

/// `LogConfig` contains `Args` flags that may be used for logging.
struct LogConfig {
    verbose: bool,
    debug: bool,
    case: Case,
    sort: Sort,
    min_chars: Option<usize>,
    min_count: Option<usize>,
}

/// Log verbose and debug details to stderr.
fn log_details(
    stderr: &mut Output,
    word_tally: &WordTally,
    delimiter: &str,
    source: &str,
    log_config: LogConfig,
) -> Result<()> {
    if log_config.verbose {
        log_detail(stderr, "source", delimiter, source)?;
        log_detail(stderr, "total-words", delimiter, word_tally.count())?;
        log_detail(stderr, "unique-words", delimiter, word_tally.uniq_count())?;
    }

    if log_config.debug {
        log_detail(stderr, "delimiter", delimiter, format!("{:?}", delimiter))?;
        log_detail(stderr, "case", delimiter, log_config.case)?;
        log_detail(stderr, "order", delimiter, log_config.sort)?;
        log_detail(
            stderr,
            "min-chars",
            delimiter,
            log_config
                .min_chars
                .map_or("none".to_string(), |count| count.to_string()),
        )?;
        log_detail(
            stderr,
            "min-count",
            delimiter,
            log_config
                .min_count
                .map_or("none".to_string(), |count| count.to_string()),
        )?;
        log_detail(stderr, "verbose", delimiter, log_config.verbose)?;
        log_detail(stderr, "debug", delimiter, log_config.debug)?;
    }

    if word_tally.count() > 0 {
        stderr.write_line("\n")?;
    }

    Ok(())
}
