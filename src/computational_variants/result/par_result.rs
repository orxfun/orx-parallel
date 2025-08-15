use crate::computations::X;
use crate::par_iter_result::ParIterResult;
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::{ParCollectInto, Params};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator.
pub struct ParResult<I, T, E, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter<Item = Result<T, E>>,
    E: Send,
{
    iter: I,
    params: Params,
    phantom: PhantomData<R>,
}

impl<I, T, E, R> ParResult<I, T, E, R>
where
    R: ParallelRunner,
    I: ConcurrentIter<Item = Result<T, E>>,
    E: Send,
{
    fn destruct(self) -> (Params, I) {
        (self.params, self.iter)
    }
}

impl<I, T, E, R> ParIterResult<R> for ParResult<I, T, E, R>
where
    R: ParallelRunner,
    I: ConcurrentIter<Item = Result<T, E>>,
    E: Send,
{
    type Item = T;

    type Error = E;

    fn con_iter_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Item>,
    {
        let (params, iter) = self.destruct();
        let x1 = |i: I::Item| i;
        let x = X::new(params, iter, x1);
        output.x_try_collect_into::<R, _, _, _>(x)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Item>, Self::Error>
    where
        Self::Item: Send,
        Reduce: Fn(Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (params, iter) = self.destruct();
        let x1 = |i: I::Item| i;
        let x = X::new(params, iter, x1);
        x.try_reduce::<R, _>(reduce).1
    }
}
