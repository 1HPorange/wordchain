#[macro_use]
extern crate clap;

use clap::{App, Arg};
use std::path::Path;
use std::io;
use std::io::Read;
use std::fs::File;
use std::str;
use std::cmp;

fn main() {

    const ARG_MIN_OVERLAP : &str = "min-overlap";
    const ARG_WORD_FILE : &str = "word-file";

    let matches = App::new("wordchain")
        .version(crate_version!())
        .author("Markus Webel <m@rkus.online>")
        .about("Finds the longest chain of non-repeating intersecting words in a file (1 word per line)")
        .arg(Arg::with_name(ARG_MIN_OVERLAP)
            .short("o")
            .long("min-overlap")
            .help("How many characters at the end/beginning of two words need to match to be considered linkable")
            .default_value("1")
            .validator(validate_min_overlap))
        .arg(Arg::with_name(ARG_WORD_FILE)
            .index(1)
            .help("A file with all unique words to be considered, separated by line-breaks")
            .required(true)
            .validator(validate_word_file))
        .get_matches();

    let words_file = matches.value_of(ARG_WORD_FILE).unwrap();

    let min_overlap = value_t_or_exit!(matches, ARG_MIN_OVERLAP, u32);
    let words = parse_words_file(words_file).unwrap_or_else(|e| {
        panic!("Could not read file: {}", e);
    });

    println!("Akali / Kalista: {}", overlapping_chars("Akali", "Kalista"));
}

fn validate_min_overlap(arg: String) -> Result<(), String> {

    const ERROR_MSG: &str = "min-overlap needs to be a positive integer";

    let num : u32 = arg.parse().map_err(|_| String::from(ERROR_MSG))?;

    if num == 0 {
        Err(String::from(ERROR_MSG))
    }
    else {
        Ok(())
    }
}

fn validate_word_file(arg: String) -> Result<(), String> {

    if Path::new(&arg).exists() {
        Ok(())
    }
    else {
        Err(format!("\"{}\" is not a valid path", arg))
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

fn overlapping_chars(left: &str, right: &str) -> usize {

    let left = left.to_lowercase();
    let right = right.to_lowercase();

    let mut left = left.chars().as_str();
    let mut right = right.chars().as_str();

    // This issue should be caught by the file parser:
    debug_assert!(left.len() > 0 && right.len() > 0);

    let max_overlap = cmp::min(left.len(), right.len()) - 1;

    // trim words to maximum potential overlap
    left = &left[left.len()-max_overlap..];
    right = &right[..max_overlap];

    for overlap in (1..=max_overlap).rev() {

        if left == right {
            return overlap;
        }

        left = &left[1..];
        right = &right[..right.len() - 1];
    };

    0
}