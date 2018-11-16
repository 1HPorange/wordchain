#[macro_use]
extern crate clap;

extern crate uint;

extern crate spmc;

extern crate num_cpus;

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
use std::thread;
use std::iter;

use std::sync::{mpsc,Arc,Mutex};

fn main() {

    const ARG_MIN_OVERLAP: &str = "min-overlap";
    const ARG_WORD_FILE: &str = "word-file";
    const ARG_SINGLE_THREAD: &str = "single-thread";

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
        .arg(Arg::with_name(ARG_SINGLE_THREAD)
            .short("s")
            .long("single-thread")
            .help("Runs the algorithm without worker threads, which might be faster in rare cases"))
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

    let before = Instant::now();

    let longest = if matches.is_present(ARG_SINGLE_THREAD) {
        find_longest_chain_single(follower_table)
    } else {
        find_longest_chain_parallel(follower_table)
    };

    duration += before.elapsed();

    println!("Longest chain ({}): {}", longest.len(), pretty_format_chain(&sorted_words, &longest));

    println!("Finished search in {}.{} s", duration.as_secs(), duration.subsec_millis());
}

fn find_longest_chain_single(follower_table: Vec<Vec<u8>>) -> Vec<u8> {

    let mut follower_table_indices = vec![0u8; follower_table.len()];

    let mut longest_estimates: Vec<Option<u8>> = vec![None; follower_table.len()];

    let follower_table = Arc::new(follower_table);

    let longest_chain = Arc::new(Mutex::new(Vec::new()));

    find_partial_longest(vec![0u8], &mut follower_table_indices, &longest_estimates.clone(), &follower_table, &longest_chain);

    println!("Finished word 1/{}", follower_table.len());

    longest_estimates[0] = Some(longest_chain.lock().unwrap().len() as u8);

    for start_index in 1..follower_table.len() as u8 {

        let est = find_partial_longest(vec![start_index], &mut follower_table_indices, &longest_estimates.clone(), &follower_table, &longest_chain);

        longest_estimates[start_index as usize] = Some(est);

        println!("Finished word {}/{}", start_index as u16 + 1, follower_table.len());
    }

    // TODO: Think about a better way to return this

    let longest_chain = longest_chain.lock().unwrap().clone();

    longest_chain
}

