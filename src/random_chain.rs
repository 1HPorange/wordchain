use super::words::pretty_format_index_chain;
use rand::rngs::SmallRng;
use rand::prelude::*;
use uint::U256;
use std::thread;
use std::sync::{Arc, Mutex};

pub fn find_longest(
    connectivity_index_table: Vec<Vec<u8>>,
    words: Vec<String>) {

    // Setup shared resources
    let longest_len_global = Arc::new(Mutex::new(0usize)); // PERF: Maybe u8?
    let mut starter_table = create_starter_table(&connectivity_index_table);
    let mut follower_table = create_follower_table(&connectivity_index_table);
    let words = Arc::new(words);

    for _ in 1..num_cpus::get() {

        // Copy/clone shared resources
        let longest_len_global = Arc::clone(&longest_len_global);
        let mut starter_table = starter_table.clone();
        let mut follower_table = follower_table.clone();
        let words = Arc::clone(&words);
        let mut rng = SmallRng::from_entropy();

        // Start search thread
        thread::spawn(move || find_longest_thread(&*longest_len_global, &mut starter_table, &mut follower_table, &*words, &mut rng));
    }

    // Start search on this thread
    let mut rng = SmallRng::from_entropy();

    find_longest_thread(&*longest_len_global, &mut starter_table, &mut follower_table, &*words, &mut rng);
}

fn find_longest_thread<R>(
    longest_len_global: &Mutex<usize>,
    starter_table: &mut Vec<Follower>,
    follower_table: &mut Vec<Vec<Follower>>,
    words: &Vec<String>,
    rng: &mut R)
    where R: Rng {

    // One-time setup
    let mut average_chain_lens_sum = starter_table.len() as f32;
    let mut longest_len_local = 0usize; // PERF: Maybe u8?

    let mut chain: Vec<u8> = Vec::new(); // PERF: Guess size
    let mut chain_mask: U256;

    loop {

        // Reset per-chain resources
        let mut latest = pick_random_follower_with_sum(&*starter_table, average_chain_lens_sum, rng);

        chain.clear();
        chain.push(latest);

        loop { // Chain growing

            chain_mask = U256::one() << latest;

            let mut followers = (&mut follower_table[latest as usize])
                .iter()
                .filter(|&follower| !chain_mask.bit(follower.follower_index as usize))
                .peekable();

            if followers.peek().is_some() {

                latest = pick_random_follower(followers, rng);

                chain.push(latest);

            } else {
                break
            }
        }

        if chain.len() > longest_len_local {

            let mut longest_global = longest_len_global.lock().unwrap();

            if chain.len() > *longest_global {

                println!("Longest chain ({}): {}",
                    chain.len(),
                    pretty_format_index_chain(&words, &chain));

                *longest_global = chain.len();
            }

            longest_len_local = *longest_global;
        }

        // Update per-chain lookups with new evidence

        let chain_flen = chain.len() as f32;

        // Update starter average length
        rolling_average_update(&mut starter_table[chain[0] as usize].average_chain_len_pair, chain_flen);

        // Re-calculate sum of average chain lengths for starters
        average_chain_lens_sum = starter_table.iter()
            .map(|f| f.average_chain_len_pair)
            .sum();

        // ... and the average length of each pair in the chain
        update_follower_averages(follower_table, &chain, chain_flen);
    }
}

#[derive(Clone)]
struct Follower {

    follower_index: u8,

    /// The average chain length for the PAIR of words where this word is the follower
    average_chain_len_pair: f32 // Think about f64

}

fn create_starter_table(connectivity_index_table: &Vec<Vec<u8>>) -> Vec<Follower> {

    (0..(connectivity_index_table.len() as u8))
        .map(|i| {
            Follower {
                follower_index: i,
                average_chain_len_pair: 1f32
            }
        })
        .collect()

}

fn create_follower_table(connectivity_index_table: &Vec<Vec<u8>>) -> Vec<Vec<Follower>> {

    connectivity_index_table.iter().map(|followers| {

            followers.iter().map(|&follower| {
                Follower {
                    follower_index: follower,
                    average_chain_len_pair: 1f32
                }
            }).collect()

        }).collect()
}

fn rolling_average_update(current: &mut f32, new_sample: f32) {

    const CONVERGENCE_RATE: f32 = 0.05; // TODO: Investigate other values

    *current = *current + CONVERGENCE_RATE * (new_sample - *current);
}

fn pick_random_follower_with_sum<'a, I, R>(starters: I, starter_avg_sum: f32, rng: &mut R) -> u8 where
    I: IntoIterator<Item=&'a Follower>,
    R: Rng {

    let target = rng.gen_range(0f32, starter_avg_sum);

    let mut acc = 0f32;

    for follower in starters {

        let next_acc = acc + follower.average_chain_len_pair;

        if next_acc > target {
            return follower.follower_index;
        }

        acc = next_acc;
    };

    unreachable!()
}

fn pick_random_follower<'a, I, R>(followers: I, rng: &mut R) -> u8 where
    I: IntoIterator<Item=&'a Follower> + Clone,
    R: Rng {

    let avg_sum = followers.clone().into_iter().map(|f| f.average_chain_len_pair).sum();

    pick_random_follower_with_sum(followers, avg_sum, rng)
}

fn update_follower_averages(
    followers: &mut Vec<Vec<Follower>>,
    chain: &Vec<u8>,
    new_sample: f32) {

    for pair in chain.windows(2) {

        if let &[a, b] = pair {

            let a_follower = followers[a as usize].iter_mut()
                .find(|f| f.follower_index == b)
                .unwrap();

            rolling_average_update(&mut a_follower.average_chain_len_pair, new_sample);

        } else {
            panic!("Windowing function failed")
        }
    }
}