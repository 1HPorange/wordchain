extern crate rayon;
extern crate uint;

mod words;
mod connectivity;
mod sorting;
mod chain;
mod tasks;

pub use sorting::SortingOrder;

pub struct Config {
    /// How many characters are at least required to chain two words together
    pub min_overlap: usize,

    /// How many tasks are generated for each word before distributing them to threads
    /// Lower values decrease management and memory overhead, but can lead to load imbalance
    /// Generally, larger workloads run faster with higher values. The smallest recommended
    /// value is the number of CPU Cores (+ Hyperthreads)
    /// If None is passed, the crate will estimate a good value based on the number of input words
    pub granularity: Option<usize>,

    /// Whether to print intermediate results to std::out
    /// Recommended for long running tasks. Note that even with this flag, you might not see
    /// any output for a VERY long time.
    pub verbose: bool,

    /// Internal sorting strategy for the words (see: SortingOrder documentation)
    pub sorting_order: SortingOrder
}

pub struct ChainInfo {
    pub len: u8,
    pub chain: String
}

pub fn find_longest_chain(words: Vec<String>, config: &Config) -> Result<ChainInfo, &'static str> {

    validate_input(&words, &config)?;

    let connectivity_map = connectivity::create_connectivity_map(&words, config.min_overlap);

    let words = sorting::sort_words(words, &connectivity_map, &config.sorting_order);

    let connectivity_index_table = connectivity::create_connectivity_index_table(&words, &connectivity_map);

    let longest_chain_indices = chain::find_longest_chain_parallel(&connectivity_index_table, &words, &config);

    Ok(ChainInfo{
        len: longest_chain_indices.len() as u8,
        chain: words::pretty_format_index_chain(&words, &longest_chain_indices)
    })

}

fn validate_input(words: &Vec<String>, config: &Config) -> Result<(), &'static str> {

    if words.len() > 256 {
        return Err("This algorithm is limited to 256 words. Please remove some words from your file.")
    };

    // TODO: Validate config

    Ok(())
}

