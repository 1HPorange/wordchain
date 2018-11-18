pub fn create_chain_tasks(start_token: u8, connectivity_table: ConnectivityTable) {

    vec![vec![start_index]];

    // TODO: Fix this abomination below...
    for _ in 1..granularity { // TODO: Make this depth configurable or dependent on something smart

        chains = chains.into_iter().flat_map(|v| {

            let last = *v.last().unwrap() as usize;

            let legal_followers = follower_table[last].iter()
                .filter(|i| !v.contains(i))
                .map(ToOwned::to_owned)
                .collect::<Vec<u8>>();

            if 0 == legal_followers.len() {
                vec![v] // TODO: Think about a better way to abort this
                // TODO: Also, much more importantly, if there is even one longer chain with the same starter, we should not have a task for the shorter one
            } else {
                iter::repeat(v).zip(legal_followers)
                    .map(|(mut l,r)| {
                        l.push(r);
                        l
                    }).collect()
            }
        }).collect();
    }
}