use super::collect::MfmCollect;
use crate::{runner::ParallelRunner, Params};
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::IntoConcurrentPinnedVec;

pub struct Mfm<I, T, O, Map1, Filter, Map2>
where
    I: ConcurrentIter,
    Map1: Fn(I::Item) -> T + Send + Sync,
    Filter: Fn(&T) -> bool + Send + Sync,
    Map2: Fn(T) -> O + Send + Sync,
    O: Send + Sync,
{
    pub(super) params: Params,
    pub(super) iter: I,
    pub(super) map1: Map1,
    pub(super) filter: Filter,
    pub(super) map2: Map2,
}

impl<I, T, O, Map1, Filter, Map2> Mfm<I, T, O, Map1, Filter, Map2>
where
    I: ConcurrentIter,
    Map1: Fn(I::Item) -> T + Send + Sync,
    Filter: Fn(&T) -> bool + Send + Sync,
    Map2: Fn(T) -> O + Send + Sync,
    O: Send + Sync,
{
    pub fn new(params: Params, iter: I, map1: Map1, filter: Filter, map2: Map2) -> Self {
        Self {
            params,
            iter,
            map1,
            filter,
            map2,
        }
    }

    pub fn collect<R, P>(self, pinned_vec: P) -> (usize, P)
    where
        R: ParallelRunner,
        P: IntoConcurrentPinnedVec<O>,
    {
        MfmCollect::compute::<R>(self, pinned_vec)
    }
}