fn find_longest_chain_parallel(follower_table: Vec<Vec<u8>>) -> Vec<u8> {

    let follower_table = Arc::new(follower_table);

    let longest_chain = Arc::new(Mutex::new(Vec::new()));

    let (task_tx, task_rx) = spmc::channel();
    let (estimates_tx, estimates_rx) = spmc::channel();
    let (result_tx, result_rx) = mpsc::channel();

    let mut threads = Vec::new();

    for _ in 0..num_cpus::get() {

        let thread_task_rx = task_rx.clone();
        let thread_estimates_rx = estimates_rx.clone();
        let thread_tx = result_tx.clone();

        let thread_follower_table = Arc::clone(&follower_table);
        let thread_longest_chain = Arc::clone(&longest_chain);

        threads.push(thread::spawn(move || {

            // POSS OPT: Can be one shorter i think ;)
            let mut follower_table_indices = vec![0u8; thread_follower_table.len()];

            loop {
                let start_chain = match thread_task_rx.recv() {
                    Ok(chain) => chain,
                    Err(_) => break
                };

                let longest_estimates = match thread_estimates_rx.recv() {
                    Ok(estimates) => estimates,
                    Err(_) => break
                };

                let partial_result = find_partial_longest(
                    start_chain,
                    &mut follower_table_indices,
                    &longest_estimates,
                    &thread_follower_table,
                    &thread_longest_chain
                );

                thread_tx.send(partial_result).unwrap();
            }

        }));
    };

    let mut longest_estimates: Vec<Option<u8>> = vec![None; follower_table.len()];

    for start_index in 0..follower_table.len() as u8 {

        let mut chains = vec![vec![start_index]];

        // TODO: Fix this abomination below...
        for _ in 1..=3 { // TODO: Make this depth configurable or dependent on something smart

            chains = chains.iter().flat_map(|v| {

                let last = *v.last().unwrap() as usize;

                let legal_followers = follower_table[last].iter()
                    .filter(|i| !v.contains(i))
                    .map(ToOwned::to_owned)
                    .collect::<Vec<u8>>();

                iter::repeat(v).zip(legal_followers)
                    .map(|(l,r)| {
                        let mut new = l.clone();

                        new.push(r);
                        new

                    })
            }).collect();
        }

        let num_chains = chains.len();

        for chain in chains {

            task_tx.send(chain).unwrap();
            estimates_tx.send(longest_estimates.clone()).unwrap();
        }

        let mut longest_estimate = 0u8;

        for _ in 0..num_chains {
            longest_estimate = cmp::max(longest_estimate, result_rx.recv().unwrap());
        }

        if 0 == start_index { // TODO: Make this nicer
            longest_estimates[0] = Some(longest_chain.lock().unwrap().len() as u8);
        }
        else {
            longest_estimates[start_index as usize] = Some(longest_estimate);
        }

        println!("Finished word {}/{}", start_index as u16 + 1, follower_table.len());
    };

    drop(task_tx);

    for thread in threads {
        thread.join().unwrap();
    }

    // TODO: Think about a better way to return this

    let longest_chain = longest_chain.lock().unwrap().clone();

    longest_chain
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

fn find_partial_longest(
    mut chain: Vec<u8>,
    follower_table_indices: &mut Vec<u8>,
    longest_estimates: &Vec<Option<u8>>,
    follower_table: &Arc<Vec<Vec<u8>>>,
    longest_chain: &Arc<Mutex<Vec<u8>>>)
    -> u8 {

    let initial_len = chain.len();

    debug_assert!(initial_len > 0);

    // Contains our best (safe) estimate of what the longest chain for our starting chain would be
    // Again, this actually contains the length - 1
    let mut estimate_for_initial_chain = 0u8;

    let mut chain_mask = chain
        .iter()
        .map(|&i| U256::one() << i)
        .fold(U256::zero(), |acc, mask| acc + mask);

    // Contains a safe guess of how long the global record chain is. Can avoid extensive mutex locking.
    let mut local_longest_hint = 0u8;

    loop {
        let index = *chain.last().unwrap() as usize;

        let followers = &follower_table[index];

        let follower_index = &mut follower_table_indices[index];

        loop {
            if let Some(follower) = followers.get(*follower_index as usize) {
                *follower_index += 1;

                // This happens before the membership test because this test is much cheaper and can lead to skipping the membership test
                let can_be_longest: bool = match longest_estimates[*follower as usize] {
                    Some(estimate) =>  match estimate.checked_add(chain.len() as u8) {
                        Some(potential_len) => {
                            estimate_for_initial_chain = cmp::max(potential_len, estimate_for_initial_chain);
                            potential_len >= local_longest_hint // we have info about a record and this can maybe be the longest chain
                        },
                        None => {
                            estimate_for_initial_chain = std::u8::MAX;
                            true // we have info about a record and this can definitely be the longest chain
                        }
                    },
                    None => true // we don't have info about a record
                };

                if can_be_longest && !chain_mask.bit(*follower as usize) {

                    chain.push(*follower);
                    chain_mask = chain_mask | U256::one() << *follower;

                    break;
                } // else: don't break
            } else {
                *follower_index = 0;

                if chain.len() as u8 > local_longest_hint {

                    let mut longest = longest_chain.lock().unwrap();

                    if chain.len() > longest.len() {
                        *longest = chain.clone();
                    }

                    local_longest_hint = longest.len() as u8;
                }

                chain.pop();

                if chain.len() < initial_len {
                    return estimate_for_initial_chain;
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