use super::connectivity::ConnectivityIndexTable;
use rayon::prelude::*;
use rayon::iter::repeat;

pub fn create_chain_tasks(
    start_index: u8,
    connectivity_index_table: &ConnectivityIndexTable,
    granularity: usize) -> Vec<Vec<u8>> {

    let mut tasks = vec![vec![start_index]];

    for _ in 1..granularity {

        let next_gen = tasks.par_iter()
            .flat_map(|t| {

                let last_index = *t.last().unwrap() as usize;

                let followers = &connectivity_index_table[last_index].iter()
                    .filter(|f| !t.contains(f))
                    .map(ToOwned::to_owned)
                    .collect::<Vec<_>>();

                repeat(t)
                    .zip(followers)
                    .map(|(old, &next)| {
                        let mut new = old.clone();
                        new.push(next);
                        new
                    }).collect::<Vec<Vec<u8>>>()

            }).collect::<Vec<Vec<u8>>>();

        if next_gen.len() > 0 {
            tasks = next_gen;
        } else {
            break;
        }
    };

    tasks
}