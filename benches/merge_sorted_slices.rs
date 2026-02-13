use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use orx_parallel::algorithms::{ExpMergeSortedSlicesParams, PivotSearch, StreakSearch};
use rand::prelude::*;
use rand_chacha::ChaCha8Rng;
use std::{
    cmp::Ordering, collections::HashSet, fmt::Display, hash::Hash, ptr::slice_from_raw_parts_mut,
};

type X = String;
fn elem(i: usize) -> X {
    i.to_string()
}
#[inline(always)]
fn is_leq(a: &X, b: &X) -> bool {
    a < b
}

fn target_slice(target: &mut Vec<X>) -> &mut [X] {
    let len = target.capacity();
    unsafe { &mut *slice_from_raw_parts_mut(target.as_mut_ptr(), len) }
}

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

struct Treatment {
    sort: SortKind,
    split: SplitKind,
    len: usize,
}

impl Treatment {
    fn new(sort: SortKind, split: SplitKind, len: usize) -> Self {
        Self { sort, split, len }
    }

    fn inputs(&self) -> (Vec<X>, Vec<X>, Vec<X>, Vec<X>) {
        let vec = new_vec(self.len, elem, self.sort);
        let (l, r) = split_to_sorted_vecs(&vec, self.split);
        let target = Vec::with_capacity(vec.len());
        let mut sorted = vec.clone();
        sorted.sort();
        (l, r, target, sorted)
    }
}

impl Display for Treatment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}_{:?}_{:?}", self.len, self.sort, self.split)
    }
}

#[derive(PartialOrd, Ord, Eq, Clone)]
struct Variant(ExpMergeSortedSlicesParams);

impl Display for Variant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let swap = match self.0.put_large_to_left {
            true => "swap",
            false => "no-swap",
        };
        let streak = match self.0.streak_search {
            StreakSearch::None => "none",
            StreakSearch::Linear => "lin",
            StreakSearch::Binary => "bin",
        };
        let pivot = match self.0.pivot_search {
            PivotSearch::Linear => "lin",
            PivotSearch::Binary => "bin",
        };

        match self.0.sequential_merge_threshold {
            0 => write!(f, "seq_0_{swap}_str:{streak}"),
            n => {
                write!(f, "seq_{n}_{swap}_str:{streak}_piv:{pivot}")
            }
        }
    }
}

impl Hash for Variant {
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

impl PartialEq for Variant {
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

impl Variant {
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

fn naive_seq(left: &[X], right: &[X], target: &mut Vec<X>) {
    let dst = target.as_mut_ptr();
    unsafe { dst.copy_from_nonoverlapping(left.as_ptr(), left.len()) };

    let dst = unsafe { target.as_mut_ptr().add(left.len()) };
    unsafe { dst.copy_from_nonoverlapping(right.as_ptr(), right.len()) };

    unsafe { target.set_len(left.len() + right.len()) };
    target.sort_by(|a, b| match is_leq(a, b) {
        true => Ordering::Less,
        false => Ordering::Greater,
    });
}

fn orx_seq(left: &[X], right: &[X], target: &mut Vec<X>, params: ExpMergeSortedSlicesParams) {
    let target = target_slice(target);
    orx_parallel::algorithms::merge_sorted_slices(is_leq, left, right, target, params);
}

fn run(c: &mut Criterion) {
    let mut group = c.benchmark_group("merge_sorted_slices");

    let len = [1 << 10, 1 << 15, 1 << 20];
    let sort = [SortKind::Mixed];
    let split = [SplitKind::Middle];
    let variants = Variant::all();

    for len in len {
        for sort in sort {
            for split in split {
                let t = Treatment::new(sort, split, len);
                let (mut left, mut right, mut target, sorted) = t.inputs();

                group.bench_with_input(BenchmarkId::new("naive_seq", &t), &t, |b, _| {
                    naive_seq(&left, &right, &mut target);
                    assert_eq!(target_slice(&mut target), &sorted);
                    b.iter(|| naive_seq(&left, &right, &mut target));
                });

                for variant in &variants {
                    group.bench_with_input(
                        BenchmarkId::new(format!("orx_{variant}"), &t),
                        &t,
                        |b, _| {
                            orx_seq(&left, &right, &mut target, variant.0.clone());
                            assert_eq!(target_slice(&mut target), &sorted);
                            b.iter(|| orx_seq(&left, &right, &mut target, variant.0.clone()));
                        },
                    );
                }

                unsafe {
                    target.set_len(left.len() + right.len());
                    left.set_len(0);
                    right.set_len(0);
                }
            }
        }
    }

    group.finish();
}

criterion_group!(benches, run);
criterion_main!(benches);
