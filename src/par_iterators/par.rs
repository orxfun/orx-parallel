use super::par_iter::{ParIter, ParIterCore};
use crate::{
    collect_into::ParCollectInto, computations::ParallelRunner, par_iterators::par_map::ParMap,
    parameters::Params,
};
use orx_concurrent_iter::ConcurrentIter;

pub struct Par<I>
where
    I: ConcurrentIter,
{
    iter: I,
    params: Params,
}

impl<I: ConcurrentIter> Par<I> {
    pub(crate) fn new(iter: I, params: Params) -> Self {
        Self { iter, params }
    }
}

impl<I: ConcurrentIter> ParIterCore for Par<I> {
    fn input_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }
}

impl<I, R> ParIter<R> for Par<I>
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
        ParIter::<R>::collect_into(map, output)
    }
}

#[inline(always)]
fn no_ops_map<T>(input: T) -> T {
    input
}
