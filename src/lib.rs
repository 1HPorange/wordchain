extern crate num_cpus;
extern crate rand;
extern crate rayon;
extern crate uint;

mod chain;
mod connectivity;
mod random_chain;
mod sorting;
mod tasks;
mod words;

use sorting::SortingOrder;

use uint::construct_uint;

construct_uint! {
    struct U256(4);
}

pub struct Config<'a> {
    /// How many characters are at least required to chain two words together
    pub min_overlap: usize,

    /// Mode of search
    pub mode: Mode<'a>,
}

pub struct SortedSearchConfig {
    /// How many levels of recursion the task generation algorithm uses
    /// Lower values decrease management and memory overhead, but can lead to load imbalance
    /// Generally, larger workloads run faster with higher values.
    pub granularity: Option<u8>,

    /// Enables more detailed intermediate output
    pub verbose: bool,
}

pub enum Mode<'a> {
    /// Normal execution, geared towards fastest definitive result
    Normal(&'a SortedSearchConfig),

    /// Sacrifices execution time for better intermediate results which are printed during execution
    QuickEstimate(&'a SortedSearchConfig),

    /// Uses a heuristic search to guess the longest chain. Will never terminate and there is
    /// no guarantee that you will get the correct result. Useful for long word lists.
    RandomSearch,
}

pub struct ChainInfo {
    pub len: u8,
    pub chain: String,
}

pub fn find_longest_chain(words: Vec<String>, config: &Config) -> Result<ChainInfo, &'static str> {
    validate_input(&words, &config)?;

    let connectivity_map = connectivity::create_connectivity_map(&words, config.min_overlap);

    match config.mode {
        Mode::Normal(ssc) => start_sorted_search(
            words,
            &connectivity_map,
            ssc,
            SortingOrder::ForFasterCompletion,
        ),

        Mode::QuickEstimate(ssc) => start_sorted_search(
            words,
            &connectivity_map,
            ssc,
            SortingOrder::ForFasterIntermediateResults,
        ),

        Mode::RandomSearch => start_random_search(words, &connectivity_map),
    }
}

fn start_sorted_search(
    words: Vec<String>,
    connectivity_map: &connectivity::ConnectivityMap,
    sorted_search_config: &SortedSearchConfig,
    sorting_order: SortingOrder,
) -> Result<ChainInfo, &'static str> {
    let words = sorting::sort_words(words, &connectivity_map, sorting_order);

    let connectivity_index_table =
        connectivity::create_connectivity_index_table(&words, &connectivity_map);

    let longest_chain_indices = chain::find_longest_chain_parallel(
        &connectivity_index_table,
        &words,
        sorted_search_config.granularity,
        sorted_search_config.verbose,
    );

    Ok(ChainInfo {
        len: longest_chain_indices.len() as u8,
        chain: words::pretty_format_index_chain(&words, &longest_chain_indices),
    })
}

fn start_random_search(
    words: Vec<String>,
    connectivity_map: &connectivity::ConnectivityMap,
) -> Result<ChainInfo, &'static str> {
    let connectivity_index_table =
        connectivity::create_connectivity_index_table(&words, &connectivity_map);

    random_chain::find_longest(connectivity_index_table, words);

    unreachable!();
}

fn validate_input(words: &Vec<String>, _config: &Config) -> Result<(), &'static str> {
    if words.len() > 256 {
        return Err(
            "This algorithm is limited to 256 words. Please remove some words from your file.",
        );
    };

    Ok(())
}
