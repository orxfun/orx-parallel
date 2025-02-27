use super::par::{Par, ParCore};
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

impl<I, O, M> ParMap<I, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
{
    pub(crate) fn new(params: Params, iter: I, map: M) -> Self {
        Self { params, iter, map }
    }

    fn destruct(self) -> (Params, I, M) {
        (self.params, self.iter, self.map)
    }
}

impl<I, O, M> ParCore for ParMap<I, O, M>
where
    I: ConcurrentIter,
    O: Send + Sync,
    M: Fn(I::Item) -> O + Send + Sync + Clone,
{
    fn input_len(&self) -> Option<usize> {
        self.iter.try_get_len()
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
        let (params, iter, map) = self.destruct();
        C::map_into::<_, _, R>(output, params, iter, map)
    }
}
