#[macro_use]
extern crate clap;

use clap::{App, Arg};
use std::path::Path;
use std::io;
use std::io::Read;
use std::fs::File;
use std::str;
use std::cmp;
use std::collections::HashMap;
use std::collections::HashSet;
use std::time::{Duration, Instant};

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

    let min_overlap = value_t_or_exit!(matches, ARG_MIN_OVERLAP, usize);
    let words = parse_words_file(words_file).unwrap_or_else(|e| {
        panic!("Could not read file: {}", e);
    });

    if words.len() > 256 {
        panic!("This algorithm is limited to 256 words. Please remove some words from your file.")
    }

    let follower_map = create_follower_map(&words, min_overlap);

    let sorted_words = sort_words(words, &follower_map, |a,b| {
        // Sorts by most incoming first, then most outgoing
        a.incoming.cmp(&b.incoming)
            .then(a.outgoing.cmp(&b.outgoing))
            .reverse()
    });

    let follower_table = create_follower_table(&sorted_words, &follower_map);

    let mut duration = Duration::from_secs(0);

    const ATTEMPTS: u32 = 5;

    for _ in 1..=ATTEMPTS {

        let before = Instant::now();

        find_longest_chain(&follower_table, &sorted_words);

       duration += before.elapsed();
    }

    duration /= ATTEMPTS;

    println!("Finished search in {}.{} s (average of {})", duration.as_secs(), duration.subsec_millis(), ATTEMPTS);
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

    // TODO:? This issue should be caught by the file parser:
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

struct WordRating {
    incoming: usize,
    outgoing: usize,
    //average_match_len: f64 TODO: Maybe include this
}

type FollowerMap = HashMap<String, HashSet<String>>;

// TODO: Figure out if this is the best way to pass generics
fn sort_words<F>(mut words: Vec<String>, follower_map: &FollowerMap, sorting_func: F) -> Vec<String> where
    F: Fn(&WordRating, &WordRating) -> cmp::Ordering
{
    // - build or receive follower table
    // - build word rating table
    // - sort by the provided sorting function

    let calc_incoming = |left : &str| {
        follower_map.iter()
            .filter(|(_, followers)| followers.contains(left))
            .count()
    };

    let ratings = follower_map.iter()
        .map(|(left, followers)|
            (left, WordRating {
                outgoing: followers.len(),
                incoming: calc_incoming(left)
            }))
        .collect::<HashMap<_,_>>();

    words.sort_unstable_by(|a,b| {
        let left = ratings.get(a).unwrap();
        let right = ratings.get(b).unwrap();

        sorting_func(left, right)
    });

    words
}

fn create_follower_map(words: &Vec<String>, min_overlap: usize) -> FollowerMap
{

    let mut map = HashMap::with_capacity(words.len());

    for left in words {
        let mut followers = HashSet::new();

        for right in words {
            if overlapping_chars(left, right) >= min_overlap
                && left != right {

                followers.insert(right.clone());
            }
        }

        map.insert(left.clone(), followers);
    };

    map
}

fn create_follower_table(sorted_words: &Vec<String>, follower_map: &FollowerMap) -> Vec<Vec<u8>> {

    let mut table = Vec::with_capacity(sorted_words.len());

    for word in sorted_words {

        let followers = follower_map.get(word).unwrap();

        let mut follower_indices = Vec::with_capacity(followers.len());

        for follower in followers {

            let index = sorted_words
                .iter()
                .position(|w| w == follower)
                .unwrap() as u8;

            follower_indices.push(index);
        };

        table.push(follower_indices);
    };

    table
}

fn find_longest_chain(follower_table: &Vec<Vec<u8>>, TEMP_SORTED_WORDS: &Vec<String>) { // TODO: Remove this param

    // Setup
    // TODO: Some of this should come from parameters

    let mut chain = Vec::with_capacity(follower_table.len());

    // POSS OPT: Can be one shorter, since the start index is not a follower of anything
    let mut follower_table_indices = vec![0u8; follower_table.len()];

    // POSS OPT: don't save this in the thread, but in a mutexed location
    // Save only the max length here and update it whenever we acquire the mutex
    // POSS OPT: Guess the longest chain length here (very minor)
    let mut longest = Vec::new();

    // Contains the longest chain length for a given starter token
    // Can be used to abort later chains early
    // Note: This actually contains the length - 1, since a chain could be 256 words long, but not 0
    let mut starter_longest_records: Vec<u8> = vec![std::u8::MAX; follower_table.len()];

    for start_index in 0..follower_table.len() as u8 {

        chain.push(start_index);

        // Again, this actually contains the length - 1
        let mut starter_longest = 0u8;

        loop {
            let index = match chain.last() {
                Some(i) => *i as usize,
                None => break
            };

            let followers = &follower_table[index];

            let follower_index = &mut follower_table_indices[index];

            loop {
                if let Some(follower) = followers.get(*follower_index as usize) {
                    *follower_index += 1;

                    // POSS OPT: Think about whether to do this check before or after membership test
                    let can_be_longest: bool = match starter_longest_records[*follower as usize].checked_add(chain.len() as u8) {
                        Some(len) => {
                            starter_longest = cmp::max(starter_longest, len); // TODO: see if max needed
                            len >= longest.len() as u8
                        },
                        None => {
                            starter_longest = std::u8::MAX;
                            true
                        }
                    };

                    if can_be_longest && !chain.contains(follower) { // OPT: Better membership test

                        chain.push(*follower);

                        break;
                    } // else: don't break
                } else {
                    *follower_index = 0;

                    if chain.len() > longest.len() {
                        longest = chain.clone();

                        println!("Chain of length {}: {}",
                            longest.len(),
                            pretty_format_chain(&TEMP_SORTED_WORDS, &longest));
                    }

                    chain.pop();

                    break;
                }
            }
        }

        starter_longest_records[start_index as usize] = starter_longest;
    }

    //println!("After: {:?}", starter_longest_records.iter().zip(TEMP_SORTED_WORDS).collect::<Vec<_>>());
}

fn pretty_format_chain(sorted_words: &Vec<String>, chain: &Vec<u8>) -> String {

    // TODO: Parser should make sure we have at least one word
    debug_assert!(chain.len() > 0);

    let mut result = String::new();

    for (left, right) in chain
        .windows(2)
        .map(|win| (&sorted_words[win[0] as usize], &sorted_words[win[1] as usize])) {

        let overlap = overlapping_chars(&left, &right);

        result.push_str(&left[..left.len() - overlap]);
    };

    result.push_str(&sorted_words[*chain.last().unwrap() as usize]);

    result
}