use super::par::{Par, ParCore};
use crate::{
    collect_into::ParCollectInto, computations::ParallelRunner, par_iter::par_map::ParMap,
    parameters::Params,
};
use orx_concurrent_iter::ConcurrentIter;

pub struct ParEmpty<I>
where
    I: ConcurrentIter,
{
    iter: I,
    params: Params,
}

impl<I: ConcurrentIter> ParEmpty<I> {
    pub(crate) fn new(iter: I, params: Params) -> Self {
        Self { iter, params }
    }
}

impl<I: ConcurrentIter> ParCore for ParEmpty<I> {
    fn input_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }
}

impl<I, R> Par<R> for ParEmpty<I>
where
    I: ConcurrentIter,
    R: ParallelRunner,
{
    type Item = I::Item;

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        let map = ParMap::new(self.params, self.iter, no_ops_map);
        Par::<R>::collect_into(map, output)
    }
}

#[inline(always)]
fn no_ops_map<T>(input: T) -> T {
    input
}
