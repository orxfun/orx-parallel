use super::par_collect_into::ParCollectIntoCore;
use crate::{computations::ParallelRunnerToArchive, parameters::Params};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<T> ParCollectIntoCore<T> for FixedVec<T>
where
    T: Send + Sync,
{
    type BridgePinnedVec = Self;

    fn empty(iter_len: Option<usize>) -> Self {
        let vec = <Vec<_> as ParCollectIntoCore<_>>::empty(iter_len);
        vec.into()
    }

    fn map_into<I, M, R>(self, params: Params, iter: I, map: M) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> T + Send + Sync + Clone,
        R: ParallelRunnerToArchive,
    {
        let vec = self.into_inner();
        vec.map_into::<_, _, R>(params, iter, map).into()
    }

    // test

    #[cfg(test)]
    fn length(&self) -> usize {
        self.len()
    }
}
