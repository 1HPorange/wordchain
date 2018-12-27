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
    let mut starter_table = create_starter_table(&connectivity_index_table);
    let mut follower_table = create_follower_table(&connectivity_index_table);
    let words = Arc::new(words);

    for _ in 1..num_cpus::get() {

        // Copy/clone shared resources
        let mut starter_table = starter_table.clone();
        let mut follower_table = follower_table.clone();
        let words = Arc::clone(words);
        let rng = SmallRng::from_entropy();

        // Start search thread
        thread::spawn(move || find_longest_thread(&mut starter_table, &mut follower_table, words, rng));
    }

    // Start search on this thread
    let rng = SmallRng::from_entropy();

    find_longest_thread( &mut starter_table, &mut follower_table, words, &rng);
}

fn find_longest_thread<R>(
    starter_table: &mut Vec<Follower>,
    follower_table: &mut Vec<Vec<Follower>>,
    words: Vec<String>,
    rng: &R)
    where R: Rng {

    // One-time setup
    let mut average_chain_lens = vec![1f32; follower_table.len()]; // todo: see if we should use better estimate here
    let mut average_chain_lens_sum = average_chain_lens.len() as f32;

    let mut chain: Vec<u8> = Vec::new(); // PERF: Guess size
    let mut chain_mask: U256;

    loop {

        // Reset per-chain resources
        let mut latest = pick_random_starter(&average_chain_lens);
        chain_mask = U256::one() << latest;
        chain.clear();

        loop { // Chain growing

            let mut followers = (&follower_table[latest as usize])
                .iter()
                .filter(|&follower| !chain_mask.bit(follower.follower_index as usize))
                .peekable();

            if followers.peek().is_some() {

                latest = pick_random_follower(&legal_followers);
                chain_mask = chain_mask | U256::one() << latest;
                chain.push(latest);

            } else {
                break
            }
        }

        let chain_flen = chain.len() as f32;

        // Update starter average length
        rolling_average_update(&mut average_chain_lens[latest], chain_flen);

        // Re-calculate sum of average chain lengths for starters
        average_chain_lens_sum = average_chain_lens.iter().sum();

        // ... and the average length of each pair in the chain
        update_follower_averages(follower_table, &chain, chain_flen);
    }
}

struct Follower {

    follower_index: u8,

    /// The average chain length for the PAIR of words where this word is the follower
    average_chain_len_pair: f32 // Think about f64

}

fn create_starter_table(connectivity_index_table: &Vec<Vec<u8>>) -> Vec<Follower> {



}

fn create_follower_table(connectivity_index_table: &Vec<Vec<u8>>) -> Vec<Vec<Follower>> {

    connectivity_index_table.iter().map(|followers| {

            followers.iter().map(|&follower| {
                Follower {
                    follower_index: follower,
                    average_chain_len_pair: 1f32 // todo: see if we should use better estimate here
                }
            }).collect()

        }).collect()
}

fn rolling_average_update(current: &mut f32, new_sample: f32) {

    const CONVERGENCE_RATE: f32 = 0.05; // TODO: Investigate other values

    *current = current + CONVERGENCE_RATE * (x - current);
}

fn pick_random_starter() -> u8 {

    unimplemented!()

}

fn pick_random_follower<I>() -> u8 {

    unimplemented!()

}

fn update_follower_averages(
    followers: &mut Vec<Vec<Follower>>,
    chain: &Vec<u8>,
    new_sample: f32) {

    for &[a, b] in chain.windows(2) {

        let a_follower = followers[a as usize].iter_mut()
            .find(|f| f.follower_index == b)
            .unwrap();

        rolling_average_update(&mut a_follower.average_chain_len_pair, new_sample);
    }
}