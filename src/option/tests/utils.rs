use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub fn inputs(n: usize, none_idx: Option<usize>) -> Vec<Option<String>> {
    let none_idx = none_idx.unwrap_or(usize::MAX);
    (0..n)
        .map(|i| match i == none_idx {
            true => None,
            false => Some(i.to_string()),
        })
        .collect()
}
