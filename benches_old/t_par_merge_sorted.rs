use criterion::{Criterion, criterion_group, criterion_main};

#[cfg(feature = "experiment")]
mod inner {
    use criterion::Criterion;
    use orx_criterion::{Experiment, Factors};
    use orx_parallel::DefaultRunner;
    use orx_parallel::experiment::algorithms::merge_sorted_slices::par::{
        ParamsParMergeSortedSlices, PivotSearch, par_merge,
    };
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
    #[allow(dead_code)]
    enum SortKind {
        Sorted,
        Mixed,
    }

    #[derive(Clone, Copy, Debug)]
    #[allow(dead_code)]
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

            let e = [15, 20];
            let sort = [SortKind::Mixed];
            let split = [SplitKind::Middle];

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

    struct Params(ParamsParMergeSortedSlices);

    impl Factors for Params {
        fn factor_names() -> Vec<&'static str> {
            vec![
                "streak_search",
                "pivot_search",
                "put_large_to_left",
                "min_split_len",
                "chunk_size",
                "num_threads",
            ]
        }

        fn factor_names_short() -> Vec<&'static str> {
            vec!["ss", "ps", "ll", "min", "ch", "nt"]
        }

        fn factor_levels(&self) -> Vec<String> {
            vec![
                format!("{:?}", self.0.seq_params.streak_search),
                format!("{:?}", self.0.pivot_search),
                self.0.put_large_to_left.to_string(),
                self.0.min_split_len.to_string(),
                self.0.chunk_size.to_string(),
                self.0.num_threads.to_string(),
            ]
        }

        fn factor_levels_short(&self) -> Vec<String> {
            vec![
                match self.0.seq_params.streak_search {
                    StreakSearch::None => "X",
                    StreakSearch::Linear => "L",
                    StreakSearch::Binary => "B",
                }
                .to_string(),
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
                self.0.min_split_len.to_string(),
                self.0.chunk_size.to_string(),
                self.0.num_threads.to_string(),
            ]
        }
    }

    impl Params {
        fn all() -> Vec<Self> {
            let mut all = vec![];
            let put_large_to_left = [false, true];
            let min_split_len = [1024];
            let streak_search = [StreakSearch::None, StreakSearch::Linear];
            let pivot_search = [PivotSearch::Linear, PivotSearch::Binary];
            let chunk_size = [1, 1024];
            let num_threads = [1, 8];

            for put_large_to_left in put_large_to_left[..1].to_vec() {
                for min_split_len in min_split_len[..1].to_vec() {
                    for streak_search in streak_search[..1].to_vec() {
                        for pivot_search in pivot_search[..1].to_vec() {
                            for chunk_size in chunk_size[..1].to_vec() {
                                for num_threads in num_threads {
                                    all.push(Self(ParamsParMergeSortedSlices {
                                        seq_params: ParamsSeqMergeSortedSlices {
                                            streak_search,
                                            put_large_to_left,
                                        },
                                        put_large_to_left,
                                        min_split_len,
                                        pivot_search,
                                        num_threads,
                                        chunk_size,
                                    }));
                                }
                            }
                        }
                    }
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
            par_merge(
                is_leq,
                left,
                right,
                target,
                &params,
                DefaultRunner::default(),
            );
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
