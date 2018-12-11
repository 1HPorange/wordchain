use rayon::prelude::*;
use super::words::pretty_format_index_chain;
use std::sync::{Mutex};
use rand::rngs::SmallRng;
use rand::prelude::*;
use uint::U256;

fn find_longest(
    connectivity_index_table: &Vec<Vec<u8>>,
    sorted_words: &Vec<String>,
    longest_global: Mutex<Vec<u8>>) {

    let mut rng = SmallRng::from_entropy();

    // Actually contains (index, length - 1) so we can potentially store chains w/ length 256
    let mut longest_known = (0u8..)
        .zip(vec![0u8; connectivity_index_table.len()])
        .collect::<Vec<_>>();

    let mut longest_known_sum = 0u16;

    // MIN OPT: Guess length
    let mut chain: Vec<u8> = Vec::new();

    loop {

        // Chose starter index randomly based on longest_known
        let starter = rnd_elem(&mut rng, &longest_known, longest_known_sum);

        // Clear chain and add starter
        chain.clear();
        chain.push(starter);

        // Init chain mask
        let chain_mask = U256::zero();

        // Set current index to starter
        let mut current = starter;

        loop {
            // Fetch follower table and filter to legal followers
            let legal_followers =
                connectivity_index_table[current as usize].iter()
                .filter(|&&f| !chain_mask.bit(f as usize));

            // Convert to index length pairs
            let mut follower_len_pairs = legal_followers
                .map(|&f| &longest_known[f as usize])
                .peekable();

            // Chose one randomly if result not empty, otherwise check longest and break
            if follower_len_pairs.peek().is_some() {

                let next = rnd_follower(&mut rng,follower_len_pairs);

                // Add to chain and set current
                chain.push(next);
                current = next;

            } else {
                let mut longest_global = longest_global.lock().unwrap();

                if chain.len() > longest_global.len() {
                    *longest_global = chain.clone();

                    println!("Longest chain: {}: {}",
                        chain.len(),
                        pretty_format_index_chain(sorted_words, &chain));
                }

                break
            }
        }
    }

    // TODO: Also try with uniform distribution and compare results

    // TODO: This method doesn't need sorting. See that it doesn't get sorting!

    // TODO: This method won't work with the quick-results and verbose flags. Think of sth!
    // Maybe introduce a -m[ode] switch that can chose between those things

    // Every thread has an internal list of (starter,longest).
    // The starter list has a prob. distribution attached
    // Also every follower table also has prob. distribution attached

    // Use distribution to chose first
    // Then do monte carlo
    // At each step, use the distribution attached to the follower list to chose a follower

    // mutex checks for global longest chain (local lookup copy of length)

    // When new longest for a starter is discovered:
    // Update starter list distribution
    // Update every follower list distribution that the starter is in

    // Note: We cannot re-sort the table bc. it's an INDEX table, but that would make stuff easier.
    // We could for example map a half-normal distr. to the list index for faster "random" lookup
    // Maybe think about doing this in the future (tm)
}

/// Make sure not to call on empty iterator
fn rnd_elem<'a, I>(rng: &mut SmallRng, pairs: I, length_sum: u16) -> u8
    where I: IntoIterator<Item = &'a (u8, u8)> { // 1. tuple element is index, 2. is length... yikes

    // TODO: Try boring old uniform distribution here

    let target = rng.gen_range(1u16, length_sum + 1);

    let mut acc = 1u16;

    for &(index,length) in pairs {

        acc = acc + length as u16;

        if acc > target {
            return index;
        }
    };

    unreachable!();
}

/// Make sure not to call on empty iterator
fn rnd_follower<'a, I>(rng: &mut SmallRng, followers: I) -> u8
    where I: IntoIterator<Item = &'a (u8, u8)> + Clone { // 1. tuple element is index, 2. is length... yikes

    let length_sum = followers.clone().into_iter().fold(0u16, |acc, &(_, l)| acc + l as u16);

    rnd_elem(rng,followers, length_sum)
}


