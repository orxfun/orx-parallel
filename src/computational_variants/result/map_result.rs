use crate::computations::X;
use crate::par_iter_result::ParIterResult;
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::{ParCollectInto, Params};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator.
pub struct ParMapResult<I, T, E, Mr, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    E: Send,
    Mr: Fn(I::Item) -> Result<T, E> + Sync,
{
    iter: I,
    params: Params,
    map_res: Mr,
    phantom: PhantomData<R>,
}

impl<I, T, E, Mr, R> ParMapResult<I, T, E, Mr, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    E: Send,
    Mr: Fn(I::Item) -> Result<T, E> + Sync,
{
    pub(crate) fn new(iter: I, params: Params, map_res: Mr) -> Self {
        Self {
            iter,
            params,
            map_res,
            phantom: PhantomData,
        }
    }

    fn destruct(self) -> (Params, I, Mr) {
        (self.params, self.iter, self.map_res)
    }
}

impl<I, T, E, Mr, R> ParIterResult<R> for ParMapResult<I, T, E, Mr, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    E: Send,
    Mr: Fn(I::Item) -> Result<T, E> + Sync,
{
    type Success = T;

    type Error = E;

    fn con_iter_len(&self) -> Option<usize> {
        self.iter.try_get_len()
    }

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Success>,
    {
        let (params, iter, map_res) = self.destruct();
        let x1 = move |i: I::Item| map_res(i);
        let x = X::new(params, iter, x1);
        output.x_try_collect_into::<R, _, _, _>(x)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Success>, Self::Error>
    where
        Self::Success: Send,
        Reduce: Fn(Self::Success, Self::Success) -> Self::Success + Sync,
    {
        let (params, iter, map_res) = self.destruct();
        let x1 = move |i: I::Item| map_res(i);
        let x = X::new(params, iter, x1);
        x.try_reduce::<R, _>(reduce).1
    }
}
