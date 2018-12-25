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

    for _ in 1..num_cpus::get() {

        // Copy/clone shared resources

        // Start search thread

    }

    // Start search on this thread
}

fn find_longest_thread() {

    // One-time setup

    loop {

        // Starter setup

        loop {

            // Chain growing

        }

        // Result evaluation
    }
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


