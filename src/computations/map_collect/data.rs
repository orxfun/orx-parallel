use crate::parameters::Params;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_fixed_vec::IntoConcurrentPinnedVec;

pub struct MapCollectData<I, O, Map, P>
where
    I: ConcurrentIter,
    O: Send + Sync,
    Map: Fn(I::Item) -> O + Send + Sync,
    P: IntoConcurrentPinnedVec<O>,
{
    pub params: Params,
    pub iter: I,
    pub map: Map,
    pub collected: ConcurrentOrderedBag<O, P>,
}
