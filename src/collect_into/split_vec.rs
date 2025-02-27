use super::par_collect_into::ParCollectIntoCore;
use orx_concurrent_iter::ConcurrentIter;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{GrowthWithConstantTimeAccess, SplitVec};

impl<O, G> ParCollectIntoCore<O> for SplitVec<O, G>
where
    O: Send + Sync,
    G: GrowthWithConstantTimeAccess,
{
    type BridgePinnedVec = Self;

    fn map_into<I>(mut self, iter: I) -> Self
    where
        I: ConcurrentIter,
    {
        match iter.try_get_len() {
            None => self.reserve_maximum_concurrent_capacity(1 << 32),
            Some(len) => self.reserve_maximum_concurrent_capacity(self.len() + len),
        };

        todo!()
    }
}
