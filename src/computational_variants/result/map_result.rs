use crate::computations::X;
use crate::par_iter_result::ParIterResult;
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::values::WhilstOk;
use crate::{ParCollectInto, Params};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator.
pub struct ParMapResult<I, T, E, M, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    M: Fn(I::Item) -> Result<T, E> + Sync,
{
    iter: I,
    params: Params,
    map: M,
    phantom: PhantomData<R>,
}

impl<I, T, E, M, R> ParMapResult<I, T, E, M, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    M: Fn(I::Item) -> Result<T, E> + Sync,
{
    fn destruct(self) -> (Params, I, M) {
        (self.params, self.iter, self.map)
    }
}

impl<I, T, E, M, R> ParIterResult<R> for ParMapResult<I, T, E, M, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    M: Fn(I::Item) -> Result<T, E> + Sync,
{
    type Item = T;

    type Error = E;

    fn con_iter_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }

    fn collect_into<C>(self, output: C) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Item>,
        Self::Error: Send,
    {
        let (params, iter, map) = self.destruct();
        let x1 = move |i: I::Item| WhilstOk::<T, E>::new(map(i));
        let x = X::new(params, iter, x1);
        output.x_try_collect_into::<R, _, _, _>(x)
    }
}
