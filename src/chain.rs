use std::cmp;
use rayon::prelude::*;
use super::{tasks, words};
use uint::U256;

pub fn find_longest_chain_parallel(
    connectivity_index_table: &Vec<Vec<u8>>,
    sorted_words: &Vec<String>,
    granularity: Option<u8>,
    verbose: bool) -> Vec<u8> {

    let mut global_longest = Vec::new(); // MIN OPT: Guess length

    let mut longest_estimates: Vec<Option<u8>> = vec![None; connectivity_index_table.len()];

    for start_index in 0..connectivity_index_table.len() as u8 {

        // TODO: Think about the constant value here and what to pass instead
        let mut chains = tasks::create_chain_tasks(start_index, &connectivity_index_table, granularity.unwrap_or(6));

        let (local_longest, global_estimate) = chains.into_par_iter()
            .map(|c| find_partial_longest_chain(c, &longest_estimates, &connectivity_index_table))
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
                     connectivity_index_table.len(),
                     global_longest.len(),
                     words::pretty_format_index_chain(&sorted_words, &global_longest));
        }
    };

    global_longest
}

fn find_partial_longest_chain(
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