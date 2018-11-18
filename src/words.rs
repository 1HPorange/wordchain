use std::cmp;

pub fn overlapping_chars(left: &str, right: &str) -> usize {

    debug_assert!(left.len() > 0 && right.len() > 0);

    let left = left.to_lowercase();
    let right = right.to_lowercase();

    let mut left = left.chars().as_str();
    let mut right = right.chars().as_str();

    let max_overlap = cmp::min(left.len(), right.len()) - 1;

    // trim words to maximum potential overlap
    left = &left[left.len()-max_overlap..];
    right = &right[..max_overlap];

    for overlap in (1..=max_overlap).rev() {

        if left == right {
            return overlap;
        }

        left = &left[1..];
        right = &right[..right.len() - 1];
    };

    0
}

pub fn pretty_format_index_chain(sorted_words: &Vec<String>, chain: &Vec<u8>) -> String {

    // TODO: Parser should make sure we have at least one word
    debug_assert!(chain.len() > 0);

    let mut result = String::new();

    for (left, right) in chain
        .windows(2)
        .map(|win| (&sorted_words[win[0] as usize], &sorted_words[win[1] as usize])) {

        let overlap = overlapping_chars(&left, &right);

        result.push_str(&left[..left.len() - overlap]);
    };

    result.push_str(&sorted_words[*chain.last().unwrap() as usize]);

    result
}