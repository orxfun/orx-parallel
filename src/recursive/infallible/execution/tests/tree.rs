use alloc::vec;
use alloc::vec::Vec;
use core::ops::Range;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

const RANGE: Range<u64> = 0..100_000;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct Node {
    pub value: u64,
    pub children: Vec<Node>,
}

impl Node {
    pub fn build_tree(depth: usize, fan_out: usize, rng: &mut ChaCha8Rng) -> Self {
        let children = if depth == 0 {
            vec![]
        } else {
            (0..fan_out)
                .map(|_| Self::build_tree(depth - 1, fan_out, rng))
                .collect()
        };

        let value = rng.random_range(RANGE);
        Self { value, children }
    }

    #[allow(dead_code)]
    pub fn matches(self: &Node, threshold: u64) -> bool {
        self.value % 10_000 < threshold
    }

    #[allow(dead_code)]
    pub fn corresponds(self: &Node, search_value: u64) -> bool {
        self.value == search_value
    }
}
