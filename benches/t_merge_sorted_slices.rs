use criterion::{Criterion, criterion_group, criterion_main};
use orx_criterion::{Experiment, Factors};
use orx_parallel::algorithms::{ExpMergeSortedSlicesParams, PivotSearch, StreakSearch};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::{collections::HashSet, hash::Hash, ptr::slice_from_raw_parts_mut};

type X = usize;

fn elem(i: usize) -> X {
    i
}

#[inline(always)]
fn is_leq(a: &X, b: &X) -> bool {
    a < b
}

fn new_vec<T: Ord>(len: usize, elem: impl Fn(usize) -> T, sort_kind: SortKind) -> Vec<T> {
    let mut vec: Vec<_> = (0..len).map(elem).collect();
    match sort_kind {
        SortKind::Sorted => vec.sort(),
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

fn split_to_sorted_vecs<T: Ord + Clone>(vec: &[T], split_kind: SplitKind) -> (Vec<T>, Vec<T>) {
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

fn target_slice(target: &Vec<X>) -> &mut [X] {
    let len = target.capacity();
    unsafe { &mut *slice_from_raw_parts_mut(target.as_ptr() as *mut X, len) }
}

// treatments

#[derive(Clone, Copy, Debug)]
enum SortKind {
    Sorted,
    Mixed,
}

#[derive(Clone, Copy, Debug)]
enum SplitKind {
    MoreInLeft,
    MoreInRight,
    Middle,
}

impl SplitKind {
    fn split_point(&self, len: usize) -> usize {
        match self {
            Self::MoreInLeft => len * 3 / 4,
            Self::MoreInRight => len / 4,
            Self::Middle => len / 2,
        }
    }
}

struct Input {
    left: Vec<X>,
    right: Vec<X>,
    target: Vec<X>,
}

impl Drop for Input {
    fn drop(&mut self) {
        unsafe {
            self.target.set_len(self.left.len() + self.right.len());
            self.left.set_len(0);
            self.right.set_len(0);
        }
    }
}

struct MergeData {
    e: usize,
    sort: SortKind,
    split: SplitKind,
}

impl Factors for MergeData {
    fn factor_names() -> Vec<&'static str> {
        vec!["e (len=2^e)", "sort", "split"]
    }

    fn factor_names_short() -> Vec<&'static str> {
        vec!["e", "so", "sp"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            self.e.to_string(),
            format!("{:?}", self.sort),
            format!("{:?}", self.split),
        ]
    }

    fn factor_levels_short(&self) -> Vec<String> {
        vec![
            self.e.to_string(),
            match self.sort {
                SortKind::Sorted => "T",
                SortKind::Mixed => "F",
            }
            .to_string(),
            match self.split {
                SplitKind::Middle => "M",
                SplitKind::MoreInLeft => "L",
                SplitKind::MoreInRight => "R",
            }
            .to_string(),
        ]
    }
}

impl MergeData {
    fn all() -> Vec<Self> {
        let mut all = vec![];

        let e = [10, 15, 20];
        let sort = [SortKind::Mixed, SortKind::Sorted];
        let split = [
            SplitKind::Middle,
            SplitKind::MoreInLeft,
            SplitKind::MoreInRight,
        ];

        for e in e {
            for sort in sort {
                for split in split {
                    all.push(MergeData { e, sort, split });
                }
            }
        }
        all
    }
}

// factors

#[derive(PartialOrd, Ord, Eq, Clone)]
struct Params(ExpMergeSortedSlicesParams);

impl Factors for Params {
    fn factor_names() -> Vec<&'static str> {
        vec![
            "num_threads",
            "streak_search",
            "sequential_merge_threshold",
            "pivot_search",
            "put_large_to_left",
        ]
    }

    fn factor_names_short() -> Vec<&'static str> {
        vec!["nt", "ss", "thr", "ps", "sw"]
    }

    fn factor_levels(&self) -> Vec<String> {
        vec![
            self.0.num_threads.to_string(),
            format!("{:?}", self.0.streak_search),
            self.0.sequential_merge_threshold.to_string(),
            format!("{:?}", self.0.pivot_search),
            self.0.put_large_to_left.to_string(),
        ]
    }

    fn factor_levels_short(&self) -> Vec<String> {
        vec![
            self.0.num_threads.to_string(),
            match self.0.streak_search {
                StreakSearch::None => "X",
                StreakSearch::Linear => "L",
                StreakSearch::Binary => "B",
            }
            .to_string(),
            self.0.sequential_merge_threshold.to_string(),
            match self.0.pivot_search {
                PivotSearch::Linear => "L",
                PivotSearch::Binary => "B",
            }
            .to_string(),
            match self.0.put_large_to_left {
                true => "T",
                false => "F",
            }
            .to_string(),
        ]
    }
}

