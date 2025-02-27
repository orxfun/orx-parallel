use super::par_collect_into::ParCollectIntoCore;
use orx_fixed_vec::FixedVec;

impl<T> ParCollectIntoCore<T> for Vec<T>
where
    T: Send + Sync,
{
    type BridgePinnedVec = FixedVec<T>;

    fn map_into<I, M, R>(self, params: crate::parameters::Params, iter: I, map: M) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        M: Fn(I::Item) -> T + Send + Sync + Clone,
        R: crate::computations::ParallelRunner,
    {
        todo!()
    }
}
