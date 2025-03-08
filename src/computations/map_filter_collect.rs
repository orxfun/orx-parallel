use crate::{
    parameters::Params,
    runner::{ComputationKind, ParallelRunner},
};
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub struct MapFilterCollect<I, O, Map, Filter, P>
where
    I: ConcurrentIter,
    O: Send + Sync,
    Map: Fn(I::Item) -> O + Send + Sync,
    Filter: Fn(&O) -> bool + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    params: Params,
    iter: I,
    map: Map,
    filter: Filter,
    bag: ConcurrentOrderedBag<O, P>,
}

impl<I, O, Map, Filter, P> MapFilterCollect<I, O, Map, Filter, P>
where
    I: ConcurrentIter,
    O: Send + Sync,
    Map: Fn(I::Item) -> O + Send + Sync,
    Filter: Fn(&O) -> bool + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    fn compute_in_place<R: ParallelRunner>(self) -> (usize, usize) {
        (0, 0)
    }
}
