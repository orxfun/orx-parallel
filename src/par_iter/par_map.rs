use super::par::Par;
use crate::{collect_into::ParCollectInto, computations::ParallelRunner, parameters::Params};
use orx_concurrent_iter::ConcurrentIter;

pub struct ParMap<I, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
{
    iter: I,
    params: Params,
    map: M,
}

impl<I, O, M, R> Par<R> for ParMap<I, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
    R: ParallelRunner,
{
    type Item = O;

    // collect

    fn collect_into<C>(self, output: C) -> C
    where
        C: ParCollectInto<Self::Item>,
    {
        C::map_into::<_, _, R>(output, self.params, self.iter, self.map)
    }
}
