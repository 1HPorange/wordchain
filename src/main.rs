#[macro_use]
extern crate clap;
extern crate wordchain;

use clap::{App, Arg};
use std::io;
use std::io::Read;
use std::fs::File;
use std::str;
use std::time::Instant;

use wordchain::SortingOrder;

fn main() {

    const ARG_MIN_OVERLAP: &str = "min-overlap";
    const ARG_WORD_FILE: &str = "word-file";
    const ARG_GRANULARITY: &str = "granularity";
    const ARG_VERBOSE: &str = "verbose";
    const ARG_QUICK_RESULTS: &str = "quick-results";

    let matches = App::new("wordchain")
        .version(crate_version!())
        .author("Markus Webel <m@rkus.online>")
        .about("Finds the longest chain of non-repeating overlapping words in a file (1 word per line)")
        .arg(Arg::with_name(ARG_WORD_FILE)
            .index(1)
            .help("A file with all unique words to be considered, separated by line-breaks")
            .required(true))
        .arg(Arg::with_name(ARG_MIN_OVERLAP)
            .short("o")
            .long(ARG_MIN_OVERLAP)
            .help("How many characters at the end/beginning of two words need to match to be considered linkable")
            .default_value("1"))
        .arg(Arg::with_name(ARG_GRANULARITY)
            .short("g")
            .long(ARG_GRANULARITY)
            .help("Set how many levels of recursion the task generation algorithm uses (0-255). \
            Lower values decrease management and memory overhead, but can lead to load imbalance.")
            .takes_value(true))
        .arg(Arg::with_name(ARG_VERBOSE)
            .short("v")
            .long(ARG_VERBOSE)
            .help("Print intermediate results after each starting word has been handled by the \
            search algorithm."))
        .arg(Arg::with_name(ARG_QUICK_RESULTS)
            .long(ARG_QUICK_RESULTS)
            .help("Instead of internally sorting the words for fast completion, this flag sorts the \
            words for longer chains in intermediate results at the cost of overall execution time. \
            Setting this flag automatically enables the --verbose flag."))
        .get_matches();

    let word_file = matches.value_of(ARG_WORD_FILE).unwrap();

    let words = parse_words_file(word_file).unwrap_or_else(|e| {
        panic!("ERROR: Could not read word file ({})", e);
    });

    let granularity = if matches.is_present(ARG_GRANULARITY) {
        Some(value_t_or_exit!(matches, ARG_GRANULARITY, u8))
    } else {
        None
    };

    let config = wordchain::Config {
        min_overlap: value_t_or_exit!(matches, ARG_MIN_OVERLAP, usize),
        granularity,
        verbose: matches.is_present(ARG_VERBOSE) || matches.is_present(ARG_QUICK_RESULTS),
        sorting_order: if matches.is_present(ARG_QUICK_RESULTS) {
            SortingOrder::ForFasterIntermediateResults
        } else {
            SortingOrder::ForFasterCompletion
        }
    };

    let before = Instant::now();

    let longest_chain_info = wordchain::find_longest_chain(words, &config).unwrap_or_else(|err| {
        panic!("ERROR: {}", err);
    });

    let duration = before.elapsed();

    println!("Finished search in {}.{} s", duration.as_secs(), duration.subsec_millis());

    println!("Longest chain ({}): {}", longest_chain_info.len, longest_chain_info.chain);
}

fn parse_words_file(path : &str) -> Result<Vec<String>, io::Error> {

    let mut content = String::new();

    File::open(path)?.read_to_string(&mut content)?;

    Ok(content.lines()
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}