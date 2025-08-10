use super::par_collect_into::ParCollectIntoCore;
use crate::collect_into::utils::extend_vec_from_split;
use crate::computations::{M, Values, X};
use crate::runner::ParallelRunner;
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;
use orx_split_vec::SplitVec;

impl<O> ParCollectIntoCore<O> for Vec<O>
where
    O: Send + Sync,
{
    type BridgePinnedVec = FixedVec<O>;

    fn empty(iter_len: Option<usize>) -> Self {
        match iter_len {
            Some(len) => Vec::with_capacity(len),
            None => Vec::new(),
        }
    }

    fn m_collect_into<R, I, M1>(mut self, m: M<I, O, M1>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        M1: Fn(I::Item) -> O + Sync,
        O: Send,
    {
        match m.par_len() {
            None => {
                let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
                let split_vec = split_vec.m_collect_into::<R, _, _>(m);
                extend_vec_from_split(self, split_vec)
            }
            Some(len) => {
                self.reserve(len);
                let fixed_vec = FixedVec::from(self);
                let (_num_spawned, fixed_vec) = m.collect_into::<R, _>(fixed_vec);
                Vec::from(fixed_vec)
            }
        }
    }

    fn x_collect_into<R, I, Vo, M1>(self, x: X<I, Vo, M1>) -> Self
    where
        R: ParallelRunner,
        I: ConcurrentIter,
        Vo: Values<Item = O>,
        M1: Fn(I::Item) -> Vo + Sync,
    {
        let split_vec = SplitVec::with_doubling_growth_and_max_concurrent_capacity();
        let split_vec = split_vec.x_collect_into::<R, _, _, _>(x);
        extend_vec_from_split(self, split_vec)
    }

    // test

    #[cfg(test)]
    fn length(&self) -> usize {
        self.len()
    }
}
