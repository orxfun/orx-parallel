use crate::computational_variants::ParXap;
use crate::computations::X;
use crate::par_iter_result::{IntoResult, ParIterResult};
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::values::TransformableValues;
use crate::values::runner_results::Infallible;
use crate::{IterationOrder, ParCollectInto, ParIter};
use orx_concurrent_iter::ConcurrentIter;
use std::marker::PhantomData;

pub struct ParXapFallible<I, T, E, Vo, M1, R = DefaultRunner>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    Vo::Item: IntoResult<T, E>,
    M1: Fn(I::Item) -> Vo + Sync,
    E: Send,
{
    par: ParXap<I, Vo, M1, R>,
    phantom: PhantomData<(T, E)>,
}

impl<I, T, E, Vo, M1, R> ParXapFallible<I, T, E, Vo, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    Vo::Item: IntoResult<T, E>,
    M1: Fn(I::Item) -> Vo + Sync,
    E: Send,
{
    pub(crate) fn new(par: ParXap<I, Vo, M1, R>) -> Self {
        Self {
            par,
            phantom: PhantomData,
        }
    }
}

impl<I, T, E, Vo, M1, R> ParIterResult<R> for ParXapFallible<I, T, E, Vo, M1, R>
where
    R: ParallelRunner,
    I: ConcurrentIter,
    Vo: TransformableValues<Fallibility = Infallible>,
    Vo::Item: IntoResult<T, E>,
    M1: Fn(I::Item) -> Vo + Sync,
    E: Send,
    T: Send,
    Vo::Item: Send,
{
    type Success = T;

    type Error = E;

    fn con_iter_len(&self) -> Option<usize> {
        self.par.con_iter().try_get_len()
    }

    // computation transformations

    fn map<Out, Map>(self, map: Map) -> impl ParIterResult<R, Success = Out, Error = Self::Error>
    where
        Map: Fn(Self::Success) -> Out + Sync + Clone,
        Out: Send,
    {
        let (params, iter, x1) = self.par.destruct();
        let x1 = move |i: I::Item| {
            let map = map.clone();
            x1(i).map(move |x| x.into_result().map(&map))
        };
        let xap = ParXap::new(params, iter, x1);
        xap.into_fallible()
    }

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Error>
    where
        C: ParCollectInto<Self::Success>,
    {
        let (params, iter, x1) = self.par.destruct();
        let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());
        let x = X::new(params, iter, x1);
        output.x_try_collect_into::<R, _, _, _>(x)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Success>, Self::Error>
    where
        Self::Success: Send,
        Reduce: Fn(Self::Success, Self::Success) -> Self::Success + Sync,
    {
        let (params, iter, x1) = self.par.destruct();
        let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());
        let x = X::new(params, iter, x1);
        x.try_reduce::<R, _>(reduce).1
    }

    // early exit

    fn first(self) -> Result<Option<Self::Success>, Self::Error>
    where
        Self::Success: Send,
    {
        let (params, iter, x1) = self.par.destruct();
        let x1 = |i: I::Item| x1(i).map_while_ok(|x| x.into_result());
        let x = X::new(params, iter, x1);
        match params.iteration_order {
            IterationOrder::Ordered => x.try_next::<R>().1,
            IterationOrder::Arbitrary => x.try_next_any::<R>().1,
        }
    }
}
