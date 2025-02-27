use super::par::Par;
use crate::{collect_into::ParCollectInto, computations::ParallelRunner, parameters::Params};
use orx_concurrent_iter::ConcurrentIter;

pub struct ParMap<I, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
{
    params: Params,
    iter: I,
    map: M,
}

impl<I, O, M> From<ParMap<I, O, M>> for (Params, I, M)
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
{
    fn from(value: ParMap<I, O, M>) -> Self {
        (value.params, value.iter, value.map)
    }
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
        let (params, iter, map) = self.into();
        C::map_into::<_, _, R>(output, params, iter, map)
    }
}
