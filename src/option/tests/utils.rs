use alloc::string::{String, ToString};
use alloc::vec::Vec;

pub fn inputs(n: usize, error_idx: Option<usize>) -> Vec<Option<String>> {
    let error_idx = error_idx.unwrap_or(usize::MAX);
    (0..n)
        .map(|i| match i == error_idx {
            true => None,
            false => Some(i.to_string()),
        })
        .collect()
}
