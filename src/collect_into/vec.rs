use super::par_collect_into::ParCollectIntoCore;
use crate::{computations::MapCollect, parameters::Params, runner::ParallelRunner};
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::FixedVec;
use orx_pinned_vec::PinnedVec;
use orx_split_vec::{GrowthWithConstantTimeAccess, SplitVec};

impl<T> ParCollectIntoCore<T> for Vec<T>
where
    T: Send + Sync,
{
    type BridgePinnedVec = FixedVec<T>;

    fn empty(iter_len: Option<usize>) -> Self {
        match iter_len {
            Some(len) => Vec::with_capacity(len),
            None => Vec::new(),
        }
    }

    fn map_into<I, M, R>(mut self, params: Params, iter: I, map: M) -> Self
    where
        I: orx_concurrent_iter::ConcurrentIter,
        M: Fn(I::Item) -> T + Send + Sync + Clone,
        R: ParallelRunner,
    {
        match iter.try_get_len() {
            None => {
                let split_vec = SplitVec::with_doubling_growth_and_fragments_capacity(32)
                    .map_into::<_, _, R>(params, iter, map);
                extend_from_split(self, split_vec)
            }
            Some(len) => {
                self.reserve(len);
                let fixed = FixedVec::from(self);
                let bag = ConcurrentOrderedBag::from(fixed);
                let (_num_spawned, pinned_vec) =
                    MapCollect::new(params, iter, map, bag).compute::<R>();
                Vec::from(pinned_vec)
            }
        }
    }

    // test

    #[cfg(test)]
    fn length(&self) -> usize {
        self.len()
    }
}

fn extend_from_split<T, G>(mut initial_vec: Vec<T>, collected_split_vec: SplitVec<T, G>) -> Vec<T>
where
    G: GrowthWithConstantTimeAccess,
{
    match initial_vec.len() {
        0 => collected_split_vec.to_vec(),
        _ => {
            initial_vec.reserve(collected_split_vec.len());
            initial_vec.extend(collected_split_vec);
            initial_vec
        }
    }
}
