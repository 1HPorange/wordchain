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
    let follower_table = create_extended_follower_table(&connectivity_index_table);
    let words = Arc::new(words);

    for _ in 1..num_cpus::get() {

        // Copy/clone shared resources
        let follower_table = follower_table.clone();
        let words = Arc::clone(words);
        let rng =

        // Start search thread
        thread::spawn(move || find_longest_thread(follower_table, words));
    }

    // Start search on this thread
    find_longest_thread(follower_table, words);
}

fn find_longest_thread<R>(
    follower_table: &mut Vec<Vec<Follower>>,
    words: Vec<String>,
    rng: &R)
    where R: Rng {

    // One-time setup
    let mut average_chain_lens = vec![0f32; follower_table.len()];
    let mut chain = Vec::new(); // TODO: Check Type PERF: Guess size
    let mut chain_mask: U256;

    loop {

        // Reset per-chain resources
        let mut latest = pick_random_starter(&average_chain_lens);
        chain_mask = U256::one() << latest;
        chain.clear();

        loop { // Chain growing

            let followers = filter_legal_followers(&chain_mask, &follower_table[latest as usize]);

            if followers.peek().is_some() {

                latest = pick_random_follower(&legal_followers);
                chain_mask = chain_mask | U256::one() << latest;
                chain.push(latest);

            } else {
                break
            }
        }

        let safe_len = (chain.len() - 1) as u8;

        // Update starter average length
        rolling_average_update(&mut average_chain_lens[latest], safe_len);

        // ... and the average length of each pair in the chain
        update_follower_averages(&mut follower_table, &chain, safe_len);
    }
}

struct Follower {

    follower_index: u8,

    /// The average chain length for the PAIR of words where this word is the follower
    average_chain_len_pair: f32 // Think about f64

}

fn create_extended_follower_table(connectivity_index_table: &Vec<Vec<u8>>) -> Vec<Vec<Follower>> {

    connectivity_index_table.iter().map(|followers| {

            followers.iter().map(|&follower| {
                Follower {
                    follower_index: follower,
                    average_chain_len_pair: 1f32 // todo: see if we should use better estimate here
                }
            }).collect()

        }).collect()

}

//pub fn find_longest(
//    connectivity_index_table: Vec<Vec<u8>>,
//    words: Vec<String>,) {
//
//    let words = Arc::new(words);
//    let cit = Arc::new(connectivity_index_table);
//    let longest_global = Arc::new(Mutex::new(0u8));
//
//    for _ in 1..num_cpus::get() {
//
//        let words = Arc::clone(&words);
//        let cit = Arc::clone(&cit);
//        let longest_global = Arc::clone(&longest_global);
//
//        thread::spawn(move || {
//            find_longest_internal(
//                &*cit,
//                &*words,
//                &*longest_global
//            )
//        });
//    }
//
//    find_longest_internal(
//        &cit,
//        &words,
//        &*longest_global
//    )
//}
//
//fn find_longest_internal(
//    connectivity_index_table: &Vec<Vec<u8>>,
//    words: &Vec<String>,
//    longest_global: &Mutex<u8>) {
//
//    let mut rng = SmallRng::from_entropy();
//
//    // Actually contains (index, length - 1) so we can potentially store chains w/ length 256
//    let mut longest_known = (0u8..)
//        .zip(vec![0u8; connectivity_index_table.len()])
//        .collect::<Vec<_>>();
//
//    // Actually contains the sum of the length, NOT length - 1
//    let mut longest_known_sum = connectivity_index_table.len() as u16;
//
//    // MIN OPT: Guess length
//    let mut chain: Vec<u8> = Vec::new();
//
//    // Local longest chain length - 1, used so we don't have to acquire the mutex as often
//    let mut longest_local = 0u8;
//
//    loop {
//
//        // Chose starter index randomly based on longest_known
//        let starter = rnd_elem(&mut rng, &longest_known, longest_known_sum);
//
//        // Clear chain
//        chain.clear();
//
//        // Init chain mask
//        let mut chain_mask = U256::zero();
//
//        // Set current index to starter
//        let mut current = starter;
//
//        loop {
//            // Add to chain
//            chain.push(current);
//
//            // Update bit-mask
//            chain_mask = chain_mask | U256::one() << current;
//
//            // Fetch follower table and filter to legal followers
//            let legal_followers =
//                connectivity_index_table[current as usize].iter()
//                    .filter(|&&f| !chain_mask.bit(f as usize));
//
//            // Convert to index length pairs
//            let mut follower_len_pairs =
//                legal_followers
//                .map(|&f| &longest_known[f as usize])
//                .peekable();
//
//            // break if there is no legal follower
//            if follower_len_pairs.peek().is_none() {
//                break
//            }
//
//            current = rnd_follower(&mut rng,follower_len_pairs);
//        }
//
//        // We can't grow the chain, so now we check if it is the longest, and
//        // if we need to update our lookup table
//
//        if chain.len() - 1 > longest_local as usize {
//
//            let mut longest_global = longest_global.lock().unwrap();
//
//            if chain.len() - 1 > *longest_global as usize {
//
//                println!("Longest chain: {}: {}",
//                         chain.len(),
//                         pretty_format_index_chain(words, &chain));
//
//                *longest_global = (chain.len() - 1) as u8;
//            }
//
//            longest_local = *longest_global;
//        }
//
//        let (_, longest_for_starter) = &mut longest_known[*chain.first().unwrap() as usize];
//
//        if chain.len() - 1 > *longest_for_starter as usize {
//            longest_known_sum = longest_known_sum - *longest_for_starter as u16 + (chain.len() - 1) as u16;
//            *longest_for_starter = (chain.len() - 1) as u8;
//
//        }
//    }
//}
//
///// Make sure not to call on empty iterator
//fn rnd_elem<'a, I>(rng: &mut SmallRng, pairs: I, length_sum: u16) -> u8
//    where I: IntoIterator<Item = &'a (u8, u8)> { // 1. tuple element is index, 2. is length... yikes
//
//    // TODO: Try boring old uniform distribution here
//
//    let target = rng.gen_range(1u16, length_sum + 1);
//
//    let mut acc = 1u16;
//
//    for &(index,length) in pairs {
//
//        acc = acc + length as u16 + 1;
//
//        if acc > target {
//            return index;
//        }
//    };
//
//    println!("acc: {}, target: {}, length_sum: {}", acc, target, length_sum);
//
//    unreachable!();
//}
//
///// Make sure not to call on empty iterator
//fn rnd_follower<'a, I>(rng: &mut SmallRng, followers: I) -> u8
//    where I: IntoIterator<Item = &'a (u8, u8)> + Clone { // 1. tuple element is index, 2. is length... yikes
//
//    let length_sum = followers.clone().into_iter().fold(0u16, |acc, &(_, l)| acc + l as u16 + 1);
//
//    rnd_elem(rng,followers, length_sum)
//}


