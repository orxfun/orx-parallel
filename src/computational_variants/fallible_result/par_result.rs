use crate::computational_variants::Par;
use crate::computations::X;
use crate::par_iter_result::{IntoResult, ParIterResult};
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::{IterationOrder, ParCollectInto, ParIter};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParResult<I, T, E, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    I::Item: IntoResult<T, E>,
    E: Send,
{
    par: Par<I, R>,
    phantom: PhantomData<(T, E)>,
}

impl<I, T, E, R> ParResult<I, T, E, R>
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

impl<I, T, E, R> ParIterResult<R> for ParResult<I, T, E, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    I::Item: IntoResult<T, E>,
    E: Send,
{
    type Ok = T;

    type Err = E;

    type RegularItem = I::Item;

    type RegularParIter = Par<I, R>;

    fn con_iter_len(&self) -> Option<usize> {
        self.par.con_iter().try_get_len()
    }

    fn into_regular_par(self) -> Self::RegularParIter {
        self.par
    }

    fn from_regular_par(regular_par: Self::RegularParIter) -> Self {
        Self {
            par: regular_par,
            phantom: PhantomData,
        }
    }

    // params transformations

    fn with_runner<Q: ParallelRunner>(
        self,
    ) -> impl ParIterResult<Q, Ok = Self::Ok, Err = Self::Err> {
        let (params, iter) = self.par.destruct();
        ParResult {
            par: Par::new(params, iter),
            phantom: PhantomData,
        }
    }

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Err>
    where
        C: ParCollectInto<Self::Ok>,
    {
        let (params, iter) = self.par.destruct();
        let x1 = |i: I::Item| i.into_result();
        let x = X::new(params, iter, x1);
        output.x_try_collect_into::<R, _, _, _>(x)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Ok>, Self::Err>
    where
        Self::Ok: Send,
        Reduce: Fn(Self::Ok, Self::Ok) -> Self::Ok + Sync,
    {
        let (params, iter) = self.par.destruct();
        let x1 = |i: I::Item| i.into_result();
        let x = X::new(params, iter, x1);
        x.try_reduce::<R, _>(reduce).1
    }

    // early exit

    fn first(self) -> Result<Option<Self::Ok>, Self::Err>
    where
        Self::Ok: Send,
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
