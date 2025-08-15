use crate::computational_variants::Par;
use crate::computations::X;
use crate::par_iter_result::ParIterResult;
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::{ParCollectInto, ParIter, Params};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

/// A parallel iterator.
pub struct ParResult<I, T, E, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter<Item = Result<T, E>>,
    Par<I, R>: ParIter<R, Item = Result<T, E>>,
    E: Send,
{
    par: Par<I, R>,
    phantom: PhantomData<R>,
}

impl<I, T, E, R> ParResult<I, T, E, R>
where
    R: ParallelRunner,
    I: ConcurrentIter<Item = Result<T, E>>,
    Par<I, R>: ParIter<R, Item = Result<T, E>>,
    E: Send,
{
    pub(crate) fn new(par: Par<I, R>) -> Self {
        Self {
            par,
            phantom: PhantomData,
        }
    }
}

impl<I, T, E, R> ParIterResult<R> for ParResult<I, T, E, R>
where
    R: ParallelRunner,
    I: ConcurrentIter<Item = Result<T, E>>,
    Par<I, R>: ParIter<R, Item = Result<T, E>>,
    E: Send,
{
    type Success = T;

    type Error = E;

    fn con_iter_len(&self) -> Option<usize> {
        self.par.con_iter().try_get_len()
    }

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Success>,
    {
        let (params, iter) = self.par.destruct();
        let x1 = |i: I::Item| i;
        let x = X::new(params, iter, x1);
        output.x_try_collect_into::<R, _, _, _>(x)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Success>, Self::Error>
    where
        Self::Success: Send,
        Reduce: Fn(Self::Success, Self::Success) -> Self::Success + Sync,
    {
        let (params, iter) = self.par.destruct();
        let x1 = |i: I::Item| i;
        let x = X::new(params, iter, x1);
        x.try_reduce::<R, _>(reduce).1
    }
}
