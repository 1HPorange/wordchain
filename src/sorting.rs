use std::cmp;
use std::collections::HashMap;

use super::connectivity::{ConnectivityMap};

struct WordRating {
    incoming: usize,
    outgoing: usize,
    //average_match_len: f64 TODO: Maybe include this
}

pub enum SortingOrder {
    /// Sorting that favors faster overall completion of the algorithm
    ForFasterCompletion,

    /// Sorting that favors longer chains as intermediate results, but with an overall longer runtime
    ForFasterIntermediateResults
}

pub fn sort_words<F>(mut words: Vec<String>, follower_map: &ConnectivityMap, sorting_func: F) -> Vec<String> where
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