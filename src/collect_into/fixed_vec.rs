use super::par_collect_into::ParCollectIntoCore;
use crate::{
    map_filter_map::{Mfm, Values},
    parameters::Params,
    runner::ParallelRunner,
};
use orx_concurrent_iter::ConcurrentIter;
use orx_fixed_vec::FixedVec;

impl<O> ParCollectIntoCore<O> for FixedVec<O>
where
    O: Send + Sync,
{
    type BridgePinnedVec = Self;

    fn empty(iter_len: Option<usize>) -> Self {
        let vec = <Vec<_> as ParCollectIntoCore<_>>::empty(iter_len);
        vec.into()
    }

    fn collect_into<R, I, T, Vt, Vo, M1, F, M2>(
        self,
        mfm: Mfm<I, T, Vt, O, Vo, M1, F, M2>,
        in_input_order: bool,
    ) -> Self
    where
        R: ParallelRunner,
        I: orx_concurrent_iter::ConcurrentIter,
        Vt: Values<Item = T>,
        O: Send + Sync,
        Vo: Values<Item = O>,
        M1: Fn(I::Item) -> Vt + Send + Sync,
        F: Fn(&T) -> bool + Send + Sync,
        M2: Fn(T) -> Vo + Send + Sync,
    {
        let vec = Vec::from(self);
        FixedVec::from(vec.collect_into::<R, _, _, _, _, _, _, _>(mfm, in_input_order))
    }

    fn map_into<I, M, R>(self, params: Params, iter: I, map: M) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> O + Send + Sync + Clone,
        R: ParallelRunner,
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
