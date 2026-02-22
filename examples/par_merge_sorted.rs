use clap::Parser;
use orx_parallel::{
    DefaultRunner,
    experiment::{
        algorithms::merge_sorted_slices::{
            par::{ParamsParMergeSortedSlices, PivotSearch, par_merge},
            seq::{ParamsSeqMergeSortedSlices, StreakSearch, seq_merge},
        },
        data_structures::{slice_dst::SliceDst, slice_src::SliceSrc},
    },
};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::{cell::UnsafeCell, time::Instant};

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

impl Input {
    fn is_target_sorted(&mut self) -> bool {
        let target = self.target.get_mut();
        let mut sorted = target.clone();
        sorted.sort();
        target == &sorted
    }
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

#[derive(Parser, Debug)]
struct Args {
    #[arg(long, default_value_t = false)]
    with_diagnostics: bool,
    #[arg(long, default_value_t = 10)]
    min_split_len_e: usize,
    #[arg(long, default_value_t = 8)]
    num_threads: usize,
    #[arg(long, default_value_t = 1024)]
    chunk_size: usize,
    #[arg(long, default_value_t = 23)]
    len_e: usize,
}

fn main() {
    let args = Args::parse();

    let Args {
        with_diagnostics,
        min_split_len_e,
        num_threads,
        chunk_size,
        len_e,
    } = args;

    let len = 1 << len_e;
    let min_split_len = 1 << min_split_len_e;
    let par = num_threads != 1;
    let vec = new_vec(len, elem);
    let (left, right) = split_to_sorted_vecs(&vec);
    let target = Vec::with_capacity(vec.len()).into();
    let mut input = Input {
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
        min_split_len,
        chunk_size,
        num_threads,
    };

    let begin = Instant::now();
    match par {
        true => match with_diagnostics {
            true => par_merge(
                is_leq,
                left,
                right,
                target,
                &params,
                DefaultRunner::default().with_diagnostics(),
            ),
            false => par_merge(
                is_leq,
                left,
                right,
                target,
                &params,
                DefaultRunner::default(),
            ),
        },
        false => seq_merge(is_leq, left, right, target, &params.seq_params),
    }
    println!("{:?}", begin.elapsed());
    assert!(input.is_target_sorted());
}
