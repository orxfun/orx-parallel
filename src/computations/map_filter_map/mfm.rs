use crate::Params;
use orx_concurrent_iter::ConcurrentIter;

pub struct Mfm<I, T, O, Map1, Filter, Map2>
where
    I: ConcurrentIter,
    Map1: Fn(I::Item) -> T,
    Filter: Fn(&T) -> bool,
    Map2: Fn(T) -> O,
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
    Map1: Fn(I::Item) -> T,
    Filter: Fn(&T) -> bool,
    Map2: Fn(T) -> O,
{
    pub(super) fn new(params: Params, iter: I, map1: Map1, filter: Filter, map2: Map2) -> Self {
        Self {
            params,
            iter,
            map1,
            filter,
            map2,
        }
    }
}