impl Hash for Params {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        let pivot_search = match self.0.sequential_merge_threshold {
            0 => PivotSearch::Binary,
            _ => self.0.pivot_search,
        };

        let params = ExpMergeSortedSlicesParams {
            num_threads: self.0.num_threads,
            sequential_merge_threshold: self.0.sequential_merge_threshold,
            put_large_to_left: self.0.put_large_to_left,
            streak_search: self.0.streak_search,
            pivot_search,
        };

        params.hash(state);
    }
}

impl PartialEq for Params {
    fn eq(&self, other: &Self) -> bool {
        match (
            self.0.sequential_merge_threshold,
            other.0.sequential_merge_threshold,
        ) {
            (0, 0) => {
                (self.0.put_large_to_left, self.0.streak_search)
                    == (other.0.put_large_to_left, other.0.streak_search)
            }
            _ => {
                (
                    self.0.put_large_to_left,
                    self.0.streak_search,
                    self.0.sequential_merge_threshold,
                    self.0.pivot_search,
                ) == (
                    other.0.put_large_to_left,
                    other.0.streak_search,
                    other.0.sequential_merge_threshold,
                    other.0.pivot_search,
                )
            }
        }
    }
}

impl Params {
    fn all() -> Vec<Self> {
        let mut all = HashSet::new();
        let num_threads = 1;
        let swaps = [false, true];
        let streaks = [
            StreakSearch::None,
            StreakSearch::Linear,
            StreakSearch::Binary,
        ];
        let pivots = [PivotSearch::Linear, PivotSearch::Binary];
        let thresholds = [0, 128, 1024, 4096];
        for put_large_to_left in swaps {
            for streak_search in streaks {
                for pivot_search in pivots {
                    for sequential_merge_threshold in thresholds {
                        all.insert(Self(ExpMergeSortedSlicesParams {
                            streak_search,
                            num_threads,
                            sequential_merge_threshold,
                            pivot_search,
                            put_large_to_left,
                        }));
                    }
                }
            }
        }
        let mut all: Vec<_> = all.into_iter().collect();
        all.sort();
        all
    }
}

// exp

struct TuneExperiment;

impl Experiment for TuneExperiment {
    type InputFactors = MergeData;

    type AlgFactors = Params;

    type Input = Input;

    type Output = ();

    fn input(&mut self, treatment: &Self::InputFactors) -> Self::Input {
        let len = 1 << treatment.e;
        let vec = new_vec(len, elem, treatment.sort);
        let (left, right) = split_to_sorted_vecs(&vec, treatment.split);
        let target = Vec::with_capacity(vec.len());
        Input {
            left,
            right,
            target,
        }
    }

    fn execute(&mut self, variant: &Self::AlgFactors, input: &Self::Input) -> Self::Output {
        let target = target_slice(&input.target);
        let params = variant.0;
        orx_parallel::algorithms::merge_sorted_slices(
            is_leq,
            &input.left,
            &input.right,
            target,
            params,
        );
    }
}

fn run(c: &mut Criterion) {
    let treatments = MergeData::all();
    let variants = Params::all();
    TuneExperiment.bench(c, "t_merge_sorted_slices", &treatments, &variants);
}

criterion_group!(benches, run);
criterion_main!(benches);
