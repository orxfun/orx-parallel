use super::par_collect_into::ParCollectIntoCore;
use crate::{computations::MapCollect, parameters::Params, runner::ParallelRunner};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{GrowthWithConstantTimeAccess, PseudoDefault, SplitVec};

impl<T, G> ParCollectIntoCore<T> for SplitVec<T, G>
where
    T: Send + Sync,
    G: GrowthWithConstantTimeAccess,
    Self: PseudoDefault,
{
    type BridgePinnedVec = Self;

    fn empty(iter_len: Option<usize>) -> Self {
        let mut vec = Self::pseudo_default();
        reserve(&mut vec, iter_len);
        vec
    }

    fn map_into<I, M, R>(mut self, params: Params, iter: I, map: M) -> Self
    where
        I: ConcurrentIter,
        M: Fn(I::Item) -> T + Send + Sync + Clone,
        R: ParallelRunner,
    {
        reserve(&mut self, iter.try_get_len());
        let bag: ConcurrentOrderedBag<_, _> = self.into();
        let (_num_spawned, pinned_vec) = MapCollect::new(params, iter, map, bag).compute::<R>();
        pinned_vec
    }

    // test

    #[cfg(test)]
    fn length(&self) -> usize {
        self.len()
    }
}

fn reserve<T, G: GrowthWithConstantTimeAccess>(
    split_vec: &mut SplitVec<T, G>,
    len_to_extend: Option<usize>,
) {
    match len_to_extend {
        None => split_vec.reserve_maximum_concurrent_capacity(1 << 32),
        Some(len) => split_vec.reserve_maximum_concurrent_capacity(split_vec.len() + len),
    };
}
