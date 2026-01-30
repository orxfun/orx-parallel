use alloc::vec::Vec;
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;

pub enum SortKind {
    Sorted,
    ReverseSorted,
    Mixed,
}

pub enum SplitKind {
    AllInLeft,
    AllInRight,
    OneInLeft,
    OneInRight,
    MoreInLeft,
    MoreInRight,
    Middle,
}

impl SplitKind {
    pub fn split_point(&self, len: usize) -> usize {
        match self {
            Self::AllInLeft => len,
            Self::AllInRight => 0,
            Self::OneInLeft => len.min(1),
            Self::OneInRight => len.saturating_sub(1),
            Self::MoreInLeft => len * 3 / 4,
            Self::MoreInRight => len / 4,
            Self::Middle => len / 2,
        }
    }
}

pub fn new_vec<T: Ord>(len: usize, elem: impl Fn(usize) -> T, sort_kind: SortKind) -> Vec<T> {
    let mut vec: Vec<_> = (0..len).map(elem).collect();
    match sort_kind {
        SortKind::Sorted => vec.sort(),
        SortKind::ReverseSorted => {
            vec.sort();
            vec.reverse();
        }
        SortKind::Mixed => {
            let num_shuffles = 10 * len;
            let mut rng = ChaCha8Rng::seed_from_u64(42);
            for _ in 0..num_shuffles {
                let i = rng.random_range(0..len);
                let j = rng.random_range(0..len);
                vec.swap(i, j);
            }
        }
    }
    vec
}

pub fn split_to_sorted_vecs<T: Ord + Clone>(vec: &[T], split_kind: SplitKind) -> (Vec<T>, Vec<T>) {
    split_at(vec, split_kind.split_point(vec.len()))
}

fn split_at<T: Ord + Clone>(vec: &[T], split_at: usize) -> (Vec<T>, Vec<T>) {
    let (left, right) = vec.split_at(split_at);
    let mut left: Vec<_> = left.iter().cloned().collect();
    let mut right: Vec<_> = right.iter().cloned().collect();
    left.sort();
    right.sort();
    (left, right)
}
