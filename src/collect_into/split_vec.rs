use super::par_collect_into::ParCollectIntoCore;
use orx_concurrent_iter::ConcurrentIter;
use orx_split_vec::{GrowthWithConstantTimeAccess, SplitVec};

impl<O, G> ParCollectIntoCore<O> for SplitVec<O, G>
where
    O: Send + Sync,
    G: GrowthWithConstantTimeAccess,
{
    type BridgePinnedVec = Self;

    fn collect_into<I>(self, con_iter: I) -> Self
    where
        I: ConcurrentIter,
    {
        todo!()
    }
}
