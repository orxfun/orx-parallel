use crate::computational_variants::Par;
use crate::computations::X;
use crate::par_iter_result::{IntoResult, ParIterResult};
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::{IterationOrder, ParCollectInto, ParIter};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParFallible<I, T, E, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    I::Item: IntoResult<T, E>,
    E: Send,
{
    par: Par<I, R>,
    phantom: PhantomData<(T, E)>,
}

impl<I, T, E, R> ParFallible<I, T, E, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    I::Item: IntoResult<T, E>,
    E: Send,
{
    pub(crate) fn new(par: Par<I, R>) -> Self {
        Self {
            par,
            phantom: PhantomData,
        }
    }
}

impl<I, T, E, R> ParIterResult<R> for ParFallible<I, T, E, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    I::Item: IntoResult<T, E>,
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
        let x1 = |i: I::Item| i.into_result();
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
        let x1 = |i: I::Item| i.into_result();
        let x = X::new(params, iter, x1);
        x.try_reduce::<R, _>(reduce).1
    }

    // early exit

    fn first(self) -> Result<Option<Self::Success>, Self::Error>
    where
        Self::Success: Send,
    {
        let (params, iter) = self.par.destruct();
        let x1 = |i: I::Item| i.into_result();
        let x = X::new(params, iter, x1);
        match params.iteration_order {
            IterationOrder::Ordered => x.try_next::<R>().1,
            IterationOrder::Arbitrary => x.try_next_any::<R>().1,
        }
    }
}
