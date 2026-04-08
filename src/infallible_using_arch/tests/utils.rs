use alloc::vec::Vec;
use std::string::{String, ToString};

pub fn inputs(n: usize) -> Vec<String> {
    (0..n).map(|x| x.to_string()).collect()
}
