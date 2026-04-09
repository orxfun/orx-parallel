use crate::infallible::collectables::col_into::ColIntoInf;
use crate::infallible::collectables::utils::extend_vec_from_split;
use crate::infallible::par_runner::ParRunnerInfallible;
use crate::infallible::{Par, Xap};
use crate::runner::ParRunner;
use alloc::vec::Vec;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;

impl<T> ColIntoInf<T> for Vec<T> {
    fn empty(exact_len: Option<usize>) -> Self {
        match exact_len {
            Some(len) => Vec::with_capacity(len),
            None => Vec::new(),
        }
    }

    fn collect_into<I, X, R>(mut self, par: Par<I, X, R>, exact_len: Option<usize>) -> Self
    where
        I: ConcurrentIter,
        X: Xap<I = I::Item, O = T>,
        R: ParRunner,
        T: Send,
    {
        let (iter, x, mut exe, params) = par.destruct();
        match exact_len {
            Some(len) => {
                self.reserve(len);
                let fixed_vec = FixedVec::from(self);
                exe.collect(params, iter, x, fixed_vec).into()
            }
            None => {
                let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
                let split_vec = exe.collect(params, iter, x, split_vec);
                extend_vec_from_split(self, split_vec)
            }
        }
    }
}
