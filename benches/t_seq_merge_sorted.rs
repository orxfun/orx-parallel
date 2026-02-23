use criterion::{Criterion, criterion_group, criterion_main};

#[cfg(feature = "experiment")]
mod inner {
    use criterion::Criterion;
    use orx_criterion::{Experiment, Factors};
    use orx_parallel::experiment::algorithms::merge_sorted_slices::seq::seq_merge;
    use orx_parallel::experiment::algorithms::merge_sorted_slices::seq::{
        ParamsSeqMergeSortedSlices, StreakSearch,
    };
    use orx_parallel::experiment::data_structures::slice_dst::SliceDst;
    use orx_parallel::experiment::data_structures::slice_src::SliceSrc;
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
        let mut left = left.to_vec();
        let mut right = right.to_vec();
        left.sort();
        right.sort();
        (left, right)
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

            let e = [15, 20, 22, 25];
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

    struct Params(ParamsSeqMergeSortedSlices);

    impl Factors for Params {
        fn factor_names() -> Vec<&'static str> {
            vec!["streak_search", "put_large_to_left"]
        }

        fn factor_names_short() -> Vec<&'static str> {
            vec!["nt", "ss"]
        }

        fn factor_levels(&self) -> Vec<String> {
            vec![
                format!("{:?}", self.0.streak_search),
                self.0.put_large_to_left.to_string(),
            ]
        }

        fn factor_levels_short(&self) -> Vec<String> {
            vec![
                match self.0.streak_search {
                    StreakSearch::None => "X",
                    StreakSearch::Linear => "L",
                    StreakSearch::Binary => "B",
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

    impl Params {
        fn all() -> Vec<Self> {
            let mut all = vec![];
            let put_large_to_left = [false, true];
            let streak_search = [
                StreakSearch::None,
                StreakSearch::Linear,
                StreakSearch::Binary,
            ];
            for put_large_to_left in put_large_to_left {
                for streak_search in streak_search.iter().cloned() {
                    all.push(Self(ParamsSeqMergeSortedSlices {
                        streak_search,
                        put_large_to_left,
                    }));
                }
            }
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
            let target = Vec::with_capacity(vec.len()).into();
            Input {
                left,
                right,
                target,
            }
        }

        fn execute(&mut self, variant: &Self::AlgFactors, input: &Self::Input) -> Self::Output {
            let target = unsafe { &mut *input.target.get() };
            let target = SliceDst::from_vec(target);
            let left = SliceSrc::from_slice(input.left.as_slice());
            let right = SliceSrc::from_slice(input.right.as_slice());
            let params = variant.0;
            seq_merge(is_leq, left, right, target, &params);
        }
    }

    pub fn run(c: &mut Criterion) {
        let treatments = MergeData::all();
        let variants = Params::all();
        TuneExperiment.bench(c, "t_seq_merge_sorted", &treatments, &variants);
    }
}

#[cfg(feature = "experiment")]
fn run(c: &mut Criterion) {
    inner::run(c);
}
#[cfg(not(feature = "experiment"))]
fn run(_: &mut Criterion) {
    panic!("REQUIRES FEATURE: experiment");
}
criterion_group!(benches, run);
criterion_main!(benches);
