use alloc::vec;
use alloc::vec::Vec;
use core::ops::Range;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

pub const RANGE: Range<u64> = 0..100_000;

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

pub fn flatten<I, E>(initial: impl IntoIterator<Item = I::Item>, extend: &E) -> Vec<I::Item>
where
    I: IntoIterator,
    E: Fn(&I::Item) -> I + Send + Sync,
{
    fn collect_into<I, E>(
        extend: &E,
        initial: impl IntoIterator<Item = I::Item>,
        dest: &mut Vec<I::Item>,
    ) where
        I: IntoIterator,
        E: Fn(&I::Item) -> I + Send + Sync,
    {
        for x in initial {
            let children = extend(&x);
            collect_into(extend, children, dest);
            dest.push(x);
        }
    }

    let mut dest = vec![];
    collect_into(extend, initial, &mut dest);
    dest
}
