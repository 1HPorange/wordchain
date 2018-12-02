use rayon::prelude::*;
use super::words;
use std::sync::{Mutex};
use rand::rngs::SmallRng;
use rand::prelude::*;

fn find_longest(
    connectivity_index_table: &Vec<Vec<u8>>,
    sorted_words: &Vec<String>,
    longest_global: Mutex<Vec<u8>>) {

    let mut longest_known = vec![0u8; connectivity_index_table.len()];

    loop {



    }

    // TODO: Move this to own module randomChain.rs

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

fn get_rnd_index<'a, I>(
    rng: &mut SmallRng,
    chain_lengths: I,
    lengths_sum: u16) -> usize where
    I: IntoIterator<Item = &'a u8> {

    // TODO: Try with uniform distrib. here

    let target = rng.gen_range(1u16, lengths_sum);

    let mut acc = 0u16;

    for (i, &prob) in chain_lengths.into_iter().enumerate() {

        acc += prob as u16;

        if acc >= target {
            return i;
        }
    }

    panic!("Unreachable code")
}