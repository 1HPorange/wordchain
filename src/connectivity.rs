use std::collections::{HashMap,HashSet};
use super::words::*;

pub type ConnectivityMap = HashMap<String, HashSet<String>>;

pub fn create_connectivity_map(words: &Vec<String>, min_overlap: usize) -> ConnectivityMap
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

pub type ConnectivityIndexTable = Vec<Vec<u8>>;

pub fn create_connectivity_index_table(sorted_words: &Vec<String>, connectivity_map: &ConnectivityMap) -> ConnectivityIndexTable {

    let mut table = Vec::with_capacity(sorted_words.len());

    for word in sorted_words {

        let followers = connectivity_map.get(word).unwrap();

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