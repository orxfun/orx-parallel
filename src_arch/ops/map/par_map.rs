use crate::par::Par;
use orx_concurrent_iter::ConcurrentIter;
use orx_concurrent_ordered_bag::ConcurrentOrderedBag;
use orx_pinned_vec::PinnedVec;
use std::marker::PhantomData;

pub struct ParMap<Data, Out, Map>
where
    Data: ConcurrentIter,
    Map: Fn(Data::Item) -> Out + Send + Sync,
    Out: Send + Sync,
{
    par: Par<Data>,
    map: Map,
    phantom: PhantomData<Out>,
}

impl<Data, Out, Map> ParMap<Data, Out, Map>
where
    Data: ConcurrentIter,
    Map: Fn(Data::Item) -> Out + Send + Sync,
    Out: Send + Sync,
{
    pub(crate) fn new(par: Par<Data>, map: Map) -> Self {
        Self {
            par,
            map,
            phantom: Default::default(),
        }
    }

    #[inline]
    fn map_into<P: PinnedVec<Out>>(self, out: ConcurrentOrderedBag<Out, P>) -> P {
        super::core::map_collect(self.par, self.map, out)
    }
}
