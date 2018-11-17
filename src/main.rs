#[macro_use]
extern crate clap;
extern crate uint;
extern crate rayon;

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
use uint::U256;
use std::iter;
use rayon::prelude::*;

fn main() {

    const ARG_MIN_OVERLAP: &str = "min-overlap";
    const ARG_WORD_FILE: &str = "word-file";
    const ARG_GRANULARITY: &str = "granularity";
    const ARG_VERBOSE: &str = "verbose";

    let matches = App::new("wordchain")
        .version(crate_version!())
        .author("Markus Webel <m@rkus.online>")
        .about("Finds the longest chain of non-repeating intersecting words in a file (1 word per line)")
        .arg(Arg::with_name(ARG_MIN_OVERLAP)
            .short("o")
            .long(ARG_MIN_OVERLAP)
            .help("How many characters at the end/beginning of two words need to match to be considered linkable")
            .default_value("1")
            .validator(validate_min_overlap))
        .arg(Arg::with_name(ARG_WORD_FILE)
            .index(1)
            .help("A file with all unique words to be considered, separated by line-breaks")
            .required(true)
            .validator(validate_word_file))
        .arg(Arg::with_name(ARG_GRANULARITY)
            .short("g")
            .long(ARG_GRANULARITY)
            .default_value("6")
            .help("Granularity of the task-distribution: Higher values help with load-balancing, but create more orchestration overhead. \
            Just try some values to find the best fit for your system. Larger workloads usually benefit from slightly increased values.")
            .validator(validate_granularity))
        .arg(Arg::with_name(ARG_VERBOSE)
            .short("v")
            .long(ARG_VERBOSE)
            .takes_value(false))
        .get_matches();

    let words_file = matches.value_of(ARG_WORD_FILE).unwrap();

    let min_overlap = value_t_or_exit!(matches, ARG_MIN_OVERLAP, usize);

    let granularity = value_t_or_exit!(matches, ARG_GRANULARITY, u8);

    let verbose = matches.is_present(ARG_VERBOSE);

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

    let before = Instant::now();

    let longest_chain = find_longest_chain_parallel(&follower_table, granularity, &sorted_words, verbose);

    if !verbose {
        println!("Longest chain ({}): {}",
            longest_chain.len(),
            pretty_format_chain(&sorted_words, &longest_chain));
    }

    duration += before.elapsed();

    println!("Finished search in {}.{} s", duration.as_secs(), duration.subsec_millis());
}

fn find_longest_chain_parallel(
    follower_table: &Vec<Vec<u8>>,
    granularity: u8,
    sorted_words: &Vec<String>,
    verbose: bool) -> Vec<u8> {

    let mut global_longest = Vec::new(); // MIN OPT: Guess length

    let mut longest_estimates: Vec<Option<u8>> = vec![None; follower_table.len()];

    for start_index in 0..follower_table.len() as u8 {

        // TODO: Avoid multiple collects here by boxing this?
        let mut chains = vec![vec![start_index]];

        // TODO: Fix this abomination below...
        for _ in 1..granularity { // TODO: Make this depth configurable or dependent on something smart

            chains = chains.into_iter().flat_map(|v| {

                let last = *v.last().unwrap() as usize;

                let legal_followers = follower_table[last].iter()
                    .filter(|i| !v.contains(i))
                    .map(ToOwned::to_owned)
                    .collect::<Vec<u8>>();

                if 0 == legal_followers.len() {
                    vec![v] // TODO: Think about a better way to abort this
                    // TODO: Also, much more importantly, if there is even one longer chain with the same starter, we should not have a task for the shorter one
                } else {
                    iter::repeat(v).zip(legal_followers)
                        .map(|(mut l,r)| {
                            l.push(r);
                            l
                        }).collect()
                }
            }).collect();
        }

        let (local_longest, global_estimate) = chains.into_par_iter()
            .map(|c| find_partial_longest(c, &longest_estimates, &follower_table))
            .reduce(|| (Vec::new(), None), |(acc_longest, acc_estimate),(next_longest, next_estimate)| {
                (if next_longest.len() > acc_longest.len() {
                    next_longest
                } else {
                    acc_longest
                }, cmp::max(next_estimate, acc_estimate))
            });

        longest_estimates[start_index as usize] = global_estimate.or(Some(local_longest.len() as u8));

        if local_longest.len() > global_longest.len() {
            global_longest = local_longest;
        }

        if verbose {
            println!("Finished word {}/{} - Longest chain until now ({}):\n{}",
                     start_index as u16 + 1,
                     follower_table.len(),
                     global_longest.len(),
                     pretty_format_chain(&sorted_words, &global_longest));
        }
    };

    global_longest
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

fn validate_granularity(arg: String) -> Result<(), String> {

    const ERROR_MSG: &str = "granularity needs to be between 1 and 255 (inclusive)";

    let num : u8 = arg.parse().map_err(|_| String::from(ERROR_MSG))?;

    if num == 0 {
        Err(String::from(ERROR_MSG))
    }
    else {
        Ok(())
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

fn find_partial_longest(
    mut chain: Vec<u8>,
    longest_estimates: &Vec<Option<u8>>,
    follower_table: &Vec<Vec<u8>>)
    -> (Vec<u8>, Option<u8>) {

    let initial_len = chain.len();

    debug_assert!(initial_len > 0);

    // Contains our best (safe) estimate of what the longest chain for our starting chain would be
    // Again, this actually contains the length - 1
    let mut estimate_for_initial_chain: Option<u8> = None;

    let mut chain_mask = chain
        .iter()
        .map(|&i| U256::one() << i)
        .fold(U256::zero(), |acc, mask| acc + mask);

    // MIN OPT: Guess the size here.
    let mut local_longest = Vec::new();

    let mut follower_table_indices = vec![0u8; follower_table.len()];

    loop {
        let index = *chain.last().unwrap() as usize;

        let followers = &follower_table[index];

        let follower_index = &mut follower_table_indices[index];

        loop {
            if let Some(follower) = followers.get(*follower_index as usize) {
                *follower_index += 1;

                let can_be_longest = longest_estimates[*follower as usize]
                    .and_then(|est| est.checked_add(chain.len() as u8))
                    .map(|potential_len| {
                        estimate_for_initial_chain = Some(cmp::max(potential_len, estimate_for_initial_chain.unwrap_or(0)));
                        potential_len >= local_longest.len() as u8 // we have info about a record and this can maybe be the longest chain
                    })
                    .unwrap_or(true);

                if can_be_longest && !chain_mask.bit(*follower as usize) {

                    chain.push(*follower);
                    chain_mask = chain_mask | U256::one() << *follower;

                    break;
                } // else: don't break
            } else {
                *follower_index = 0;

                if chain.len() > local_longest.len() {
                    local_longest = chain.clone();
                }

                chain.pop();

                if chain.len() < initial_len {

                    return (local_longest, estimate_for_initial_chain);
                }

                chain_mask = chain_mask & !(U256::one() << index);

                break;
            }
        }
    };
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