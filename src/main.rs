#[macro_use]
extern crate clap;
extern crate wordchain;

use clap::{App, Arg};
use std::io;
use std::io::Read;
use std::fs::File;
use std::str;
use std::time::Instant;
use wordchain::{SortedSearchConfig, Config};

arg_enum!{
    enum Mode {
        normal,
        quickestimate,
        random
    }
}

const ARG_MIN_OVERLAP: &str = "min-overlap";
const ARG_WORD_FILE: &str = "word-file";
const ARG_MODE: &str = "mode";
const ARG_GRANULARITY: &str = "granularity";
const ARG_VERBOSE: &str = "verbose";

fn main() {

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
        .arg(Arg::with_name(ARG_MODE)
            .short("m")
            .long(ARG_MODE)
            .default_value("normal")
            .possible_values(&Mode::variants())
            .help("The search algorithm that is used. Normal mode aims for fastest completion, but gives no intermediate results. \
            Quick estimation mode sacrifices execution time for longer intermediate results, which are printed as soon as they are \
            available. Random mode never terminates and uses heuristic search to guess the longest chain. This mode is NOT guaranteed \
            to find the correct result."))
        .arg(Arg::with_name(ARG_GRANULARITY)
            .short("g")
            .long(ARG_GRANULARITY)
            .takes_value(true)
            .help("Determines the granularity of tasks that are distributed to each thread. Usually a fairly small value (~6) is enough \
            to get good load balancing, but higher values might be beneficial for large workloads. If this argument is omitted, a default \
            value will be used. This argument is not permitted in random mode."))
        .arg(Arg::with_name(ARG_VERBOSE)
            .short("v")
            .long(ARG_VERBOSE)
            .help("Enables more detailed intermediate output."))
        .get_matches();

    let word_file = matches.value_of(ARG_WORD_FILE).unwrap();

    let words = parse_words_file(word_file).unwrap_or_else(|e| {
        panic!("ERROR: Could not read word file ({})", e);
    });

    let min_overlap = value_t_or_exit!(matches, ARG_MIN_OVERLAP, usize);

    let mode = value_t_or_exit!(matches, ARG_MODE, Mode);

    match mode {
        Mode::normal => exec_sorted_search(words, min_overlap, mode, &matches),
        Mode::quickestimate => exec_sorted_search(words, min_overlap, mode, &matches),
        Mode::random => exec_random_search(words, min_overlap, &matches)
    }
}

fn parse_words_file(path : &str) -> Result<Vec<String>, io::Error> {

    let mut content = String::new();

    File::open(path)?.read_to_string(&mut content)?;

    Ok(content.lines()
        .filter(|s| !s.is_empty())
        .map(ToOwned::to_owned)
        .collect())
}

fn exec_sorted_search(
    words: Vec<String>,
    min_overlap: usize,
    mode: Mode,
    matches: &clap::ArgMatches) {

    let granularity = if matches.is_present(ARG_GRANULARITY) {
        Some(value_t_or_exit!(matches, ARG_GRANULARITY, u8))
    } else {
        None
    };

    let verbose = matches.is_present(ARG_VERBOSE);

    let search_config = SortedSearchConfig { granularity, verbose };

    let lib_mode = match mode {
        Mode::normal => wordchain::Mode::Normal(&search_config),
        Mode::quickestimate => wordchain::Mode::QuickEstimate(&search_config),
        _ => unreachable!()
    };

    let config = Config {
        min_overlap,
        mode: lib_mode
    };

    let before = Instant::now();

    let longest_chain_info = wordchain::find_longest_chain(words, &config).unwrap_or_else(|err| {
        panic!("ERROR: {}", err);
    });

    let duration = before.elapsed();

    println!("Finished search in {}.{} s", duration.as_secs(), duration.subsec_millis());

    println!("Longest chain ({}): {}", longest_chain_info.len, longest_chain_info.chain);

}

fn exec_random_search(
    words: Vec<String>,
    min_overlap: usize,
    matches: &clap::ArgMatches) {

    if matches.is_present(ARG_GRANULARITY) {
        panic!("Cannot specify granularity when operating in random mode");
    }

    if matches.is_present(ARG_VERBOSE) {
        panic!("Verbose mode is not available when operating in random mode");
    }

    let config = Config {
        min_overlap,
        mode: wordchain::Mode::RandomSearch
    };

    wordchain::find_longest_chain(words, &config).unwrap_or_else(|err| {
        panic!("ERROR: {}", err);
    });
}