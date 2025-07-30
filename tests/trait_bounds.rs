use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;
use std::collections::VecDeque;

#[test]
fn trait_bounds_parallelizable() {
    use orx_parallel::Parallelizable;
    fn fun(source: impl Parallelizable) {
        let _iter = source.par();
    }

    fun(vec![1, 2, 3].as_slice());
    fun(&vec![1, 2, 3]);
    fun(&VecDeque::<String>::new());
    fun(0..9);
    fun(&FixedVec::<usize>::new(3));
    fun(&SplitVec::<usize>::new());
}

#[test]
fn trait_bounds_parallelizable_collection() {
    use orx_parallel::ParallelizableCollection;
    fn fun(source: impl ParallelizableCollection) {
        let _iter = source.par();
    }

    fun(vec![1, 2, 3]);
    fun(VecDeque::<String>::new());
    fun(FixedVec::<usize>::new(3));
    fun(SplitVec::<usize>::new());
}

#[test]
fn trait_bounds_into_par_iter() {
    use orx_parallel::IntoParIter;
    fn fun(source: impl IntoParIter) {
        let _iter = source.into_par();
    }

    // owned
    fun(vec![1, 2, 3]);
    fun(VecDeque::<String>::new());
    fun(FixedVec::<usize>::new(3));
    fun(SplitVec::<usize>::new());

    // ref
    fun(vec![1, 2, 3].as_slice());
    fun(&vec![1, 2, 3]);
    fun(&VecDeque::<String>::new());
    fun(0..9);
    fun(&FixedVec::<usize>::new(3));
    fun(&SplitVec::<usize>::new());
}
