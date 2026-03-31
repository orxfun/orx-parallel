use crate::par_iter_result::IntoResult;
use crate::runner::{DefaultRunner, ParallelRunner};
use crate::using::executor::parallel_compute as prc;
use crate::using::{ParIterResultUsing, UParMap, Using};
use crate::{IterationOrder, ParCollectInto, ParIterUsing};
use core::marker::PhantomData;
use orx_concurrent_iter::ConcurrentIter;

/// A parallel iterator for which the computation either completely succeeds,
/// or fails and **early exits** with an error.
pub struct UParMapResult<'using, U, I, T, E, O, M1, R = DefaultRunner>
where
    U: Using<'using>,
    R: ParallelRunner,
    I: ConcurrentIter,
    O: IntoResult<T, E>,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
    par: UParMap<'using, U, I, O, M1, R>,
    phantom: PhantomData<(T, E)>,
}

impl<'using, U, I, T, E, O, M1, R> UParMapResult<'using, U, I, T, E, O, M1, R>
where
    U: Using<'using>,
    R: ParallelRunner,
    I: ConcurrentIter,
    O: IntoResult<T, E>,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
    pub(crate) fn new(par: UParMap<'using, U, I, O, M1, R>) -> Self {
        Self {
            par,
            phantom: PhantomData,
        }
    }
}

impl<'using, U, I, T, E, O, M1, R> ParIterResultUsing<'using, U, R>
    for UParMapResult<'using, U, I, T, E, O, M1, R>
where
    U: Using<'using>,
    R: ParallelRunner,
    I: ConcurrentIter,
    O: IntoResult<T, E>,
    M1: Fn(&mut U::Item, I::Item) -> O + Sync,
{
    type Item = T;

    type Err = E;

    type RegularItem = O;

    type RegularParIter = UParMap<'using, U, I, O, M1, R>;

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
        orchestrator: Q,
    ) -> impl ParIterResultUsing<'using, U, Q, Item = Self::Item, Err = Self::Err> {
        let (using, _, params, iter, m1) = self.par.destruct();
        UParMapResult {
            par: UParMap::new(using, orchestrator, params, iter, m1),
            phantom: PhantomData,
        }
    }

    // collect

    fn collect_into<C>(self, output: C) -> Result<C, Self::Err>
    where
        C: ParCollectInto<Self::Item>,
        Self::Item: Send,
        Self::Err: Send,
    {
        let (using, orchestrator, params, iter, m1) = self.par.destruct();
        let x1 = |u: *mut U::Item, i: I::Item| {
            // SAFETY: TODO-USING
            let u = unsafe { &mut *u };
            m1(u, i).into_result()
        };
        output.u_x_try_collect_into(using, orchestrator, params, iter, x1)
    }

    // reduce

    fn reduce<Reduce>(self, reduce: Reduce) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
        Reduce: Fn(&mut U::Item, Self::Item, Self::Item) -> Self::Item + Sync,
    {
        let (using, orchestrator, params, iter, m1) = self.par.destruct();
        let x1 = |u: *mut U::Item, i: I::Item| {
            // SAFETY: TODO-USING
            let u = unsafe { &mut *u };
            m1(u, i).into_result()
        };
        let reduce = move |u: *mut U::Item, a: Self::Item, b: Self::Item| {
            // SAFETY: TODO-USING
            let u = unsafe { &mut *u };
            reduce(u, a, b)
        };
        prc::reduce::x(using, orchestrator, params, iter, x1, reduce).1
    }

    // early exit

    fn first(self) -> Result<Option<Self::Item>, Self::Err>
    where
        Self::Item: Send,
        Self::Err: Send,
    {
        let (using, orchestrator, params, iter, m1) = self.par.destruct();
        let x1 = |u: *mut U::Item, i: I::Item| {
            // SAFETY: TODO-USING
            let u = unsafe { &mut *u };
            m1(u, i).into_result()
        };
        match params.iteration_order {
            IterationOrder::Ordered => {
                let (_, result) = prc::next::x(using, orchestrator, params, iter, x1);
                result.map(|x: Option<(usize, T)>| x.map(|y| y.1))
            }
            IterationOrder::Arbitrary => {
                let (_, result) = prc::next_any::x(using, orchestrator, params, iter, x1);
                result
            }
        }
    }
}
