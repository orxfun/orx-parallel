use crate::computational_variants::ParMap;
use crate::computations::X;
use crate::par_iter_result::{IntoResult, ParIterResult};
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::{IterationOrder, ParCollectInto, ParIter};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParMapResult<I, T, E, O, M1, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    O: IntoResult<T, E>,
    M1: Fn(I::Item) -> O + Sync,
    E: Send,
{
    par: ParMap<I, O, M1, R>,
    phantom: PhantomData<(T, E)>,
}

impl<I, T, E, O, M1, R> ParMapResult<I, T, E, O, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    O: IntoResult<T, E>,
    M1: Fn(I::Item) -> O + Sync,
    E: Send,
{
    pub(crate) fn new(par: ParMap<I, O, M1, R>) -> Self {
        Self {
            par,
            phantom: PhantomData,
        }
    }
}

impl<I, T, E, O, M1, R> ParIterResult<R> for ParMapResult<I, T, E, O, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    O: IntoResult<T, E>,
    M1: Fn(I::Item) -> O + Sync,
    E: Send,
{
    type Success = T;

    type Error = E;

    type RegularItem = O;

    type RegularParIter = ParMap<I, O, M1, R>;

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
    ) -> impl ParIterResult<Q, Success = Self::Success, Error = Self::Error> {
        let (params, iter, m1) = self.par.destruct();
        ParMapResult {
            par: ParMap::new(params, iter, m1),
            phantom: PhantomData,
        }
    }

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Success>,
    {
        let (params, iter, m1) = self.par.destruct();
        let x1 = |i: I::Item| m1(i).into_result();
        let x = X::new(params, iter, x1);
        output.x_try_collect_into::<R, _, _, _>(x)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Success>, Self::Error>
    where
        Self::Success: Send,
        Reduce: Fn(Self::Success, Self::Success) -> Self::Success + Sync,
    {
        let (params, iter, m1) = self.par.destruct();
        let x1 = |i: I::Item| m1(i).into_result();
        let x = X::new(params, iter, x1);
        x.try_reduce::<R, _>(reduce).1
    }

    // early exit

    fn first(self) -> Result<Option<Self::Success>, Self::Error>
    where
        Self::Success: Send,
    {
        let (params, iter, m1) = self.par.destruct();
        let x1 = |i: I::Item| m1(i).into_result();
        let x = X::new(params, iter, x1);
        match params.iteration_order {
            IterationOrder::Ordered => x.try_next::<R>().1,
            IterationOrder::Arbitrary => x.try_next_any::<R>().1,
        }
    }
}
