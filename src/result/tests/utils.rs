use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;

pub fn inputs_id(n: usize, error_idx: Option<usize>) -> Vec<Result<String, Vec<char>>> {
    let error_idx = error_idx.unwrap_or(usize::MAX);
    (0..n)
        .map(|i| match i == error_idx {
            true => Err(vec!['a']),
            false => Ok(i.to_string()),
        })
        .collect()
}

pub fn inputs_map(n: usize) -> Vec<String> {
    (0..n).map(|i| i.to_string()).collect()
}
