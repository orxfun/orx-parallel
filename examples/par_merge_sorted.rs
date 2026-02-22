use orx_parallel::{
    DefaultRunner,
    experiment::{
        algorithms::merge_sorted_slices::{
            par::{ParamsParMergeSortedSlices, PivotSearch, par_merge},
            seq::{ParamsSeqMergeSortedSlices, StreakSearch},
        },
        data_structures::{slice_dst::SliceDst, slice_src::SliceSrc},
    },
};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::cell::UnsafeCell;

type X = usize;

fn elem(i: usize) -> X {
    i
}

#[inline(always)]
fn is_leq(a: &X, b: &X) -> bool {
    a < b
}

fn new_vec<T: Ord>(len: usize, elem: impl Fn(usize) -> T) -> Vec<T> {
    let mut vec: Vec<_> = (0..len).map(elem).collect();

    let num_shuffles = 10 * len;
    let mut rng = ChaCha8Rng::seed_from_u64(42);
    for _ in 0..num_shuffles {
        let i = rng.random_range(0..len);
        let j = rng.random_range(0..len);
        vec.swap(i, j);
    }
    vec
}

fn split_to_sorted_vecs<T: Ord + Clone>(vec: &[T]) -> (Vec<T>, Vec<T>) {
    split_at(vec, vec.len() / 2)
}

fn split_at<T: Ord + Clone>(vec: &[T], split_at: usize) -> (Vec<T>, Vec<T>) {
    let (left, right) = vec.split_at(split_at);
    let mut left = left.to_vec();
    let mut right = right.to_vec();
    left.sort();
    right.sort();
    (left, right)
}

struct Input {
    left: Vec<X>,
    right: Vec<X>,
    target: UnsafeCell<Vec<X>>,
}

impl Drop for Input {
    fn drop(&mut self) {
        unsafe {
            let target = &mut *self.target.get();
            target.set_len(self.left.len() + self.right.len());
            self.left.set_len(0);
            self.right.set_len(0);
        }
    }
}

fn main() {
    let len = 1 << 15;
    let vec = new_vec(len, elem);
    let (left, right) = split_to_sorted_vecs(&vec);
    let target = Vec::with_capacity(vec.len()).into();
    let input = Input {
        left,
        right,
        target,
    };

    let target = unsafe { &mut *input.target.get() };
    let target = SliceDst::from_vec(target);
    let left = SliceSrc::from_slice(input.left.as_slice());
    let right = SliceSrc::from_slice(input.right.as_slice());
    let params = ParamsParMergeSortedSlices {
        seq_params: ParamsSeqMergeSortedSlices {
            streak_search: StreakSearch::None,
            put_large_to_left: true,
        },
        pivot_search: PivotSearch::Binary,
        put_large_to_left: true,
        chunk_size: 1,
        num_threads: 8,
    };
    par_merge(
        is_leq,
        left,
        right,
        target,
        &params,
        DefaultRunner::default(),
    );
}
